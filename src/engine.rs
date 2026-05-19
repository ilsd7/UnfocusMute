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
    #[cfg(test)]
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

    pub(crate) fn from_normalized(
        pid: u32,
        process_name: String,
        instance_id: Option<String>,
    ) -> Self {
        debug_assert!(!process_name.is_empty());
        debug_assert!(!process_name.contains('\0'));
        debug_assert_eq!(process_name, process_name.to_ascii_lowercase());
        Self {
            pid,
            process_name,
            instance_id,
        }
    }
}

#[cfg(test)]
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

#[cfg(test)]
pub fn plan_mute_actions_with_matcher(
    matcher: &TargetMatcher,
    foreground_pid: Option<u32>,
    foreground_process_name: Option<&str>,
    managed_muted_sessions: &HashSet<AudioSessionKey>,
    sessions: &[AudioSessionSnapshot],
) -> Vec<MuteAction> {
    let planner = MutePlanner::new(matcher, foreground_pid, foreground_process_name);
    let mut actions = Vec::with_capacity(sessions.len());

    for session in sessions {
        if let Some(action) =
            planner.plan_session(managed_muted_sessions, &session.key, session.muted)
        {
            actions.push(action);
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
    names_by_pid: HashMap<u32, Vec<String>>,
}

impl TargetMatcher {
    pub fn new(targets: &[TargetProcess]) -> Self {
        let mut names = HashSet::with_capacity(targets.len());
        let mut names_by_pid = HashMap::<u32, Vec<String>>::with_capacity(targets.len());

        for target in targets.iter().filter(|target| target.enabled) {
            let Some(name) = normalize_process_name(&target.name) else {
                continue;
            };
            if let Some(pid) = target.pid {
                let names = names_by_pid.entry(pid).or_default();
                if !names.iter().any(|existing| existing == &name) {
                    names.push(name);
                }
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

    pub fn needs_foreground_process_name(&self) -> bool {
        !self.names.is_empty()
    }

    fn match_kind(&self, name: &str, pid: u32) -> Option<TargetMatchKind> {
        if self.names.contains(name) {
            return Some(TargetMatchKind::ProcessName);
        }

        self.names_by_pid
            .get(&pid)
            .is_some_and(|names| names.iter().any(|target| target == name))
            .then_some(TargetMatchKind::Pid)
    }
}

pub struct MutePlanner<'a> {
    matcher: &'a TargetMatcher,
    foreground_pid: Option<u32>,
    foreground_process_name: Option<String>,
}

impl<'a> MutePlanner<'a> {
    #[cfg(test)]
    pub fn new(
        matcher: &'a TargetMatcher,
        foreground_pid: Option<u32>,
        foreground_process_name: Option<&str>,
    ) -> Self {
        Self::new_with_normalized_foreground(
            matcher,
            foreground_pid,
            foreground_process_name.and_then(normalize_process_name),
        )
    }

    pub(crate) fn new_with_normalized_foreground(
        matcher: &'a TargetMatcher,
        foreground_pid: Option<u32>,
        foreground_process_name: Option<String>,
    ) -> Self {
        if let Some(name) = &foreground_process_name {
            debug_assert!(!name.is_empty());
            debug_assert!(!name.contains('\0'));
            debug_assert_eq!(name, &name.to_ascii_lowercase());
        }
        Self {
            matcher,
            foreground_pid,
            foreground_process_name,
        }
    }

    #[cfg(test)]
    pub fn plan_session(
        &self,
        managed_muted_sessions: &HashSet<AudioSessionKey>,
        key: &AudioSessionKey,
        muted: bool,
    ) -> Option<MuteAction> {
        let managed = managed_muted_sessions.contains(key);
        self.plan_session_with_managed(managed, key, muted)
    }

    pub(crate) fn plan_session_with_managed(
        &self,
        managed: bool,
        key: &AudioSessionKey,
        muted: bool,
    ) -> Option<MuteAction> {
        let match_kind = self.matcher.match_kind(&key.process_name, key.pid);
        if match_kind.is_none() && !managed {
            return None;
        }

        let should_mute = match match_kind {
            Some(TargetMatchKind::ProcessName) => {
                self.foreground_process_name.as_deref() != Some(key.process_name.as_str())
                    && self.foreground_pid != Some(key.pid)
            }
            Some(TargetMatchKind::Pid) => self.foreground_pid != Some(key.pid),
            None => false,
        };
        let should_change = if should_mute {
            !muted
        } else {
            muted && managed
        };

        should_change.then(|| MuteAction {
            key: key.clone(),
            mute: should_mute,
        })
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
    fn pid_only_targets_do_not_need_foreground_process_name() {
        let pid_matcher = TargetMatcher::new(&[TargetProcess::for_pid("game.exe", 10).unwrap()]);
        let exe_matcher = TargetMatcher::new(&[TargetProcess::new("game.exe").unwrap()]);

        assert!(!pid_matcher.needs_foreground_process_name());
        assert!(exe_matcher.needs_foreground_process_name());
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
