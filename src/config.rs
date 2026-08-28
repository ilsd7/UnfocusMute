#![cfg_attr(not(windows), allow(dead_code))]

use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
#[cfg(windows)]
use std::cell::RefCell;
use std::cmp::Ordering;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(windows)]
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{
    FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE,
    FindCloseChangeNotification, FindFirstChangeNotificationW, FindNextChangeNotification,
    MOVE_FILE_FLAGS, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
};
#[cfg(windows)]
use windows::Win32::System::Threading::WaitForSingleObject;
#[cfg(windows)]
use windows::core::PCWSTR;

const CONFIG_VERSION: u32 = 7;
const LEGACY_DEFAULT_POLLING_INTERVAL_MS: u64 = 350;
const EVENT_FALLBACK_DEFAULT_POLLING_INTERVAL_MS: u64 = 5_000;
const DEFAULT_POLLING_INTERVAL_MS: u64 = 3_000;
const MIN_POLLING_INTERVAL_MS: u64 = 100;
const MAX_POLLING_INTERVAL_MS: u64 = 10_000;
const MAX_CONFIG_FILE_BYTES: u64 = 64 * 1024;
pub(crate) const MAX_TARGET_NOTE_CHARS: usize = 120;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TargetProcess {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default = "target_enabled_default", skip_serializing_if = "is_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub managed_muted: bool,
}

impl TargetProcess {
    pub(crate) fn matches_session(&self, process_name: &str, pid: u32) -> bool {
        self.name == process_name && self.pid.is_none_or(|target_pid| target_pid == pid)
    }

    #[cfg(test)]
    pub fn new(name: impl AsRef<str>) -> Option<Self> {
        let name = normalize_process_name(name.as_ref())?;
        if !is_supported_normalized_target_process_name(&name) {
            return None;
        }
        Some(Self {
            name,
            pid: None,
            note: None,
            enabled: true,
            managed_muted: false,
        })
    }

    #[cfg(test)]
    pub fn for_pid(name: impl AsRef<str>, pid: u32) -> Option<Self> {
        if pid == 0 {
            return None;
        }
        let name = normalize_process_name(name.as_ref())?;
        if !is_supported_normalized_target_process_name(&name) {
            return None;
        }
        Some(Self {
            name,
            pid: Some(pid),
            note: None,
            enabled: true,
            managed_muted: false,
        })
    }

    pub fn display_identity_into(&self, output: &mut String) {
        output.clear();
        self.push_identity_into(output);
    }

    pub(crate) fn push_identity_into(&self, output: &mut String) {
        output.push_str(&self.name);
        if let Some(pid) = self.pid {
            output.push_str(" (PID ");
            push_decimal_u32(output, pid);
            output.push(')');
        }
    }

    pub fn display_name_into(&self, output: &mut String) {
        output.clear();
        self.push_identity_into(output);
        if let Some(note) = &self.note {
            output.push_str(" - ");
            output.push_str(note);
        }
    }
}

const fn target_enabled_default() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct AppConfig {
    pub version: u32,
    pub language: Language,
    pub theme: ThemePreference,
    pub window_position: Option<WindowPosition>,
    pub window_size: Option<WindowSize>,
    pub polling_interval_ms: u64,
    pub launch_on_startup: bool,
    pub start_minimized: bool,
    pub hide_to_tray_on_close: bool,
    pub targets: Vec<TargetProcess>,
}

pub struct AppConfigLoad {
    pub config: AppConfig,
    pub first_run: bool,
    pub recovered_invalid_config: bool,
    pub(crate) source_stamp: ConfigSourceStamp,
}

pub(crate) enum ExistingConfigLoad {
    Loaded(AppConfig),
    Missing(io::Error),
    Malformed(io::Error),
}

#[derive(Clone, Copy)]
pub(crate) enum ConfigSourceStamp {
    Unknown,
    Known(Option<ConfigFileStamp>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConfigDurability {
    Durable,
    ProcessCrashSafe,
}

#[derive(Clone, Copy)]
enum ExistingConfigGuard {
    #[cfg(test)]
    Any,
    Unchanged(Option<ConfigFileStamp>),
}

struct ExistingConfigValidation {
    stamp: Option<ConfigFileStamp>,
    invalid_backup: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ConfigFileStamp {
    pub(crate) modified: SystemTime,
    pub(crate) len: u64,
    fingerprint: Option<u64>,
}

#[cfg(windows)]
#[derive(Default)]
struct ConfigStampCache {
    change_notification: Option<HANDLE>,
    stamp: Option<ConfigFileStamp>,
}

#[cfg(windows)]
impl ConfigStampCache {
    fn current(&mut self, path: &Path) -> Option<ConfigFileStamp> {
        if let Some(notification) = self.change_notification {
            match unsafe { WaitForSingleObject(notification, 0) } {
                WAIT_TIMEOUT => return self.stamp,
                WAIT_OBJECT_0 => {
                    if unsafe { FindNextChangeNotification(notification) }.is_err() {
                        self.close_notification();
                    }
                }
                _ => self.close_notification(),
            }
        }

        if self.change_notification.is_none()
            && let Some(parent) = path.parent()
        {
            let parent = path_to_wide(parent);
            let filter = FILE_NOTIFY_CHANGE_FILE_NAME
                | FILE_NOTIFY_CHANGE_LAST_WRITE
                | FILE_NOTIFY_CHANGE_SIZE;
            self.change_notification =
                unsafe { FindFirstChangeNotificationW(PCWSTR(parent.as_ptr()), false, filter) }
                    .ok();
        }

        match try_config_file_stamp(path) {
            Ok(stamp) => {
                self.stamp = stamp;
                stamp
            }
            Err(_) => {
                // A sharing violation or another transient read failure must be retried on the
                // next check even when no additional directory notification arrives.
                self.close_notification();
                self.stamp = None;
                None
            }
        }
    }

    fn close_notification(&mut self) {
        if let Some(notification) = self.change_notification.take() {
            unsafe {
                let _ = FindCloseChangeNotification(notification);
            }
        }
    }
}

#[cfg(windows)]
impl Drop for ConfigStampCache {
    fn drop(&mut self) {
        self.close_notification();
    }
}

impl ConfigFileStamp {
    #[cfg(test)]
    pub(crate) fn has_content_fingerprint(self) -> bool {
        self.fingerprint.is_some()
    }
}

#[cfg(test)]
pub(crate) fn config_reload_needed(
    current: Option<ConfigFileStamp>,
    cached: Option<ConfigFileStamp>,
    previous_load_failed: bool,
) -> bool {
    previous_load_failed || current != cached
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            language: Language::default(),
            theme: ThemePreference::default(),
            window_position: None,
            window_size: None,
            polling_interval_ms: DEFAULT_POLLING_INTERVAL_MS,
            launch_on_startup: false,
            start_minimized: true,
            hide_to_tray_on_close: true,
            targets: Vec::new(),
        }
    }
}

impl Default for AppConfigLoad {
    fn default() -> Self {
        Self {
            config: AppConfig::default(),
            first_run: true,
            recovered_invalid_config: false,
            source_stamp: ConfigSourceStamp::Known(None),
        }
    }
}

impl AppConfig {
    pub fn load_or_default_with_status() -> io::Result<AppConfigLoad> {
        let path = cached_config_file_path()?;
        load_or_default_from_path(path)
    }

    pub fn load_existing() -> io::Result<Self> {
        match Self::load_existing_with_status()? {
            ExistingConfigLoad::Loaded(config) => Ok(config),
            ExistingConfigLoad::Missing(error) | ExistingConfigLoad::Malformed(error) => Err(error),
        }
    }

    pub(crate) fn load_existing_with_status() -> io::Result<ExistingConfigLoad> {
        let path = cached_config_file_path()?;
        load_existing_from_path(path)
    }

    pub(crate) fn save_from_source(&self, source_stamp: ConfigSourceStamp) -> io::Result<()> {
        let ConfigSourceStamp::Known(expected_stamp) = source_stamp else {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "config source changed before startup settings could be saved",
            ));
        };
        if self.save_if_unchanged(expected_stamp, ConfigDurability::Durable)? {
            return Ok(());
        }
        Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "config file changed before startup settings could be saved",
        ))
    }

    pub(crate) fn save_if_unchanged(
        &self,
        expected_stamp: Option<ConfigFileStamp>,
        durability: ConfigDurability,
    ) -> io::Result<bool> {
        let path = cached_config_file_path()?;
        self.save_to_path_guarded(
            path,
            true,
            ExistingConfigGuard::Unchanged(expected_stamp),
            durability,
        )
    }

    #[cfg(test)]
    fn save_to_path(&self, path: &Path, validate_existing: bool) -> io::Result<()> {
        for _ in 0..3 {
            if self.save_to_path_guarded(
                path,
                validate_existing,
                ExistingConfigGuard::Any,
                ConfigDurability::Durable,
            )? {
                return Ok(());
            }
        }
        Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "config file kept changing while the app was saving",
        ))
    }

    fn save_to_path_guarded(
        &self,
        path: &Path,
        validate_existing: bool,
        guard: ExistingConfigGuard,
        durability: ConfigDurability,
    ) -> io::Result<bool> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut config = self.clone();
        config.sanitize();

        let config_bytes = serialized_config_bytes(&config)?;
        let (mut temp_file, temp_path) = create_temp_config_file(path)?;
        let write_result = {
            let mut writer = BufWriter::new(&mut temp_file);
            writer
                .write_all(&config_bytes)
                .and_then(|()| writer.flush())
        };
        if let Err(error) = write_result {
            discard_open_file(temp_file, &temp_path);
            return Err(error);
        }
        if durability == ConfigDurability::Durable
            && let Err(error) = temp_file.sync_all()
        {
            discard_open_file(temp_file, &temp_path);
            return Err(error);
        }
        drop(temp_file);
        match existing_config_matches_guard(path, guard) {
            Ok(true) => {}
            Ok(false) => {
                let _ = fs::remove_file(&temp_path);
                return Ok(false);
            }
            Err(error) => {
                let _ = fs::remove_file(&temp_path);
                return Err(error);
            }
        }
        let validation = if validate_existing {
            match validate_existing_config(path) {
                Ok(validation) => Some(validation),
                Err(error) => {
                    let _ = fs::remove_file(&temp_path);
                    return Err(error);
                }
            }
        } else {
            None
        };
        match existing_config_matches_guard(path, guard) {
            Ok(true) => {}
            Ok(false) => {
                let _ = fs::remove_file(&temp_path);
                discard_invalid_backup(validation.as_ref());
                return Ok(false);
            }
            Err(error) => {
                let _ = fs::remove_file(&temp_path);
                discard_invalid_backup(validation.as_ref());
                return Err(error);
            }
        }
        if let Some(validation) = &validation {
            match try_config_file_stamp(path) {
                Ok(stamp) if stamp == validation.stamp => {}
                Ok(_) => {
                    let _ = fs::remove_file(&temp_path);
                    discard_invalid_backup(Some(validation));
                    return Ok(false);
                }
                Err(error) => {
                    let _ = fs::remove_file(&temp_path);
                    discard_invalid_backup(Some(validation));
                    return Err(error);
                }
            }
        }
        replace_file(&temp_path, path, durability)?;
        Ok(true)
    }

    #[cfg(test)]
    pub fn add_target(&mut self, name: impl AsRef<str>) -> bool {
        let Some(target) = TargetProcess::new(name) else {
            return false;
        };
        self.add_target_process(target)
    }

    #[cfg(test)]
    pub fn add_pid_target(&mut self, name: impl AsRef<str>, pid: u32) -> bool {
        let Some(target) = TargetProcess::for_pid(name, pid) else {
            return false;
        };
        self.add_target_process(target)
    }

    pub(crate) fn add_normalized_target(&mut self, name: String) -> bool {
        debug_assert!(is_normalized_process_name(&name));
        if !is_supported_normalized_target_process_name(&name) {
            return false;
        }
        self.add_target_process(TargetProcess {
            name,
            pid: None,
            note: None,
            enabled: true,
            managed_muted: false,
        })
    }

    pub(crate) fn add_normalized_pid_target(&mut self, name: String, pid: u32) -> bool {
        debug_assert!(is_normalized_process_name(&name));
        if pid == 0 || !is_supported_normalized_target_process_name(&name) {
            return false;
        }
        self.add_target_process(TargetProcess {
            name,
            pid: Some(pid),
            note: None,
            enabled: true,
            managed_muted: false,
        })
    }

    fn add_target_process(&mut self, target: TargetProcess) -> bool {
        debug_assert!(is_supported_target_process_name(&target.name));
        debug_assert!(
            self.targets
                .iter()
                .all(|target| is_supported_target_process_name(&target.name))
        );
        match self
            .targets
            .binary_search_by(|existing| compare_targets(existing, &target))
        {
            Ok(_) => false,
            Err(index) => {
                self.targets.insert(index, target);
                true
            }
        }
    }

    pub fn remove_target_at(&mut self, index: usize) -> bool {
        if index >= self.targets.len() {
            return false;
        }
        self.targets.remove(index);
        true
    }

    pub(crate) fn set_target_note_at(&mut self, index: usize, note: Option<String>) -> bool {
        let Some(target) = self.targets.get_mut(index) else {
            return false;
        };
        let note = note.and_then(|note| normalize_target_note(&note));
        if target.note == note {
            return false;
        }
        target.note = note;
        true
    }

    pub(crate) fn set_target_enabled_at(&mut self, index: usize, enabled: bool) -> bool {
        let Some(target) = self.targets.get_mut(index) else {
            return false;
        };
        if target.enabled == enabled {
            return false;
        }
        target.enabled = enabled;
        true
    }

    pub(crate) fn set_target_managed_muted_at(
        &mut self,
        index: usize,
        managed_muted: bool,
    ) -> bool {
        let Some(target) = self.targets.get_mut(index) else {
            return false;
        };
        if target.managed_muted == managed_muted {
            return false;
        }
        target.managed_muted = managed_muted;
        true
    }

    pub(crate) fn contains_normalized_target(&self, name: &str, pid: Option<u32>) -> bool {
        debug_assert!(is_normalized_process_name(name));
        self.targets
            .binary_search_by(|target| compare_target_key(target, name, pid))
            .is_ok()
    }

    pub fn deduplicate_targets(&mut self) {
        self.targets.retain_mut(|target| {
            if target.pid == Some(0) {
                return false;
            }
            let Some(name) = normalize_process_name_owned(std::mem::take(&mut target.name)) else {
                return false;
            };
            if !is_supported_normalized_target_process_name(&name) {
                return false;
            }
            target.name = name;
            if let Some(note) = target.note.take() {
                target.note = normalize_target_note(&note);
            }
            true
        });
        if self.targets.len() > 1 {
            self.sort_targets();
            self.merge_duplicate_targets();
        }
    }

    fn sanitize(&mut self) {
        let previous_version = self.version;
        self.version = CONFIG_VERSION;
        let should_migrate_default_interval = (previous_version < 2
            && self.polling_interval_ms == LEGACY_DEFAULT_POLLING_INTERVAL_MS)
            || (previous_version < 3
                && self.polling_interval_ms == EVENT_FALLBACK_DEFAULT_POLLING_INTERVAL_MS);
        if should_migrate_default_interval {
            self.polling_interval_ms = DEFAULT_POLLING_INTERVAL_MS;
        }
        self.polling_interval_ms = self
            .polling_interval_ms
            .clamp(MIN_POLLING_INTERVAL_MS, MAX_POLLING_INTERVAL_MS);
        if self
            .window_size
            .is_some_and(|size| size.width <= 0 || size.height <= 0)
        {
            self.window_size = None;
        }
        self.deduplicate_targets();
    }

    fn sort_targets(&mut self) {
        self.targets.sort_by(compare_targets);
    }

    fn merge_duplicate_targets(&mut self) {
        self.targets.dedup_by(|target, previous| {
            if previous.name == target.name && previous.pid == target.pid {
                previous.enabled |= target.enabled;
                previous.managed_muted |= target.managed_muted;
                if previous.note.is_none() {
                    previous.note = target.note.take();
                }
                true
            } else {
                false
            }
        });
    }
}

pub(crate) fn merge_pending_config_changes(
    base: &AppConfig,
    local: &AppConfig,
    disk: &mut AppConfig,
) {
    if local.version != base.version {
        disk.version = local.version;
    }
    if local.language != base.language {
        disk.language = local.language;
    }
    if local.theme != base.theme {
        disk.theme = local.theme;
    }
    if local.window_position != base.window_position {
        disk.window_position = local.window_position;
    }
    if local.window_size != base.window_size {
        disk.window_size = local.window_size;
    }
    if local.polling_interval_ms != base.polling_interval_ms {
        disk.polling_interval_ms = local.polling_interval_ms;
    }
    if local.launch_on_startup != base.launch_on_startup {
        disk.launch_on_startup = local.launch_on_startup;
    }
    if local.start_minimized != base.start_minimized {
        disk.start_minimized = local.start_minimized;
    }
    if local.hide_to_tray_on_close != base.hide_to_tray_on_close {
        disk.hide_to_tray_on_close = local.hide_to_tray_on_close;
    }
    merge_pending_target_changes(&base.targets, &local.targets, &mut disk.targets);
}

fn merge_pending_target_changes(
    base: &[TargetProcess],
    local: &[TargetProcess],
    disk: &mut Vec<TargetProcess>,
) {
    for target in base {
        if target_index_by_key(local, &target.name, target.pid).is_none() {
            remove_target_by_key(disk, &target.name, target.pid);
        }
    }

    for target in local {
        let base_target =
            target_index_by_key(base, &target.name, target.pid).map(|index| &base[index]);
        match base_target {
            None => upsert_target(disk, target.clone()),
            Some(base_target) => merge_pending_target_fields(base_target, target, disk),
        }
    }
}

fn merge_pending_target_fields(
    base: &TargetProcess,
    local: &TargetProcess,
    disk: &mut [TargetProcess],
) {
    let Some(index) = target_index_by_key(disk, &local.name, local.pid) else {
        return;
    };
    let disk_target = &mut disk[index];
    if local.note != base.note {
        disk_target.note.clone_from(&local.note);
    }
    if local.enabled != base.enabled {
        disk_target.enabled = local.enabled;
    }
    if local.managed_muted != base.managed_muted {
        disk_target.managed_muted = local.managed_muted;
    }
}

fn target_index_by_key(targets: &[TargetProcess], name: &str, pid: Option<u32>) -> Option<usize> {
    targets
        .binary_search_by(|target| compare_target_key(target, name, pid))
        .ok()
}

fn remove_target_by_key(targets: &mut Vec<TargetProcess>, name: &str, pid: Option<u32>) -> bool {
    let Some(index) = target_index_by_key(targets, name, pid) else {
        return false;
    };
    targets.remove(index);
    true
}

fn upsert_target(targets: &mut Vec<TargetProcess>, target: TargetProcess) {
    match targets.binary_search_by(|existing| compare_targets(existing, &target)) {
        Ok(index) => targets[index] = target,
        Err(index) => targets.insert(index, target),
    }
}

fn compare_targets(left: &TargetProcess, right: &TargetProcess) -> Ordering {
    compare_target_key(left, &right.name, right.pid)
}

fn compare_target_key(left: &TargetProcess, name: &str, pid: Option<u32>) -> Ordering {
    compare_target_identity(left.name.as_str(), left.pid, name, pid)
}

pub(crate) fn compare_target_identity(
    left_name: &str,
    left_pid: Option<u32>,
    right_name: &str,
    right_pid: Option<u32>,
) -> Ordering {
    left_name
        .cmp(right_name)
        .then_with(|| left_pid.cmp(&right_pid))
}

pub(crate) fn target_index_by_identity(
    targets: &[TargetProcess],
    name: &str,
    pid: Option<u32>,
) -> Option<usize> {
    targets
        .binary_search_by(|target| compare_target_identity(&target.name, target.pid, name, pid))
        .ok()
}

fn load_or_default_from_path(path: &Path) -> io::Result<AppConfigLoad> {
    for _ in 0..3 {
        let expected_stamp = try_config_file_stamp(path)?;
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if expected_stamp.is_none() {
                    return Ok(AppConfigLoad::default());
                }
                continue;
            }
            Err(error) => return Err(error),
        };
        let bytes = read_config_file(file)?;
        match parse_config_bytes(&bytes) {
            Ok(mut config) => {
                if try_config_file_stamp(path)? != expected_stamp {
                    continue;
                }
                config.sanitize();
                return Ok(AppConfigLoad {
                    config,
                    first_run: false,
                    recovered_invalid_config: false,
                    source_stamp: ConfigSourceStamp::Known(expected_stamp),
                });
            }
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                if try_config_file_stamp(path)? != expected_stamp {
                    continue;
                }
                if let Some(recovered) =
                    recover_invalid_config_if_unchanged(path, expected_stamp, &bytes)?
                {
                    return Ok(recovered);
                }
            }
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        "config file kept changing while the app was loading",
    ))
}

#[cfg(test)]
fn parse_config_file(file: fs::File) -> io::Result<AppConfig> {
    let bytes = read_config_file(file)?;
    parse_config_bytes(&bytes)
}

fn load_existing_from_path(path: &Path) -> io::Result<ExistingConfigLoad> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ExistingConfigLoad::Missing(error));
        }
        Err(error) => return Err(error),
    };
    let bytes = read_config_file(file)?;
    match parse_config_bytes(&bytes) {
        Ok(mut config) => {
            config.sanitize();
            Ok(ExistingConfigLoad::Loaded(config))
        }
        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
            Ok(ExistingConfigLoad::Malformed(error))
        }
        Err(error) => Err(error),
    }
}

fn read_config_file(file: fs::File) -> io::Result<Vec<u8>> {
    reject_oversized_config_file(&file)?;
    let mut bytes = Vec::with_capacity(file.metadata()?.len() as usize);
    Read::take(file, MAX_CONFIG_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CONFIG_FILE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "config file is too large",
        ));
    }
    Ok(bytes)
}

fn parse_config_bytes(bytes: &[u8]) -> io::Result<AppConfig> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(json_error_to_io)?;
    if let Some(version) = value.get("version").and_then(serde_json::Value::as_u64)
        && version > u64::from(CONFIG_VERSION)
    {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "config version {} is newer than supported version {CONFIG_VERSION}",
                version
            ),
        ));
    }
    serde_json::from_value(value).map_err(json_error_to_io)
}

fn json_error_to_io(error: serde_json::Error) -> io::Error {
    io::Error::new(
        error.io_error_kind().unwrap_or(io::ErrorKind::InvalidData),
        error,
    )
}

fn recover_invalid_config_if_unchanged(
    path: &Path,
    expected_stamp: Option<ConfigFileStamp>,
    invalid_bytes: &[u8],
) -> io::Result<Option<AppConfigLoad>> {
    if try_config_file_stamp(path)? != expected_stamp {
        return Ok(None);
    }

    let backup_path = backup_invalid_config_bytes(path, invalid_bytes)?;
    let config = AppConfig::default();
    let save_result = config.save_to_path_guarded(
        path,
        false,
        ExistingConfigGuard::Unchanged(expected_stamp),
        ConfigDurability::Durable,
    );
    match save_result {
        Ok(true) => Ok(Some(AppConfigLoad {
            config,
            first_run: false,
            recovered_invalid_config: true,
            source_stamp: ConfigSourceStamp::Unknown,
        })),
        Ok(false) => {
            let _ = fs::remove_file(backup_path);
            Ok(None)
        }
        Err(error) => {
            let _ = fs::remove_file(backup_path);
            Err(error)
        }
    }
}

fn existing_config_matches_guard(path: &Path, guard: ExistingConfigGuard) -> io::Result<bool> {
    match guard {
        #[cfg(test)]
        ExistingConfigGuard::Any => Ok(true),
        ExistingConfigGuard::Unchanged(expected) => Ok(try_config_file_stamp(path)? == expected),
    }
}

fn serialized_config_bytes(config: &AppConfig) -> io::Result<Vec<u8>> {
    let bytes = serde_json::to_vec_pretty(config).map_err(io::Error::other)?;
    if bytes.len() as u64 <= MAX_CONFIG_FILE_BYTES {
        return Ok(bytes);
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "config file would exceed maximum size",
    ))
}

fn reject_oversized_config_file(file: &fs::File) -> io::Result<()> {
    let len = file.metadata()?.len();
    if len <= MAX_CONFIG_FILE_BYTES {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "config file is too large",
    ))
}

fn validate_existing_config(path: &Path) -> io::Result<ExistingConfigValidation> {
    for _ in 0..3 {
        let stamp_before_load = try_config_file_stamp(path)?;
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if stamp_before_load.is_none() {
                    return Ok(ExistingConfigValidation {
                        stamp: None,
                        invalid_backup: None,
                    });
                }
                continue;
            }
            Err(error) => return Err(error),
        };
        let bytes = read_config_file(file)?;
        let parse_result = parse_config_bytes(&bytes);
        if try_config_file_stamp(path)? != stamp_before_load {
            continue;
        }

        match parse_result {
            Ok(_) => {
                return Ok(ExistingConfigValidation {
                    stamp: stamp_before_load,
                    invalid_backup: None,
                });
            }
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                let backup_path = backup_invalid_config_bytes(path, &bytes)?;
                if try_config_file_stamp(path)? == stamp_before_load {
                    return Ok(ExistingConfigValidation {
                        stamp: stamp_before_load,
                        invalid_backup: Some(backup_path),
                    });
                }
                let _ = fs::remove_file(backup_path);
            }
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        "config file kept changing while the app was validating it",
    ))
}

fn discard_invalid_backup(validation: Option<&ExistingConfigValidation>) {
    if let Some(backup_path) = validation.and_then(|validation| validation.invalid_backup.as_ref())
    {
        let _ = fs::remove_file(backup_path);
    }
}

fn replace_file(
    temp_path: &Path,
    destination: &Path,
    durability: ConfigDurability,
) -> io::Result<()> {
    #[cfg(windows)]
    let result = replace_file_windows(temp_path, destination, durability);

    #[cfg(not(windows))]
    let result = {
        let _ = durability;
        fs::rename(temp_path, destination)
    };

    if result.is_err() {
        let _ = fs::remove_file(temp_path);
    }
    result
}

fn discard_open_file(file: fs::File, path: &Path) {
    drop(file);
    let _ = fs::remove_file(path);
}

fn create_temp_config_file(destination: &Path) -> io::Result<(fs::File, PathBuf)> {
    let temp_path = destination.with_extension("json.tmp");
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
    {
        Ok(file) => return Ok((file, temp_path)),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }

    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config.json");
    let pid = std::process::id();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());

    for index in 0..100 {
        let temp_path = parent.join(format!("{file_name}.{pid}-{timestamp}-{index}.tmp"));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((file, temp_path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique config temp path",
    ))
}

#[cfg(windows)]
fn replace_file_windows(
    temp_path: &Path,
    destination: &Path,
    durability: ConfigDurability,
) -> io::Result<()> {
    let temp_path = path_to_wide(temp_path);
    let destination = path_to_wide(destination);
    let flags = replace_file_flags(durability);

    unsafe {
        MoveFileExW(
            PCWSTR(temp_path.as_ptr()),
            PCWSTR(destination.as_ptr()),
            flags,
        )
    }
    .map_err(|error| io::Error::other(format!("replace config file: {error}")))
}

#[cfg(windows)]
fn replace_file_flags(durability: ConfigDurability) -> MOVE_FILE_FLAGS {
    match durability {
        ConfigDurability::Durable => MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        ConfigDurability::ProcessCrashSafe => MOVEFILE_REPLACE_EXISTING,
    }
}

#[cfg(windows)]
fn path_to_wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn backup_invalid_config_bytes(path: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    for index in 0..100 {
        let backup_path = invalid_config_backup_path(parent, timestamp, index);
        let mut backup = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup_path)
        {
            Ok(backup) => backup,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        if let Err(error) = backup.write_all(bytes).and_then(|()| backup.sync_all()) {
            discard_open_file(backup, &backup_path);
            return Err(error);
        }
        return Ok(backup_path);
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique invalid config backup path",
    ))
}

fn invalid_config_backup_path(parent: &Path, timestamp: u64, index: usize) -> PathBuf {
    let file_name = if index == 0 {
        format!("config.invalid-{timestamp}.json")
    } else {
        format!("config.invalid-{timestamp}-{index}.json")
    };
    parent.join(file_name)
}

struct ProcessNameCandidate<'a> {
    name: &'a str,
    has_uppercase: bool,
}

fn process_name_candidate(input: &str) -> Option<ProcessNameCandidate<'_>> {
    if input.as_bytes().contains(&0) {
        return None;
    }

    let name = input
        .trim_matches('"')
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches('"')
        .trim();

    validated_process_name_candidate(name)
}

fn direct_process_name_candidate(input: &str) -> Option<ProcessNameCandidate<'_>> {
    if input.as_bytes().contains(&0) {
        return None;
    }

    let name = input.trim().trim_matches('"').trim();
    if name.is_empty() || name.bytes().any(|byte| matches!(byte, b'\\' | b'/')) {
        return None;
    }

    validated_process_name_candidate(name)
}

fn validated_process_name_candidate(name: &str) -> Option<ProcessNameCandidate<'_>> {
    if name.is_empty() {
        return None;
    }

    let mut has_uppercase = false;
    for byte in name.bytes() {
        if is_invalid_process_file_name_byte(byte) {
            return None;
        }
        has_uppercase |= byte.is_ascii_uppercase();
    }

    Some(ProcessNameCandidate {
        name,
        has_uppercase,
    })
}

#[cfg(test)]
pub fn normalize_process_name(input: &str) -> Option<String> {
    Some(normalize_process_name_cow(input)?.into_owned())
}

#[cfg(test)]
pub fn normalize_process_name_cow(input: &str) -> Option<Cow<'_, str>> {
    let candidate = process_name_candidate(input)?;

    normalized_process_name_cow(candidate)
}

pub(crate) fn normalize_manual_process_name(input: &str) -> Option<String> {
    Some(normalize_manual_process_name_cow(input)?.into_owned())
}

pub(crate) fn normalize_manual_process_name_cow(input: &str) -> Option<Cow<'_, str>> {
    let candidate = direct_process_name_candidate(input)?;

    normalized_process_name_cow(candidate)
}

fn normalized_process_name_cow(candidate: ProcessNameCandidate<'_>) -> Option<Cow<'_, str>> {
    if candidate.name.is_ascii() {
        return if candidate.has_uppercase {
            Some(Cow::Owned(candidate.name.to_ascii_lowercase()))
        } else {
            Some(Cow::Borrowed(candidate.name))
        };
    }

    let lowercase = candidate.name.to_lowercase();
    if lowercase == candidate.name {
        Some(Cow::Borrowed(candidate.name))
    } else {
        Some(Cow::Owned(lowercase))
    }
}

pub fn normalize_process_name_owned(mut input: String) -> Option<String> {
    let candidate = process_name_candidate(&input)?;

    if candidate.name.len() == input.len() && candidate.name.as_ptr() == input.as_ptr() {
        if input.is_ascii() {
            if candidate.has_uppercase {
                input.make_ascii_lowercase();
            }
            return Some(input);
        }

        let lowercase = input.to_lowercase();
        if lowercase == input {
            return Some(input);
        }
        return Some(lowercase);
    }

    if candidate.name.is_ascii() {
        if candidate.has_uppercase {
            Some(candidate.name.to_ascii_lowercase())
        } else {
            Some(candidate.name.to_owned())
        }
    } else {
        let lowercase = candidate.name.to_lowercase();
        if lowercase == candidate.name {
            Some(candidate.name.to_owned())
        } else {
            Some(lowercase)
        }
    }
}

fn is_normalized_process_name_candidate(input: &str, candidate: ProcessNameCandidate<'_>) -> bool {
    if candidate.name.len() != input.len() || candidate.name.as_ptr() != input.as_ptr() {
        return false;
    }

    if input.is_ascii() {
        !candidate.has_uppercase
    } else {
        input.to_lowercase() == input
    }
}

fn normalized_utf16_process_name(candidate: Utf16ProcessNameCandidate<'_>) -> String {
    if candidate.is_ascii {
        return ascii_utf16_process_name(candidate.name, candidate.has_uppercase);
    }

    let name = String::from_utf16_lossy(candidate.name);
    let lowercase = name.to_lowercase();
    if lowercase == name { name } else { lowercase }
}

pub fn is_normalized_process_name(input: &str) -> bool {
    process_name_candidate(input)
        .is_some_and(|candidate| is_normalized_process_name_candidate(input, candidate))
}

pub(crate) fn is_supported_target_process_name(input: &str) -> bool {
    is_normalized_process_name(input) && is_supported_normalized_target_process_name(input)
}

pub(crate) fn is_supported_normalized_target_process_name(input: &str) -> bool {
    input
        .strip_suffix(".exe")
        .is_some_and(|name| !name.is_empty() && !windows_reserved_device_name(name))
}

fn windows_reserved_device_name(name: &str) -> bool {
    let device_name = name.split('.').next().unwrap_or(name);
    match device_name.len() {
        3 => matches!(device_name, "con" | "prn" | "aux" | "nul"),
        4 => {
            let bytes = device_name.as_bytes();
            matches!(&bytes[..3], b"com" | b"lpt") && matches!(bytes[3], b'1'..=b'9')
        }
        6 => device_name == "conin$",
        7 => device_name == "conout$",
        _ => false,
    }
}

pub(crate) fn normalize_target_note(input: &str) -> Option<String> {
    let mut output = String::with_capacity(input.len().min(MAX_TARGET_NOTE_CHARS * 4));
    let mut previous_was_space = false;
    let mut chars = 0;

    for ch in input.trim().chars() {
        let is_space = ch.is_whitespace() || ch.is_control();
        if is_space {
            if !output.is_empty() && !previous_was_space {
                output.push(' ');
                previous_was_space = true;
                chars += 1;
            }
        } else {
            output.push(ch);
            previous_was_space = false;
            chars += 1;
        }

        if chars >= MAX_TARGET_NOTE_CHARS {
            break;
        }
    }

    while output.ends_with(' ') {
        output.pop();
    }

    (!output.is_empty()).then_some(output)
}

#[cfg(test)]
pub(crate) fn normalize_process_name_utf16(input: &[u16]) -> Option<String> {
    let candidate = utf16_process_name_candidate(input)?;
    Some(normalized_utf16_process_name(candidate))
}

pub(crate) fn normalize_supported_process_name_utf16(input: &[u16]) -> Option<String> {
    let candidate = utf16_process_name_candidate(input)?;
    if !is_supported_utf16_target_process_name(candidate.name) {
        return None;
    }
    Some(normalized_utf16_process_name(candidate))
}

fn is_supported_utf16_target_process_name(input: &[u16]) -> bool {
    strip_ascii_exe_suffix_utf16(input)
        .is_some_and(|name| !name.is_empty() && !windows_reserved_device_name_utf16(name))
}

fn strip_ascii_exe_suffix_utf16(input: &[u16]) -> Option<&[u16]> {
    let stem_len = input.len().checked_sub(4)?;
    let (stem, suffix) = input.split_at(stem_len);
    (suffix[0] == u16::from(b'.')
        && ascii_u16_eq_ignore_case(suffix[1], b'e')
        && ascii_u16_eq_ignore_case(suffix[2], b'x')
        && ascii_u16_eq_ignore_case(suffix[3], b'e'))
    .then_some(stem)
}

fn windows_reserved_device_name_utf16(name: &[u16]) -> bool {
    let device_name = name
        .split(|ch| *ch == u16::from(b'.'))
        .next()
        .unwrap_or(name);

    match device_name.len() {
        3 => {
            ascii_utf16_eq_ignore_case(device_name, b"con")
                || ascii_utf16_eq_ignore_case(device_name, b"prn")
                || ascii_utf16_eq_ignore_case(device_name, b"aux")
                || ascii_utf16_eq_ignore_case(device_name, b"nul")
        }
        4 => {
            (ascii_utf16_eq_ignore_case(&device_name[..3], b"com")
                || ascii_utf16_eq_ignore_case(&device_name[..3], b"lpt"))
                && matches!(device_name[3], 0x31..=0x39)
        }
        6 => ascii_utf16_eq_ignore_case(device_name, b"conin$"),
        7 => ascii_utf16_eq_ignore_case(device_name, b"conout$"),
        _ => false,
    }
}

fn ascii_utf16_eq_ignore_case(input: &[u16], ascii: &[u8]) -> bool {
    input.len() == ascii.len()
        && input
            .iter()
            .zip(ascii)
            .all(|(ch, byte)| ascii_u16_eq_ignore_case(*ch, *byte))
}

fn ascii_u16_eq_ignore_case(ch: u16, ascii: u8) -> bool {
    ch <= 0x7f && (ch as u8).eq_ignore_ascii_case(&ascii)
}

#[derive(Clone, Copy)]
struct Utf16ProcessNameCandidate<'a> {
    name: &'a [u16],
    has_uppercase: bool,
    is_ascii: bool,
}

fn utf16_process_name_candidate(input: &[u16]) -> Option<Utf16ProcessNameCandidate<'_>> {
    let nul = input.iter().position(|ch| *ch == 0).unwrap_or(input.len());
    let mut name = trim_ascii_utf16(&input[..nul]);
    name = trim_ascii_quote_utf16(name);
    if let Some(index) = name
        .iter()
        .rposition(|ch| *ch == u16::from(b'\\') || *ch == u16::from(b'/'))
    {
        name = &name[index + 1..];
    }
    name = trim_ascii_utf16(name);
    name = trim_ascii_quote_utf16(name);
    name = trim_ascii_utf16(name);

    if name.is_empty() {
        return None;
    }

    let mut has_uppercase = false;
    let mut is_ascii = true;
    for ch in name {
        if is_invalid_process_file_name_u16(*ch) {
            return None;
        } else if *ch > 0x7f {
            is_ascii = false;
        } else {
            has_uppercase |= (*ch as u8).is_ascii_uppercase();
        }
    }
    Some(Utf16ProcessNameCandidate {
        name,
        has_uppercase,
        is_ascii,
    })
}

fn ascii_utf16_process_name(input: &[u16], has_uppercase: bool) -> String {
    let mut output = String::with_capacity(input.len());
    if has_uppercase {
        for ch in input {
            output.push((*ch as u8).to_ascii_lowercase() as char);
        }
    } else {
        for ch in input {
            output.push(*ch as u8 as char);
        }
    }
    output
}

fn is_invalid_process_file_name_byte(byte: u8) -> bool {
    byte < 0x20 || matches!(byte, b'<' | b'>' | b':' | b'"' | b'|' | b'?' | b'*')
}

fn is_invalid_process_file_name_u16(ch: u16) -> bool {
    ch < 0x20 || matches!(ch, 0x3c | 0x3e | 0x3a | 0x22 | 0x7c | 0x3f | 0x2a)
}

fn trim_ascii_utf16(mut input: &[u16]) -> &[u16] {
    while let Some((first, rest)) = input.split_first()
        && is_ascii_whitespace_u16(*first)
    {
        input = rest;
    }
    while let Some((last, rest)) = input.split_last()
        && is_ascii_whitespace_u16(*last)
    {
        input = rest;
    }
    input
}

fn trim_ascii_quote_utf16(mut input: &[u16]) -> &[u16] {
    while let Some((first, rest)) = input.split_first()
        && *first == u16::from(b'"')
    {
        input = rest;
    }
    while let Some((last, rest)) = input.split_last()
        && *last == u16::from(b'"')
    {
        input = rest;
    }
    input
}

fn is_ascii_whitespace_u16(ch: u16) -> bool {
    matches!(ch, 0x09..=0x0d | 0x20)
}

fn push_decimal_u32(output: &mut String, mut number: u32) {
    let mut digits = [0u8; 10];
    let mut len = 0;
    loop {
        digits[len] = b'0' + (number % 10) as u8;
        len += 1;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    for digit in digits[..len].iter().rev() {
        output.push(*digit as char);
    }
}

pub fn config_dir() -> io::Result<PathBuf> {
    config_dir_from_env(
        cfg!(windows),
        env::var_os("APPDATA"),
        env::var_os("XDG_CONFIG_HOME"),
        env::var_os("HOME"),
        env::var_os("USERPROFILE"),
    )
}

fn config_dir_from_env(
    is_windows: bool,
    appdata: Option<OsString>,
    xdg_config_home: Option<OsString>,
    home: Option<OsString>,
    userprofile: Option<OsString>,
) -> io::Result<PathBuf> {
    if is_windows && let Some(appdata) = non_empty_os_string(appdata) {
        return Ok(PathBuf::from(appdata).join("UnfocusMute"));
    }

    if let Some(xdg) = non_empty_os_string(xdg_config_home) {
        return Ok(PathBuf::from(xdg).join("unfocusmute"));
    }

    let home = non_empty_os_string(home)
        .or_else(|| non_empty_os_string(userprofile))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory not found"))?;
    Ok(PathBuf::from(home).join(".config").join("unfocusmute"))
}

fn non_empty_os_string(value: Option<OsString>) -> Option<OsString> {
    value.filter(|value| !value.as_os_str().is_empty())
}

pub(crate) fn cached_config_file_path() -> io::Result<&'static Path> {
    static CONFIG_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();

    if let Some(path) = CONFIG_FILE_PATH.get() {
        return Ok(path);
    }

    let path = config_dir()?.join("config.json");
    let _ = CONFIG_FILE_PATH.set(path);
    CONFIG_FILE_PATH
        .get()
        .map(PathBuf::as_path)
        .ok_or_else(|| io::Error::other("config path cache unavailable"))
}

pub(crate) fn current_config_stamp() -> Option<ConfigFileStamp> {
    let path = cached_config_file_path().ok()?;

    #[cfg(windows)]
    {
        thread_local! {
            static CONFIG_STAMP_CACHE: RefCell<ConfigStampCache> =
                RefCell::new(ConfigStampCache::default());
        }
        CONFIG_STAMP_CACHE.with_borrow_mut(|cache| cache.current(path))
    }

    #[cfg(not(windows))]
    config_file_stamp(path)
}

#[cfg(any(not(windows), test))]
fn config_file_stamp(path: &Path) -> Option<ConfigFileStamp> {
    try_config_file_stamp(path).ok().flatten()
}

fn try_config_file_stamp(path: &Path) -> io::Result<Option<ConfigFileStamp>> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let len = metadata.len();
    let fingerprint = if len <= MAX_CONFIG_FILE_BYTES {
        Some(config_file_fingerprint(path)?)
    } else {
        None
    };
    Ok(Some(ConfigFileStamp {
        modified: metadata.modified()?,
        len,
        fingerprint,
    }))
}

fn config_file_fingerprint(path: &Path) -> io::Result<u64> {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut file = fs::File::open(path)?;
    let mut buffer = [0u8; 8192];
    let mut hash = FNV_OFFSET_BASIS;

    loop {
        let len = file.read(&mut buffer)?;
        if len == 0 {
            return Ok(hash);
        }
        for byte in &buffer[..len] {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let base = env::temp_dir();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos());

            for index in 0..100 {
                let path = base.join(format!(
                    "unfocusmute-test-{}-{timestamp}-{index}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self { path },
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create test directory: {error}"),
                }
            }

            panic!("could not create a unique test directory");
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn os_string(value: &str) -> OsString {
        OsString::from(value)
    }

    fn invalid_backup_count(path: &Path) -> usize {
        fs::read_dir(path)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.invalid-")
            })
            .count()
    }

    #[test]
    fn config_dir_ignores_empty_xdg_config_home() {
        assert_eq!(
            config_dir_from_env(
                false,
                None,
                Some(os_string("")),
                Some(os_string("/home/user")),
                None
            )
            .unwrap(),
            PathBuf::from("/home/user")
                .join(".config")
                .join("unfocusmute")
        );
    }

    #[test]
    fn config_dir_ignores_empty_windows_appdata() {
        assert_eq!(
            config_dir_from_env(
                true,
                Some(os_string("")),
                Some(os_string("/xdg/config")),
                Some(os_string("/home/user")),
                None,
            )
            .unwrap(),
            PathBuf::from("/xdg/config").join("unfocusmute")
        );
    }

    #[test]
    fn config_dir_falls_back_from_empty_home_to_userprofile() {
        assert_eq!(
            config_dir_from_env(
                false,
                None,
                None,
                Some(os_string("")),
                Some(os_string("/users/example")),
            )
            .unwrap(),
            PathBuf::from("/users/example")
                .join(".config")
                .join("unfocusmute")
        );
    }

    #[test]
    fn config_dir_rejects_empty_home_values() {
        assert_eq!(
            config_dir_from_env(false, None, None, Some(os_string("")), Some(os_string("")))
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotFound
        );
    }

    #[test]
    fn normalizes_paths_and_case() {
        assert_eq!(
            normalize_process_name(r"C:\Games\Example.EXE"),
            Some("example.exe".to_owned())
        );
        assert_eq!(
            normalize_process_name("  game.exe  "),
            Some("game.exe".to_owned())
        );
        assert_eq!(normalize_process_name(""), None);
    }

    #[test]
    fn normalizes_quoted_names_after_path_split() {
        assert_eq!(
            normalize_process_name(r#"C:\Games\"Example.EXE""#),
            Some("example.exe".to_owned())
        );
        assert_eq!(
            normalize_process_name(r#"C:\Games\ "Example.EXE" "#),
            Some("example.exe".to_owned())
        );
    }

    #[test]
    fn rejects_invalid_windows_file_name_chars_inside_process_file_name() {
        assert_eq!(normalize_process_name(r#"bad"game.exe"#), None);
        assert_eq!(normalize_process_name(r#"C:\Games\bad"game.exe"#), None);
        assert_eq!(normalize_process_name("bad:game.exe"), None);
        assert_eq!(normalize_process_name("bad*game.exe"), None);
        assert_eq!(normalize_process_name("bad|game.exe"), None);
        assert_eq!(normalize_process_name("bad\tgame.exe"), None);
        assert_eq!(
            normalize_process_name_utf16(&wide_null_terminated(r#"bad"game.exe"#)),
            None
        );
        assert_eq!(
            normalize_process_name_utf16(&wide_null_terminated("bad:game.exe")),
            None
        );
    }

    #[test]
    fn normalizes_utf16_paths_and_quoted_names() {
        assert_eq!(
            normalize_process_name_utf16(&wide_null_terminated(r"C:\Games\Example.EXE")),
            Some("example.exe".to_owned())
        );
        assert_eq!(
            normalize_process_name_utf16(&wide_null_terminated("  \"Mixer.EXE\"  ")),
            Some("mixer.exe".to_owned())
        );
        assert_eq!(
            normalize_process_name_utf16(&wide_null_terminated(r#"C:\Games\"Example.EXE""#)),
            Some("example.exe".to_owned())
        );
        assert_eq!(
            normalize_process_name_utf16(&wide_null_terminated("")),
            None
        );
    }

    #[test]
    fn normalizes_unicode_case_consistently() {
        assert_eq!(
            normalize_process_name("ÄPP.EXE"),
            Some("äpp.exe".to_owned())
        );
        assert_eq!(
            normalize_process_name_utf16(&wide_null_terminated("ÄPP.EXE")),
            Some("äpp.exe".to_owned())
        );
        assert!(is_normalized_process_name("äpp.exe"));
        assert!(!is_normalized_process_name("Äpp.exe"));
    }

    #[test]
    fn manual_process_names_accept_only_direct_supported_exe_names() {
        let plain = normalize_manual_process_name("  Game.EXE  ").unwrap();
        let quoted = normalize_manual_process_name(r#""Game.EXE""#).unwrap();
        let command_line = normalize_manual_process_name("game.exe --fullscreen").unwrap();

        assert_eq!(plain, "game.exe");
        assert_eq!(quoted, "game.exe");
        assert!(is_supported_normalized_target_process_name(&plain));
        assert!(is_supported_normalized_target_process_name(&quoted));
        assert!(!is_supported_normalized_target_process_name(&command_line));
        assert_eq!(normalize_manual_process_name(r"C:\Games\Game.EXE"), None);
        assert_eq!(
            normalize_manual_process_name(r#""C:\Games\Game.EXE" --fullscreen"#),
            None
        );
    }

    #[test]
    fn accepts_only_supported_utf16_process_names() {
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated(r"C:\Games\Example.EXE")),
            Some("example.exe".to_owned())
        );
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated("System")),
            None
        );
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated(".EXE")),
            None
        );
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated("NUL.EXE")),
            None
        );
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated("CON.any.EXE")),
            None
        );
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated("CONOUT$.EXE")),
            None
        );
    }

    #[test]
    fn utf16_normalization_preserves_non_ascii_names() {
        let name = [
            0xac8c,
            0xc784,
            u16::from(b'.'),
            u16::from(b'E'),
            u16::from(b'X'),
            u16::from(b'E'),
            0,
        ];

        assert_eq!(
            normalize_process_name_utf16(&name),
            Some("\u{ac8c}\u{c784}.exe".to_owned())
        );
    }

    #[test]
    fn owned_normalization_reuses_already_normalized_name() {
        let name = "game.exe".to_owned();
        let ptr = name.as_ptr();
        let normalized = normalize_process_name_owned(name).unwrap();

        assert_eq!(normalized, "game.exe");
        assert_eq!(normalized.as_ptr(), ptr);
    }

    #[test]
    fn borrowed_normalization_reuses_already_normalized_name() {
        let name = "game.exe";
        let normalized = normalize_process_name_cow(name).unwrap();

        assert!(matches!(normalized, Cow::Borrowed("game.exe")));
    }

    #[test]
    fn owned_normalization_matches_borrowed_normalization() {
        let inputs = [
            r"C:\Games\Example.EXE",
            "  game.exe  ",
            "\"MIXER.EXE\"",
            "C:/Tools/player.exe",
        ];

        for input in inputs {
            assert_eq!(
                normalize_process_name_owned(input.to_owned()),
                normalize_process_name(input)
            );
        }
    }

    #[test]
    fn detects_already_normalized_process_names() {
        assert!(is_normalized_process_name("game.exe"));
        assert!(!is_normalized_process_name("Game.EXE"));
        assert!(!is_normalized_process_name(r"C:\Games\game.exe"));
        assert!(!is_normalized_process_name("game.exe\0"));
    }

    #[test]
    fn supported_target_names_must_be_normalized_exe_files() {
        assert!(is_supported_target_process_name("game.exe"));
        assert!(!is_supported_target_process_name("Game.EXE"));
        assert!(!is_supported_target_process_name(".exe"));
        assert!(!is_supported_target_process_name("system"));
        assert!(!is_supported_target_process_name("game.exe."));
    }

    #[test]
    fn supported_target_names_reject_windows_reserved_devices() {
        assert!(!is_supported_target_process_name("con.exe"));
        assert!(!is_supported_target_process_name("nul.exe"));
        assert!(!is_supported_target_process_name("com1.exe"));
        assert!(!is_supported_target_process_name("lpt9.exe"));
        assert!(!is_supported_target_process_name("con.any.exe"));
        assert!(!is_supported_target_process_name("conin$.exe"));
        assert!(!is_supported_target_process_name("conout$.any.exe"));
        assert!(is_supported_target_process_name("com0.exe"));
        assert!(is_supported_target_process_name("console.exe"));
    }

    #[test]
    fn rejects_nul_anywhere_in_process_name_input() {
        assert_eq!(normalize_process_name("bad\0/path/game.exe"), None);
        assert_eq!(normalize_process_name("bad\0\\game.exe"), None);
    }

    #[test]
    fn config_file_stamp_fingerprints_same_length_content_changes() {
        let dir = TestDir::new();
        let path = dir.path().join("config.json");

        fs::write(&path, "alpha").unwrap();
        let first = config_file_stamp(&path).unwrap();
        fs::write(&path, "bravo").unwrap();
        let second = config_file_stamp(&path).unwrap();

        assert_eq!(first.len, second.len);
        assert!(first.has_content_fingerprint());
        assert!(second.has_content_fingerprint());
        assert_ne!(first.fingerprint, second.fingerprint);
    }

    #[cfg(windows)]
    #[test]
    fn config_stamp_cache_refreshes_after_directory_change_notification() {
        use std::os::windows::fs::OpenOptionsExt;

        let dir = TestDir::new();
        let path = dir.path().join("config.json");
        fs::write(&path, "alpha").unwrap();
        let mut cache = ConfigStampCache::default();
        let first = cache.current(&path).unwrap();

        let exclusive_file = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        assert_eq!(cache.current(&path), Some(first));
        drop(exclusive_file);
        fs::write(&path, "bravo").unwrap();

        let second = (0..50).find_map(|_| {
            let stamp = cache.current(&path);
            if stamp != Some(first) {
                stamp
            } else {
                std::thread::sleep(std::time::Duration::from_millis(10));
                None
            }
        });

        assert!(second.is_some());
        assert_ne!(first.fingerprint, second.unwrap().fingerprint);
    }

    #[test]
    fn oversized_config_files_are_preserved_without_json_parsing() {
        let dir = TestDir::new();
        let path = dir.path().join("config.json");
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_CONFIG_FILE_BYTES + 1).unwrap();
        drop(file);

        let error = parse_config_file(fs::File::open(&path).unwrap()).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn oversized_serialized_config_is_rejected_before_save() {
        let dir = TestDir::new();
        let path = dir.path().join("config.json");
        let config = AppConfig {
            targets: (0..2_000)
                .map(|index| TargetProcess {
                    name: format!("app{index}.exe"),
                    pid: None,
                    note: Some("x".repeat(MAX_TARGET_NOTE_CHARS)),
                    enabled: true,
                    managed_muted: false,
                })
                .collect(),
            ..AppConfig::default()
        };

        let error = config.save_to_path(&path, false).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(!path.exists());
    }

    #[test]
    fn oversized_save_does_not_move_existing_config_before_size_check() {
        let dir = TestDir::new();
        let path = dir.path().join("config.json");
        let existing_len = MAX_CONFIG_FILE_BYTES + 1;
        fs::write(&path, vec![b'x'; existing_len as usize]).unwrap();
        let config = AppConfig {
            targets: (0..2_000)
                .map(|index| TargetProcess {
                    name: format!("app{index}.exe"),
                    pid: None,
                    note: Some("x".repeat(MAX_TARGET_NOTE_CHARS)),
                    enabled: true,
                    managed_muted: false,
                })
                .collect(),
            ..AppConfig::default()
        };

        let error = config.save_to_path(&path, true).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::metadata(&path).unwrap().len(), existing_len);
        assert_eq!(invalid_backup_count(dir.path()), 0);
    }

    #[test]
    fn oversized_config_files_skip_content_fingerprints() {
        let dir = TestDir::new();
        let path = dir.path().join("config.json");
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_CONFIG_FILE_BYTES + 1).unwrap();
        drop(file);

        let stamp = config_file_stamp(&path).unwrap();

        assert_eq!(stamp.len, MAX_CONFIG_FILE_BYTES + 1);
        assert!(!stamp.has_content_fingerprint());
    }

    #[test]
    fn config_reload_retries_after_previous_load_failure_even_when_stamp_matches() {
        let dir = TestDir::new();
        let path = dir.path().join("config.json");
        fs::write(&path, "{}").unwrap();
        let stamp = config_file_stamp(&path);

        assert!(!config_reload_needed(stamp, stamp, false));
        assert!(config_reload_needed(stamp, stamp, true));
        assert!(config_reload_needed(None, stamp, false));
    }

    fn wide_null_terminated(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[test]
    fn add_and_remove_targets_are_case_insensitive() {
        let mut config = AppConfig::default();

        assert!(config.add_target("Game.EXE"));
        assert!(!config.add_target("game.exe"));
        assert_eq!(config.targets.len(), 1);
        assert!(config.remove_target_at(0));
        assert!(config.targets.is_empty());
    }

    #[test]
    fn single_target_lookup_and_insert_keep_sorted_order() {
        let mut config = AppConfig::default();

        assert!(config.add_target("game.exe"));
        assert!(config.contains_normalized_target("game.exe", None));
        assert!(!config.contains_normalized_target("chat.exe", None));
        assert!(config.add_target("chat.exe"));

        assert_eq!(config.targets[0].name, "chat.exe");
        assert_eq!(config.targets[1].name, "game.exe");
    }

    #[test]
    fn pid_zero_targets_are_rejected() {
        let mut config = AppConfig::default();

        assert!(!config.add_pid_target("game.exe", 0));
        assert!(config.targets.is_empty());
    }

    #[test]
    fn non_exe_targets_are_rejected() {
        let mut config = AppConfig::default();

        assert!(!config.add_target("system"));
        assert!(!config.add_pid_target("service", 42));
        assert!(config.targets.is_empty());
    }

    #[test]
    fn exe_and_pid_targets_can_coexist() {
        let mut config = AppConfig::default();

        assert!(config.add_target("browser.exe"));
        assert!(config.add_pid_target("browser.exe", 42));
        assert!(!config.add_pid_target("browser.exe", 42));
        assert!(config.contains_normalized_target("browser.exe", None));
        assert!(config.contains_normalized_target("browser.exe", Some(42)));
        assert!(!config.contains_normalized_target("browser.exe", Some(43)));
        assert_eq!(config.targets.len(), 2);
        let mut display_name = String::new();
        config.targets[0].display_name_into(&mut display_name);
        assert_eq!(display_name, "browser.exe");
        config.targets[1].display_name_into(&mut display_name);
        assert_eq!(display_name, "browser.exe (PID 42)");
        assert!(config.remove_target_at(1));
        assert_eq!(config.targets.len(), 1);
    }

    #[test]
    fn target_note_is_shown_after_identity() {
        let mut target = TargetProcess::for_pid("browser.exe", 42).unwrap();
        target.note = Some("main window".to_owned());
        let mut display_name = String::new();

        target.display_name_into(&mut display_name);

        assert_eq!(display_name, "browser.exe (PID 42) - main window");
    }

    #[test]
    fn target_notes_are_trimmed_collapsed_and_limited() {
        let mut config = AppConfig::default();
        assert!(config.add_target("browser.exe"));
        let long_note = format!(
            "  main\twindow\n{}  ",
            "x".repeat(MAX_TARGET_NOTE_CHARS + 10)
        );

        assert!(config.set_target_note_at(0, Some(long_note)));
        let note = config.targets[0].note.as_deref().unwrap();

        assert!(!note.contains('\n'));
        assert!(!note.contains('\t'));
        assert!(note.chars().count() <= MAX_TARGET_NOTE_CHARS);
        assert!(note.starts_with("main window "));
    }

    #[test]
    fn empty_target_note_is_not_stored() {
        let mut config = AppConfig::default();
        assert!(config.add_target("browser.exe"));

        assert!(!config.set_target_note_at(0, Some("  ".to_owned())));
        assert_eq!(config.targets[0].note, None);
        assert!(config.set_target_note_at(0, Some("game".to_owned())));
        assert!(config.set_target_note_at(0, Some("  ".to_owned())));
        assert_eq!(config.targets[0].note, None);
    }

    #[test]
    fn target_enabled_state_changes_only_when_needed() {
        let mut config = AppConfig::default();
        assert!(config.add_target("browser.exe"));

        assert!(!config.set_target_enabled_at(0, true));
        assert!(config.targets[0].enabled);
        assert!(config.set_target_enabled_at(0, false));
        assert!(!config.targets[0].enabled);
        assert!(!config.set_target_enabled_at(0, false));
        assert!(!config.set_target_enabled_at(1, true));
    }

    #[test]
    fn target_managed_mute_state_changes_only_when_needed() {
        let mut config = AppConfig::default();
        assert!(config.add_target("browser.exe"));

        assert!(!config.set_target_managed_muted_at(0, false));
        assert!(config.set_target_managed_muted_at(0, true));
        assert!(config.targets[0].managed_muted);
        assert!(!config.set_target_managed_muted_at(0, true));
        assert!(!config.set_target_managed_muted_at(1, true));
    }

    #[test]
    fn legacy_target_without_enabled_defaults_to_enabled() {
        let config: AppConfig =
            serde_json::from_str(r#"{"targets":[{"name":"game.exe"}]}"#).unwrap();

        assert!(config.targets[0].enabled);
    }

    #[test]
    fn enabled_targets_skip_default_enabled_when_serialized() {
        let mut target = TargetProcess::new("game.exe").unwrap();

        let enabled = serde_json::to_string(&target).unwrap();
        assert!(!enabled.contains("enabled"));
        assert!(!enabled.contains("managed_muted"));

        target.enabled = false;
        let disabled = serde_json::to_string(&target).unwrap();
        assert!(disabled.contains(r#""enabled":false"#));

        target.managed_muted = true;
        let managed_muted = serde_json::to_string(&target).unwrap();
        assert!(managed_muted.contains(r#""managed_muted":true"#));
    }

    #[test]
    fn legacy_target_without_managed_mute_defaults_to_not_managed() {
        let config: AppConfig =
            serde_json::from_str(r#"{"targets":[{"name":"game.exe"}]}"#).unwrap();

        assert!(!config.targets[0].managed_muted);
    }

    #[test]
    fn persisted_managed_mute_state_is_restored() {
        let bytes = br#"{"targets":[{"name":"game.exe","managed_muted":true}]}"#;
        let config: AppConfig = serde_json::from_slice(bytes).unwrap();

        assert!(config.targets[0].managed_muted);
    }

    #[test]
    fn malformed_managed_mute_state_is_rejected() {
        assert!(
            serde_json::from_str::<AppConfig>(
                r#"{"targets":[{"name":"game.exe","managed_muted":"yes"}]}"#,
            )
            .is_err()
        );
    }

    #[test]
    fn loaded_targets_are_normalized_and_deduplicated() {
        let mut config = AppConfig {
            targets: vec![
                TargetProcess {
                    name: r"C:\Games\Game.EXE".to_owned(),
                    pid: None,
                    note: None,
                    enabled: true,
                    managed_muted: false,
                },
                TargetProcess {
                    name: "game.exe".to_owned(),
                    pid: None,
                    note: None,
                    enabled: true,
                    managed_muted: true,
                },
            ],
            ..AppConfig::default()
        };

        config.deduplicate_targets();

        assert_eq!(config.targets.len(), 1);
        assert_eq!(config.targets[0].name, "game.exe");
        assert!(config.targets[0].managed_muted);
    }

    #[test]
    fn loaded_pid_zero_targets_are_removed() {
        let mut config = AppConfig {
            targets: vec![TargetProcess {
                name: "game.exe".to_owned(),
                pid: Some(0),
                note: None,
                enabled: true,
                managed_muted: false,
            }],
            ..AppConfig::default()
        };

        config.deduplicate_targets();

        assert!(config.targets.is_empty());
    }

    #[test]
    fn loaded_non_exe_targets_are_removed() {
        let mut config = AppConfig {
            targets: vec![TargetProcess {
                name: "system".to_owned(),
                pid: None,
                note: None,
                enabled: true,
                managed_muted: false,
            }],
            ..AppConfig::default()
        };

        config.deduplicate_targets();

        assert!(config.targets.is_empty());
    }

    #[test]
    fn loaded_reserved_device_targets_are_removed() {
        let mut config = AppConfig {
            targets: vec![TargetProcess {
                name: "nul.exe".to_owned(),
                pid: None,
                note: None,
                enabled: true,
                managed_muted: false,
            }],
            ..AppConfig::default()
        };

        config.deduplicate_targets();

        assert!(config.targets.is_empty());
    }

    #[test]
    fn deduplicate_targets_merges_duplicate_state_after_normalization() {
        let mut config = AppConfig {
            targets: vec![
                TargetProcess {
                    name: "Game.EXE".to_owned(),
                    pid: None,
                    note: Some("primary".to_owned()),
                    enabled: false,
                    managed_muted: true,
                },
                TargetProcess {
                    name: "game.exe".to_owned(),
                    pid: None,
                    note: Some("duplicate".to_owned()),
                    enabled: true,
                    managed_muted: false,
                },
            ],
            ..AppConfig::default()
        };

        config.deduplicate_targets();

        assert_eq!(config.targets.len(), 1);
        assert_eq!(config.targets[0].name, "game.exe");
        assert!(config.targets[0].enabled);
        assert!(config.targets[0].managed_muted);
        assert_eq!(config.targets[0].note.as_deref(), Some("primary"));
    }

    #[test]
    fn deduplicate_targets_keeps_note_from_later_duplicate_when_missing() {
        let mut config = AppConfig {
            targets: vec![
                TargetProcess {
                    name: "game.exe".to_owned(),
                    pid: Some(42),
                    note: None,
                    enabled: true,
                    managed_muted: false,
                },
                TargetProcess {
                    name: "GAME.EXE".to_owned(),
                    pid: Some(42),
                    note: Some("main".to_owned()),
                    enabled: true,
                    managed_muted: true,
                },
            ],
            ..AppConfig::default()
        };

        config.deduplicate_targets();

        assert_eq!(config.targets.len(), 1);
        assert_eq!(config.targets[0].name, "game.exe");
        assert_eq!(config.targets[0].pid, Some(42));
        assert!(config.targets[0].managed_muted);
        assert_eq!(config.targets[0].note.as_deref(), Some("main"));
    }

    #[test]
    fn save_writes_sanitized_config_snapshot() {
        let dir = TestDir::new();
        let path = dir.path().join("config.json");
        let config = AppConfig {
            version: 0,
            polling_interval_ms: 1,
            targets: vec![
                TargetProcess {
                    name: "Game.EXE".to_owned(),
                    pid: None,
                    note: Some("  primary\twindow  ".to_owned()),
                    enabled: true,
                    managed_muted: false,
                },
                TargetProcess {
                    name: "game.exe".to_owned(),
                    pid: None,
                    note: None,
                    enabled: true,
                    managed_muted: true,
                },
                TargetProcess {
                    name: "system".to_owned(),
                    pid: None,
                    note: None,
                    enabled: true,
                    managed_muted: false,
                },
            ],
            ..AppConfig::default()
        };

        config.save_to_path(&path, false).unwrap();
        let saved = parse_config_file(fs::File::open(path).unwrap()).unwrap();

        assert_eq!(saved.version, CONFIG_VERSION);
        assert_eq!(saved.polling_interval_ms, MIN_POLLING_INTERVAL_MS);
        assert_eq!(saved.targets.len(), 1);
        assert_eq!(saved.targets[0].name, "game.exe");
        assert_eq!(saved.targets[0].note.as_deref(), Some("primary window"));
        assert!(saved.targets[0].managed_muted);
    }

    #[test]
    fn default_startup_preferences_match_release_defaults() {
        let config = AppConfig::default();

        assert!(!config.launch_on_startup);
        assert!(config.start_minimized);
        assert!(config.hide_to_tray_on_close);
    }

    #[test]
    fn retired_restore_on_exit_setting_is_ignored() {
        let config: AppConfig =
            serde_json::from_str(r#"{"version":7,"restore_muted_on_exit":false}"#).unwrap();

        assert_eq!(config, AppConfig::default());
        assert!(
            !serde_json::to_string(&config)
                .unwrap()
                .contains("restore_muted_on_exit")
        );
    }

    #[test]
    fn theme_defaults_to_system_for_new_and_legacy_configs() {
        assert_eq!(AppConfig::default().theme, ThemePreference::System);

        let legacy: AppConfig = serde_json::from_str(r#"{"version":6}"#).unwrap();
        assert_eq!(legacy.theme, ThemePreference::System);
    }

    #[test]
    fn theme_preference_uses_stable_config_names() {
        assert_eq!(
            serde_json::to_string(&ThemePreference::System).unwrap(),
            r#""system""#
        );
        assert_eq!(
            serde_json::to_string(&ThemePreference::Dark).unwrap(),
            r#""dark""#
        );
    }

    #[test]
    fn legacy_config_defaults_to_hiding_on_close() {
        let config: AppConfig = serde_json::from_str(r#"{"version":5}"#).unwrap();

        assert!(config.hide_to_tray_on_close);
    }

    #[test]
    fn default_language_is_english() {
        let config = AppConfig::default();

        assert_eq!(config.language, Language::En);
    }

    #[test]
    fn sanitize_clamps_polling_interval() {
        let mut config = AppConfig {
            polling_interval_ms: 1,
            version: CONFIG_VERSION,
            ..AppConfig::default()
        };

        config.sanitize();

        assert_eq!(config.polling_interval_ms, MIN_POLLING_INTERVAL_MS);

        config.polling_interval_ms = 400;
        config.sanitize();

        assert_eq!(config.polling_interval_ms, 400);
    }

    #[test]
    fn legacy_config_without_window_size_uses_default_size() {
        let config: AppConfig = serde_json::from_str(r#"{"version":4}"#).unwrap();

        assert_eq!(config.window_size, None);
    }

    #[test]
    fn sanitize_discards_non_positive_window_sizes() {
        let mut config = AppConfig {
            window_size: Some(WindowSize {
                width: 660,
                height: 0,
            }),
            ..AppConfig::default()
        };

        config.sanitize();

        assert_eq!(config.window_size, None);
    }

    #[test]
    fn legacy_default_polling_interval_migrates_to_event_based_fallback() {
        let mut config = AppConfig {
            version: 1,
            polling_interval_ms: LEGACY_DEFAULT_POLLING_INTERVAL_MS,
            ..AppConfig::default()
        };

        config.sanitize();

        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.polling_interval_ms, DEFAULT_POLLING_INTERVAL_MS);
    }

    #[test]
    fn previous_event_fallback_default_migrates_to_current_default() {
        let mut config = AppConfig {
            version: 2,
            polling_interval_ms: EVENT_FALLBACK_DEFAULT_POLLING_INTERVAL_MS,
            ..AppConfig::default()
        };

        config.sanitize();

        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.polling_interval_ms, DEFAULT_POLLING_INTERVAL_MS);
    }

    #[test]
    fn custom_legacy_polling_interval_is_preserved() {
        let mut config = AppConfig {
            version: 1,
            polling_interval_ms: 700,
            ..AppConfig::default()
        };

        config.sanitize();

        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.polling_interval_ms, 700);
    }

    #[test]
    fn replace_file_replaces_existing_destination() {
        let dir = TestDir::new();
        let destination = dir.path().join("config.json");
        let temp_path = dir.path().join("config.json.tmp");

        fs::write(&destination, "old").unwrap();
        fs::write(&temp_path, "new").unwrap();

        replace_file(&temp_path, &destination, ConfigDurability::Durable).unwrap();

        assert_eq!(fs::read_to_string(&destination).unwrap(), "new");
        assert!(!temp_path.exists());
    }

    #[cfg(windows)]
    #[test]
    fn process_crash_safe_replace_skips_write_through() {
        assert_eq!(
            replace_file_flags(ConfigDurability::Durable),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH
        );
        assert_eq!(
            replace_file_flags(ConfigDurability::ProcessCrashSafe),
            MOVEFILE_REPLACE_EXISTING
        );
    }

    #[test]
    fn discard_open_file_closes_and_removes_file() {
        let dir = TestDir::new();
        let temp_path = dir.path().join("config.json.tmp");
        let temp_file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .unwrap();

        discard_open_file(temp_file, &temp_path);

        assert!(!temp_path.exists());
    }

    #[test]
    fn oversized_future_config_load_preserves_original_in_place() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        let mut future_config = format!(r#"{{"version":{}}}"#, CONFIG_VERSION + 1).into_bytes();
        future_config.resize(MAX_CONFIG_FILE_BYTES as usize + 1, b' ');
        fs::write(&config_path, &future_config).unwrap();

        let error = load_or_default_from_path(&config_path).err().unwrap();

        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
        assert_eq!(fs::read(&config_path).unwrap(), future_config);
        assert_eq!(invalid_backup_count(dir.path()), 0);
    }

    #[test]
    fn invalid_config_load_reports_recovery() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, "{not valid json").unwrap();

        let loaded = load_or_default_from_path(&config_path).unwrap();

        assert!(!loaded.first_run);
        assert!(loaded.recovered_invalid_config);
        assert_eq!(loaded.config, AppConfig::default());
        assert!(
            !load_or_default_from_path(&config_path)
                .unwrap()
                .recovered_invalid_config
        );
        assert!(
            fs::read_to_string(&config_path)
                .unwrap()
                .contains("\"version\"")
        );
        let backups = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.invalid-")
            })
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            fs::read_to_string(backups[0].path()).unwrap(),
            "{not valid json"
        );
    }

    #[test]
    fn future_config_version_is_preserved_instead_of_recovered() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        let future_config = format!(
            r#"{{"version":{},"start-minimized":"future-mode"}}"#,
            CONFIG_VERSION + 1
        );
        fs::write(&config_path, &future_config).unwrap();

        let error = load_or_default_from_path(&config_path).err().unwrap();

        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
        assert_eq!(fs::read_to_string(&config_path).unwrap(), future_config);
        assert_eq!(invalid_backup_count(dir.path()), 0);
    }

    #[test]
    fn existing_load_keeps_read_errors_blocking() {
        let dir = TestDir::new();
        let unreadable_path = dir.path().join("unreadable.json");
        fs::create_dir(&unreadable_path).unwrap();

        assert!(load_existing_from_path(&unreadable_path).is_err());
    }

    #[test]
    fn existing_load_keeps_future_config_version_blocking() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        let future_config = format!(r#"{{"version":{}}}"#, CONFIG_VERSION + 1);
        fs::write(&config_path, future_config).unwrap();

        let result = load_existing_from_path(&config_path);

        assert!(matches!(
            result,
            Err(error) if error.kind() == io::ErrorKind::Unsupported
        ));
    }

    #[test]
    fn invalid_recovery_does_not_replace_config_changed_after_parse() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        let invalid_config = b"{not valid json";
        fs::write(&config_path, invalid_config).unwrap();
        let expected_stamp = config_file_stamp(&config_path);
        let future_config = format!(r#"{{"version":{}}}"#, CONFIG_VERSION + 1);
        fs::write(&config_path, &future_config).unwrap();

        let recovered =
            recover_invalid_config_if_unchanged(&config_path, expected_stamp, invalid_config)
                .unwrap();

        assert!(recovered.is_none());
        assert_eq!(fs::read_to_string(&config_path).unwrap(), future_config);
        assert_eq!(invalid_backup_count(dir.path()), 0);
    }

    #[test]
    fn save_does_not_replace_future_config_version() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        let future_config = format!(
            r#"{{"version":{},"start-minimized":"future-mode"}}"#,
            CONFIG_VERSION + 1
        );
        fs::write(&config_path, &future_config).unwrap();

        let error = AppConfig::default()
            .save_to_path(&config_path, true)
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
        assert_eq!(fs::read_to_string(&config_path).unwrap(), future_config);
        assert_eq!(invalid_backup_count(dir.path()), 0);
    }

    #[test]
    fn guarded_save_does_not_replace_config_changed_after_stamp_check() {
        for durability in [
            ConfigDurability::Durable,
            ConfigDurability::ProcessCrashSafe,
        ] {
            let dir = TestDir::new();
            let config_path = dir.path().join("config.json");
            AppConfig::default()
                .save_to_path(&config_path, false)
                .unwrap();
            let expected_stamp = config_file_stamp(&config_path);
            let future_config = format!(r#"{{"version":{}}}"#, CONFIG_VERSION + 1);
            fs::write(&config_path, &future_config).unwrap();

            let saved = AppConfig::default()
                .save_to_path_guarded(
                    &config_path,
                    true,
                    ExistingConfigGuard::Unchanged(expected_stamp),
                    durability,
                )
                .unwrap();

            assert!(!saved);
            assert_eq!(fs::read_to_string(&config_path).unwrap(), future_config);
            assert_eq!(invalid_backup_count(dir.path()), 0);
        }
    }

    #[test]
    fn process_crash_safe_save_is_atomic_and_immediately_readable() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        let mut config = AppConfig::default();
        assert!(config.add_target("game.exe"));
        config.save_to_path(&config_path, false).unwrap();
        let expected_stamp = config_file_stamp(&config_path);
        assert!(config.set_target_managed_muted_at(0, true));

        let saved = config
            .save_to_path_guarded(
                &config_path,
                true,
                ExistingConfigGuard::Unchanged(expected_stamp),
                ConfigDurability::ProcessCrashSafe,
            )
            .unwrap();
        let reloaded = parse_config_file(fs::File::open(&config_path).unwrap()).unwrap();

        assert!(saved);
        assert!(reloaded.targets[0].managed_muted);
    }

    #[test]
    fn malformed_existing_load_can_be_recovered_by_guarded_save() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, "{not valid json").unwrap();

        let malformed = load_existing_from_path(&config_path).unwrap();

        assert!(matches!(
            malformed,
            ExistingConfigLoad::Malformed(error)
                if error.kind() == io::ErrorKind::InvalidData
        ));
        let expected_stamp = config_file_stamp(&config_path);

        let saved = AppConfig::default()
            .save_to_path_guarded(
                &config_path,
                true,
                ExistingConfigGuard::Unchanged(expected_stamp),
                ConfigDurability::Durable,
            )
            .unwrap();

        assert!(saved);
        assert!(
            fs::read_to_string(&config_path)
                .unwrap()
                .contains("\"version\"")
        );
        let backups = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.invalid-")
            })
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            fs::read_to_string(backups[0].path()).unwrap(),
            "{not valid json"
        );
    }

    #[test]
    fn load_read_errors_are_not_recovered_as_invalid_config() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        fs::create_dir(&config_path).unwrap();

        assert!(load_or_default_from_path(&config_path).is_err());
        assert_eq!(invalid_backup_count(dir.path()), 0);
    }

    #[test]
    fn existing_config_read_errors_are_not_validated_as_invalid() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        fs::create_dir(&config_path).unwrap();

        assert!(validate_existing_config(&config_path).is_err());
        assert_eq!(invalid_backup_count(dir.path()), 0);
    }

    #[test]
    fn save_backs_up_invalid_existing_config_before_replacing() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, "{not valid json").unwrap();

        AppConfig::default()
            .save_to_path(&config_path, true)
            .unwrap();

        assert!(
            fs::read_to_string(&config_path)
                .unwrap()
                .contains("\"version\"")
        );
        let backups = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.invalid-")
            })
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            fs::read_to_string(backups[0].path()).unwrap(),
            "{not valid json"
        );
    }

    #[test]
    fn save_does_not_overwrite_existing_temp_config() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        let existing_temp = dir.path().join("config.json.tmp");
        fs::write(&existing_temp, "keep").unwrap();

        AppConfig::default()
            .save_to_path(&config_path, false)
            .unwrap();

        assert!(
            fs::read_to_string(&config_path)
                .unwrap()
                .contains("\"version\"")
        );
        assert_eq!(fs::read_to_string(existing_temp).unwrap(), "keep");
        let temp_files = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .count();
        assert_eq!(temp_files, 1);
    }

    #[test]
    fn merge_pending_config_changes_preserves_external_target_additions() {
        let mut base = AppConfig::default();
        assert!(base.add_target("game.exe"));

        let mut local = base.clone();
        assert!(local.set_target_note_at(0, Some("local".to_owned())));

        let mut disk = base.clone();
        assert!(disk.add_target("chat.exe"));

        merge_pending_config_changes(&base, &local, &mut disk);

        assert_eq!(disk.targets.len(), 2);
        assert_eq!(disk.targets[0].name, "chat.exe");
        assert_eq!(disk.targets[1].name, "game.exe");
        assert_eq!(disk.targets[1].note.as_deref(), Some("local"));
    }

    #[test]
    fn merge_pending_config_changes_keeps_runtime_mute_updates() {
        let mut base = AppConfig::default();
        assert!(base.add_target("game.exe"));

        let mut local = base.clone();
        assert!(local.set_target_managed_muted_at(0, true));

        let mut disk = base.clone();
        assert!(disk.add_target("chat.exe"));

        merge_pending_config_changes(&base, &local, &mut disk);

        let game = target_index_by_key(&disk.targets, "game.exe", None).unwrap();
        assert!(disk.targets[game].managed_muted);
        assert!(target_index_by_key(&disk.targets, "chat.exe", None).is_some());
    }

    #[test]
    fn merge_pending_config_changes_keeps_external_runtime_mute_for_local_target_edits() {
        let mut base = AppConfig::default();
        assert!(base.add_target("game.exe"));

        let mut local = base.clone();
        assert!(local.set_target_note_at(0, Some("local".to_owned())));

        let mut disk = base.clone();
        assert!(disk.set_target_managed_muted_at(0, true));

        merge_pending_config_changes(&base, &local, &mut disk);

        assert_eq!(disk.targets[0].note.as_deref(), Some("local"));
        assert!(disk.targets[0].managed_muted);
    }

    #[test]
    fn merge_pending_config_changes_merges_runtime_and_user_fields_independently() {
        let mut base = AppConfig::default();
        assert!(base.add_target("game.exe"));
        assert!(base.set_target_note_at(0, Some("base".to_owned())));

        let mut local = base.clone();
        assert!(local.set_target_note_at(0, Some("local".to_owned())));
        assert!(local.set_target_managed_muted_at(0, true));

        let mut disk = base.clone();
        assert!(disk.set_target_enabled_at(0, false));

        merge_pending_config_changes(&base, &local, &mut disk);

        assert_eq!(disk.targets[0].note.as_deref(), Some("local"));
        assert!(!disk.targets[0].enabled);
        assert!(disk.targets[0].managed_muted);
    }

    #[test]
    fn merge_pending_runtime_mute_does_not_resurrect_external_target_removal() {
        let mut base = AppConfig::default();
        assert!(base.add_target("game.exe"));
        let mut local = base.clone();
        assert!(local.set_target_managed_muted_at(0, true));
        let mut disk = AppConfig::default();

        merge_pending_config_changes(&base, &local, &mut disk);

        assert!(disk.targets.is_empty());
    }

    #[test]
    fn merge_pending_config_changes_keeps_local_window_size() {
        let base = AppConfig::default();
        let mut local = base.clone();
        local.window_size = Some(WindowSize {
            width: 825,
            height: 875,
        });
        let mut disk = base.clone();
        disk.language = Language::Ko;

        merge_pending_config_changes(&base, &local, &mut disk);

        assert_eq!(disk.window_size, local.window_size);
        assert_eq!(disk.language, Language::Ko);
    }

    #[test]
    fn merge_pending_config_changes_merges_target_fields_independently() {
        let mut base = AppConfig::default();
        assert!(base.add_target("game.exe"));
        assert!(base.set_target_note_at(0, Some("base".to_owned())));

        let mut local = base.clone();
        assert!(local.set_target_note_at(0, Some("local".to_owned())));

        let mut disk = base.clone();
        assert!(disk.set_target_enabled_at(0, false));

        merge_pending_config_changes(&base, &local, &mut disk);

        assert_eq!(disk.targets[0].note.as_deref(), Some("local"));
        assert!(!disk.targets[0].enabled);
    }

    #[test]
    fn merge_pending_config_changes_keeps_local_target_removals() {
        let mut base = AppConfig::default();
        assert!(base.add_target("chat.exe"));
        assert!(base.add_target("game.exe"));

        let mut local = base.clone();
        assert!(local.remove_target_at(1));

        let mut disk = base.clone();
        assert!(disk.add_target("music.exe"));

        merge_pending_config_changes(&base, &local, &mut disk);

        assert!(target_index_by_key(&disk.targets, "chat.exe", None).is_some());
        assert!(target_index_by_key(&disk.targets, "game.exe", None).is_none());
        assert!(target_index_by_key(&disk.targets, "music.exe", None).is_some());
    }

    #[test]
    fn trusted_save_skips_invalid_existing_backup() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, "{not valid json").unwrap();

        AppConfig::default()
            .save_to_path(&config_path, false)
            .unwrap();

        assert!(
            fs::read_to_string(&config_path)
                .unwrap()
                .contains("\"version\"")
        );
        let backups = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.invalid-")
            })
            .count();
        assert_eq!(backups, 0);
    }
}
