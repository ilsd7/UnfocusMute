#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct IssueState {
    flags: u8,
    visible: Option<StatusIssue>,
}

impl IssueState {
    pub(super) fn set(&mut self, issue: StatusIssue) -> bool {
        let bit = issue.bit();
        if self.flags & bit != 0 {
            return false;
        }

        let old_visible = self.visible;
        self.flags |= bit;
        self.refresh_visible();
        old_visible != self.visible
    }

    pub(super) fn clear(&mut self, issue: StatusIssue) -> bool {
        self.clear_mask(issue.bit())
    }

    pub(super) fn clear_mask(&mut self, mask: u8) -> bool {
        if self.flags & mask == 0 {
            return false;
        }

        let old_visible = self.visible;
        self.flags &= !mask;
        self.refresh_visible();
        old_visible != self.visible
    }

    pub(super) fn merge(&mut self, issues: Self) -> bool {
        let flags = self.flags | issues.flags;
        if flags == self.flags {
            return false;
        }

        let old_visible = self.visible;
        self.flags = flags;
        self.refresh_visible();
        old_visible != self.visible
    }

    pub(super) fn contains(self, issue: StatusIssue) -> bool {
        self.flags & issue.bit() != 0
    }

    pub(super) fn visible(self) -> Option<StatusIssue> {
        self.visible
    }

    fn refresh_visible(&mut self) {
        self.visible = STATUS_ISSUE_PRIORITY_ORDER
            .iter()
            .copied()
            .find(|issue| self.flags & issue.bit() != 0);
    }
}

const STATUS_ISSUE_PRIORITY_ORDER: [StatusIssue; 7] = [
    StatusIssue::ConfigSaveFailed,
    StatusIssue::ConfigLoadFailed,
    StatusIssue::StartupUpdateFailed,
    StatusIssue::TimerSetupFailed,
    StatusIssue::TrayIconUnavailable,
    StatusIssue::AudioUnavailable,
    StatusIssue::AudioUpdateFailed,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum StatusIssue {
    AudioUnavailable,
    AudioUpdateFailed,
    ConfigLoadFailed,
    ConfigSaveFailed,
    StartupUpdateFailed,
    TimerSetupFailed,
    TrayIconUnavailable,
}

impl StatusIssue {
    pub(super) fn bit(self) -> u8 {
        1 << self as u8
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
    pub(super) add_selected: bool,
    pub(super) add_manual: bool,
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
        assert_eq!(issues.visible(), Some(StatusIssue::ConfigSaveFailed));

        issues.clear(StatusIssue::ConfigSaveFailed);
        assert_eq!(issues.visible(), Some(StatusIssue::AudioUnavailable));
    }

    #[test]
    fn lower_priority_issue_does_not_hide_visible_critical_issue() {
        let mut issues = IssueState::default();

        issues.set(StatusIssue::ConfigSaveFailed);
        issues.set(StatusIssue::AudioUnavailable);

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
    fn merging_issues_preserves_priority_order() {
        let mut issues = IssueState::default();
        let mut incoming = IssueState::default();

        issues.set(StatusIssue::AudioUnavailable);
        incoming.set(StatusIssue::ConfigLoadFailed);

        assert!(issues.merge(incoming));
        assert!(issues.contains(StatusIssue::AudioUnavailable));
        assert!(issues.contains(StatusIssue::ConfigLoadFailed));
        assert_eq!(issues.visible(), Some(StatusIssue::ConfigLoadFailed));
    }

    #[test]
    fn process_refresh_result_only_reports_refreshed_for_successful_refresh() {
        assert!(ProcessRefreshResult::Refreshed.refreshed());
        assert!(!ProcessRefreshResult::Unchanged.refreshed());
        assert!(!ProcessRefreshResult::Skipped.refreshed());
        assert!(!ProcessRefreshResult::Failed.refreshed());
    }
}
