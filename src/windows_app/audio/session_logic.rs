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
            .any(|target| target.managed_muted && target_matches_session(target, process_name, pid))
    }

    pub(super) fn for_each_matching(
        &self,
        process_name: &str,
        pid: u32,
        mut apply: impl FnMut(TargetMuteIdentity<'a>),
    ) {
        for target in self.targets.iter().filter(|target| {
            target.managed_muted && target_matches_session(target, process_name, pid)
        }) {
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

fn target_matches_session(target: &TargetProcess, process_name: &str, pid: u32) -> bool {
    target.name == process_name && target.pid.is_none_or(|target_pid| target_pid == pid)
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
