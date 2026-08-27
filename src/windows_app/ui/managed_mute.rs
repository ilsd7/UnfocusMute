use crate::config::TargetProcess;
use crate::engine::AudioSessionKey;
#[cfg(test)]
use crate::engine::ManagedSessionLookup;
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

pub(super) fn target_matches_session_key(target: &TargetProcess, key: &AudioSessionKey) -> bool {
    target.name.as_str() == key.process_name.as_str() && target.pid.is_none_or(|pid| pid == key.pid)
}

#[cfg(test)]
pub(super) fn matching_session_keys_for_target(
    target: &TargetProcess,
    muted_by_app: &HashSet<AudioSessionKey>,
) -> HashSet<AudioSessionKey> {
    let mut matches = HashSet::new();
    for key in muted_by_app {
        if target_matches_session_key(target, key) {
            matches.insert(key.clone());
        }
    }
    matches
}

pub(super) fn session_keys_exclusive_to_target(
    target: &TargetProcess,
    targets: &[TargetProcess],
    muted_by_app: &HashSet<AudioSessionKey>,
) -> HashSet<AudioSessionKey> {
    muted_by_app
        .iter()
        .filter(|key| {
            target_matches_session_key(target, key)
                && !targets.iter().any(|other| {
                    other.enabled
                        && (other.name != target.name || other.pid != target.pid)
                        && target_matches_session_key(other, key)
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
fn managed_mute_count(targets: &[TargetProcess], muted_by_app: &HashSet<AudioSessionKey>) -> usize {
    let mute_lookup = ManagedSessionLookup::new(muted_by_app);
    targets
        .iter()
        .filter(|target| mute_lookup.target_has_managed_mute(target))
        .count()
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
    fn exe_target_is_muted_when_any_matching_session_is_managed() {
        let target = TargetProcess::new("game.exe").unwrap();
        let muted_by_app = HashSet::from([
            AudioSessionKey::new(10, "chat.exe", None).unwrap(),
            AudioSessionKey::new(20, "game.exe", None).unwrap(),
        ]);

        assert!(ManagedSessionLookup::new(&muted_by_app).target_has_managed_mute(&target));
    }

    #[test]
    fn managed_mute_count_counts_targets_not_sessions() {
        let game = TargetProcess::new("game.exe").unwrap();
        let chat = TargetProcess::new("chat.exe").unwrap();
        let muted_by_app = HashSet::from([
            AudioSessionKey::new(20, "game.exe", Some("a".to_owned())).unwrap(),
            AudioSessionKey::new(20, "game.exe", Some("b".to_owned())).unwrap(),
        ]);

        assert_eq!(managed_mute_count(&[game, chat], &muted_by_app), 1);
    }

    #[test]
    fn pid_target_is_muted_only_for_matching_pid() {
        let target = TargetProcess::for_pid("game.exe", 20).unwrap();
        let wrong_pid = HashSet::from([AudioSessionKey::new(21, "game.exe", None).unwrap()]);
        let matching_pid = HashSet::from([AudioSessionKey::new(20, "game.exe", None).unwrap()]);

        assert!(!ManagedSessionLookup::new(&wrong_pid).target_has_managed_mute(&target));
        assert!(ManagedSessionLookup::new(&matching_pid).target_has_managed_mute(&target));
    }

    #[test]
    fn managed_mute_lookup_reuses_indexed_session_state() {
        let exe_target = TargetProcess::new("game.exe").unwrap();
        let pid_target = TargetProcess::for_pid("chat.exe", 7).unwrap();
        let wrong_pid_target = TargetProcess::for_pid("chat.exe", 8).unwrap();
        let muted_by_app = HashSet::from([
            AudioSessionKey::new(20, "game.exe", None).unwrap(),
            AudioSessionKey::new(7, "chat.exe", None).unwrap(),
        ]);
        let lookup = ManagedSessionLookup::new(&muted_by_app);

        assert!(lookup.target_has_managed_mute(&exe_target));
        assert!(lookup.target_has_managed_mute(&pid_target));
        assert!(!lookup.target_has_managed_mute(&wrong_pid_target));
    }

    #[test]
    fn managed_mute_lookup_matches_after_deduplication() {
        let muted_by_app = (1..=9)
            .map(|pid| AudioSessionKey::new(pid as u32, format!("app{pid}.exe"), None).unwrap())
            .collect::<HashSet<_>>();
        let lookup = ManagedSessionLookup::new(&muted_by_app);
        let matching_target = TargetProcess::for_pid("app3.exe", 3).unwrap();
        let same_name_wrong_pid = TargetProcess::for_pid("app3.exe", 30).unwrap();

        assert!(lookup.target_has_managed_mute(&matching_target));
        assert!(!lookup.target_has_managed_mute(&same_name_wrong_pid));
    }

    #[test]
    fn matching_session_keys_for_target_keeps_only_matching_sessions() {
        let target = TargetProcess::new("game.exe").unwrap();
        let matching_a = AudioSessionKey::new(20, "game.exe", Some("a".to_owned())).unwrap();
        let matching_b = AudioSessionKey::new(21, "game.exe", Some("b".to_owned())).unwrap();
        let ignored = AudioSessionKey::new(22, "chat.exe", None).unwrap();
        let muted_by_app = HashSet::from([matching_a.clone(), matching_b.clone(), ignored]);

        let matches = matching_session_keys_for_target(&target, &muted_by_app);

        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&matching_a));
        assert!(matches.contains(&matching_b));
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

    #[test]
    fn disabled_target_does_not_report_managed_mute() {
        let mut target = TargetProcess::new("game.exe").unwrap();
        target.enabled = false;
        target.managed_muted = true;
        let muted_by_app = HashSet::from([AudioSessionKey::new(20, "game.exe", None).unwrap()]);

        assert!(!ManagedSessionLookup::new(&muted_by_app).target_has_managed_mute(&target));
    }
}
