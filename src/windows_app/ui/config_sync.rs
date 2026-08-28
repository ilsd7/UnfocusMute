use super::*;

#[derive(Clone, Copy)]
struct ConfigChangeEffects {
    language_changed: bool,
    theme_changed: bool,
    target_list_changed: bool,
    target_matcher_changed: bool,
    interval_changed: bool,
    should_schedule_managed_mute_recovery: bool,
}

impl ConfigChangeEffects {
    fn between(previous: &AppConfig, next: &AppConfig) -> Self {
        Self {
            language_changed: previous.language != next.language,
            theme_changed: previous.theme != next.theme,
            target_list_changed: previous.targets != next.targets,
            target_matcher_changed: target_matcher_inputs_changed(&previous.targets, &next.targets),
            interval_changed: previous.polling_interval_ms != next.polling_interval_ms,
            should_schedule_managed_mute_recovery: next
                .targets
                .iter()
                .any(|target| target.managed_muted)
                && !previous.targets.iter().any(|target| target.managed_muted),
        }
    }
}

fn startup_setting_changed(previous: &AppConfig, next: &AppConfig) -> bool {
    previous.launch_on_startup != next.launch_on_startup
        || (next.launch_on_startup && previous.start_minimized != next.start_minimized)
}

impl AppWindow {
    pub(super) fn reload_config_if_due(&mut self) -> ConfigReloadResult {
        let reload = self.config_store.reload_if_due();
        self.apply_config_reload(reload)
    }

    pub(super) fn reload_config_if_changed(&mut self) -> ConfigReloadResult {
        let reload = self.config_store.reload_if_changed();
        self.apply_config_reload(reload)
    }

    fn apply_config_reload(&mut self, reload: ConfigReload) -> ConfigReloadResult {
        self.apply_config_reload_with_startup_sync(reload, true)
    }

    pub(super) fn apply_config_reload_without_startup_sync(
        &mut self,
        reload: ConfigReload,
    ) -> ConfigReloadResult {
        self.apply_config_reload_with_startup_sync(reload, false)
    }

    fn apply_config_reload_with_startup_sync(
        &mut self,
        reload: ConfigReload,
        sync_startup: bool,
    ) -> ConfigReloadResult {
        match reload {
            ConfigReload::Unchanged => ConfigReloadResult::UNCHANGED,
            ConfigReload::Loaded(mut config) => {
                self.clear_issue(StatusIssue::ConfigLoadFailed);
                let persisted_config = self
                    .runtime
                    .runtime_mute_state_dirty()
                    .then(|| config.clone());
                self.preserve_dirty_runtime_mute_state(&mut config);
                let target_matcher_changed = self.apply_external_config(config, sync_startup);
                if let Some(persisted_config) = persisted_config {
                    self.config_store.accept_loaded(&persisted_config);
                } else {
                    self.config_store.accept_loaded(&self.config);
                }
                ConfigReloadResult::changed(target_matcher_changed)
            }
            ConfigReload::Missing => {
                self.clear_issue(StatusIssue::ConfigLoadFailed);
                ConfigReloadResult::UNCHANGED
            }
            ConfigReload::Failed(error) => {
                self.set_issue_with_detail(StatusIssue::ConfigLoadFailed, error.to_string());
                ConfigReloadResult::UNCHANGED
            }
        }
    }

    fn apply_external_config(&mut self, mut config: AppConfig, sync_startup: bool) -> bool {
        let previous_config = self.config.clone();
        if sync_startup && startup_setting_changed(&previous_config, &config) {
            let startup_sync =
                sync_external_startup_config(&mut config, previous_config.launch_on_startup);
            self.update_startup_sync_issue(startup_sync);
        }

        let effects = ConfigChangeEffects::between(&previous_config, &config);
        self.config = config;
        self.apply_config_change_effects(effects);
        effects.target_matcher_changed
    }

    pub(super) fn save_config(&mut self) -> bool {
        self.save_config_with_intent(ConfigSaveIntent::Settings)
    }

    pub(super) fn save_runtime_mute_state(&mut self) -> bool {
        self.save_config_with_intent(ConfigSaveIntent::RuntimeMuteState)
    }

    pub(super) fn retry_runtime_mute_state_save(&mut self) {
        if self.runtime.runtime_mute_state_dirty() {
            self.save_runtime_mute_state();
        }
    }

    fn preserve_dirty_runtime_mute_state(&self, config: &mut AppConfig) {
        if !self.runtime.runtime_mute_state_dirty() {
            return;
        }
        for target in &self.config.targets {
            let Some(index) = target_index_by_identity(&config.targets, &target.name, target.pid)
            else {
                continue;
            };
            config.targets[index].managed_muted = target.managed_muted;
        }
    }

    fn save_config_with_intent(&mut self, intent: ConfigSaveIntent) -> bool {
        const SAVE_RETRY_LIMIT: usize = 3;

        for _ in 0..SAVE_RETRY_LIMIT {
            match self.config_store.merge_external_before_save(&self.config) {
                Ok(ConfigMerge::Ready(Some(disk_config))) => {
                    let previous_config = std::mem::replace(&mut self.config, disk_config);
                    self.refresh_after_external_save_merge(&previous_config);
                }
                Ok(ConfigMerge::Ready(None)) => {}
                Ok(ConfigMerge::Retry) => continue,
                Err(error) => {
                    self.set_issue_with_detail(StatusIssue::ConfigSaveFailed, error.to_string());
                    return false;
                }
            }

            match self.config_store.save(&self.config, intent) {
                Ok(ConfigSave::Saved) => {
                    self.window_placement_dirty = false;
                    self.runtime.mark_runtime_mute_state_saved();
                    self.clear_config_issues();
                    return true;
                }
                Ok(ConfigSave::Retry) => {}
                Err(error) => {
                    self.set_issue_with_detail(StatusIssue::ConfigSaveFailed, error.to_string());
                    return false;
                }
            }
        }

        self.set_issue_with_detail(
            StatusIssue::ConfigSaveFailed,
            "config file kept changing while the app was saving",
        );
        false
    }

    fn refresh_after_external_save_merge(&mut self, previous_config: &AppConfig) {
        if startup_setting_changed(previous_config, &self.config) {
            let startup_sync =
                sync_external_startup_config(&mut self.config, previous_config.launch_on_startup);
            self.update_startup_sync_issue(startup_sync);
        }

        let effects = ConfigChangeEffects::between(previous_config, &self.config);
        self.apply_config_change_effects(effects);
    }

    fn update_startup_sync_issue(&mut self, startup_sync: std::result::Result<(), String>) {
        match startup_sync {
            Ok(()) => self.clear_issue(StatusIssue::StartupUpdateFailed),
            Err(error) => self.set_issue_with_detail(StatusIssue::StartupUpdateFailed, error),
        }
    }

    fn apply_config_change_effects(&mut self, effects: ConfigChangeEffects) {
        if effects.theme_changed {
            self.apply_resolved_theme(resolve_theme(self.config.theme));
        }
        if effects.should_schedule_managed_mute_recovery {
            self.runtime.schedule_managed_mute_foreground_retry(true);
        }
        if effects.target_matcher_changed {
            self.refresh_targets();
        } else if effects.target_list_changed {
            self.refresh_target_list();
        }
        if effects.interval_changed {
            self.reset_polling_timer();
            self.last_status = None;
        } else if effects.target_matcher_changed || effects.should_schedule_managed_mute_recovery {
            self.sync_audio_fallback_timer();
        }
        if effects.language_changed {
            self.refresh_text();
        } else {
            self.update_status();
        }
    }

    fn clear_config_issues(&mut self) {
        if self.clear_issue_mask(active_config_failure_mask()) {
            self.last_status = None;
            self.update_status();
        }
    }
}
