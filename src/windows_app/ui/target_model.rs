use super::win32::storage_bytes_hint;
use crate::config::TargetProcess;
use crate::i18n::Strings;
use std::mem::size_of;

const PID_DISPLAY_DECORATION_UTF16_UNITS: usize = " (PID )".len();

pub(super) fn target_display_name_into(
    target: &TargetProcess,
    strings: &Strings,
    output: &mut String,
) {
    if target.enabled {
        target.display_name_into(output);
        return;
    }

    output.clear();
    output.push_str(strings.target_paused_prefix);
    target.push_identity_into(output);
    if let Some(note) = &target.note {
        output.push_str(" - ");
        output.push_str(note);
    }
}

pub(super) fn target_matcher_inputs_changed(
    left: &[TargetProcess],
    right: &[TargetProcess],
) -> bool {
    left.len() != right.len()
        || left.iter().zip(right).any(|(left, right)| {
            left.name != right.name || left.pid != right.pid || left.enabled != right.enabled
        })
}

pub(super) fn target_display_storage_bytes_hint(
    target: &TargetProcess,
    strings: &Strings,
) -> usize {
    let mut bytes = storage_bytes_hint(&target.name);
    if !target.enabled {
        bytes += strings.target_paused_prefix.encode_utf16().count() * size_of::<u16>();
    }
    if let Some(pid) = target.pid {
        bytes += (PID_DISPLAY_DECORATION_UTF16_UNITS + decimal_digit_count(pid)) * size_of::<u16>();
    }
    if let Some(note) = &target.note {
        bytes += (" - ".len() + note.encode_utf16().count()) * size_of::<u16>();
    }
    bytes
}

fn decimal_digit_count(value: u32) -> usize {
    if value == 0 {
        return 1;
    }
    value.ilog10() as usize + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn target_display_storage_hint_counts_one_nul_for_plain_target() {
        let target = TargetProcess::new("abc.exe").unwrap();

        assert_eq!(
            target_display_storage_bytes_hint(&target, Language::En.strings()),
            ("abc.exe".encode_utf16().count() + 1) * size_of::<u16>()
        );
    }

    #[test]
    fn target_display_storage_hint_counts_one_nul_for_pid_target() {
        let target = TargetProcess::for_pid("abc.exe", 42).unwrap();
        let mut display_name = String::new();
        target.display_name_into(&mut display_name);

        assert_eq!(
            target_display_storage_bytes_hint(&target, Language::En.strings()),
            (display_name.encode_utf16().count() + 1) * size_of::<u16>()
        );
    }

    #[test]
    fn target_display_storage_hint_counts_note_text() {
        let mut target = TargetProcess::new("abc.exe").unwrap();
        target.note = Some("게임".to_owned());
        let mut display_name = String::new();
        target.display_name_into(&mut display_name);

        assert_eq!(
            target_display_storage_bytes_hint(&target, Language::En.strings()),
            (display_name.encode_utf16().count() + 1) * size_of::<u16>()
        );
    }

    #[test]
    fn target_display_storage_hint_counts_paused_prefix() {
        let mut target = TargetProcess::new("abc.exe").unwrap();
        target.enabled = false;
        let strings = Language::Ko.strings();
        let mut display_name = String::new();
        target_display_name_into(&target, strings, &mut display_name);

        assert_eq!(
            target_display_storage_bytes_hint(&target, strings),
            (display_name.encode_utf16().count() + 1) * size_of::<u16>()
        );
    }

    #[test]
    fn target_matcher_inputs_ignore_note_only_changes() {
        let mut left = TargetProcess::new("abc.exe").unwrap();
        let mut right = left.clone();
        left.note = Some("before".to_owned());
        right.note = Some("after".to_owned());

        assert!(!target_matcher_inputs_changed(&[left], &[right]));
    }

    #[test]
    fn target_matcher_inputs_include_identity_and_enabled_changes() {
        let base = TargetProcess::new("abc.exe").unwrap();
        let mut renamed = base.clone();
        renamed.name = "other.exe".to_owned();
        let mut pid_target = base.clone();
        pid_target.pid = Some(42);
        let mut disabled = base.clone();
        disabled.enabled = false;

        assert!(target_matcher_inputs_changed(
            std::slice::from_ref(&base),
            &[renamed]
        ));
        assert!(target_matcher_inputs_changed(
            std::slice::from_ref(&base),
            &[pid_target]
        ));
        assert!(target_matcher_inputs_changed(&[base], &[disabled]));
    }
}
