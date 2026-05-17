#![cfg_attr(not(windows), allow(dead_code))]

use crate::config::{TargetProcess, normalize_process_name};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioSessionSnapshot {
    pub pid: u32,
    pub process_name: String,
    pub muted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MuteAction {
    pub pid: u32,
    pub process_name: String,
    pub mute: bool,
}

pub fn plan_mute_actions_with_matcher(
    matcher: &TargetMatcher,
    foreground_pid: Option<u32>,
    managed_muted_pids: &HashSet<u32>,
    sessions: &[AudioSessionSnapshot],
) -> Vec<MuteAction> {
    let mut actions = Vec::new();

    for session in sessions {
        let Some(name) = normalize_process_name(&session.process_name) else {
            continue;
        };
        if !matcher.matches(&name, session.pid) {
            continue;
        }

        let should_mute = foreground_pid != Some(session.pid);
        let should_change = if should_mute {
            !session.muted
        } else {
            session.muted && managed_muted_pids.contains(&session.pid)
        };

        if should_change {
            actions.push(MuteAction {
                pid: session.pid,
                process_name: name,
                mute: should_mute,
            });
        }
    }

    actions
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

    fn matches(&self, name: &str, pid: u32) -> bool {
        self.names.contains(name)
            || self
                .names_by_pid
                .get(&pid)
                .is_some_and(|names| names.contains(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TargetProcess;

    fn plan_mute_actions(
        targets: &[TargetProcess],
        foreground_pid: Option<u32>,
        managed_muted_pids: &HashSet<u32>,
        sessions: &[AudioSessionSnapshot],
    ) -> Vec<MuteAction> {
        let matcher = TargetMatcher::new(targets);
        plan_mute_actions_with_matcher(&matcher, foreground_pid, managed_muted_pids, sessions)
    }

    #[test]
    fn mutes_target_sessions_that_are_not_foreground() {
        let targets = vec![TargetProcess::new("game.exe").unwrap()];
        let sessions = vec![AudioSessionSnapshot {
            pid: 10,
            process_name: "game.exe".to_owned(),
            muted: false,
        }];

        assert_eq!(
            plan_mute_actions(&targets, Some(20), &HashSet::new(), &sessions),
            vec![MuteAction {
                pid: 10,
                process_name: "game.exe".to_owned(),
                mute: true,
            }]
        );
    }

    #[test]
    fn unmutes_target_session_when_it_returns_to_foreground() {
        let targets = vec![TargetProcess::new("game.exe").unwrap()];
        let sessions = vec![AudioSessionSnapshot {
            pid: 10,
            process_name: "game.exe".to_owned(),
            muted: true,
        }];

        assert_eq!(
            plan_mute_actions(&targets, Some(10), &HashSet::from([10]), &sessions),
            vec![MuteAction {
                pid: 10,
                process_name: "game.exe".to_owned(),
                mute: false,
            }]
        );
    }

    #[test]
    fn ignores_unregistered_processes() {
        let targets = vec![TargetProcess::new("game.exe").unwrap()];
        let sessions = vec![AudioSessionSnapshot {
            pid: 10,
            process_name: "browser.exe".to_owned(),
            muted: false,
        }];

        assert!(plan_mute_actions(&targets, Some(20), &HashSet::new(), &sessions).is_empty());
    }

    #[test]
    fn does_not_unmute_a_session_that_the_user_muted_manually() {
        let targets = vec![TargetProcess::new("game.exe").unwrap()];
        let sessions = vec![AudioSessionSnapshot {
            pid: 10,
            process_name: "game.exe".to_owned(),
            muted: true,
        }];

        assert!(plan_mute_actions(&targets, Some(10), &HashSet::new(), &sessions).is_empty());
    }

    #[test]
    fn pid_target_matches_only_that_process_instance() {
        let targets = vec![TargetProcess::for_pid("browser.exe", 20).unwrap()];
        let sessions = vec![
            AudioSessionSnapshot {
                pid: 10,
                process_name: "browser.exe".to_owned(),
                muted: false,
            },
            AudioSessionSnapshot {
                pid: 20,
                process_name: "browser.exe".to_owned(),
                muted: false,
            },
        ];

        assert_eq!(
            plan_mute_actions(&targets, Some(30), &HashSet::new(), &sessions),
            vec![MuteAction {
                pid: 20,
                process_name: "browser.exe".to_owned(),
                mute: true,
            }]
        );
    }
}
