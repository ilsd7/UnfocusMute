use crate::config::TargetProcess;
use crate::engine::AudioSessionKey;
use crate::i18n::Strings;
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashSet;

const LINEAR_MANAGED_MUTE_LOOKUP_LIMIT: usize = 8;

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

pub(super) fn target_has_managed_mute(
    target: &TargetProcess,
    muted_by_app: &HashSet<AudioSessionKey>,
) -> bool {
    ManagedMuteLookup::new(muted_by_app).target_has_managed_mute(target)
}

pub(super) enum ManagedMuteLookup<'a> {
    Empty,
    One {
        pid: u32,
        process_name: &'a str,
    },
    Two {
        first_pid: u32,
        first_process_name: &'a str,
        second_pid: u32,
        second_process_name: &'a str,
    },
    Few {
        identities: [(u32, &'a str); LINEAR_MANAGED_MUTE_LOOKUP_LIMIT],
        len: usize,
    },
    Many {
        process_names: Vec<&'a str>,
        process_names_by_pid: Vec<(&'a str, u32)>,
    },
}

impl<'a> ManagedMuteLookup<'a> {
    pub(super) fn new(muted_by_app: &'a HashSet<AudioSessionKey>) -> Self {
        let mut keys = muted_by_app.iter();
        let Some(first) = keys.next() else {
            return Self::Empty;
        };
        let Some(second) = keys.next() else {
            return Self::One {
                pid: first.pid,
                process_name: first.process_name.as_str(),
            };
        };
        if muted_by_app.len() == 2 {
            return Self::Two {
                first_pid: first.pid,
                first_process_name: first.process_name.as_str(),
                second_pid: second.pid,
                second_process_name: second.process_name.as_str(),
            };
        }

        if muted_by_app.len() <= LINEAR_MANAGED_MUTE_LOOKUP_LIMIT {
            let mut identities = [(0, ""); LINEAR_MANAGED_MUTE_LOOKUP_LIMIT];
            identities[0] = (first.pid, first.process_name.as_str());
            identities[1] = (second.pid, second.process_name.as_str());
            let mut len = 2;
            for key in keys {
                identities[len] = (key.pid, key.process_name.as_str());
                len += 1;
            }
            return Self::Few { identities, len };
        }

        let mut process_names = Vec::with_capacity(muted_by_app.len());
        let mut process_names_by_pid = Vec::with_capacity(muted_by_app.len());
        process_names.push(first.process_name.as_str());
        process_names.push(second.process_name.as_str());
        process_names_by_pid.push((first.process_name.as_str(), first.pid));
        process_names_by_pid.push((second.process_name.as_str(), second.pid));
        for key in keys {
            let name = key.process_name.as_str();
            process_names.push(name);
            process_names_by_pid.push((name, key.pid));
        }

        process_names.sort_unstable();
        process_names.dedup();
        process_names_by_pid.sort_unstable_by(compare_managed_mute_pid_entry);
        process_names_by_pid.dedup();

        Self::Many {
            process_names,
            process_names_by_pid,
        }
    }

    pub(super) fn target_has_managed_mute(&self, target: &TargetProcess) -> bool {
        if !target.enabled {
            return false;
        }
        if target.managed_muted {
            return true;
        }

        match target.pid {
            Some(pid) => self.has_process_pid(target.name.as_str(), pid),
            None => self.has_process_name(target.name.as_str()),
        }
    }

    fn has_process_name(&self, name: &str) -> bool {
        match self {
            Self::Empty => false,
            Self::One { process_name, .. } => *process_name == name,
            Self::Two {
                first_process_name,
                second_process_name,
                ..
            } => *first_process_name == name || *second_process_name == name,
            Self::Few { identities, len } => identities[..*len]
                .iter()
                .any(|(_, process_name)| *process_name == name),
            Self::Many { process_names, .. } => process_names.binary_search(&name).is_ok(),
        }
    }

    fn has_process_pid(&self, name: &str, pid: u32) -> bool {
        match self {
            Self::Empty => false,
            Self::One {
                pid: managed_pid,
                process_name,
            } => *managed_pid == pid && *process_name == name,
            Self::Two {
                first_pid,
                first_process_name,
                second_pid,
                second_process_name,
            } => {
                (*first_pid == pid && *first_process_name == name)
                    || (*second_pid == pid && *second_process_name == name)
            }
            Self::Few { identities, len } => identities[..*len]
                .iter()
                .any(|(managed_pid, process_name)| *managed_pid == pid && *process_name == name),
            Self::Many {
                process_names_by_pid,
                ..
            } => process_names_by_pid
                .binary_search_by(|entry| compare_managed_mute_pid_key(*entry, name, pid))
                .is_ok(),
        }
    }
}

fn compare_managed_mute_pid_entry(left: &(&str, u32), right: &(&str, u32)) -> CmpOrdering {
    left.0.cmp(right.0).then_with(|| left.1.cmp(&right.1))
}

fn compare_managed_mute_pid_key(entry: (&str, u32), name: &str, pid: u32) -> CmpOrdering {
    entry.0.cmp(name).then_with(|| entry.1.cmp(&pid))
}

pub(super) fn target_matches_session_key(target: &TargetProcess, key: &AudioSessionKey) -> bool {
    target.name.as_str() == key.process_name.as_str() && target.pid.is_none_or(|pid| pid == key.pid)
}

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

#[cfg(test)]
fn managed_mute_count(targets: &[TargetProcess], muted_by_app: &HashSet<AudioSessionKey>) -> usize {
    let mute_lookup = ManagedMuteLookup::new(muted_by_app);
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

        assert!(target_has_managed_mute(&target, &muted_by_app));
    }

    #[test]
    fn exe_target_is_muted_when_persisted_target_state_is_managed() {
        let mut target = TargetProcess::new("game.exe").unwrap();
        target.managed_muted = true;

        assert!(target_has_managed_mute(&target, &HashSet::new()));
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

        assert!(!target_has_managed_mute(&target, &wrong_pid));
        assert!(target_has_managed_mute(&target, &matching_pid));
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
        let lookup = ManagedMuteLookup::new(&muted_by_app);

        assert!(lookup.target_has_managed_mute(&exe_target));
        assert!(lookup.target_has_managed_mute(&pid_target));
        assert!(!lookup.target_has_managed_mute(&wrong_pid_target));
    }

    #[test]
    fn managed_mute_lookup_uses_many_lookup_after_inline_limit() {
        let muted_by_app = (1..=LINEAR_MANAGED_MUTE_LOOKUP_LIMIT + 1)
            .map(|pid| AudioSessionKey::new(pid as u32, format!("app{pid}.exe"), None).unwrap())
            .collect::<HashSet<_>>();
        let lookup = ManagedMuteLookup::new(&muted_by_app);
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
    fn disabled_target_does_not_report_managed_mute() {
        let mut target = TargetProcess::new("game.exe").unwrap();
        target.enabled = false;
        let muted_by_app = HashSet::from([AudioSessionKey::new(20, "game.exe", None).unwrap()]);

        assert!(!target_has_managed_mute(&target, &muted_by_app));
    }
}
