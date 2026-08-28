use crate::config::TargetProcess;
use crate::engine::AudioSessionKey;
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetMuteStateUpdate {
    pub process_name: String,
    pub pid: Option<u32>,
    pub muted: bool,
}

#[derive(Clone, Copy)]
pub(super) struct SessionControlIdentityRef<'a> {
    pub(super) key: &'a AudioSessionKey,
    pub(super) token: usize,
}

pub(super) fn same_session_control(
    left: SessionControlIdentityRef<'_>,
    right: SessionControlIdentityRef<'_>,
) -> bool {
    match (&left.key.instance_id, &right.key.instance_id) {
        (Some(_), Some(_)) => left.key == right.key,
        _ => left.token == right.token,
    }
}

pub(super) fn same_session_group(left: &AudioSessionKey, right: &AudioSessionKey) -> bool {
    left.pid == right.pid && left.process_name == right.process_name
}

pub(super) fn same_session_ownership(left: &AudioSessionKey, right: &AudioSessionKey) -> bool {
    match (&left.instance_id, &right.instance_id) {
        (Some(_), Some(_)) => left == right,
        _ => same_session_group(left, right),
    }
}

pub(super) fn cached_session_requires_unmute(
    cached: SessionControlIdentityRef<'_>,
    observed: SessionControlIdentityRef<'_>,
) -> bool {
    same_session_control(cached, observed)
        || ((cached.key.instance_id.is_none() || observed.key.instance_id.is_none())
            && same_session_group(cached.key, observed.key))
}

pub(super) fn mute_state_needs_write<E>(
    desired_mute: bool,
    current_mute: &std::result::Result<bool, E>,
) -> bool {
    current_mute
        .as_ref()
        .map_or(true, |current_mute| *current_mute != desired_mute)
}

pub(super) fn should_retain_cached_session(
    desired_mute: bool,
    process_has_exited: bool,
    session_has_expired: bool,
) -> bool {
    desired_mute && !process_has_exited && !session_has_expired
}

#[derive(Clone, Copy)]
pub(super) struct TargetMuteIdentity<'a> {
    pub(super) process_name: &'a str,
    pub(super) pid: Option<u32>,
}

impl TargetMuteStateUpdate {
    pub(super) fn new(identity: TargetMuteIdentity<'_>, muted: bool) -> Self {
        Self {
            process_name: identity.process_name.to_owned(),
            pid: identity.pid,
            muted,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct AudioSessionIdentity<'a> {
    pub(super) process_name: &'a str,
    pub(super) pid: u32,
}

pub(super) fn target_update_matches(
    update: &TargetMuteStateUpdate,
    identity: TargetMuteIdentity<'_>,
) -> bool {
    update.process_name == identity.process_name && update.pid == identity.pid
}

pub(super) struct ManagedTargetLookup<'a> {
    targets: &'a [TargetProcess],
}

impl<'a> ManagedTargetLookup<'a> {
    pub(super) fn new(targets: &'a [TargetProcess]) -> Self {
        Self { targets }
    }

    pub(super) fn may_include_pid(&self, pid: u32) -> bool {
        self.targets.iter().any(|target| {
            target.managed_muted && target.pid.is_none_or(|target_pid| target_pid == pid)
        })
    }

    pub(super) fn has_recovery_match(&self, process_name: &str, pid: u32) -> bool {
        self.targets
            .iter()
            .any(|target| target.managed_muted && target.matches_session(process_name, pid))
    }

    pub(super) fn for_each_matching(
        &self,
        process_name: &str,
        pid: u32,
        mut apply: impl FnMut(TargetMuteIdentity<'a>),
    ) {
        for target in self
            .targets
            .iter()
            .filter(|target| target.managed_muted && target.matches_session(process_name, pid))
        {
            apply(TargetMuteIdentity {
                process_name: &target.name,
                pid: target.pid,
            });
        }
    }

    pub(super) fn for_each_pid_target(
        &self,
        pid: u32,
        mut apply: impl FnMut(TargetMuteIdentity<'a>),
    ) {
        for target in self
            .targets
            .iter()
            .filter(|target| target.managed_muted && target.pid == Some(pid))
        {
            apply(TargetMuteIdentity {
                process_name: &target.name,
                pid: target.pid,
            });
        }
    }

    pub(super) fn for_each_process_target(&self, mut apply: impl FnMut(TargetMuteIdentity<'a>)) {
        for target in self
            .targets
            .iter()
            .filter(|target| target.managed_muted && target.pid.is_none())
        {
            apply(TargetMuteIdentity {
                process_name: &target.name,
                pid: None,
            });
        }
    }
}

pub(super) struct PlanApplyOutcome {
    pub(super) active_managed_sessions: Vec<AudioSessionKey>,
    pub(super) target_updates: Vec<TargetMuteStateUpdate>,
    blocked_unmuted_target_updates: Vec<TargetMuteStateUpdate>,
    pub(super) uncertain_unmuted_target_updates: Vec<TargetMuteStateUpdate>,
    pub(super) block_all_unmuted_target_updates: bool,
    pub(super) had_failures: bool,
    pub(super) failure_detail: Option<String>,
}

impl PlanApplyOutcome {
    pub(super) fn new(managed_session_count: usize) -> Self {
        Self {
            active_managed_sessions: Vec::with_capacity(managed_session_count),
            target_updates: Vec::new(),
            blocked_unmuted_target_updates: Vec::new(),
            uncertain_unmuted_target_updates: Vec::new(),
            block_all_unmuted_target_updates: false,
            had_failures: false,
            failure_detail: None,
        }
    }

    pub(super) fn mark_failure(&mut self, detail: impl Into<String>) {
        self.had_failures = true;
        if self.failure_detail.is_none() {
            self.failure_detail = Some(detail.into());
        }
    }

    pub(super) fn keep_session_ownership(
        &mut self,
        key: AudioSessionKey,
        session_keys: &HashSet<AudioSessionKey>,
    ) {
        if key.instance_id.is_none() {
            self.keep_active_sessions_for_identity(&key.process_name, key.pid, session_keys);
        }
        self.active_managed_sessions.push(key);
    }

    pub(super) fn keep_active_sessions_for_pid(
        &mut self,
        pid: u32,
        session_keys: &HashSet<AudioSessionKey>,
    ) {
        self.active_managed_sessions
            .extend(session_keys.iter().filter(|key| key.pid == pid).cloned());
    }

    pub(super) fn keep_active_sessions(&mut self, session_keys: &HashSet<AudioSessionKey>) {
        self.active_managed_sessions
            .extend(session_keys.iter().cloned());
    }

    fn keep_active_sessions_for_identity(
        &mut self,
        process_name: &str,
        pid: u32,
        session_keys: &HashSet<AudioSessionKey>,
    ) {
        self.active_managed_sessions.extend(
            session_keys
                .iter()
                .filter(|key| key.pid == pid && key.process_name == process_name)
                .cloned(),
        );
    }

    pub(super) fn block_unmuted_target_states_for_unresolved_pid(
        &mut self,
        pid: u32,
        session_keys: &HashSet<AudioSessionKey>,
        target_lookup: &ManagedTargetLookup<'_>,
    ) {
        for key in session_keys.iter().filter(|key| key.pid == pid) {
            target_lookup.for_each_matching(&key.process_name, pid, |identity| {
                self.block_unmuted_target_state(identity);
            });
        }
        let _ = self.block_unmuted_pid_target_states(pid, target_lookup, Some(session_keys));
    }

    pub(super) fn block_unmuted_pid_target_states(
        &mut self,
        pid: u32,
        target_lookup: &ManagedTargetLookup<'_>,
        known_sessions: Option<&HashSet<AudioSessionKey>>,
    ) -> bool {
        let mut blocked = false;
        target_lookup.for_each_pid_target(pid, |identity| {
            blocked = true;
            let has_known_session = known_sessions.is_some_and(|sessions| {
                sessions
                    .iter()
                    .any(|key| key.pid == pid && key.process_name.as_str() == identity.process_name)
            });
            if has_known_session {
                self.block_unmuted_target_state(identity);
            } else {
                self.mark_unmuted_target_uncertain(identity);
            }
        });
        blocked
    }

    pub(super) fn block_unmuted_process_target_states(
        &mut self,
        target_lookup: &ManagedTargetLookup<'_>,
    ) {
        target_lookup.for_each_process_target(|identity| {
            self.mark_unmuted_target_uncertain(identity);
        });
    }

    fn mark_unmuted_target_uncertain(&mut self, identity: TargetMuteIdentity<'_>) {
        self.block_unmuted_target_state(identity);
        if !self.block_all_unmuted_target_updates
            && !self
                .uncertain_unmuted_target_updates
                .iter()
                .any(|update| target_update_matches(update, identity))
        {
            self.uncertain_unmuted_target_updates
                .push(TargetMuteStateUpdate::new(identity, false));
        }
    }

    pub(super) fn block_unmuted_target_state(&mut self, identity: TargetMuteIdentity<'_>) {
        if !self.block_all_unmuted_target_updates
            && !self
                .blocked_unmuted_target_updates
                .iter()
                .any(|update| target_update_matches(update, identity))
        {
            self.blocked_unmuted_target_updates
                .push(TargetMuteStateUpdate::new(identity, false));
        }
        self.target_updates
            .retain(|update| update.muted || !target_update_matches(update, identity));
    }

    pub(super) fn block_all_unmuted_target_states(&mut self) {
        self.block_all_unmuted_target_updates = true;
        self.blocked_unmuted_target_updates.clear();
        self.uncertain_unmuted_target_updates.clear();
        self.target_updates.retain(|update| update.muted);
    }

    pub(super) fn set_target_state(&mut self, identity: TargetMuteIdentity<'_>, muted: bool) {
        if !muted
            && (self.block_all_unmuted_target_updates
                || self
                    .blocked_unmuted_target_updates
                    .iter()
                    .any(|update| target_update_matches(update, identity)))
        {
            return;
        }

        if let Some(update) = self
            .target_updates
            .iter_mut()
            .find(|update| target_update_matches(update, identity))
        {
            update.muted |= muted;
        } else {
            self.target_updates
                .push(TargetMuteStateUpdate::new(identity, muted));
        }
    }

    pub(super) fn set_target_states_for_session(
        &mut self,
        target_lookup: &ManagedTargetLookup<'_>,
        session: AudioSessionIdentity<'_>,
        fallback_identity: Option<TargetMuteIdentity<'_>>,
        muted: bool,
    ) {
        let mut updated_persisted_target = false;
        target_lookup.for_each_matching(session.process_name, session.pid, |identity| {
            self.set_target_state(identity, muted);
            updated_persisted_target = true;
        });
        if (muted || !updated_persisted_target)
            && let Some(identity) = fallback_identity
        {
            self.set_target_state(identity, muted);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ManagedSessionSelection {
    Exact,
    CoarseRuntime,
    Unmanaged,
}

impl ManagedSessionSelection {
    pub(super) fn is_managed(self) -> bool {
        matches!(self, Self::Exact | Self::CoarseRuntime)
    }
}

pub(super) fn select_runtime_managed_session(
    key: &AudioSessionKey,
    managed_muted_sessions: &HashSet<AudioSessionKey>,
) -> ManagedSessionSelection {
    if managed_muted_sessions.contains(key) {
        return ManagedSessionSelection::Exact;
    }

    let mut has_matching_runtime_key = false;
    let mut has_matching_exact_key = false;
    let mut has_matching_coarse_key = false;
    for managed_key in managed_muted_sessions.iter().filter(|managed_key| {
        managed_key.pid == key.pid && managed_key.process_name == key.process_name
    }) {
        has_matching_runtime_key = true;
        if managed_key.instance_id.is_some() {
            has_matching_exact_key = true;
        } else {
            has_matching_coarse_key = true;
        }
    }

    // A missing Core Audio instance ID turns ownership into an explicit PID/name group. Keep
    // existing exact keys alongside that coarse token so a transient API failure cannot discard
    // ownership. If IDs return, an exact sibling suppresses the coarse token for other siblings.
    if (key.instance_id.is_none() && has_matching_runtime_key)
        || (key.instance_id.is_some() && has_matching_coarse_key && !has_matching_exact_key)
    {
        ManagedSessionSelection::CoarseRuntime
    } else {
        ManagedSessionSelection::Unmanaged
    }
}

pub(super) fn matching_unresolved_session_keys(
    session_keys: &HashSet<AudioSessionKey>,
    pid: u32,
    instance_id: Option<&str>,
) -> Vec<AudioSessionKey> {
    session_keys
        .iter()
        .filter(|key| {
            key.pid == pid
                && (key.instance_id.as_deref() == instance_id
                    || key.instance_id.is_none()
                    || instance_id.is_none())
        })
        .cloned()
        .collect()
}

pub(super) fn single_session_process_name(session_keys: &[AudioSessionKey]) -> Option<&str> {
    let process_name = session_keys.first()?.process_name.as_str();
    session_keys
        .iter()
        .all(|key| key.process_name == process_name)
        .then_some(process_name)
}

pub(super) fn unresolved_session_keys_for_pid(
    session_keys: &HashSet<AudioSessionKey>,
    pid: u32,
) -> Option<Vec<AudioSessionKey>> {
    let matches = session_keys
        .iter()
        .filter(|key| key.pid == pid)
        .cloned()
        .collect::<Vec<_>>();
    (!matches.is_empty()).then_some(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn managed(mut target: TargetProcess) -> TargetProcess {
        target.managed_muted = true;
        target
    }

    #[test]
    fn target_recovery_uses_only_persisted_matching_identities() {
        let mut broad = managed(TargetProcess::new("game.exe").unwrap());
        broad.enabled = false;
        let broad_targets = [broad];
        let broad_lookup = ManagedTargetLookup::new(&broad_targets);

        assert!(broad_lookup.may_include_pid(8));
        assert!(broad_lookup.has_recovery_match("game.exe", 8));
        assert!(!broad_lookup.has_recovery_match("other.exe", 8));

        let pid_target = managed(TargetProcess::for_pid("chat.exe", 42).unwrap());
        let inactive = TargetProcess::for_pid("music.exe", 7).unwrap();
        let targets = [pid_target, inactive];
        let lookup = ManagedTargetLookup::new(&targets);

        assert!(lookup.may_include_pid(42));
        assert!(!lookup.may_include_pid(7));
        assert!(lookup.has_recovery_match("chat.exe", 42));
        assert!(!lookup.has_recovery_match("other.exe", 42));
    }

    #[test]
    fn persisted_target_recovery_covers_restarted_and_replacement_sessions() {
        let target = managed(TargetProcess::new("game.exe").unwrap());
        let targets = [target];
        let lookup = ManagedTargetLookup::new(&targets);
        let owned =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("owned".to_owned()));
        let replacement = AudioSessionKey::from_normalized(
            7,
            "game.exe".to_owned(),
            Some("replacement".to_owned()),
        );
        let runtime_ownership = HashSet::from([owned.clone()]);

        assert_eq!(
            select_runtime_managed_session(&owned, &runtime_ownership),
            ManagedSessionSelection::Exact
        );
        assert_eq!(
            select_runtime_managed_session(&replacement, &runtime_ownership),
            ManagedSessionSelection::Unmanaged
        );
        assert!(lookup.has_recovery_match("game.exe", 7));
        assert!(lookup.has_recovery_match("game.exe", 8));
        assert_eq!(
            select_runtime_managed_session(&replacement, &HashSet::new()),
            ManagedSessionSelection::Unmanaged
        );
    }

    #[test]
    fn coarse_runtime_ownership_retains_exact_siblings() {
        let exact =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("owned".to_owned()));
        let coarse = AudioSessionKey::from_normalized(7, "game.exe".to_owned(), None);
        let runtime_ownership = HashSet::from([exact.clone()]);

        assert_eq!(
            select_runtime_managed_session(&coarse, &runtime_ownership),
            ManagedSessionSelection::CoarseRuntime
        );

        let mut outcome = PlanApplyOutcome::new(runtime_ownership.len());
        outcome.keep_session_ownership(coarse.clone(), &runtime_ownership);
        assert!(outcome.active_managed_sessions.contains(&exact));
        assert!(outcome.active_managed_sessions.contains(&coarse));
    }

    #[test]
    fn target_updates_follow_persisted_owners_before_fallback_identities() {
        let broad = managed(TargetProcess::new("game.exe").unwrap());
        let pid_target = TargetProcess::for_pid("game.exe", 42).unwrap();
        let targets = [broad.clone(), pid_target];
        let lookup = ManagedTargetLookup::new(&targets);
        let session = AudioSessionIdentity {
            process_name: "game.exe",
            pid: 42,
        };
        let fallback = Some(TargetMuteIdentity {
            process_name: "game.exe",
            pid: Some(42),
        });
        let mut muted = PlanApplyOutcome::new(0);
        muted.set_target_states_for_session(&lookup, session, fallback, true);
        assert_eq!(muted.target_updates.len(), 2);
        assert!(muted.target_updates.iter().all(|update| update.muted));

        let broad_targets = [broad];
        let broad_lookup = ManagedTargetLookup::new(&broad_targets);
        let mut unmuted = PlanApplyOutcome::new(0);
        unmuted.set_target_states_for_session(&broad_lookup, session, fallback, false);
        assert_eq!(
            unmuted.target_updates,
            vec![TargetMuteStateUpdate {
                process_name: "game.exe".to_owned(),
                pid: None,
                muted: false,
            }]
        );
    }

    #[test]
    fn target_update_reconciliation_keeps_mutes_and_blocks_unsafe_clears() {
        let game = TargetMuteIdentity {
            process_name: "game.exe",
            pid: None,
        };
        let chat = TargetMuteIdentity {
            process_name: "chat.exe",
            pid: Some(7),
        };

        let mut merged = PlanApplyOutcome::new(0);
        merged.set_target_state(game, false);
        merged.set_target_state(game, true);
        assert_eq!(
            merged.target_updates,
            vec![TargetMuteStateUpdate::new(game, true)]
        );

        let mut locally_blocked = PlanApplyOutcome::new(0);
        locally_blocked.set_target_state(game, false);
        locally_blocked.block_unmuted_target_state(game);
        locally_blocked.set_target_state(game, false);
        locally_blocked.set_target_state(chat, false);
        assert_eq!(
            locally_blocked.target_updates,
            vec![TargetMuteStateUpdate::new(chat, false)]
        );

        let mut globally_blocked = PlanApplyOutcome::new(0);
        globally_blocked.set_target_state(game, false);
        globally_blocked.set_target_state(chat, true);
        globally_blocked.block_all_unmuted_target_states();
        globally_blocked.set_target_state(game, false);
        assert_eq!(
            globally_blocked.target_updates,
            vec![TargetMuteStateUpdate::new(chat, true)]
        );
    }

    #[test]
    fn unresolved_sessions_block_only_the_clear_updates_they_make_uncertain() {
        let pid_target = managed(TargetProcess::for_pid("game.exe", 42).unwrap());
        let pid_targets = [pid_target];
        let pid_lookup = ManagedTargetLookup::new(&pid_targets);
        let pid_identity = TargetMuteIdentity {
            process_name: "game.exe",
            pid: Some(42),
        };

        let mut unknown = PlanApplyOutcome::new(0);
        unknown.set_target_state(pid_identity, false);
        assert!(unknown.block_unmuted_pid_target_states(42, &pid_lookup, None));
        assert!(unknown.target_updates.is_empty());
        assert_eq!(unknown.uncertain_unmuted_target_updates.len(), 1);

        let known_sessions = HashSet::from([AudioSessionKey::from_normalized(
            42,
            "game.exe".to_owned(),
            Some("owned".to_owned()),
        )]);
        let mut known = PlanApplyOutcome::new(known_sessions.len());
        known.block_unmuted_target_states_for_unresolved_pid(42, &known_sessions, &pid_lookup);
        assert!(known.uncertain_unmuted_target_updates.is_empty());

        let broad = managed(TargetProcess::new("game.exe").unwrap());
        let other_pid = managed(TargetProcess::for_pid("chat.exe", 7).unwrap());
        let mixed_targets = [broad, other_pid];
        let mixed_lookup = ManagedTargetLookup::new(&mixed_targets);
        let broad_identity = TargetMuteIdentity {
            process_name: "game.exe",
            pid: None,
        };
        let other_identity = TargetMuteIdentity {
            process_name: "chat.exe",
            pid: Some(7),
        };
        let mut mixed = PlanApplyOutcome::new(0);
        mixed.set_target_state(broad_identity, false);
        mixed.set_target_state(other_identity, false);
        mixed.block_unmuted_process_target_states(&mixed_lookup);
        assert_eq!(
            mixed.target_updates,
            vec![TargetMuteStateUpdate::new(other_identity, false)]
        );
    }

    #[test]
    fn plan_failures_preserve_the_first_diagnostic() {
        let mut outcome = PlanApplyOutcome::new(0);

        outcome.mark_failure("first");
        outcome.mark_failure("second");

        assert!(outcome.had_failures);
        assert_eq!(outcome.failure_detail.as_deref(), Some("first"));
    }
}
