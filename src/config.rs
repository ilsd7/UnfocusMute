#![cfg_attr(not(windows), allow(dead_code))]

use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

const CONFIG_VERSION: u32 = 1;
const DEFAULT_POLLING_INTERVAL_MS: u64 = 350;

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

    pub fn display_name(&self) -> String {
        match self.pid {
            Some(pid) => format!("{} (PID {pid})", self.name),
            None => self.name.clone(),
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
            launch_on_startup: true,
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

        let raw = fs::read_to_string(path)?;
        let mut config = serde_json::from_str::<Self>(&raw).unwrap_or_default();
        config.deduplicate_targets();
        Ok(config)
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_file_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(&temp_path, json)?;
        fs::rename(temp_path, path)
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
        let mut names = BTreeSet::new();
        self.targets.retain(|target| {
            let Some(name) = normalize_process_name(&target.name) else {
                return false;
            };
            names.insert((name, target.pid))
        });
        self.sort_targets();
    }

    fn sort_targets(&mut self) {
        self.targets
            .sort_by_key(|target| (target.name.clone(), target.pid.unwrap_or(0)));
    }
}

pub fn normalize_process_name(input: &str) -> Option<String> {
    let name = input
        .trim()
        .trim_matches('"')
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    if name.is_empty() || name.contains('\0') {
        return None;
    }

    Some(name)
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
    fn default_startup_preferences_match_release_defaults() {
        let config = AppConfig::default();

        assert!(config.launch_on_startup);
        assert!(config.start_minimized);
        assert!(config.restore_muted_on_exit);
    }

    #[test]
    fn default_language_is_english() {
        let config = AppConfig::default();

        assert_eq!(config.language, Language::En);
    }
}
