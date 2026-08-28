use crate::config::TargetProcess;
use crate::engine::AudioSessionKey;
use crate::i18n::Strings;
use std::collections::HashSet;

pub(super) fn target_status_text(
    target: &TargetProcess,
    muted_by_app: bool,
    strings: &Strings,
) -> &'static str {
    if !target.enabled {
        strings.target_excluded
    } else if muted_by_app {
        strings.target_muted
    } else {
        strings.target_ready
    }
}

pub(super) fn session_keys_exclusive_to_target(
    target: &TargetProcess,
    targets: &[TargetProcess],
    muted_by_app: &HashSet<AudioSessionKey>,
) -> HashSet<AudioSessionKey> {
    muted_by_app
        .iter()
        .filter(|key| {
            target.matches_session(&key.process_name, key.pid)
                && !targets.iter().any(|other| {
                    other.enabled
                        && (other.name != target.name || other.pid != target.pid)
                        && other.matches_session(&key.process_name, key.pid)
                })
        })
        .cloned()
        .collect()
}

pub(super) fn removal_restore_targets(
    target: &TargetProcess,
    targets: &[TargetProcess],
) -> Option<Vec<TargetProcess>> {
    let mut transition = targets.to_vec();
    let removed = transition
        .iter_mut()
        .find(|candidate| candidate.name == target.name && candidate.pid == target.pid)?;
    removed.enabled = false;
    Some(transition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn target_status_reports_muted_only_for_enabled_managed_target() {
        let strings = Language::Ko.strings();
        let mut target = TargetProcess::new("game.exe").unwrap();

        assert_eq!(
            target_status_text(&target, false, strings),
            strings.target_ready
        );
        assert_eq!(
            target_status_text(&target, true, strings),
            strings.target_muted
        );

        target.enabled = false;
        assert_eq!(
            target_status_text(&target, true, strings),
            strings.target_excluded
        );
    }

    #[test]
    fn persisted_broad_target_restores_only_sessions_exclusive_to_it() {
        let mut exe_target = TargetProcess::new("game.exe").unwrap();
        exe_target.managed_muted = true;
        let pid_target = TargetProcess::for_pid("game.exe", 20).unwrap();
        let covered = AudioSessionKey::new(20, "game.exe", Some("a".to_owned())).unwrap();
        let exclusive = AudioSessionKey::new(21, "game.exe", Some("b".to_owned())).unwrap();
        let muted_by_app = HashSet::from([covered.clone(), exclusive.clone()]);

        let sessions = session_keys_exclusive_to_target(
            &exe_target,
            &[exe_target.clone(), pid_target],
            &muted_by_app,
        );

        assert!(!sessions.contains(&covered));
        assert!(sessions.contains(&exclusive));
    }

    #[test]
    fn removal_restore_disables_only_the_removed_target() {
        let mut exe_target = TargetProcess::new("game.exe").unwrap();
        exe_target.managed_muted = true;
        let pid_target = TargetProcess::for_pid("game.exe", 20).unwrap();

        let targets =
            removal_restore_targets(&exe_target, &[exe_target.clone(), pid_target.clone()])
                .unwrap();

        assert!(!targets[0].enabled);
        assert!(targets[0].managed_muted);
        assert_eq!(targets[1], pid_target);
    }
}
