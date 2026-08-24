use crate::i18n::{APP_TITLE, CountText, CountTextOrder, Strings};

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

    push_count_text(output, strings.target_count, target_count);
    output.push_str(" · ");
    push_count_text(output, strings.muted_count, muted_count);
}

pub(super) fn tray_tip_text_into(status: &str, detail: &str, output: &mut String) {
    output.clear();
    output.reserve(APP_TITLE.len() + status.len() + detail.len() + 8);
    output.push_str(APP_TITLE);
    output.push_str(" - ");
    output.push_str(status);
    if !detail.is_empty() {
        output.push_str(" | ");
        output.push_str(detail);
    }
}

#[cfg(test)]
fn status_summary_text_into(status: &str, detail: &str, output: &mut String) {
    output.clear();
    output.reserve(status.len() + detail.len() + 3);
    output.push_str(status);
    if !detail.is_empty() {
        output.push_str(" - ");
        output.push_str(detail);
    }
}

fn push_decimal(output: &mut String, mut number: usize) {
    let mut digits = [0u8; 20];
    let mut len = 0;
    loop {
        digits[len] = b'0' + (number % 10) as u8;
        len += 1;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    for digit in digits[..len].iter().rev() {
        output.push(*digit as char);
    }
}

fn push_count_text(output: &mut String, text: CountText, count: usize) {
    match text.order {
        CountTextOrder::LabelFirst => {
            output.push_str(text.label(count));
            output.push(' ');
            push_decimal(output, count);
        }
        CountTextOrder::LabelFirstTight => {
            output.push_str(text.label(count));
            push_decimal(output, count);
        }
        CountTextOrder::CountFirst => {
            push_decimal(output, count);
            output.push(' ');
            output.push_str(text.label(count));
        }
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

        assert_eq!(detail, "등록 앱 12 · 음소거 중 3");
    }

    #[test]
    fn localized_status_detail_labels_read_naturally() {
        let cases = [
            (Language::En, "Registered apps: 12 · Muted: 3"),
            (Language::Ja, "登録アプリ 12 · ミュート中 3"),
            (Language::ZhHans, "已添加应用：12 · 已静音：3"),
            (Language::Es, "12 apps registradas · 3 apps silenciadas"),
            (
                Language::Fr,
                "12 applications ajoutées · 3 applications au son coupé",
            ),
            (Language::Pt, "12 apps registrados · 3 apps silenciados"),
            (Language::Hi, "12 रजिस्टर किए गए ऐप · 3 म्यूट किए गए ऐप"),
            (
                Language::Ar,
                "التطبيقات المسجلة: 12 · التطبيقات المكتومة: 3",
            ),
        ];

        let mut detail = String::new();
        for (language, expected) in cases {
            status_detail_text_into(language.strings(), None, 12, 3, &mut detail);
            assert_eq!(detail, expected);
        }
    }

    #[test]
    fn localized_status_detail_uses_singular_count_labels() {
        let cases = [
            (Language::Es, "1 app registrada · 1 app silenciada"),
            (
                Language::Fr,
                "1 application ajoutée · 1 application au son coupé",
            ),
            (Language::Pt, "1 app registrado · 1 app silenciado"),
            (Language::Hi, "1 रजिस्टर किया गया ऐप · 1 म्यूट किया गया ऐप"),
        ];

        let mut detail = String::new();
        for (language, expected) in cases {
            status_detail_text_into(language.strings(), None, 1, 1, &mut detail);
            assert_eq!(detail, expected);
        }
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
            strings.status_running,
            "Registered apps: 2 · Muted: 1",
            &mut tip,
        );

        assert_eq!(
            tip,
            "UnfocusMute - Monitoring | Registered apps: 2 · Muted: 1"
        );
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
