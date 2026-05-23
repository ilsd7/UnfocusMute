use super::state::{IssueState, StatusIssue};
use crate::config::AppConfig;
use crate::windows_app::startup;

#[derive(Default)]
pub(super) struct StartupSyncResult {
    pub(super) issues: IssueState,
    pub(super) config_changed: bool,
}

pub(super) fn sync_startup_setting(config: &mut AppConfig) -> StartupSyncResult {
    apply_startup_sync_result(
        config,
        startup::set_launch_on_startup(config.launch_on_startup, config.start_minimized).is_err(),
    )
}

pub(super) fn apply_startup_preference(config: &mut AppConfig, launch_on_startup: bool) -> bool {
    if startup::set_launch_on_startup(launch_on_startup, config.start_minimized).is_err() {
        return false;
    }

    config.launch_on_startup = launch_on_startup;
    true
}

pub(super) fn apply_startup_command_preference(config: &AppConfig) -> bool {
    !config.launch_on_startup
        || startup::set_launch_on_startup(true, config.start_minimized).is_ok()
}

pub(super) fn apply_external_startup_config(
    config: &mut AppConfig,
    previous_launch_on_startup: bool,
) -> bool {
    if startup::set_launch_on_startup(config.launch_on_startup, config.start_minimized).is_ok() {
        return true;
    }

    config.launch_on_startup = previous_launch_on_startup;
    false
}

fn apply_startup_sync_result(config: &mut AppConfig, update_failed: bool) -> StartupSyncResult {
    let mut issues = IssueState::default();
    let mut config_changed = false;

    if update_failed {
        issues.set(StatusIssue::StartupUpdateFailed);
        if config.launch_on_startup {
            config.launch_on_startup = false;
            config_changed = true;
        }
    }

    StartupSyncResult {
        issues,
        config_changed,
    }
}

pub(super) fn should_save_startup_config(
    accepted_initial_preferences: bool,
    startup_config_changed: bool,
) -> bool {
    accepted_initial_preferences || startup_config_changed
}

pub(super) fn should_sync_startup_setting(
    first_run: bool,
    accepted_initial_preferences: bool,
) -> bool {
    !first_run || accepted_initial_preferences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabling_failure_disables_config_and_reports_issue() {
        let mut config = AppConfig {
            launch_on_startup: true,
            ..AppConfig::default()
        };

        let result = apply_startup_sync_result(&mut config, true);

        assert!(!config.launch_on_startup);
        assert!(result.config_changed);
        assert!(result.issues.contains(StatusIssue::StartupUpdateFailed));
    }

    #[test]
    fn disabling_failure_reports_issue_without_reenabling_config() {
        let mut config = AppConfig {
            launch_on_startup: false,
            ..AppConfig::default()
        };

        let result = apply_startup_sync_result(&mut config, true);

        assert!(!config.launch_on_startup);
        assert!(!result.config_changed);
        assert!(result.issues.contains(StatusIssue::StartupUpdateFailed));
    }

    #[test]
    fn successful_sync_preserves_startup_config() {
        let mut config = AppConfig {
            launch_on_startup: true,
            ..AppConfig::default()
        };

        let result = apply_startup_sync_result(&mut config, false);

        assert!(config.launch_on_startup);
        assert!(!result.config_changed);
        assert!(!result.issues.contains(StatusIssue::StartupUpdateFailed));
    }

    #[test]
    fn initial_config_is_saved_only_after_accept_or_sync_change() {
        assert!(!should_save_startup_config(false, false));
        assert!(should_save_startup_config(true, false));
        assert!(should_save_startup_config(false, true));
    }

    #[test]
    fn startup_sync_waits_for_first_run_acceptance() {
        assert!(!should_sync_startup_setting(true, false));
        assert!(should_sync_startup_setting(true, true));
        assert!(should_sync_startup_setting(false, false));
    }

    #[test]
    fn startup_command_sync_is_not_needed_when_startup_is_disabled() {
        let config = AppConfig {
            launch_on_startup: false,
            start_minimized: false,
            ..AppConfig::default()
        };

        assert!(apply_startup_command_preference(&config));
    }
}
