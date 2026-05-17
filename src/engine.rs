#![cfg_attr(not(windows), allow(dead_code))]

use crate::config::{TargetProcess, normalize_process_name};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AudioSessionKey {
    pub pid: u32,
    pub process_name: String,
    pub instance_id: Option<String>,
}

impl AudioSessionKey {
    pub fn new(
        pid: u32,
        process_name: impl AsRef<str>,
        instance_id: Option<String>,
    ) -> Option<Self> {
        let process_name = normalize_process_name(process_name.as_ref())?;
        Some(Self {
            pid,
            process_name,
            instance_id,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioSessionSnapshot {
    pub key: AudioSessionKey,
    pub muted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MuteAction {
    pub key: AudioSessionKey,
    pub mute: bool,
}

pub fn plan_mute_actions_with_matcher(
    matcher: &TargetMatcher,
    foreground_pid: Option<u32>,
    foreground_process_name: Option<&str>,
    managed_muted_sessions: &HashSet<AudioSessionKey>,
    sessions: &[AudioSessionSnapshot],
) -> Vec<MuteAction> {
    let mut actions = Vec::new();
    let foreground_process_name = foreground_process_name.and_then(normalize_process_name);

    for session in sessions {
        let managed = managed_muted_sessions.contains(&session.key);
        let match_kind = matcher.match_kind(&session.key.process_name, session.key.pid);
        if match_kind.is_none() && !managed {
            continue;
        }

        let should_mute = match match_kind {
            Some(TargetMatchKind::ProcessName) => {
                foreground_process_name.as_deref() != Some(session.key.process_name.as_str())
                    && foreground_pid != Some(session.key.pid)
            }
            Some(TargetMatchKind::Pid) => foreground_pid != Some(session.key.pid),
            None => false,
        };
        let should_change = if should_mute {
            !session.muted
        } else {
            session.muted && managed
        };

        if should_change {
            actions.push(MuteAction {
                key: session.key.clone(),
                mute: should_mute,
            });
        }
    }

    actions
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TargetMatchKind {
    ProcessName,
    Pid,
}

#[derive(Clone, Debug, Default)]
pub struct TargetMatcher {
    names: HashSet<String>,
    names_by_pid: HashMap<u32, HashSet<String>>,
}

impl TargetMatcher {
    pub fn new(targets: &[TargetProcess]) -> Self {
        let mut names = HashSet::new();
        let mut names_by_pid = HashMap::<u32, HashSet<String>>::new();

        for target in targets.iter().filter(|target| target.enabled) {
            let Some(name) = normalize_process_name(&target.name) else {
                continue;
            };
            if let Some(pid) = target.pid {
                names_by_pid.entry(pid).or_default().insert(name);
            } else {
                names.insert(name);
            }
        }

        Self {
            names,
            names_by_pid,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && self.names_by_pid.is_empty()
    }

    fn match_kind(&self, name: &str, pid: u32) -> Option<TargetMatchKind> {
        if self.names.contains(name) {
            return Some(TargetMatchKind::ProcessName);
        }

        self.names_by_pid
            .get(&pid)
            .is_some_and(|names| names.contains(name))
            .then_some(TargetMatchKind::Pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TargetProcess;

    fn plan_mute_actions(
        targets: &[TargetProcess],
        foreground_pid: Option<u32>,
        managed_muted_sessions: &HashSet<AudioSessionKey>,
        sessions: &[AudioSessionSnapshot],
    ) -> Vec<MuteAction> {
        plan_mute_actions_with_foreground_name(
            targets,
            foreground_pid,
            None,
            managed_muted_sessions,
            sessions,
        )
    }

    fn plan_mute_actions_with_foreground_name(
        targets: &[TargetProcess],
        foreground_pid: Option<u32>,
        foreground_process_name: Option<&str>,
        managed_muted_sessions: &HashSet<AudioSessionKey>,
        sessions: &[AudioSessionSnapshot],
    ) -> Vec<MuteAction> {
        let matcher = TargetMatcher::new(targets);
        plan_mute_actions_with_matcher(
            &matcher,
            foreground_pid,
            foreground_process_name,
            managed_muted_sessions,
            sessions,
        )
    }

    fn session(pid: u32, process_name: &str, muted: bool) -> AudioSessionSnapshot {
        AudioSessionSnapshot {
            key: AudioSessionKey::new(pid, process_name, None).unwrap(),
            muted,
        }
    }

    #[test]
    fn mutes_target_sessions_that_are_not_foreground() {
        let targets = vec![TargetProcess::new("game.exe").unwrap()];
        let sessions = vec![session(10, "game.exe", false)];

        assert_eq!(
            plan_mute_actions(&targets, Some(20), &HashSet::new(), &sessions),
            vec![MuteAction {
                key: AudioSessionKey::new(10, "game.exe", None).unwrap(),
                mute: true,
            }]
        );
    }

    #[test]
    fn unmutes_target_session_when_it_returns_to_foreground() {
        let targets = vec![TargetProcess::new("game.exe").unwrap()];
        let sessions = vec![session(10, "game.exe", true)];

        assert_eq!(
            plan_mute_actions(
                &targets,
                Some(10),
                &HashSet::from([AudioSessionKey::new(10, "game.exe", None).unwrap()]),
                &sessions
            ),
            vec![MuteAction {
                key: AudioSessionKey::new(10, "game.exe", None).unwrap(),
                mute: false,
            }]
        );
    }

    #[test]
    fn ignores_unregistered_processes() {
        let targets = vec![TargetProcess::new("game.exe").unwrap()];
        let sessions = vec![session(10, "browser.exe", false)];

        assert!(plan_mute_actions(&targets, Some(20), &HashSet::new(), &sessions).is_empty());
    }

    #[test]
    fn does_not_unmute_a_session_that_the_user_muted_manually() {
        let targets = vec![TargetProcess::new("game.exe").unwrap()];
        let sessions = vec![session(10, "game.exe", true)];

        assert!(plan_mute_actions(&targets, Some(10), &HashSet::new(), &sessions).is_empty());
    }

    #[test]
    fn pid_target_matches_only_that_process_instance() {
        let targets = vec![TargetProcess::for_pid("browser.exe", 20).unwrap()];
        let sessions = vec![
            session(10, "browser.exe", false),
            session(20, "browser.exe", false),
        ];

        assert_eq!(
            plan_mute_actions(&targets, Some(30), &HashSet::new(), &sessions),
            vec![MuteAction {
                key: AudioSessionKey::new(20, "browser.exe", None).unwrap(),
                mute: true,
            }]
        );
    }

    #[test]
    fn exe_target_keeps_same_process_name_unmuted_across_pids() {
        let targets = vec![TargetProcess::new("browser.exe").unwrap()];
        let sessions = vec![session(20, "browser.exe", false)];

        assert!(
            plan_mute_actions_with_foreground_name(
                &targets,
                Some(10),
                Some("browser.exe"),
                &HashSet::new(),
                &sessions,
            )
            .is_empty()
        );
    }

    #[test]
    fn exe_target_unmutes_managed_same_process_name_across_pids() {
        let targets = vec![TargetProcess::new("browser.exe").unwrap()];
        let session_key = AudioSessionKey::new(20, "browser.exe", None).unwrap();
        let sessions = vec![AudioSessionSnapshot {
            key: session_key.clone(),
            muted: true,
        }];

        assert_eq!(
            plan_mute_actions_with_foreground_name(
                &targets,
                Some(10),
                Some("browser.exe"),
                &HashSet::from([session_key.clone()]),
                &sessions,
            ),
            vec![MuteAction {
                key: session_key,
                mute: false,
            }]
        );
    }

    #[test]
    fn pid_target_stays_strict_even_when_process_name_is_foreground() {
        let targets = vec![TargetProcess::for_pid("browser.exe", 20).unwrap()];
        let sessions = vec![session(20, "browser.exe", false)];

        assert_eq!(
            plan_mute_actions_with_foreground_name(
                &targets,
                Some(10),
                Some("browser.exe"),
                &HashSet::new(),
                &sessions,
            ),
            vec![MuteAction {
                key: AudioSessionKey::new(20, "browser.exe", None).unwrap(),
                mute: true,
            }]
        );
    }

    #[test]
    fn session_instance_id_keeps_same_pid_sessions_separate() {
        let targets = vec![TargetProcess::new("game.exe").unwrap()];
        let session_a = AudioSessionKey::new(10, "game.exe", Some("a".to_owned())).unwrap();
        let session_b = AudioSessionKey::new(10, "game.exe", Some("b".to_owned())).unwrap();
        let sessions = vec![
            AudioSessionSnapshot {
                key: session_a.clone(),
                muted: true,
            },
            AudioSessionSnapshot {
                key: session_b.clone(),
                muted: true,
            },
        ];

        assert_eq!(
            plan_mute_actions(
                &targets,
                Some(10),
                &HashSet::from([session_a.clone()]),
                &sessions
            ),
            vec![MuteAction {
                key: session_a,
                mute: false,
            }]
        );
    }

    #[test]
    fn unmutes_managed_session_after_target_is_removed() {
        let session_key = AudioSessionKey::new(10, "game.exe", None).unwrap();
        let sessions = vec![AudioSessionSnapshot {
            key: session_key.clone(),
            muted: true,
        }];

        assert_eq!(
            plan_mute_actions(
                &[],
                Some(20),
                &HashSet::from([session_key.clone()]),
                &sessions
            ),
            vec![MuteAction {
                key: session_key,
                mute: false,
            }]
        );
    }
}
