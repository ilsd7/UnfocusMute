use crate::i18n::Strings;
use std::fmt::Write as _;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(super) fn status_text(strings: &Strings, paused: bool, has_issue: bool) -> &'static str {
    if has_issue {
        strings.status_issue
    } else if paused {
        strings.status_paused
    } else {
        strings.status_running
    }
}

pub(super) fn status_text_and_detail_into(
    strings: &Strings,
    issue_text: Option<&str>,
    paused: bool,
    target_count: usize,
    muted_count: usize,
    output: &mut String,
) -> &'static str {
    let status = status_text(strings, paused, issue_text.is_some());
    status_detail_text_into(strings, issue_text, target_count, muted_count, output);
    status
}

pub(super) fn status_detail_text_into(
    strings: &Strings,
    issue_text: Option<&str>,
    target_count: usize,
    muted_count: usize,
    output: &mut String,
) {
    output.clear();
    if let Some(issue_text) = issue_text {
        output.push_str(issue_text);
        return;
    }

    output.push_str(strings.target_count);
    output.push(' ');
    let _ = write!(output, "{target_count}");
    output.push_str(" · ");
    output.push_str(strings.muted_count);
    output.push(' ');
    let _ = write!(output, "{muted_count}");
}

pub(super) fn tray_tip_text_into(
    strings: &Strings,
    status: &str,
    detail: &str,
    output: &mut String,
) {
    output.clear();
    output.reserve(strings.app_title.len() + APP_VERSION.len() + status.len() + detail.len() + 8);
    output.push_str(strings.app_title);
    output.push_str(" - ");
    output.push_str(status);
    if !detail.is_empty() {
        output.push_str(" | ");
        output.push_str(detail);
    }
}

pub(super) fn app_title_with_version_into(strings: &Strings, output: &mut String) {
    output.clear();
    output.push_str(strings.app_title);
    output.push_str(" v");
    output.push_str(APP_VERSION);
}

pub(super) fn status_summary_text_into(status: &str, detail: &str, output: &mut String) {
    output.clear();
    output.reserve(status.len() + detail.len() + 3);
    output.push_str(status);
    if !detail.is_empty() {
        output.push_str(" - ");
        output.push_str(detail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn running_status_detail_includes_target_and_muted_counts() {
        let strings = Language::Ko.strings();
        let mut detail = String::new();

        status_detail_text_into(strings, None, 12, 3, &mut detail);

        assert_eq!(detail, "등록 12 · 음소거 3");
    }

    #[test]
    fn issue_status_detail_prefers_issue_text() {
        let strings = Language::En.strings();
        let mut detail = String::new();

        status_detail_text_into(
            strings,
            Some("Could not access audio sessions"),
            12,
            3,
            &mut detail,
        );

        assert_eq!(detail, "Could not access audio sessions");
    }

    #[test]
    fn status_text_and_detail_share_issue_decision() {
        let strings = Language::En.strings();
        let mut detail = String::new();

        let status = status_text_and_detail_into(
            strings,
            Some(strings.audio_unavailable),
            true,
            12,
            3,
            &mut detail,
        );

        assert_eq!(status, strings.status_issue);
        assert_eq!(detail, "Audio device unavailable");
    }

    #[test]
    fn tray_tip_combines_app_status_and_detail() {
        let strings = Language::En.strings();
        let mut tip = String::new();

        tray_tip_text_into(
            strings,
            strings.status_running,
            "Apps 2 · Muted 1",
            &mut tip,
        );

        assert_eq!(tip, "UnfocusMute - Monitoring | Apps 2 · Muted 1");
    }

    #[test]
    fn app_title_includes_package_version() {
        let strings = Language::Ko.strings();
        let mut title = String::new();

        app_title_with_version_into(strings, &mut title);

        assert_eq!(title, format!("UnfocusMute v{APP_VERSION}"));
    }

    #[test]
    fn status_summary_combines_status_and_detail_for_tray_menu() {
        let mut summary = String::new();

        status_summary_text_into("모니터링 중", "등록 2 · 음소거 1", &mut summary);

        assert_eq!(summary, "모니터링 중 - 등록 2 · 음소거 1");
    }

    #[test]
    fn status_text_prioritizes_issue_over_pause() {
        let strings = Language::En.strings();

        assert_eq!(status_text(strings, true, true), strings.status_issue);
        assert_eq!(status_text(strings, true, false), strings.status_paused);
        assert_eq!(status_text(strings, false, false), strings.status_running);
    }
}
