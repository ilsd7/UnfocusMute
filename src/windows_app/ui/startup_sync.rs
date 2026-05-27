use super::state::{IssueState, StatusIssue};
use crate::config::AppConfig;
use crate::windows_app::startup;

#[derive(Default)]
pub(super) struct StartupSyncResult {
    pub(super) issues: IssueState,
    pub(super) issue_detail: Option<String>,
    pub(super) config_changed: bool,
}

pub(super) fn sync_startup_setting(config: &mut AppConfig) -> StartupSyncResult {
    let update_error =
        startup::set_launch_on_startup(config.launch_on_startup, config.start_minimized)
            .err()
            .map(|error| error.to_string());
    apply_startup_sync_result(config, update_error)
}

pub(super) fn apply_startup_preference(
    config: &mut AppConfig,
    launch_on_startup: bool,
) -> std::result::Result<(), String> {
    startup::set_launch_on_startup(launch_on_startup, config.start_minimized)
        .map_err(|error| error.to_string())?;

    config.launch_on_startup = launch_on_startup;
    Ok(())
}

pub(super) fn apply_startup_command_preference(
    config: &AppConfig,
) -> std::result::Result<(), String> {
    if !config.launch_on_startup {
        return Ok(());
    }
    startup::set_launch_on_startup(true, config.start_minimized).map_err(|error| error.to_string())
}

pub(super) fn apply_external_startup_config(
    config: &mut AppConfig,
    previous_launch_on_startup: bool,
) -> std::result::Result<(), String> {
    match startup::set_launch_on_startup(config.launch_on_startup, config.start_minimized) {
        Ok(()) => Ok(()),
        Err(error) => {
            config.launch_on_startup = previous_launch_on_startup;
            Err(error.to_string())
        }
    }
}

fn apply_startup_sync_result(
    config: &mut AppConfig,
    update_error: Option<String>,
) -> StartupSyncResult {
    let mut issues = IssueState::default();
    let mut config_changed = false;

    if update_error.is_some() {
        issues.set(StatusIssue::StartupUpdateFailed);
        if config.launch_on_startup {
            config.launch_on_startup = false;
            config_changed = true;
        }
    }

    StartupSyncResult {
        issues,
        issue_detail: update_error,
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

        let result = apply_startup_sync_result(&mut config, Some("registry failed".to_owned()));

        assert!(!config.launch_on_startup);
        assert!(result.config_changed);
        assert!(result.issues.contains(StatusIssue::StartupUpdateFailed));
        assert_eq!(result.issue_detail.as_deref(), Some("registry failed"));
    }

    #[test]
    fn disabling_failure_reports_issue_without_reenabling_config() {
        let mut config = AppConfig {
            launch_on_startup: false,
            ..AppConfig::default()
        };

        let result = apply_startup_sync_result(&mut config, Some("registry failed".to_owned()));

        assert!(!config.launch_on_startup);
        assert!(!result.config_changed);
        assert!(result.issues.contains(StatusIssue::StartupUpdateFailed));
        assert_eq!(result.issue_detail.as_deref(), Some("registry failed"));
    }

    #[test]
    fn successful_sync_preserves_startup_config() {
        let mut config = AppConfig {
            launch_on_startup: true,
            ..AppConfig::default()
        };

        let result = apply_startup_sync_result(&mut config, None);

        assert!(config.launch_on_startup);
        assert!(!result.config_changed);
        assert!(!result.issues.contains(StatusIssue::StartupUpdateFailed));
        assert_eq!(result.issue_detail, None);
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

        assert!(apply_startup_command_preference(&config).is_ok());
    }
}
