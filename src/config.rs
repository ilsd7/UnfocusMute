#![cfg_attr(not(windows), allow(dead_code))]

use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{
    MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
};
#[cfg(windows)]
use windows::core::PCWSTR;

const CONFIG_VERSION: u32 = 3;
const LEGACY_DEFAULT_POLLING_INTERVAL_MS: u64 = 350;
const EVENT_FALLBACK_DEFAULT_POLLING_INTERVAL_MS: u64 = 5_000;
const DEFAULT_POLLING_INTERVAL_MS: u64 = 3_000;
const MIN_POLLING_INTERVAL_MS: u64 = 100;
const MAX_POLLING_INTERVAL_MS: u64 = 10_000;
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
}

impl TargetProcess {
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
        })
    }

    pub fn display_identity_into(&self, output: &mut String) {
        output.clear();
        output.push_str(&self.name);
        if let Some(pid) = self.pid {
            output.push_str(" (PID ");
            push_decimal_u32(output, pid);
            output.push(')');
        }
    }

    pub fn display_name_into(&self, output: &mut String) {
        self.display_identity_into(output);
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct AppConfig {
    pub version: u32,
    pub language: Language,
    pub window_position: Option<WindowPosition>,
    pub polling_interval_ms: u64,
    pub launch_on_startup: bool,
    pub start_minimized: bool,
    pub restore_muted_on_exit: bool,
    pub targets: Vec<TargetProcess>,
}

pub struct AppConfigLoad {
    pub config: AppConfig,
    pub first_run: bool,
    pub recovered_invalid_config: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            language: Language::default(),
            window_position: None,
            polling_interval_ms: DEFAULT_POLLING_INTERVAL_MS,
            launch_on_startup: false,
            start_minimized: true,
            restore_muted_on_exit: true,
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
        }
    }
}

impl AppConfig {
    pub fn load_or_default_with_status() -> io::Result<AppConfigLoad> {
        let path = cached_config_file_path()?;
        load_or_default_from_path(path)
    }

    pub fn load_existing() -> io::Result<Self> {
        let path = cached_config_file_path()?;
        let mut config = parse_config_file(fs::File::open(path)?)?;
        config.sanitize();
        Ok(config)
    }

    pub fn save(&self) -> io::Result<()> {
        self.save_with_existing_validation(true)
    }

    pub(crate) fn save_trusting_existing_file(&self) -> io::Result<()> {
        self.save_with_existing_validation(false)
    }

    fn save_with_existing_validation(&self, validate_existing: bool) -> io::Result<()> {
        let path = cached_config_file_path()?;
        self.save_to_path(path, validate_existing)
    }

    fn save_to_path(&self, path: &Path, validate_existing: bool) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if validate_existing {
            backup_invalid_existing_config(path)?;
        }

        let (mut temp_file, temp_path) = create_temp_config_file(path)?;
        if let Err(error) = serde_json::to_writer_pretty(&mut temp_file, self) {
            let _ = fs::remove_file(&temp_path);
            return Err(io::Error::other(error));
        }
        if let Err(error) = temp_file.sync_all() {
            let _ = fs::remove_file(&temp_path);
            return Err(error);
        }
        drop(temp_file);
        replace_file(&temp_path, path)
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
        })
    }

    fn add_target_process(&mut self, target: TargetProcess) -> bool {
        debug_assert!(is_supported_target_process_name(&target.name));
        debug_assert!(
            self.targets
                .iter()
                .all(|target| is_supported_target_process_name(&target.name))
        );
        match self.targets.len() {
            0 => {
                self.targets.push(target);
                return true;
            }
            1 => match compare_targets(&self.targets[0], &target) {
                Ordering::Equal => return false,
                Ordering::Less => {
                    self.targets.push(target);
                    return true;
                }
                Ordering::Greater => {
                    self.targets.insert(0, target);
                    return true;
                }
            },
            _ => {}
        }
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

    pub(crate) fn contains_normalized_target(&self, name: &str, pid: Option<u32>) -> bool {
        debug_assert!(is_normalized_process_name(name));
        match self.targets.as_slice() {
            [] => false,
            [target] => compare_target_key(target, name, pid).is_eq(),
            targets => targets
                .binary_search_by(|target| compare_target_key(target, name, pid))
                .is_ok(),
        }
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
            self.targets
                .dedup_by(|right, left| right.name == left.name && right.pid == left.pid);
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
        self.deduplicate_targets();
    }

    fn sort_targets(&mut self) {
        self.targets.sort_by(compare_targets);
    }
}

fn compare_targets(left: &TargetProcess, right: &TargetProcess) -> Ordering {
    compare_target_key(left, &right.name, right.pid)
}

fn compare_target_key(left: &TargetProcess, name: &str, pid: Option<u32>) -> Ordering {
    left.name
        .as_str()
        .cmp(name)
        .then_with(|| left.pid.cmp(&pid))
}

fn load_or_default_from_path(path: &Path) -> io::Result<AppConfigLoad> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(AppConfigLoad::default());
        }
        Err(error) => return Err(error),
    };
    if let Ok(mut config) = parse_config_file(file) {
        config.sanitize();
        return Ok(AppConfigLoad {
            config,
            first_run: false,
            recovered_invalid_config: false,
        });
    }

    let _ = backup_invalid_config(path);
    Ok(AppConfigLoad {
        config: AppConfig::default(),
        first_run: false,
        recovered_invalid_config: true,
    })
}

fn parse_config_file(file: fs::File) -> io::Result<AppConfig> {
    serde_json::from_reader::<_, AppConfig>(file).map_err(io::Error::other)
}

fn backup_invalid_existing_config(path: &Path) -> io::Result<()> {
    match fs::File::open(path) {
        Ok(file) => {
            if parse_config_file(file).is_err() {
                backup_invalid_config(path)?;
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    Ok(())
}

fn replace_file(temp_path: &Path, destination: &Path) -> io::Result<()> {
    #[cfg(windows)]
    let result = replace_file_windows(temp_path, destination);

    #[cfg(not(windows))]
    let result = fs::rename(temp_path, destination);

    if result.is_err() {
        let _ = fs::remove_file(temp_path);
    }
    result
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
fn replace_file_windows(temp_path: &Path, destination: &Path) -> io::Result<()> {
    let temp_path = path_to_wide(temp_path);
    let destination = path_to_wide(destination);
    let flags = MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH;

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
fn path_to_wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn backup_invalid_config(path: &Path) -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    backup_invalid_config_with_timestamp(path, timestamp)
}

fn backup_invalid_config_with_timestamp(path: &Path, timestamp: u64) -> io::Result<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut source = fs::File::open(path)?;
    for index in 0..100 {
        let file_name = if index == 0 {
            format!("config.invalid-{timestamp}.json")
        } else {
            format!("config.invalid-{timestamp}-{index}.json")
        };
        let backup_path = parent.join(file_name);
        let mut backup = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup_path)
        {
            Ok(backup) => backup,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };

        if let Err(error) = io::copy(&mut source, &mut backup) {
            let _ = fs::remove_file(&backup_path);
            return Err(error);
        }
        if let Err(error) = backup.sync_all() {
            let _ = fs::remove_file(&backup_path);
            return Err(error);
        }
        return Ok(backup_path);
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique invalid config backup path",
    ))
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
        .trim()
        .trim_matches('"')
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches('"')
        .trim();

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

pub fn normalize_process_name(input: &str) -> Option<String> {
    Some(normalize_process_name_cow(input)?.into_owned())
}

pub fn normalize_process_name_cow(input: &str) -> Option<Cow<'_, str>> {
    let candidate = process_name_candidate(input)?;

    if candidate.has_uppercase {
        Some(Cow::Owned(candidate.name.to_ascii_lowercase()))
    } else {
        Some(Cow::Borrowed(candidate.name))
    }
}

pub fn normalize_process_name_owned(mut input: String) -> Option<String> {
    let candidate = process_name_candidate(&input)?;

    if candidate.name.len() == input.len() && candidate.name.as_ptr() == input.as_ptr() {
        if candidate.has_uppercase {
            input.make_ascii_lowercase();
        }
        return Some(input);
    }

    if candidate.has_uppercase {
        Some(candidate.name.to_ascii_lowercase())
    } else {
        Some(candidate.name.to_owned())
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

fn normalized_utf16_process_name(candidate: Utf16ProcessNameCandidate<'_>) -> String {
    if candidate.is_ascii {
        return ascii_utf16_process_name(candidate.name, candidate.has_uppercase);
    }

    let mut name = String::from_utf16_lossy(candidate.name);
    if candidate.has_uppercase {
        name.make_ascii_lowercase();
    }
    name
}

pub fn is_normalized_process_name(input: &str) -> bool {
    process_name_candidate(input).is_some_and(|candidate| {
        !candidate.has_uppercase
            && candidate.name.len() == input.len()
            && candidate.name.as_ptr() == input.as_ptr()
    })
}

pub(crate) fn is_supported_target_process_name(input: &str) -> bool {
    is_normalized_process_name(input) && is_supported_normalized_target_process_name(input)
}

pub(crate) fn is_supported_normalized_target_process_name(input: &str) -> bool {
    input
        .strip_suffix(".exe")
        .is_some_and(|name| !windows_reserved_device_name(name))
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

fn is_supported_utf16_target_process_name(input: &[u16]) -> bool {
    strip_ascii_exe_suffix_utf16(input)
        .is_some_and(|name| !windows_reserved_device_name_utf16(name))
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
    fn normalizes_utf16_paths_before_allocating_name() {
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
    fn supported_utf16_names_are_filtered_before_allocating_name() {
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated(r"C:\Games\Example.EXE")),
            Some("example.exe".to_owned())
        );
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated("System")),
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

        target.enabled = false;
        let disabled = serde_json::to_string(&target).unwrap();
        assert!(disabled.contains(r#""enabled":false"#));
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
                },
                TargetProcess {
                    name: "game.exe".to_owned(),
                    pid: None,
                    note: None,
                    enabled: true,
                },
            ],
            ..AppConfig::default()
        };

        config.deduplicate_targets();

        assert_eq!(config.targets.len(), 1);
        assert_eq!(config.targets[0].name, "game.exe");
    }

    #[test]
    fn loaded_pid_zero_targets_are_removed() {
        let mut config = AppConfig {
            targets: vec![TargetProcess {
                name: "game.exe".to_owned(),
                pid: Some(0),
                note: None,
                enabled: true,
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
            }],
            ..AppConfig::default()
        };

        config.deduplicate_targets();

        assert!(config.targets.is_empty());
    }

    #[test]
    fn deduplicate_targets_keeps_first_duplicate_after_normalization() {
        let mut config = AppConfig {
            targets: vec![
                TargetProcess {
                    name: "Game.EXE".to_owned(),
                    pid: None,
                    note: None,
                    enabled: false,
                },
                TargetProcess {
                    name: "game.exe".to_owned(),
                    pid: None,
                    note: None,
                    enabled: true,
                },
            ],
            ..AppConfig::default()
        };

        config.deduplicate_targets();

        assert_eq!(config.targets.len(), 1);
        assert!(!config.targets[0].enabled);
    }

    #[test]
    fn default_startup_preferences_match_release_defaults() {
        let config = AppConfig::default();

        assert!(!config.launch_on_startup);
        assert!(config.start_minimized);
        assert!(config.restore_muted_on_exit);
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

        replace_file(&temp_path, &destination).unwrap();

        assert_eq!(fs::read_to_string(&destination).unwrap(), "new");
        assert!(!temp_path.exists());
    }

    #[test]
    fn invalid_config_backup_preserves_original_contents() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, "{not valid json").unwrap();

        let backup_path = backup_invalid_config(&config_path).unwrap();

        assert_ne!(backup_path, config_path);
        assert_eq!(fs::read_to_string(backup_path).unwrap(), "{not valid json");
        assert_eq!(fs::read_to_string(config_path).unwrap(), "{not valid json");
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
    }

    #[test]
    fn invalid_config_backup_does_not_overwrite_existing_backup() {
        let dir = TestDir::new();
        let config_path = dir.path().join("config.json");
        let existing_backup = dir.path().join("config.invalid-7.json");
        fs::write(&config_path, "{not valid json").unwrap();
        fs::write(&existing_backup, "keep").unwrap();

        let backup_path = backup_invalid_config_with_timestamp(&config_path, 7).unwrap();

        assert_eq!(backup_path, dir.path().join("config.invalid-7-1.json"));
        assert_eq!(fs::read_to_string(existing_backup).unwrap(), "keep");
        assert_eq!(fs::read_to_string(backup_path).unwrap(), "{not valid json");
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
