#![cfg_attr(not(windows), allow(dead_code))]

use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TargetProcess {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub enabled: bool,
}

impl TargetProcess {
    pub fn new(name: impl AsRef<str>) -> Option<Self> {
        let name = normalize_process_name(name.as_ref())?;
        Some(Self {
            name,
            pid: None,
            enabled: true,
        })
    }

    pub fn for_pid(name: impl AsRef<str>, pid: u32) -> Option<Self> {
        let name = normalize_process_name(name.as_ref())?;
        Some(Self {
            name,
            pid: Some(pid),
            enabled: true,
        })
    }

    pub fn display_name(&self) -> Cow<'_, str> {
        match self.pid {
            Some(pid) => Cow::Owned(format!("{} (PID {pid})", self.name)),
            None => Cow::Borrowed(&self.name),
        }
    }
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

impl AppConfig {
    pub fn load_or_default() -> io::Result<Self> {
        let path = config_file_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&path)?;
        match parse_config(&raw) {
            Ok(mut config) => {
                config.sanitize();
                Ok(config)
            }
            Err(_) => {
                let _ = backup_invalid_config(&path);
                Ok(Self::default())
            }
        }
    }

    pub fn load_existing() -> io::Result<Self> {
        let path = config_file_path()?;
        let raw = fs::read_to_string(&path)?;
        let mut config = parse_config(&raw)?;
        config.sanitize();
        Ok(config)
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_file_path()?;
        self.save_to_path(&path)
    }

    fn save_to_path(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        backup_invalid_existing_config(path)?;

        let temp_path = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(&temp_path, json)?;
        replace_file(&temp_path, path)
    }

    pub fn add_target(&mut self, name: impl AsRef<str>) -> bool {
        let Some(target) = TargetProcess::new(name) else {
            return false;
        };
        self.add_target_process(target)
    }

    pub fn add_pid_target(&mut self, name: impl AsRef<str>, pid: u32) -> bool {
        let Some(target) = TargetProcess::for_pid(name, pid) else {
            return false;
        };
        self.add_target_process(target)
    }

    fn add_target_process(&mut self, target: TargetProcess) -> bool {
        if self.targets.iter().any(|existing| {
            existing.name.eq_ignore_ascii_case(&target.name) && existing.pid == target.pid
        }) {
            return false;
        }
        self.targets.push(target);
        self.sort_targets();
        true
    }

    pub fn remove_target_at(&mut self, index: usize) -> bool {
        if index >= self.targets.len() {
            return false;
        }
        self.targets.remove(index);
        true
    }

    pub fn deduplicate_targets(&mut self) {
        self.targets.retain_mut(|target| {
            let Some(name) = normalize_process_name_owned(std::mem::take(&mut target.name)) else {
                return false;
            };
            target.name = name;
            true
        });
        self.sort_targets();
        self.targets
            .dedup_by(|right, left| right.name == left.name && right.pid == left.pid);
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
        self.targets.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.pid.unwrap_or(0).cmp(&right.pid.unwrap_or(0)))
        });
    }
}

fn parse_config(raw: &str) -> io::Result<AppConfig> {
    serde_json::from_str::<AppConfig>(raw).map_err(io::Error::other)
}

fn backup_invalid_existing_config(path: &Path) -> io::Result<()> {
    match fs::read_to_string(path) {
        Ok(raw) => {
            if parse_config(&raw).is_err() {
                backup_invalid_config(path)?;
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
            backup_invalid_config(path)?;
        }
        Err(error) => return Err(error),
    }

    Ok(())
}

fn replace_file(temp_path: &Path, destination: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        replace_file_windows(temp_path, destination)
    }

    #[cfg(not(windows))]
    {
        fs::rename(temp_path, destination).inspect_err(|_| {
            let _ = fs::remove_file(temp_path);
        })
    }
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
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    for index in 0..100 {
        let file_name = if index == 0 {
            format!("config.invalid-{timestamp}.json")
        } else {
            format!("config.invalid-{timestamp}-{index}.json")
        };
        let backup_path = parent.join(file_name);
        if backup_path.exists() {
            continue;
        }
        fs::copy(path, &backup_path)?;
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
    let name = input
        .trim()
        .trim_matches('"')
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .trim();

    if name.is_empty() {
        return None;
    }

    let mut has_uppercase = false;
    for byte in name.bytes() {
        if byte == b'\0' {
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
    let candidate = process_name_candidate(input)?;

    if candidate.has_uppercase {
        Some(candidate.name.to_ascii_lowercase())
    } else {
        Some(candidate.name.to_owned())
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

pub fn is_normalized_process_name(input: &str) -> bool {
    process_name_candidate(input).is_some_and(|candidate| {
        !candidate.has_uppercase
            && candidate.name.len() == input.len()
            && candidate.name.as_ptr() == input.as_ptr()
    })
}

pub fn config_dir() -> io::Result<PathBuf> {
    if cfg!(windows)
        && let Some(appdata) = env::var_os("APPDATA")
    {
        return Ok(PathBuf::from(appdata).join("UnfocusMute"));
    }

    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("unfocusmute"));
    }

    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory not found"))?;
    Ok(PathBuf::from(home).join(".config").join("unfocusmute"))
}

pub fn config_file_path() -> io::Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn config_file_exists() -> bool {
    config_file_path().is_ok_and(|path| path.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_paths_and_case() {
        assert_eq!(
            normalize_process_name(r#"C:\Games\Example.EXE"#),
            Some("example.exe".to_owned())
        );
        assert_eq!(
            normalize_process_name("  game.exe  "),
            Some("game.exe".to_owned())
        );
        assert_eq!(normalize_process_name(""), None);
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
    fn owned_normalization_matches_borrowed_normalization() {
        let inputs = [
            r#"C:\Games\Example.EXE"#,
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
        assert!(!is_normalized_process_name(r#"C:\Games\game.exe"#));
        assert!(!is_normalized_process_name("game.exe\0"));
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
    fn exe_and_pid_targets_can_coexist() {
        let mut config = AppConfig::default();

        assert!(config.add_target("browser.exe"));
        assert!(config.add_pid_target("browser.exe", 42));
        assert!(!config.add_pid_target("browser.exe", 42));
        assert_eq!(config.targets.len(), 2);
        assert_eq!(config.targets[1].display_name(), "browser.exe (PID 42)");
        assert!(config.remove_target_at(1));
        assert_eq!(config.targets.len(), 1);
    }

    #[test]
    fn loaded_targets_are_normalized_and_deduplicated() {
        let mut config = AppConfig {
            targets: vec![
                TargetProcess {
                    name: r#"C:\Games\Game.EXE"#.to_owned(),
                    pid: None,
                    enabled: true,
                },
                TargetProcess {
                    name: "game.exe".to_owned(),
                    pid: None,
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
    fn deduplicate_targets_keeps_first_duplicate_after_normalization() {
        let mut config = AppConfig {
            targets: vec![
                TargetProcess {
                    name: "Game.EXE".to_owned(),
                    pid: None,
                    enabled: false,
                },
                TargetProcess {
                    name: "game.exe".to_owned(),
                    pid: None,
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
    fn default_language_is_korean() {
        let config = AppConfig::default();

        assert_eq!(config.language, Language::Ko);
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
        let dir = tempfile::tempdir().unwrap();
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
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, "{not valid json").unwrap();

        let backup_path = backup_invalid_config(&config_path).unwrap();

        assert_ne!(backup_path, config_path);
        assert_eq!(fs::read_to_string(backup_path).unwrap(), "{not valid json");
        assert_eq!(fs::read_to_string(config_path).unwrap(), "{not valid json");
    }

    #[test]
    fn save_backs_up_invalid_existing_config_before_replacing() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, "{not valid json").unwrap();

        AppConfig::default().save_to_path(&config_path).unwrap();

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
}
