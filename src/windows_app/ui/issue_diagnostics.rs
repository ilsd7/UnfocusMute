use super::state::StatusIssue;

#[derive(Default)]
pub(super) struct IssueDiagnostics {
    details: Vec<IssueDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IssueDiagnostic {
    issue: StatusIssue,
    detail: String,
}

impl IssueDiagnostics {
    pub(super) fn set(&mut self, issue: StatusIssue, detail: impl Into<String>) -> bool {
        let Some(detail) = normalized_detail(detail.into()) else {
            return self.clear(issue);
        };

        match self.details.iter_mut().find(|entry| entry.issue == issue) {
            Some(entry) if entry.detail == detail => false,
            Some(entry) => {
                entry.detail = detail;
                true
            }
            None => {
                self.details.push(IssueDiagnostic { issue, detail });
                true
            }
        }
    }

    pub(super) fn clear(&mut self, issue: StatusIssue) -> bool {
        let Some(index) = self.details.iter().position(|entry| entry.issue == issue) else {
            return false;
        };
        self.details.remove(index);
        true
    }

    pub(super) fn clear_mask(&mut self, mask: u16) -> bool {
        let old_len = self.details.len();
        self.details.retain(|entry| entry.issue.bit() & mask == 0);
        old_len != self.details.len()
    }

    pub(super) fn detail(&self, issue: StatusIssue) -> Option<&str> {
        self.details
            .iter()
            .find(|entry| entry.issue == issue)
            .map(|entry| entry.detail.as_str())
    }
}

fn normalized_detail(detail: String) -> Option<String> {
    if detail.trim().is_empty() {
        return None;
    }

    Some(detail.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detail_is_stored_by_issue() {
        let mut diagnostics = IssueDiagnostics::default();

        assert!(diagnostics.set(StatusIssue::AudioUnavailable, "core audio failed"));

        assert_eq!(
            diagnostics.detail(StatusIssue::AudioUnavailable),
            Some("core audio failed")
        );
        assert_eq!(diagnostics.detail(StatusIssue::ConfigSaveFailed), None);
    }

    #[test]
    fn blank_detail_clears_existing_issue() {
        let mut diagnostics = IssueDiagnostics::default();
        diagnostics.set(StatusIssue::AudioUnavailable, "core audio failed");

        assert!(diagnostics.set(StatusIssue::AudioUnavailable, "  "));

        assert_eq!(diagnostics.detail(StatusIssue::AudioUnavailable), None);
    }

    #[test]
    fn clear_mask_removes_matching_issues() {
        let mut diagnostics = IssueDiagnostics::default();
        diagnostics.set(StatusIssue::AudioUnavailable, "audio");
        diagnostics.set(StatusIssue::ConfigSaveFailed, "config");

        assert!(diagnostics.clear_mask(StatusIssue::AudioUnavailable.bit()));

        assert_eq!(diagnostics.detail(StatusIssue::AudioUnavailable), None);
        assert_eq!(
            diagnostics.detail(StatusIssue::ConfigSaveFailed),
            Some("config")
        );
    }

    #[test]
    fn long_detail_is_preserved_in_full() {
        let detail = "x".repeat(10_000);
        let mut diagnostics = IssueDiagnostics::default();

        diagnostics.set(StatusIssue::AudioUpdateFailed, detail.clone());

        assert_eq!(
            diagnostics.detail(StatusIssue::AudioUpdateFailed),
            Some(detail.as_str())
        );
    }
}
