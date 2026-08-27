#![cfg_attr(not(windows), allow(dead_code))]

#[cfg(test)]
use crate::config::normalize_process_name;
use crate::config::{TargetProcess, compare_target_identity, is_normalized_process_name};
use std::collections::HashSet;

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
        debug_assert!(is_normalized_process_name(&process_name));
        Self {
            pid,
            process_name,
            instance_id,
        }
    }
}

/// Canonical process/PID view of the exact audio sessions owned by this process.
///
/// Audio planning and target-status rendering share this lookup so identity matching cannot
/// drift between the engine and its Windows adapters.
pub(crate) struct ManagedSessionLookup<'a> {
    session_keys: &'a HashSet<AudioSessionKey>,
}

impl<'a> ManagedSessionLookup<'a> {
    pub(crate) fn new(session_keys: &'a HashSet<AudioSessionKey>) -> Self {
        Self { session_keys }
    }

    pub(crate) fn may_include_pid(&self, pid: u32) -> bool {
        self.session_keys.iter().any(|key| key.pid == pid)
    }

    pub(crate) fn contains(&self, process_name: &str, pid: u32) -> bool {
        self.session_keys
            .iter()
            .any(|key| key.pid == pid && key.process_name == process_name)
    }

    pub(crate) fn target_has_managed_mute(&self, target: &TargetProcess) -> bool {
        if !target.enabled {
            return false;
        }
        if target.managed_muted {
            return true;
        }
        match target.pid {
            Some(pid) => self.contains(&target.name, pid),
            None => self
                .session_keys
                .iter()
                .any(|key| key.process_name == target.name),
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioSessionSnapshot {
    pub key: AudioSessionKey,
    pub muted: bool,
}

#[cfg(test)]
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
pub(crate) enum TargetMatchKind {
    ProcessName,
    Pid,
}

#[derive(Clone, Debug, Default)]
pub struct TargetMatcher {
    identities: Vec<(String, Option<u32>)>,
}

impl TargetMatcher {
    pub fn new(targets: &[TargetProcess]) -> Self {
        let mut identities = Vec::with_capacity(targets.len());
        for target in targets.iter().filter(|target| target.enabled) {
            debug_assert!(is_normalized_process_name(&target.name));
            identities.push((target.name.clone(), target.pid));
        }
        if identities.len() > 1 {
            identities.sort_unstable_by(|left, right| {
                compare_target_identity(&left.0, left.1, &right.0, right.1)
            });
            identities.dedup();
        }
        Self { identities }
    }

    pub fn is_empty(&self) -> bool {
        self.identities.is_empty()
    }

    pub fn needs_foreground_process_name(&self) -> bool {
        !self.is_empty()
    }

    pub fn needs_all_session_process_names(&self) -> bool {
        self.identities.iter().any(|(_, pid)| pid.is_none())
    }

    pub(crate) fn has_pid_target(&self, pid: u32) -> bool {
        self.identities
            .iter()
            .any(|(_, target_pid)| *target_pid == Some(pid))
    }

    fn match_kind(&self, name: &str, pid: u32) -> Option<TargetMatchKind> {
        if self.has_identity(name, None) {
            Some(TargetMatchKind::ProcessName)
        } else if self.has_identity(name, Some(pid)) {
            Some(TargetMatchKind::Pid)
        } else {
            None
        }
    }

    fn has_identity(&self, name: &str, pid: Option<u32>) -> bool {
        self.identities
            .binary_search_by(|(target_name, target_pid)| {
                compare_target_identity(target_name, *target_pid, name, pid)
            })
            .is_ok()
    }
}

pub struct MutePlanner<'a> {
    matcher: &'a TargetMatcher,
    foreground_pid: Option<u32>,
    foreground_process_name: Option<&'a str>,
}

impl<'a> MutePlanner<'a> {
    #[cfg(test)]
    pub fn new(
        matcher: &'a TargetMatcher,
        foreground_pid: Option<u32>,
        foreground_process_name: Option<&'a str>,
    ) -> Self {
        Self::new_with_normalized_foreground(matcher, foreground_pid, foreground_process_name)
    }

    pub(crate) fn new_with_normalized_foreground(
        matcher: &'a TargetMatcher,
        foreground_pid: Option<u32>,
        foreground_process_name: Option<&'a str>,
    ) -> Self {
        if let Some(name) = &foreground_process_name {
            debug_assert!(is_normalized_process_name(name));
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

    #[cfg(test)]
    pub fn plan_session_with_managed(
        &self,
        managed: bool,
        key: &AudioSessionKey,
        muted: bool,
    ) -> Option<MuteAction> {
        self.plan_identity_with_managed(&key.process_name, key.pid, managed, muted)
            .map(|mute| MuteAction {
                key: key.clone(),
                mute,
            })
    }

    pub(crate) fn match_kind(&self, process_name: &str, pid: u32) -> Option<TargetMatchKind> {
        self.matcher.match_kind(process_name, pid)
    }

    #[cfg(test)]
    pub fn plan_identity_with_managed(
        &self,
        process_name: &str,
        pid: u32,
        managed: bool,
        muted: bool,
    ) -> Option<bool> {
        self.plan_identity_with_match(
            self.match_kind(process_name, pid),
            process_name,
            pid,
            managed,
            muted,
        )
    }

    #[cfg(test)]
    pub(crate) fn plan_identity_with_match(
        &self,
        match_kind: Option<TargetMatchKind>,
        process_name: &str,
        pid: u32,
        managed: bool,
        muted: bool,
    ) -> Option<bool> {
        let should_mute = self.desired_mute_with_match(match_kind, process_name, pid, managed)?;
        (should_mute != muted).then_some(should_mute)
    }

    #[cfg(test)]
    pub(crate) fn desired_mute_with_match(
        &self,
        match_kind: Option<TargetMatchKind>,
        process_name: &str,
        pid: u32,
        managed: bool,
    ) -> Option<bool> {
        self.desired_mute_with_foreground(
            match_kind,
            managed,
            self.session_is_foreground(process_name, pid),
        )
    }

    pub(crate) fn desired_mute_with_foreground(
        &self,
        match_kind: Option<TargetMatchKind>,
        managed: bool,
        session_is_foreground: bool,
    ) -> Option<bool> {
        if match_kind.is_none() && !managed {
            return None;
        }

        let should_mute = match match_kind {
            Some(TargetMatchKind::ProcessName | TargetMatchKind::Pid) => !session_is_foreground,
            None => false,
        };

        (should_mute || managed).then_some(should_mute)
    }

    pub(crate) fn session_is_foreground(&self, process_name: &str, pid: u32) -> bool {
        self.foreground_pid == Some(pid) || self.foreground_process_name == Some(process_name)
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
    fn pid_only_targets_need_foreground_name_without_full_session_scan() {
        let pid_matcher = TargetMatcher::new(&[TargetProcess::for_pid("game.exe", 10).unwrap()]);
        let exe_matcher = TargetMatcher::new(&[TargetProcess::new("game.exe").unwrap()]);

        assert!(pid_matcher.needs_foreground_process_name());
        assert!(!pid_matcher.needs_all_session_process_names());
        assert!(exe_matcher.needs_foreground_process_name());
        assert!(exe_matcher.needs_all_session_process_names());
    }

    #[test]
    fn pid_targets_can_prefilter_by_process_id() {
        let matcher = TargetMatcher::new(&[
            TargetProcess::for_pid("game.exe", 10).unwrap(),
            TargetProcess::for_pid("chat.exe", 20).unwrap(),
        ]);

        assert!(matcher.has_pid_target(10));
        assert!(matcher.has_pid_target(20));
        assert!(!matcher.has_pid_target(30));
    }

    #[test]
    fn single_pid_target_prefilter_matches() {
        let matcher = TargetMatcher::new(&[TargetProcess::for_pid("game.exe", 10).unwrap()]);

        assert!(matcher.has_pid_target(10));
        assert!(!matcher.has_pid_target(20));
    }

    #[test]
    fn single_targets_match_by_identity() {
        let exe_matcher = TargetMatcher::new(&[TargetProcess::new("game.exe").unwrap()]);
        let pid_matcher = TargetMatcher::new(&[TargetProcess::for_pid("chat.exe", 20).unwrap()]);

        assert_eq!(
            exe_matcher.match_kind("game.exe", 10),
            Some(TargetMatchKind::ProcessName)
        );
        assert_eq!(exe_matcher.match_kind("chat.exe", 10), None);
        assert_eq!(
            pid_matcher.match_kind("chat.exe", 20),
            Some(TargetMatchKind::Pid)
        );
        assert_eq!(pid_matcher.match_kind("chat.exe", 21), None);
    }

    #[test]
    fn single_disabled_target_is_ignored() {
        let mut target = TargetProcess::new("game.exe").unwrap();
        target.enabled = false;

        let matcher = TargetMatcher::new(&[target]);

        assert!(matcher.is_empty());
        assert!(!matcher.needs_foreground_process_name());
        assert!(!matcher.needs_all_session_process_names());
    }

    #[test]
    fn empty_targets_create_empty_matcher() {
        let matcher = TargetMatcher::new(&[]);

        assert!(matcher.is_empty());
        assert!(!matcher.needs_foreground_process_name());
        assert!(!matcher.needs_all_session_process_names());
    }

    #[test]
    fn identity_planner_skips_unmatched_unmanaged_sessions() {
        let matcher = TargetMatcher::new(&[TargetProcess::new("game.exe").unwrap()]);
        let planner = MutePlanner::new(&matcher, Some(20), Some("other.exe"));

        assert_eq!(planner.match_kind("browser.exe", 10), None);
        assert_eq!(
            planner.plan_identity_with_managed("browser.exe", 10, false, false),
            None
        );
    }

    #[test]
    fn desired_mute_enforces_managed_sessions_without_current_state() {
        let matcher = TargetMatcher::new(&[TargetProcess::new("game.exe").unwrap()]);
        let planner = MutePlanner::new(&matcher, Some(10), Some("game.exe"));

        assert_eq!(
            planner.desired_mute_with_match(
                planner.match_kind("game.exe", 10),
                "game.exe",
                10,
                true,
            ),
            Some(false)
        );
    }

    #[test]
    fn desired_mute_enforces_removed_managed_sessions_without_current_state() {
        let matcher = TargetMatcher::new(&[]);
        let planner = MutePlanner::new(&matcher, Some(20), Some("other.exe"));

        assert_eq!(
            planner.desired_mute_with_match(None, "game.exe", 10, true),
            Some(false)
        );
    }

    #[test]
    fn desired_mute_allows_external_foreground_match() {
        let matcher = TargetMatcher::new(&[TargetProcess::new("game.exe").unwrap()]);
        let planner = MutePlanner::new(&matcher, Some(20), Some("launcher.exe"));

        assert_eq!(
            planner.desired_mute_with_foreground(planner.match_kind("game.exe", 10), true, true,),
            Some(false)
        );
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
    fn pid_target_allows_same_process_name_foreground_as_technical_fallback() {
        let targets = vec![TargetProcess::for_pid("browser.exe", 20).unwrap()];
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
    fn pid_target_unmutes_managed_session_when_same_process_name_is_foreground() {
        let targets = vec![TargetProcess::for_pid("browser.exe", 20).unwrap()];
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
