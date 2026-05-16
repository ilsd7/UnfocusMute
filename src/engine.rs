#![cfg_attr(not(windows), allow(dead_code))]

use crate::config::{TargetProcess, normalize_process_name};
use std::collections::{BTreeSet, HashSet};

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

pub fn plan_mute_actions(
    targets: &[TargetProcess],
    foreground_pid: Option<u32>,
    managed_muted_pids: &HashSet<u32>,
    sessions: &[AudioSessionSnapshot],
) -> Vec<MuteAction> {
    let target_names = enabled_target_names(targets);
    let mut actions = Vec::new();

    for session in sessions {
        let Some(name) = normalize_process_name(&session.process_name) else {
            continue;
        };
        if !target_names.contains(&name) {
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

fn enabled_target_names(targets: &[TargetProcess]) -> BTreeSet<String> {
    targets
        .iter()
        .filter(|target| target.enabled)
        .filter_map(|target| normalize_process_name(&target.name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TargetProcess;

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
}
