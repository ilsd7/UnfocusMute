use crate::config::{
    AppConfig, ConfigFileStamp, current_config_stamp, merge_pending_config_changes,
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

#[derive(Clone, Copy)]
enum ConfigStampState {
    Unknown,
    Known(Option<ConfigFileStamp>),
}

pub(super) struct ConfigStore {
    persisted: AppConfig,
    stamp: ConfigStampState,
    next_check: Instant,
    reload_interval: Duration,
    load_failed: bool,
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
            load_failed: initial_load_failed,
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
            match AppConfig::load_existing() {
                Ok(config) => {
                    if current_config_stamp() != stamp_before_load {
                        continue;
                    }
                    self.stamp = ConfigStampState::Known(stamp_before_load);
                    self.load_failed = false;
                    return ConfigReload::Loaded(config);
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    if current_config_stamp() != stamp_before_load {
                        continue;
                    }
                    self.stamp = ConfigStampState::Known(None);
                    self.load_failed = false;
                    return ConfigReload::Missing;
                }
                Err(error) => {
                    self.stamp = ConfigStampState::Known(stamp_before_load);
                    self.load_failed = true;
                    return ConfigReload::Failed(error);
                }
            }
        }

        self.load_failed = true;
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

        let mut disk_config = match AppConfig::load_existing() {
            Ok(config) => config,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.stamp = ConfigStampState::Known(None);
                self.load_failed = false;
                return Ok(ConfigMerge::Ready(None));
            }
            Err(error) => {
                self.stamp = ConfigStampState::Known(stamp_before_load);
                self.load_failed = true;
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
        self.load_failed = false;
        Ok(ConfigMerge::Ready(
            (disk_config != *local).then_some(disk_config),
        ))
    }

    pub(super) fn save(&mut self, config: &AppConfig) -> io::Result<ConfigSave> {
        let current_stamp = current_config_stamp();
        let ConfigStampState::Known(expected_stamp) = self.stamp else {
            return Ok(ConfigSave::Retry);
        };
        if self.load_failed || current_stamp != expected_stamp {
            return Ok(ConfigSave::Retry);
        }
        if !config.save_if_unchanged(expected_stamp)? {
            return Ok(ConfigSave::Retry);
        }

        // A writer can replace the file immediately after our atomic save. Keeping the stamp
        // unknown forces the next operation to load and merge instead of treating that writer's
        // stamp as if it belonged to the configuration we just persisted.
        self.stamp = ConfigStampState::Unknown;
        self.persisted = config.clone();
        self.next_check = Instant::now() + self.reload_interval;
        self.load_failed = false;
        Ok(ConfigSave::Saved)
    }

    fn reload_needed(&self, current: Option<ConfigFileStamp>) -> bool {
        self.load_failed
            || match self.stamp {
                ConfigStampState::Unknown => true,
                ConfigStampState::Known(cached) => current != cached,
            }
    }
}
