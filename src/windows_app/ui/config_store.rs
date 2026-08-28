use crate::config::{
    AppConfig, ConfigDurability, ConfigFileStamp, ExistingConfigLoad, current_config_stamp,
    merge_pending_config_changes,
};
use std::io;
use std::time::{Duration, Instant};

pub(super) enum ConfigReload {
    Unchanged,
    Missing,
    Loaded(AppConfig),
    Failed(io::Error),
}

pub(super) enum ConfigMerge {
    Ready(Option<AppConfig>),
    Retry,
}

pub(super) enum ConfigSave {
    Saved,
    Retry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ConfigSaveIntent {
    Settings,
    RuntimeMuteState,
}

#[derive(Clone, Copy)]
enum ConfigStampState {
    Unknown,
    Known(Option<ConfigFileStamp>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfigLoadState {
    Ready,
    Failed,
    RecoverMalformed,
}

impl ConfigLoadState {
    fn blocks_save(self) -> bool {
        self == Self::Failed
    }
}

fn save_durability(
    intent: ConfigSaveIntent,
    existing_file: bool,
    load_state: ConfigLoadState,
    persisted: &AppConfig,
    config: &AppConfig,
) -> ConfigDurability {
    if intent == ConfigSaveIntent::RuntimeMuteState
        && existing_file
        && load_state == ConfigLoadState::Ready
        && configs_match_ignoring_runtime_mute(persisted, config)
    {
        ConfigDurability::ProcessCrashSafe
    } else {
        ConfigDurability::Durable
    }
}

fn configs_match_ignoring_runtime_mute(left: &AppConfig, right: &AppConfig) -> bool {
    let mut left = left.clone();
    let mut right = right.clone();
    for target in &mut left.targets {
        target.managed_muted = false;
    }
    for target in &mut right.targets {
        target.managed_muted = false;
    }
    left == right
}

pub(super) struct ConfigStore {
    persisted: AppConfig,
    stamp: ConfigStampState,
    next_check: Instant,
    reload_interval: Duration,
    load_state: ConfigLoadState,
}

impl ConfigStore {
    const READ_RETRY_LIMIT: usize = 3;

    pub(super) fn new(
        config: &AppConfig,
        reload_interval: Duration,
        initial_load_failed: bool,
    ) -> Self {
        Self {
            persisted: config.clone(),
            // The caller's config and a stamp sampled here are not an atomic pair. Starting
            // unknown makes the first reload or save establish that pair from one stable read.
            stamp: ConfigStampState::Unknown,
            next_check: Instant::now() + reload_interval,
            reload_interval,
            load_state: if initial_load_failed {
                ConfigLoadState::Failed
            } else {
                ConfigLoadState::Ready
            },
        }
    }

    pub(super) fn reload_if_due(&mut self) -> ConfigReload {
        let now = Instant::now();
        if now < self.next_check {
            return ConfigReload::Unchanged;
        }
        self.next_check = now + self.reload_interval;
        self.reload_if_changed()
    }

    pub(super) fn reload_if_changed(&mut self) -> ConfigReload {
        let stamp = current_config_stamp();
        if !self.reload_needed(stamp) {
            return ConfigReload::Unchanged;
        }
        self.reload_now()
    }

    pub(super) fn reload_now(&mut self) -> ConfigReload {
        for _ in 0..Self::READ_RETRY_LIMIT {
            let stamp_before_load = current_config_stamp();
            match AppConfig::load_existing_with_status() {
                Ok(ExistingConfigLoad::Loaded(config)) => {
                    if current_config_stamp() != stamp_before_load {
                        continue;
                    }
                    self.stamp = ConfigStampState::Known(stamp_before_load);
                    self.load_state = ConfigLoadState::Ready;
                    return ConfigReload::Loaded(config);
                }
                Ok(ExistingConfigLoad::Missing(_)) => {
                    if current_config_stamp() != stamp_before_load {
                        continue;
                    }
                    self.stamp = ConfigStampState::Known(None);
                    self.load_state = ConfigLoadState::Ready;
                    return ConfigReload::Missing;
                }
                Ok(ExistingConfigLoad::Malformed(error)) => {
                    if current_config_stamp() != stamp_before_load {
                        continue;
                    }
                    self.stamp = ConfigStampState::Known(stamp_before_load);
                    // A reload reports malformed content as a failure. Only an explicit in-app
                    // save may opt into guarded recovery after checking the file again.
                    self.load_state = ConfigLoadState::Failed;
                    return ConfigReload::Failed(error);
                }
                Err(error) => {
                    self.stamp = ConfigStampState::Known(stamp_before_load);
                    self.load_state = ConfigLoadState::Failed;
                    return ConfigReload::Failed(error);
                }
            }
        }

        self.load_state = ConfigLoadState::Failed;
        ConfigReload::Failed(io::Error::new(
            io::ErrorKind::WouldBlock,
            "config file kept changing while the app was reading it",
        ))
    }

    pub(super) fn accept_loaded(&mut self, config: &AppConfig) {
        self.persisted = config.clone();
    }

    pub(super) fn merge_external_before_save(
        &mut self,
        local: &AppConfig,
    ) -> io::Result<ConfigMerge> {
        let stamp_before_load = current_config_stamp();
        if !self.reload_needed(stamp_before_load) {
            return Ok(ConfigMerge::Ready(None));
        }

        let mut disk_config = match AppConfig::load_existing_with_status() {
            Ok(ExistingConfigLoad::Loaded(config)) => config,
            Ok(ExistingConfigLoad::Missing(_)) => {
                if current_config_stamp() != stamp_before_load {
                    return Ok(ConfigMerge::Retry);
                }
                self.stamp = ConfigStampState::Known(None);
                self.load_state = ConfigLoadState::Ready;
                return Ok(ConfigMerge::Ready(None));
            }
            Ok(ExistingConfigLoad::Malformed(_)) => {
                if current_config_stamp() != stamp_before_load {
                    return Ok(ConfigMerge::Retry);
                }
                self.stamp = ConfigStampState::Known(stamp_before_load);
                // The following save may recover only the exact malformed file inspected here.
                self.load_state = ConfigLoadState::RecoverMalformed;
                return Ok(ConfigMerge::Ready(None));
            }
            Err(error) => {
                self.stamp = ConfigStampState::Known(stamp_before_load);
                self.load_state = ConfigLoadState::Failed;
                return Err(error);
            }
        };
        let stamp_after_load = current_config_stamp();
        if stamp_before_load != stamp_after_load {
            return Ok(ConfigMerge::Retry);
        }

        let next_base = disk_config.clone();
        merge_pending_config_changes(&self.persisted, local, &mut disk_config);
        self.persisted = next_base;
        self.stamp = ConfigStampState::Known(stamp_after_load);
        self.load_state = ConfigLoadState::Ready;
        Ok(ConfigMerge::Ready(
            (disk_config != *local).then_some(disk_config),
        ))
    }

    pub(super) fn save(
        &mut self,
        config: &AppConfig,
        intent: ConfigSaveIntent,
    ) -> io::Result<ConfigSave> {
        let current_stamp = current_config_stamp();
        let ConfigStampState::Known(expected_stamp) = self.stamp else {
            return Ok(ConfigSave::Retry);
        };
        if self.load_state.blocks_save() || current_stamp != expected_stamp {
            return Ok(ConfigSave::Retry);
        }
        let durability = save_durability(
            intent,
            expected_stamp.is_some(),
            self.load_state,
            &self.persisted,
            config,
        );
        if !config.save_if_unchanged(expected_stamp, durability)? {
            return Ok(ConfigSave::Retry);
        }

        // A writer can replace the file immediately after our atomic save. Keeping the stamp
        // unknown forces the next operation to load and merge instead of treating that writer's
        // stamp as if it belonged to the configuration we just persisted.
        self.stamp = ConfigStampState::Unknown;
        self.persisted = config.clone();
        self.next_check = Instant::now() + self.reload_interval;
        self.load_state = ConfigLoadState::Ready;
        Ok(ConfigSave::Saved)
    }

    fn reload_needed(&self, current: Option<ConfigFileStamp>) -> bool {
        self.load_state != ConfigLoadState::Ready
            || match self.stamp {
                ConfigStampState::Unknown => true,
                ConfigStampState::Known(cached) => current != cached,
            }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigLoadState, ConfigSaveIntent, save_durability};
    use crate::config::{AppConfig, ConfigDurability, WindowPosition};

    #[test]
    fn malformed_config_is_the_only_failed_load_state_that_allows_save() {
        assert!(!ConfigLoadState::Ready.blocks_save());
        assert!(ConfigLoadState::Failed.blocks_save());
        assert!(!ConfigLoadState::RecoverMalformed.blocks_save());
    }

    #[test]
    fn save_durability_matches_the_pending_change_contract() {
        let mut persisted = AppConfig::default();
        assert!(persisted.add_target("game.exe"));
        let mut runtime_update = persisted.clone();
        assert!(runtime_update.set_target_managed_muted_at(0, true));
        let mut settings_update = runtime_update.clone();
        settings_update.window_position = Some(WindowPosition { x: 10, y: 20 });

        let cases = [
            (
                "runtime mute state only",
                ConfigSaveIntent::RuntimeMuteState,
                true,
                ConfigLoadState::Ready,
                &runtime_update,
                ConfigDurability::ProcessCrashSafe,
            ),
            (
                "settings save intent",
                ConfigSaveIntent::Settings,
                true,
                ConfigLoadState::Ready,
                &runtime_update,
                ConfigDurability::Durable,
            ),
            (
                "pending user settings",
                ConfigSaveIntent::RuntimeMuteState,
                true,
                ConfigLoadState::Ready,
                &settings_update,
                ConfigDurability::Durable,
            ),
            (
                "missing configuration file",
                ConfigSaveIntent::RuntimeMuteState,
                false,
                ConfigLoadState::Ready,
                &runtime_update,
                ConfigDurability::Durable,
            ),
            (
                "malformed configuration recovery",
                ConfigSaveIntent::RuntimeMuteState,
                true,
                ConfigLoadState::RecoverMalformed,
                &runtime_update,
                ConfigDurability::Durable,
            ),
        ];

        for (case, intent, existing_file, load_state, config, expected) in cases {
            assert_eq!(
                save_durability(intent, existing_file, load_state, &persisted, config),
                expected,
                "{case}"
            );
        }
    }
}
