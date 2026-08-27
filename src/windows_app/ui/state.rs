#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct IssueState {
    flags: u16,
    ignored: u16,
}

impl IssueState {
    pub(super) fn set(&mut self, issue: StatusIssue) -> bool {
        let bit = issue.bit();
        if self.flags & bit != 0 {
            return false;
        }

        let old_visible = self.visible();
        self.flags |= bit;
        old_visible != self.visible()
    }

    pub(super) fn clear(&mut self, issue: StatusIssue) -> bool {
        self.clear_mask(issue.bit())
    }

    pub(super) fn clear_mask(&mut self, mask: u16) -> bool {
        if self.flags & mask == 0 {
            return false;
        }

        let old_visible = self.visible();
        self.flags &= !mask;
        old_visible != self.visible()
    }

    pub(super) fn merge(&mut self, issues: Self) -> bool {
        let flags = self.flags | issues.flags;
        if flags == self.flags {
            return false;
        }

        let old_visible = self.visible();
        self.flags = flags;
        old_visible != self.visible()
    }

    pub(super) fn contains(self, issue: StatusIssue) -> bool {
        self.flags & issue.bit() != 0
    }

    pub(super) fn ignore(&mut self, issue: StatusIssue) -> bool {
        if !issue.can_ignore() || !self.contains(issue) {
            return false;
        }

        let old_visible = self.visible();
        self.ignored |= issue.bit();
        old_visible != self.visible()
    }

    pub(super) fn visible(self) -> Option<StatusIssue> {
        self.visible_issues().next()
    }

    pub(super) fn visible_issues(self) -> impl Iterator<Item = StatusIssue> {
        STATUS_ISSUE_PRIORITY_ORDER
            .into_iter()
            .filter(move |issue| self.flags & !self.ignored & issue.bit() != 0)
    }

    pub(super) fn requires_attention(self) -> bool {
        self.visible_issues().any(|issue| !issue.can_ignore())
    }
}

const STATUS_ISSUE_PRIORITY_ORDER: [StatusIssue; 9] = [
    StatusIssue::AudioUnavailable,
    StatusIssue::AudioUpdateFailed,
    StatusIssue::TimerSetupFailed,
    StatusIssue::ForegroundHookUnavailable,
    StatusIssue::ConfigLoadFailed,
    StatusIssue::ConfigSaveFailed,
    StatusIssue::ConfigRecovered,
    StatusIssue::StartupUpdateFailed,
    StatusIssue::TrayIconUnavailable,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum StatusIssue {
    AudioUnavailable,
    AudioUpdateFailed,
    ConfigLoadFailed,
    ConfigRecovered,
    ConfigSaveFailed,
    StartupUpdateFailed,
    TimerSetupFailed,
    TrayIconUnavailable,
    ForegroundHookUnavailable,
}

impl StatusIssue {
    pub(super) fn bit(self) -> u16 {
        1 << self as u8
    }

    pub(super) fn can_ignore(self) -> bool {
        matches!(self, Self::StartupUpdateFailed | Self::TrayIconUnavailable)
    }

    pub(super) fn clears_on_acknowledge(self) -> bool {
        self == Self::ConfigRecovered
    }

    pub(super) fn code(self) -> &'static str {
        match self {
            Self::AudioUnavailable => "audio-unavailable",
            Self::AudioUpdateFailed => "audio-update-failed",
            Self::ConfigLoadFailed => "config-load-failed",
            Self::ConfigRecovered => "config-recovered",
            Self::ConfigSaveFailed => "config-save-failed",
            Self::StartupUpdateFailed => "startup-update-failed",
            Self::TimerSetupFailed => "timer-setup-failed",
            Self::TrayIconUnavailable => "tray-icon-unavailable",
            Self::ForegroundHookUnavailable => "foreground-hook-unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct StatusSnapshot {
    pub(super) paused: bool,
    pub(super) issue: Option<StatusIssue>,
    pub(super) target_count: usize,
    pub(super) muted_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ActionButtonState {
    pub(super) register: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ConfigReloadResult {
    pub(super) target_matcher_changed: bool,
}

impl ConfigReloadResult {
    pub(super) const UNCHANGED: Self = Self {
        target_matcher_changed: false,
    };

    pub(super) const fn changed(target_matcher_changed: bool) -> Self {
        Self {
            target_matcher_changed,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProcessRefreshResult {
    Refreshed,
    Unchanged,
    Skipped,
    Failed,
}

impl ProcessRefreshResult {
    pub(super) fn refreshed(self) -> bool {
        self == Self::Refreshed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clearing_visible_issue_reveals_hidden_issue() {
        let mut issues = IssueState::default();

        issues.set(StatusIssue::AudioUnavailable);
        issues.set(StatusIssue::ConfigSaveFailed);
        assert_eq!(issues.visible(), Some(StatusIssue::AudioUnavailable));

        issues.clear(StatusIssue::AudioUnavailable);
        assert_eq!(issues.visible(), Some(StatusIssue::ConfigSaveFailed));
    }

    #[test]
    fn lower_priority_issue_does_not_hide_visible_critical_issue() {
        let mut issues = IssueState::default();

        issues.set(StatusIssue::AudioUnavailable);
        issues.set(StatusIssue::ConfigSaveFailed);

        assert_eq!(issues.visible(), Some(StatusIssue::AudioUnavailable));
    }

    #[test]
    fn active_save_failure_precedes_recovered_config_notice() {
        let mut issues = IssueState::default();

        issues.set(StatusIssue::ConfigRecovered);
        issues.set(StatusIssue::ConfigSaveFailed);

        assert_eq!(issues.visible(), Some(StatusIssue::ConfigSaveFailed));
    }

    #[test]
    fn contains_reports_hidden_issues() {
        let mut issues = IssueState::default();

        issues.set(StatusIssue::AudioUnavailable);
        issues.set(StatusIssue::ConfigLoadFailed);

        assert!(issues.contains(StatusIssue::AudioUnavailable));
        assert!(issues.contains(StatusIssue::ConfigLoadFailed));
        assert!(!issues.contains(StatusIssue::ConfigSaveFailed));
    }

    #[test]
    fn merging_lower_priority_issue_preserves_visible_priority() {
        let mut issues = IssueState::default();
        let mut incoming = IssueState::default();

        issues.set(StatusIssue::AudioUnavailable);
        incoming.set(StatusIssue::ConfigLoadFailed);

        assert!(!issues.merge(incoming));
        assert!(issues.contains(StatusIssue::AudioUnavailable));
        assert!(issues.contains(StatusIssue::ConfigLoadFailed));
        assert_eq!(issues.visible(), Some(StatusIssue::AudioUnavailable));
    }

    #[test]
    fn ignored_safe_issue_stays_active_but_is_hidden_for_the_session() {
        let mut issues = IssueState::default();
        issues.set(StatusIssue::StartupUpdateFailed);

        assert!(issues.ignore(StatusIssue::StartupUpdateFailed));
        assert!(issues.contains(StatusIssue::StartupUpdateFailed));
        assert_eq!(issues.visible(), None);

        issues.clear(StatusIssue::StartupUpdateFailed);
        issues.set(StatusIssue::StartupUpdateFailed);
        assert_eq!(issues.visible(), None);
    }

    #[test]
    fn critical_issue_cannot_be_ignored() {
        let mut issues = IssueState::default();
        issues.set(StatusIssue::AudioUnavailable);

        assert!(!issues.ignore(StatusIssue::AudioUnavailable));
        assert_eq!(issues.visible(), Some(StatusIssue::AudioUnavailable));
    }

    #[test]
    fn ignoring_visible_issue_reveals_next_active_issue() {
        let mut issues = IssueState::default();
        issues.set(StatusIssue::StartupUpdateFailed);
        issues.set(StatusIssue::TrayIconUnavailable);

        assert!(issues.ignore(StatusIssue::StartupUpdateFailed));
        assert_eq!(issues.visible(), Some(StatusIssue::TrayIconUnavailable));
    }

    #[test]
    fn visible_issues_lists_every_active_issue_in_priority_order() {
        let mut issues = IssueState::default();
        issues.set(StatusIssue::ConfigSaveFailed);
        issues.set(StatusIssue::AudioUnavailable);
        issues.set(StatusIssue::TrayIconUnavailable);
        issues.ignore(StatusIssue::TrayIconUnavailable);

        assert_eq!(
            issues.visible_issues().collect::<Vec<_>>(),
            [StatusIssue::AudioUnavailable, StatusIssue::ConfigSaveFailed,]
        );
    }

    #[test]
    fn only_non_ignored_critical_issues_require_attention() {
        let mut issues = IssueState::default();
        issues.set(StatusIssue::StartupUpdateFailed);
        assert!(!issues.requires_attention());

        issues.set(StatusIssue::ConfigRecovered);
        assert!(issues.requires_attention());

        issues.clear(StatusIssue::ConfigRecovered);
        issues.ignore(StatusIssue::StartupUpdateFailed);
        assert!(!issues.requires_attention());
    }

    #[test]
    fn recovered_config_is_the_only_resolved_notice() {
        for issue in STATUS_ISSUE_PRIORITY_ORDER {
            assert_eq!(
                issue.clears_on_acknowledge(),
                issue == StatusIssue::ConfigRecovered
            );
        }
    }

    #[test]
    fn process_refresh_result_only_reports_refreshed_for_successful_refresh() {
        assert!(ProcessRefreshResult::Refreshed.refreshed());
        assert!(!ProcessRefreshResult::Unchanged.refreshed());
        assert!(!ProcessRefreshResult::Skipped.refreshed());
        assert!(!ProcessRefreshResult::Failed.refreshed());
    }
}
