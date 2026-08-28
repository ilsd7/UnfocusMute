use crate::config::TargetProcess;
use crate::engine::{
    AudioSessionKey, ManagedSessionLookup, MutePlanner, TargetMatchKind, TargetMatcher,
};
use crate::windows_app::error::{Context, Result, message_error};
use crate::windows_app::process::{self, ProcessInfo, ProcessRefreshOutcome};
use std::collections::{HashMap, HashSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::{
    AudioSessionStateExpired, DEVICE_STATE, DEVICE_STATE_ACTIVE, EDataFlow, ERole,
    IAudioSessionControl2, IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator,
    IMMNotificationClient, IMMNotificationClient_Impl, ISimpleAudioVolume, MMDeviceEnumerator,
    eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree};
use windows::core::{BOOL, IUnknown, Interface, PCWSTR, PWSTR, implement};

mod session_logic;

pub use session_logic::TargetMuteStateUpdate;
use session_logic::{
    AudioSessionIdentity, ManagedTargetLookup, PlanApplyOutcome, SessionControlIdentityRef,
    TargetMuteIdentity, cached_session_requires_unmute, matching_unresolved_session_keys,
    mute_state_needs_write, same_session_control, same_session_group, same_session_ownership,
    select_runtime_managed_session, should_retain_cached_session, single_session_process_name,
    unresolved_session_keys_for_pid,
};

pub struct AudioController {
    enumerator: IMMDeviceEnumerator,
    managers: Vec<IAudioSessionManager2>,
    endpoint_ids: Vec<String>,
    endpoint_notification: Option<EndpointNotification>,
    // A closing app can leave the foreground before its process and audio session disappear.
    // Keep the exact control so a later process-exit check can clear that final mute state.
    managed_session_controls: Vec<ManagedSessionControl>,
}

pub struct MuteApplyResult {
    pub had_failures: bool,
    pub failure_detail: Option<String>,
}

pub struct PlannedMuteApplyResult {
    pub had_failures: bool,
    pub failure_detail: Option<String>,
    pub target_updates: Vec<TargetMuteStateUpdate>,
    uncertain_unmuted_target_updates: Vec<TargetMuteStateUpdate>,
    block_all_unmuted_target_updates: bool,
}

impl PlannedMuteApplyResult {
    pub fn target_unmute_is_uncertain(&self, target: &TargetProcess) -> bool {
        self.block_all_unmuted_target_updates
            || self
                .uncertain_unmuted_target_updates
                .iter()
                .any(|update| update.process_name == target.name && update.pid == target.pid)
    }
}

pub struct AudioProcessRefreshResult {
    pub outcome: ProcessRefreshOutcome,
    pub failure_detail: Option<String>,
}

impl AudioController {
    pub fn new() -> Result<Self> {
        unsafe {
            let enumerator = device_enumerator()?;
            let managers = active_render_session_managers(&enumerator)?;
            let endpoint_notification = EndpointNotification::new(&enumerator).ok();
            let endpoint_ids =
                endpoint_ids_for_change_detection(endpoint_notification.is_some(), || {
                    active_render_endpoint_ids(&enumerator)
                })?;
            Ok(Self {
                enumerator,
                managers,
                endpoint_ids,
                endpoint_notification,
                managed_session_controls: Vec::new(),
            })
        }
    }

    pub fn take_endpoint_changed(&self) -> bool {
        if let Some(notification) = &self.endpoint_notification {
            notification.take_changed()
        } else {
            unsafe { active_render_endpoint_ids(&self.enumerator) }
                .map_or(true, |endpoint_ids| endpoint_ids != self.endpoint_ids)
        }
    }

    pub fn refresh_session_processes(
        &self,
        processes: &mut Vec<ProcessInfo>,
        scratch: &mut Vec<ProcessInfo>,
    ) -> AudioProcessRefreshResult {
        let mut failure_detail = None;
        let outcome = process::replace_processes(processes, scratch, |next| {
            match self.collect_session_processes(next) {
                Ok(()) => true,
                Err(error) => {
                    failure_detail = Some(error.to_string());
                    false
                }
            }
        });
        AudioProcessRefreshResult {
            outcome,
            failure_detail,
        }
    }

    fn collect_session_processes(&self, processes: &mut Vec<ProcessInfo>) -> Result<()> {
        self.visit_sessions_matching(
            |_| true,
            |visit| {
                if let SessionVisit::Resolved(session) = visit {
                    processes.push(ProcessInfo {
                        pid: session.pid,
                        name: session.process_name.to_owned(),
                    });
                }
            },
        )
    }

    pub fn unmute_sessions(
        &mut self,
        session_keys: &mut HashSet<AudioSessionKey>,
    ) -> Result<MuteApplyResult> {
        if session_keys.is_empty() {
            return Ok(MuteApplyResult {
                had_failures: false,
                failure_detail: None,
            });
        }

        let mut restored_sessions = HashSet::with_capacity(session_keys.len());
        let mut failed_sessions = HashSet::new();
        let mut failure_detail = None;
        let mut observed_controls = Vec::new();
        {
            let lookup = ManagedSessionLookup::new(session_keys);
            let visit_result = self.visit_sessions_matching(
                |pid| lookup.may_include_pid(pid),
                |visit| match visit {
                    SessionVisit::Resolved(session) => {
                        let key = session.key();
                        let token = session.identity_token();
                        match apply_unmute_to_session(&session, &key, session_keys) {
                            Ok(keys) => {
                                if !keys.is_empty() {
                                    observed_controls.push(SessionControlIdentity { key, token });
                                }
                                restored_sessions.extend(keys);
                            }
                            Err(keys) => {
                                if !keys.is_empty() {
                                    observed_controls.push(SessionControlIdentity { key, token });
                                }
                                record_failure_detail(
                                    &mut failure_detail,
                                    format!(
                                        "unmute audio session failed for {} (PID {})",
                                        session.process_name, session.pid
                                    ),
                                );
                                failed_sessions.extend(keys);
                            }
                        }
                    }
                    SessionVisit::Unresolved(session) => {
                        let token = session.identity_token();
                        let instance_id = session.instance_id();
                        match apply_unmute_to_unresolved_session(
                            &session,
                            instance_id.as_deref(),
                            session_keys,
                        ) {
                            Ok(keys) => {
                                observe_unresolved_session(
                                    &mut observed_controls,
                                    &keys,
                                    session.pid,
                                    instance_id.clone(),
                                    token,
                                );
                                restored_sessions.extend(keys);
                            }
                            Err(keys) => {
                                observe_unresolved_session(
                                    &mut observed_controls,
                                    &keys,
                                    session.pid,
                                    instance_id.clone(),
                                    token,
                                );
                                record_failure_detail(
                                    &mut failure_detail,
                                    format!(
                                        "unmute unresolved audio session failed for PID {}",
                                        session.pid
                                    ),
                                );
                                failed_sessions.extend(keys);
                            }
                        }
                    }
                    SessionVisit::Unreadable => {
                        record_failure_detail(
                            &mut failure_detail,
                            "could not unmute because an audio session was unreadable",
                        );
                        failed_sessions.extend(session_keys.iter().cloned());
                    }
                },
            );
            if let Err(error) = visit_result {
                record_failure_detail(
                    &mut failure_detail,
                    format!("enumerate audio sessions while restoring mute state failed: {error}"),
                );
                failed_sessions.extend(session_keys.iter().cloned());
            }
        }

        let mut retained_controls = Vec::new();
        for session in std::mem::take(&mut self.managed_session_controls) {
            if !session_keys.contains(&session.key) {
                retained_controls.push(session);
                continue;
            }
            if let Some(observed) = observed_controls
                .iter()
                .find(|observed| same_session_control(session.identity(), observed.as_ref()))
            {
                if failed_sessions
                    .iter()
                    .any(|failed| same_session_ownership(failed, &observed.key))
                {
                    retained_controls.push(session);
                }
                continue;
            }
            match session.unmute() {
                Ok(()) => {
                    restored_sessions.insert(session.key);
                }
                Err(error) => {
                    record_failure_detail(
                        &mut failure_detail,
                        format!(
                            "unmute cached audio session failed for {} (PID {}): {error}",
                            session.key.process_name, session.key.pid
                        ),
                    );
                    failed_sessions.insert(session.key.clone());
                    retained_controls.push(session);
                }
            }
        }
        self.managed_session_controls = retained_controls;

        for key in restored_sessions {
            if !failed_sessions.contains(&key) {
                session_keys.remove(&key);
            }
        }
        if !session_keys.is_empty() && failure_detail.is_none() {
            failure_detail = Some(
                "could not unmute because the managed audio session is unavailable".to_owned(),
            );
        }
        let had_failures = !session_keys.is_empty();
        Ok(MuteApplyResult {
            had_failures,
            failure_detail,
        })
    }

    pub fn apply_mute_plan(
        &mut self,
        matcher: &TargetMatcher,
        foreground_pid: Option<u32>,
        foreground_process_name: Option<&str>,
        managed_muted_sessions: &mut HashSet<AudioSessionKey>,
        targets: &[TargetProcess],
    ) -> Result<PlannedMuteApplyResult> {
        let planner = MutePlanner::new_with_normalized_foreground(
            matcher,
            foreground_pid,
            foreground_process_name,
        );
        let force_unmute_controls = self
            .managed_session_controls
            .iter()
            .filter(|session| {
                managed_muted_sessions.contains(&session.key) && session.process_has_exited()
            })
            .map(ManagedSessionControl::identity)
            .collect::<Vec<_>>();
        let mut outcome = PlanApplyOutcome::new(managed_muted_sessions.len());
        let mut pending_controls = Vec::with_capacity(managed_muted_sessions.len());
        let mut observed_controls = Vec::new();
        {
            let lookup = ManagedSessionLookup::new(managed_muted_sessions);
            let target_lookup = ManagedTargetLookup::new(targets);
            let needs_all_session_process_names = matcher.needs_all_session_process_names();
            let mut preserve_existing_managed_sessions = false;
            let visit_result = self.visit_sessions_matching(
                |pid| {
                    needs_all_session_process_names
                        || matcher.has_pid_target(pid)
                        || lookup.may_include_pid(pid)
                        || target_lookup.may_include_pid(pid)
                },
                |visit| match visit {
                    SessionVisit::Resolved(session) => {
                        let identity = session.identity_token();
                        let key = session.key();
                        observed_controls.push(SessionControlIdentity {
                            key: key.clone(),
                            token: identity,
                        });
                        apply_plan_to_session(
                            &session,
                            &key,
                            &planner,
                            managed_muted_sessions,
                            &target_lookup,
                            force_unmute_controls.iter().any(|cached| {
                                cached_session_requires_unmute(
                                    *cached,
                                    SessionControlIdentityRef {
                                        key: &key,
                                        token: identity,
                                    },
                                )
                            }),
                            SessionApplyOutput {
                                outcome: &mut outcome,
                                pending_controls: &mut pending_controls,
                            },
                        );
                    }
                    SessionVisit::Unresolved(session) => {
                        let identity = session.identity_token();
                        let instance_id = session.instance_id();
                        let matching_sessions = matching_unresolved_session_keys(
                            managed_muted_sessions,
                            session.pid,
                            instance_id.as_deref(),
                        );
                        if let Some(process_name) = single_session_process_name(&matching_sessions)
                        {
                            let session = AudioSessionControl {
                                control: session.control.clone(),
                                pid: session.pid,
                                process_name,
                            };
                            let key = session.key();
                            observed_controls.push(SessionControlIdentity {
                                key: key.clone(),
                                token: identity,
                            });
                            apply_plan_to_session(
                                &session,
                                &key,
                                &planner,
                                managed_muted_sessions,
                                &target_lookup,
                                force_unmute_controls.iter().any(|cached| {
                                    cached_session_requires_unmute(
                                        *cached,
                                        SessionControlIdentityRef {
                                            key: &key,
                                            token: identity,
                                        },
                                    )
                                }),
                                SessionApplyOutput {
                                    outcome: &mut outcome,
                                    pending_controls: &mut pending_controls,
                                },
                            );
                            return;
                        }
                        if lookup.may_include_pid(session.pid) {
                            outcome
                                .keep_active_sessions_for_pid(session.pid, managed_muted_sessions);
                            outcome.block_unmuted_target_states_for_unresolved_pid(
                                session.pid,
                                managed_muted_sessions,
                                &target_lookup,
                            );
                            outcome.mark_failure(format!(
                                "audio session process name unavailable for managed PID {}",
                                session.pid
                            ));
                        } else {
                            // With only a saved broad target, an unknown process may be an audio
                            // session that still needs restoring. Delay only broad clear updates;
                            // a later successful pass can clear them without losing recovery.
                            outcome.block_unmuted_process_target_states(&target_lookup);
                            let has_managed_pid_target = outcome.block_unmuted_pid_target_states(
                                session.pid,
                                &target_lookup,
                                None,
                            );
                            if matcher.has_pid_target(session.pid) || has_managed_pid_target {
                                outcome.mark_failure(format!(
                                    "audio session process name unavailable for PID {}",
                                    session.pid
                                ));
                            }
                        }
                    }
                    SessionVisit::Unreadable => {
                        outcome.mark_failure("audio session unavailable while applying mute plan");
                        outcome.block_all_unmuted_target_states();
                        preserve_existing_managed_sessions = true;
                    }
                },
            );
            if let Err(error) = visit_result {
                outcome.mark_failure(format!("enumerate audio sessions failed: {error}"));
                outcome.block_all_unmuted_target_states();
                preserve_existing_managed_sessions = true;
            }
            if preserve_existing_managed_sessions {
                outcome.keep_active_sessions(managed_muted_sessions);
            }

            let mut reusable_observed_controls = Vec::new();
            let mut retained_controls = Vec::new();
            for session in std::mem::take(&mut self.managed_session_controls) {
                if !managed_muted_sessions.contains(&session.key) {
                    continue;
                }
                if observed_controls
                    .iter()
                    .any(|observed| same_session_control(session.identity(), observed.as_ref()))
                {
                    reusable_observed_controls.push(session);
                    continue;
                }
                let match_kind = planner.match_kind(&session.key.process_name, session.key.pid);
                let session_is_foreground =
                    planner.session_is_foreground(&session.key.process_name, session.key.pid);
                let desired_mute =
                    planner.desired_mute_with_foreground(match_kind, true, session_is_foreground)
                        == Some(true);
                if should_retain_cached_session(
                    desired_mute,
                    session.process_has_exited(),
                    session.has_expired(),
                ) {
                    outcome.active_managed_sessions.push(session.key.clone());
                    target_lookup.for_each_matching(
                        &session.key.process_name,
                        session.key.pid,
                        |identity| {
                            outcome.block_unmuted_target_state(identity);
                        },
                    );
                    retained_controls.push(session);
                    continue;
                }
                match session.unmute() {
                    Ok(()) => outcome.set_target_states_for_session(
                        &target_lookup,
                        AudioSessionIdentity {
                            process_name: &session.key.process_name,
                            pid: session.key.pid,
                        },
                        None,
                        false,
                    ),
                    Err(error) => {
                        outcome.mark_failure(format!(
                            "unmute cached audio session failed for {} (PID {}): {error}",
                            session.key.process_name, session.key.pid
                        ));
                        outcome.active_managed_sessions.push(session.key.clone());
                        target_lookup.for_each_matching(
                            &session.key.process_name,
                            session.key.pid,
                            |identity| {
                                outcome.block_unmuted_target_state(identity);
                            },
                        );
                        retained_controls.push(session);
                    }
                }
            }

            let mut pending_group_counts = HashMap::new();
            for pending in &pending_controls {
                *pending_group_counts
                    .entry((pending.key.pid, pending.key.process_name.clone()))
                    .or_insert(0usize) += 1;
            }

            let mut next_controls = retained_controls;
            for pending in pending_controls {
                if let Some(index) = reusable_observed_controls.iter().position(|existing| {
                    same_session_control(existing.identity(), pending.identity())
                }) {
                    let mut existing = reusable_observed_controls.swap_remove(index);
                    existing.refresh(pending);
                    next_controls.push(existing);
                } else if pending.key.instance_id.is_none() {
                    let required = pending_group_counts
                        .get(&(pending.key.pid, pending.key.process_name.clone()))
                        .copied()
                        .unwrap_or(1);
                    let cached = next_controls
                        .iter()
                        .chain(&reusable_observed_controls)
                        .filter(|existing| same_session_group(&existing.key, &pending.key))
                        .count();
                    if cached < required {
                        next_controls.push(ManagedSessionControl::new(pending));
                    }
                } else {
                    next_controls.push(ManagedSessionControl::new(pending));
                }
            }
            next_controls.extend(reusable_observed_controls.into_iter().filter(|control| {
                outcome
                    .active_managed_sessions
                    .iter()
                    .any(|key| key == &control.key)
            }));
            self.managed_session_controls = next_controls;
        }
        managed_muted_sessions.clear();
        managed_muted_sessions.extend(outcome.active_managed_sessions.iter().cloned());

        Ok(PlannedMuteApplyResult {
            had_failures: outcome.had_failures,
            failure_detail: outcome.failure_detail,
            target_updates: outcome.target_updates,
            uncertain_unmuted_target_updates: outcome.uncertain_unmuted_target_updates,
            block_all_unmuted_target_updates: outcome.block_all_unmuted_target_updates,
        })
    }

    fn visit_sessions_matching(
        &self,
        mut include_pid: impl FnMut(u32) -> bool,
        mut visit: impl FnMut(SessionVisit<'_>),
    ) -> Result<()> {
        unsafe {
            let mut process_names = process::ProcessNameResolver::new();

            for manager in &self.managers {
                let enumerator = manager
                    .GetSessionEnumerator()
                    .context("get audio session enumerator")?;
                let count = enumerator.GetCount().context("get audio session count")?;
                let count = audio_session_snapshot_count(count)?;

                for index in 0..count {
                    let Ok(control) = enumerator.GetSession(index as i32) else {
                        visit(SessionVisit::Unreadable);
                        continue;
                    };
                    let Ok(control2) = control.cast::<IAudioSessionControl2>() else {
                        visit(SessionVisit::Unreadable);
                        continue;
                    };
                    let pid = match session_process_id(&control2) {
                        Ok(pid) => pid,
                        Err(_) => {
                            visit(SessionVisit::Unreadable);
                            continue;
                        }
                    };
                    let Some(pid) = pid else {
                        continue;
                    };
                    if !include_pid(pid) {
                        continue;
                    }
                    let Some(process_name) = process_names.name(pid) else {
                        visit(SessionVisit::Unresolved(UnresolvedAudioSessionControl {
                            control: control2,
                            pid,
                        }));
                        continue;
                    };

                    visit(SessionVisit::Resolved(AudioSessionControl {
                        control: control2,
                        pid,
                        process_name,
                    }));
                }
            }

            Ok(())
        }
    }
}

struct AudioSessionControl<'a> {
    control: IAudioSessionControl2,
    pid: u32,
    process_name: &'a str,
}

enum SessionVisit<'a> {
    Resolved(AudioSessionControl<'a>),
    Unresolved(UnresolvedAudioSessionControl),
    Unreadable,
}

struct UnresolvedAudioSessionControl {
    control: IAudioSessionControl2,
    pid: u32,
}

struct ManagedSessionControl {
    key: AudioSessionKey,
    session: IAudioSessionControl2,
    volume: ISimpleAudioVolume,
    identity: usize,
    process: Option<process::ProcessLifetime>,
}

struct PendingManagedSessionControl {
    key: AudioSessionKey,
    session: IAudioSessionControl2,
    volume: ISimpleAudioVolume,
    identity: usize,
}

struct SessionApplyOutput<'a> {
    outcome: &'a mut PlanApplyOutcome,
    pending_controls: &'a mut Vec<PendingManagedSessionControl>,
}

struct SessionControlIdentity {
    key: AudioSessionKey,
    token: usize,
}

impl SessionControlIdentity {
    fn as_ref(&self) -> SessionControlIdentityRef<'_> {
        SessionControlIdentityRef {
            key: &self.key,
            token: self.token,
        }
    }
}

impl PendingManagedSessionControl {
    fn identity(&self) -> SessionControlIdentityRef<'_> {
        SessionControlIdentityRef {
            key: &self.key,
            token: self.identity,
        }
    }
}

impl ManagedSessionControl {
    fn new(pending: PendingManagedSessionControl) -> Self {
        let process = process::ProcessLifetime::open(pending.key.pid);
        Self {
            key: pending.key,
            session: pending.session,
            volume: pending.volume,
            identity: pending.identity,
            process,
        }
    }

    fn process_has_exited(&self) -> bool {
        self.process
            .as_ref()
            .and_then(process::ProcessLifetime::has_exited)
            .unwrap_or(false)
    }

    fn identity(&self) -> SessionControlIdentityRef<'_> {
        SessionControlIdentityRef {
            key: &self.key,
            token: self.identity,
        }
    }

    fn has_expired(&self) -> bool {
        unsafe { self.session.GetState() }.is_ok_and(|state| state == AudioSessionStateExpired)
    }

    fn refresh(&mut self, pending: PendingManagedSessionControl) {
        let process_changed = self.key.pid != pending.key.pid;
        self.key = pending.key;
        self.session = pending.session;
        self.volume = pending.volume;
        self.identity = pending.identity;
        if process_changed || self.process.is_none() {
            self.process = process::ProcessLifetime::open(self.key.pid);
        }
    }

    fn unmute(&self) -> windows::core::Result<()> {
        unsafe { self.volume.SetMute(false, std::ptr::null()) }
    }
}

impl AudioSessionControl<'_> {
    fn identity(&self) -> AudioSessionIdentity<'_> {
        AudioSessionIdentity {
            process_name: self.process_name,
            pid: self.pid,
        }
    }

    fn key(&self) -> AudioSessionKey {
        AudioSessionKey::from_normalized(self.pid, self.process_name.to_owned(), unsafe {
            session_instance_id(&self.control)
        })
    }

    fn volume(&self) -> Option<ISimpleAudioVolume> {
        self.control.cast().ok()
    }

    fn identity_token(&self) -> usize {
        session_control_identity(&self.control)
    }
}

impl UnresolvedAudioSessionControl {
    fn volume(&self) -> Option<ISimpleAudioVolume> {
        self.control.cast().ok()
    }

    fn instance_id(&self) -> Option<String> {
        unsafe { session_instance_id(&self.control) }
    }

    fn identity_token(&self) -> usize {
        session_control_identity(&self.control)
    }
}

fn session_control_identity(control: &IAudioSessionControl2) -> usize {
    control
        .cast::<IUnknown>()
        .map_or(control.as_raw() as usize, |identity| {
            identity.as_raw() as usize
        })
}

fn session_muted(volume: &ISimpleAudioVolume) -> windows::core::Result<bool> {
    unsafe { volume.GetMute() }.map(BOOL::as_bool)
}

fn keep_session_control(
    controls: &mut Vec<PendingManagedSessionControl>,
    key: AudioSessionKey,
    session: IAudioSessionControl2,
    volume: ISimpleAudioVolume,
) {
    let identity = session_control_identity(&session);
    if let Some(existing) = controls
        .iter_mut()
        .find(|existing| existing.identity == identity)
    {
        existing.key = key;
        existing.session = session;
        existing.volume = volume;
    } else {
        controls.push(PendingManagedSessionControl {
            key,
            session,
            volume,
            identity,
        });
    }
}

fn apply_plan_to_session(
    session: &AudioSessionControl<'_>,
    session_key: &AudioSessionKey,
    planner: &MutePlanner<'_>,
    managed_muted_sessions: &HashSet<AudioSessionKey>,
    target_lookup: &ManagedTargetLookup<'_>,
    force_unmute: bool,
    output: SessionApplyOutput<'_>,
) {
    let SessionApplyOutput {
        outcome,
        pending_controls,
    } = output;
    let match_kind = planner.match_kind(session.process_name, session.pid);
    let runtime_candidate = ManagedSessionLookup::new(managed_muted_sessions)
        .contains(session.process_name, session.pid);
    let target_recovery = target_lookup.has_recovery_match(session.process_name, session.pid);
    if match_kind.is_none() && !runtime_candidate && !target_recovery && !force_unmute {
        return;
    }

    // Exact instance identifiers are needed only to distinguish existing runtime ownership.
    // A saved target marker is already sufficient for restart and replacement recovery.
    let mut key = (runtime_candidate || force_unmute).then(|| session_key.clone());
    let runtime_managed = key.as_ref().is_some_and(|key| {
        select_runtime_managed_session(key, managed_muted_sessions).is_managed()
    });
    let managed = force_unmute || runtime_managed || target_recovery;
    let fallback_identity =
        target_identity_for_session(match_kind, session.process_name, session.pid);
    let session_is_foreground = planner.session_is_foreground(session.process_name, session.pid);

    let desired_mute = if force_unmute {
        false
    } else {
        let Some(desired_mute) =
            planner.desired_mute_with_foreground(match_kind, managed, session_is_foreground)
        else {
            return;
        };
        desired_mute
    };
    let Some(volume) = session.volume() else {
        outcome.mark_failure(format!(
            "audio session mute control unavailable for {} (PID {})",
            session.process_name, session.pid
        ));
        if managed {
            outcome.keep_session_ownership(
                key.take().unwrap_or_else(|| session_key.clone()),
                managed_muted_sessions,
            );
            outcome.set_target_states_for_session(
                target_lookup,
                session.identity(),
                fallback_identity,
                true,
            );
        }
        return;
    };
    let current_mute = session_muted(&volume);
    if !mute_state_needs_write(desired_mute, &current_mute) {
        if managed {
            if desired_mute {
                let owned_key = key.take().unwrap_or_else(|| session_key.clone());
                outcome.keep_session_ownership(owned_key.clone(), managed_muted_sessions);
                keep_session_control(
                    pending_controls,
                    owned_key,
                    session.control.clone(),
                    volume.clone(),
                );
            }
            outcome.set_target_states_for_session(
                target_lookup,
                session.identity(),
                fallback_identity,
                desired_mute,
            );
        }
        return;
    }
    let mute = desired_mute;

    // If GetMute failed, SetMute is still the only way to enforce the requested state. A
    // successful mute therefore establishes ownership even though an already-muted manual state
    // cannot be distinguished in this rare error path.
    if let Err(error) = unsafe { volume.SetMute(mute, std::ptr::null()) } {
        outcome.mark_failure(format!(
            "set mute state failed for {} (PID {}): {error}",
            session.process_name, session.pid
        ));
        if managed {
            let owned_key = key.take().unwrap_or_else(|| session_key.clone());
            outcome.keep_session_ownership(owned_key.clone(), managed_muted_sessions);
            keep_session_control(
                pending_controls,
                owned_key,
                session.control.clone(),
                volume.clone(),
            );
            outcome.set_target_states_for_session(
                target_lookup,
                session.identity(),
                fallback_identity,
                true,
            );
        }
        return;
    }

    if mute {
        let owned_key = key.take().unwrap_or_else(|| session_key.clone());
        outcome.keep_session_ownership(owned_key.clone(), managed_muted_sessions);
        keep_session_control(pending_controls, owned_key, session.control.clone(), volume);
    }
    outcome.set_target_states_for_session(
        target_lookup,
        session.identity(),
        fallback_identity,
        mute,
    );
}

fn target_identity_for_session<'a>(
    match_kind: Option<TargetMatchKind>,
    process_name: &'a str,
    pid: u32,
) -> Option<TargetMuteIdentity<'a>> {
    match_kind.map(|kind| TargetMuteIdentity {
        process_name,
        pid: matches!(kind, TargetMatchKind::Pid).then_some(pid),
    })
}

fn apply_unmute_to_session(
    session: &AudioSessionControl<'_>,
    key: &AudioSessionKey,
    session_keys: &HashSet<AudioSessionKey>,
) -> std::result::Result<Vec<AudioSessionKey>, Vec<AudioSessionKey>> {
    let restored_keys = if session_keys.contains(key) {
        vec![key.clone()]
    } else {
        session_keys
            .iter()
            .filter(|managed_key| {
                managed_key.pid == key.pid
                    && managed_key.process_name == key.process_name
                    && managed_key.instance_id.is_none()
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    if restored_keys.is_empty() {
        return Ok(Vec::new());
    }
    let Some(volume) = session.volume() else {
        return Err(restored_keys);
    };
    if mute_state_needs_write(false, &session_muted(&volume))
        && unsafe { volume.SetMute(false, std::ptr::null()) }.is_err()
    {
        return Err(restored_keys);
    }
    Ok(restored_keys)
}

fn apply_unmute_to_unresolved_session(
    session: &UnresolvedAudioSessionControl,
    instance_id: Option<&str>,
    session_keys: &HashSet<AudioSessionKey>,
) -> std::result::Result<Vec<AudioSessionKey>, Vec<AudioSessionKey>> {
    let matching_sessions =
        matching_unresolved_session_keys(session_keys, session.pid, instance_id);
    if matching_sessions.is_empty() {
        if let Some(unmatched_sessions) = unresolved_session_keys_for_pid(session_keys, session.pid)
        {
            return Err(unmatched_sessions);
        }
        return Ok(matching_sessions);
    }

    let Some(volume) = session.volume() else {
        return Err(matching_sessions);
    };
    if mute_state_needs_write(false, &session_muted(&volume))
        && unsafe { volume.SetMute(false, std::ptr::null()) }.is_err()
    {
        return Err(matching_sessions);
    }
    Ok(matching_sessions)
}

fn observe_unresolved_session(
    observed_controls: &mut Vec<SessionControlIdentity>,
    matching_sessions: &[AudioSessionKey],
    pid: u32,
    instance_id: Option<String>,
    token: usize,
) {
    let Some(process_name) = single_session_process_name(matching_sessions) else {
        return;
    };
    observed_controls.push(SessionControlIdentity {
        key: AudioSessionKey::from_normalized(pid, process_name.to_owned(), instance_id),
        token,
    });
}

fn record_failure_detail(target: &mut Option<String>, detail: impl Into<String>) {
    if target.is_none() {
        *target = Some(detail.into());
    }
}

struct EndpointNotification {
    enumerator: IMMDeviceEnumerator,
    client: IMMNotificationClient,
    changed: Arc<AtomicBool>,
}

impl EndpointNotification {
    unsafe fn new(enumerator: &IMMDeviceEnumerator) -> Result<Self> {
        let changed = Arc::new(AtomicBool::new(false));
        let client: IMMNotificationClient = EndpointNotificationClient {
            changed: Arc::clone(&changed),
        }
        .into();
        unsafe { enumerator.RegisterEndpointNotificationCallback(&client) }
            .context("register audio endpoint notification")?;

        Ok(Self {
            enumerator: enumerator.clone(),
            client,
            changed,
        })
    }

    fn take_changed(&self) -> bool {
        self.changed.swap(false, Ordering::Relaxed)
    }
}

impl Drop for EndpointNotification {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .enumerator
                .UnregisterEndpointNotificationCallback(&self.client);
        }
    }
}

#[implement(IMMNotificationClient)]
struct EndpointNotificationClient {
    changed: Arc<AtomicBool>,
}

#[allow(non_snake_case)]
impl IMMNotificationClient_Impl for EndpointNotificationClient_Impl {
    fn OnDeviceStateChanged(
        &self,
        _pwstrdeviceid: &PCWSTR,
        _dwnewstate: DEVICE_STATE,
    ) -> windows::core::Result<()> {
        self.changed.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn OnDeviceAdded(&self, _pwstrdeviceid: &PCWSTR) -> windows::core::Result<()> {
        self.changed.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn OnDeviceRemoved(&self, _pwstrdeviceid: &PCWSTR) -> windows::core::Result<()> {
        self.changed.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        _pwstrdefaultdeviceid: &PCWSTR,
    ) -> windows::core::Result<()> {
        if flow == eRender && role == eMultimedia {
            self.changed.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    fn OnPropertyValueChanged(
        &self,
        _pwstrdeviceid: &PCWSTR,
        _key: &PROPERTYKEY,
    ) -> windows::core::Result<()> {
        self.changed.store(true, Ordering::Relaxed);
        Ok(())
    }
}

unsafe fn session_process_id(
    control: &IAudioSessionControl2,
) -> windows::core::Result<Option<u32>> {
    let pid = unsafe { control.GetProcessId()? };
    if pid == 0 {
        return Ok(None);
    }

    Ok(Some(pid))
}

unsafe fn session_instance_id(control: &IAudioSessionControl2) -> Option<String> {
    let value = unsafe { control.GetSessionInstanceIdentifier().ok()? };
    unsafe { co_task_mem_string(value) }
}

unsafe fn device_enumerator() -> Result<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
        .context("create audio device enumerator")
}

unsafe fn active_render_session_managers(
    enumerator: &IMMDeviceEnumerator,
) -> Result<Vec<IAudioSessionManager2>> {
    let endpoints = unsafe {
        enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .context("enumerate active render endpoints")?
    };
    let count = unsafe { endpoints.GetCount().context("get render endpoint count")? };
    let mut managers = Vec::with_capacity(count as usize);

    for index in 0..count {
        let Ok(device) = (unsafe { endpoints.Item(index) }) else {
            continue;
        };
        let Ok(manager) = (unsafe { device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) })
        else {
            continue;
        };
        managers.push(manager);
    }

    if managers.is_empty() {
        return Err(message_error(
            "no active render endpoints with audio session managers",
        ));
    }

    Ok(managers)
}

unsafe fn active_render_endpoint_ids(enumerator: &IMMDeviceEnumerator) -> Result<Vec<String>> {
    let endpoints = unsafe {
        enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .context("enumerate active render endpoints")?
    };
    let count = unsafe { endpoints.GetCount().context("get render endpoint count")? };
    let mut ids = Vec::with_capacity(count as usize);

    for index in 0..count {
        let Ok(device) = (unsafe { endpoints.Item(index) }) else {
            continue;
        };
        if let Some(id) = unsafe { endpoint_id(&device) } {
            ids.push(id);
        }
    }

    if count > 0 && ids.is_empty() {
        return Err(message_error("active render endpoint ids unavailable"));
    }

    if ids.len() > 1 {
        ids.sort_unstable();
    }
    Ok(ids)
}

fn endpoint_ids_for_change_detection(
    endpoint_notification_available: bool,
    load_endpoint_ids: impl FnOnce() -> Result<Vec<String>>,
) -> Result<Vec<String>> {
    if endpoint_notification_available {
        Ok(Vec::new())
    } else {
        load_endpoint_ids()
    }
}

fn audio_session_snapshot_count(count: i32) -> Result<usize> {
    usize::try_from(count).map_err(|_| message_error("audio session count unavailable"))
}

unsafe fn endpoint_id(device: &IMMDevice) -> Option<String> {
    let value = unsafe { device.GetId().ok()? };
    unsafe { co_task_mem_string(value) }
}

unsafe fn co_task_mem_string(value: PWSTR) -> Option<String> {
    let value = CoTaskMemString::new(value)?;

    let wide = unsafe { value.as_wide() };
    let len = wide.iter().position(|ch| *ch == 0).unwrap_or(wide.len());
    if len == 0 {
        return None;
    }

    Some(string_from_wide_lossy(&wide[..len]))
}

fn string_from_wide_lossy(wide: &[u16]) -> String {
    if wide.iter().all(|ch| *ch <= 0x7f) {
        return wide.iter().map(|ch| *ch as u8 as char).collect();
    }

    String::from_utf16_lossy(wide)
}

struct CoTaskMemString(PWSTR);

impl CoTaskMemString {
    fn new(value: PWSTR) -> Option<Self> {
        (!value.is_null()).then_some(Self(value))
    }

    unsafe fn as_wide(&self) -> &[u16] {
        unsafe { self.0.as_wide() }
    }
}

impl Drop for CoTaskMemString {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(Some(self.0.as_ptr().cast()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unreadable_mute_state_still_writes_desired_state() {
        let unreadable: std::result::Result<bool, ()> = Err(());
        assert!(mute_state_needs_write(true, &unreadable));
        assert!(mute_state_needs_write(false, &unreadable));
        assert!(!mute_state_needs_write(true, &Ok::<bool, ()>(true)));
        assert!(mute_state_needs_write(true, &Ok::<bool, ()>(false)));
    }

    #[test]
    fn cached_session_is_retained_only_when_the_plan_still_requires_mute() {
        assert!(should_retain_cached_session(true, false, false));
        assert!(!should_retain_cached_session(true, true, false));
        assert!(!should_retain_cached_session(true, false, true));
        assert!(!should_retain_cached_session(false, false, false));
    }

    #[test]
    fn exact_session_identity_does_not_depend_on_com_wrapper_reuse() {
        let exact_a = AudioSessionKey::from_normalized(
            7,
            "game.exe".to_owned(),
            Some("session-a".to_owned()),
        );
        let exact_b = AudioSessionKey::from_normalized(
            7,
            "game.exe".to_owned(),
            Some("session-b".to_owned()),
        );

        assert!(same_session_control(
            SessionControlIdentityRef {
                key: &exact_a,
                token: 1,
            },
            SessionControlIdentityRef {
                key: &exact_a,
                token: 2,
            },
        ));
        assert!(!same_session_control(
            SessionControlIdentityRef {
                key: &exact_a,
                token: 1,
            },
            SessionControlIdentityRef {
                key: &exact_b,
                token: 1,
            },
        ));
    }

    #[test]
    fn coarse_session_identity_uses_com_identity_and_group_exit_fallback() {
        let coarse = AudioSessionKey::from_normalized(7, "game.exe".to_owned(), None);
        let exact = AudioSessionKey::from_normalized(
            7,
            "game.exe".to_owned(),
            Some("session-a".to_owned()),
        );

        assert!(same_session_control(
            SessionControlIdentityRef {
                key: &coarse,
                token: 1,
            },
            SessionControlIdentityRef {
                key: &exact,
                token: 1,
            },
        ));
        assert!(!same_session_control(
            SessionControlIdentityRef {
                key: &coarse,
                token: 1,
            },
            SessionControlIdentityRef {
                key: &exact,
                token: 2,
            },
        ));
        assert!(cached_session_requires_unmute(
            SessionControlIdentityRef {
                key: &coarse,
                token: 1,
            },
            SessionControlIdentityRef {
                key: &exact,
                token: 2,
            },
        ));

        let other_exact = AudioSessionKey::from_normalized(
            7,
            "game.exe".to_owned(),
            Some("session-b".to_owned()),
        );
        assert!(!cached_session_requires_unmute(
            SessionControlIdentityRef {
                key: &exact,
                token: 1,
            },
            SessionControlIdentityRef {
                key: &other_exact,
                token: 1,
            },
        ));
    }

    #[test]
    fn unresolved_session_name_is_used_only_when_all_runtime_keys_agree() {
        let game_a =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("a".to_owned()));
        let game_b =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("b".to_owned()));
        let chat = AudioSessionKey::from_normalized(7, "chat.exe".to_owned(), Some("c".to_owned()));

        assert_eq!(
            single_session_process_name(&[game_a.clone(), game_b]),
            Some("game.exe")
        );
        assert_eq!(single_session_process_name(&[game_a, chat]), None);
        assert_eq!(single_session_process_name(&[]), None);
    }

    #[test]
    fn unresolved_control_is_observed_once_instead_of_once_per_runtime_key() {
        let sessions = [
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("a".to_owned())),
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("b".to_owned())),
        ];
        let mut observed = Vec::new();

        observe_unresolved_session(&mut observed, &sessions, 7, None, 11);

        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].key.instance_id, None);
        assert_eq!(observed[0].token, 11);
    }

    #[test]
    fn target_specific_restore_uncertainty_is_independent_of_other_failures() {
        let game = TargetProcess::new("game.exe").unwrap();
        let chat = TargetProcess::for_pid("chat.exe", 7).unwrap();
        let result = PlannedMuteApplyResult {
            had_failures: true,
            failure_detail: Some("unrelated failure".to_owned()),
            target_updates: Vec::new(),
            uncertain_unmuted_target_updates: vec![TargetMuteStateUpdate {
                process_name: chat.name.clone(),
                pid: chat.pid,
                muted: false,
            }],
            block_all_unmuted_target_updates: false,
        };

        assert!(!result.target_unmute_is_uncertain(&game));
        assert!(result.target_unmute_is_uncertain(&chat));
    }

    #[test]
    fn unreadable_session_blocks_restore_for_every_target() {
        let result = PlannedMuteApplyResult {
            had_failures: true,
            failure_detail: Some("unreadable session".to_owned()),
            target_updates: Vec::new(),
            uncertain_unmuted_target_updates: Vec::new(),
            block_all_unmuted_target_updates: true,
        };

        assert!(result.target_unmute_is_uncertain(&TargetProcess::new("game.exe").unwrap()));
    }

    #[test]
    fn unresolved_session_matches_exact_instance_id() {
        let retained =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("a".to_owned()));
        let same_pid_other_instance =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("b".to_owned()));
        let ignored = AudioSessionKey::from_normalized(8, "chat.exe".to_owned(), None);
        let session_keys = HashSet::from([
            retained.clone(),
            same_pid_other_instance.clone(),
            ignored.clone(),
        ]);

        let matches = matching_unresolved_session_keys(&session_keys, 7, Some("a"));

        assert!(matches.contains(&retained));
        assert!(!matches.contains(&same_pid_other_instance));
        assert!(!matches.contains(&ignored));
    }

    #[test]
    fn unresolved_session_uses_single_pid_fallback_without_instance_id() {
        let retained =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("a".to_owned()));
        let ignored = AudioSessionKey::from_normalized(8, "chat.exe".to_owned(), None);
        let session_keys = HashSet::from([retained.clone(), ignored.clone()]);

        let matches = matching_unresolved_session_keys(&session_keys, 7, None);

        assert!(matches.contains(&retained));
        assert!(!matches.contains(&ignored));
    }

    #[test]
    fn unresolved_session_without_instance_id_uses_coarse_pid_ownership() {
        let first =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("a".to_owned()));
        let second =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("b".to_owned()));
        let session_keys = HashSet::from([first.clone(), second.clone()]);

        let matches = matching_unresolved_session_keys(&session_keys, 7, None);
        assert!(matches.contains(&first));
        assert!(matches.contains(&second));
    }

    #[test]
    fn unresolved_session_matches_coarse_runtime_key() {
        let retained = AudioSessionKey::from_normalized(7, "game.exe".to_owned(), None);
        let ignored = AudioSessionKey::from_normalized(8, "chat.exe".to_owned(), None);
        let session_keys = HashSet::from([retained.clone(), ignored.clone()]);

        let matches = matching_unresolved_session_keys(&session_keys, 7, Some("runtime"));
        assert!(matches.contains(&retained));
        assert!(!matches.contains(&ignored));
    }

    #[test]
    fn wide_lossy_string_decodes_ascii_unicode_and_invalid_input() {
        assert_eq!(
            string_from_wide_lossy(&[
                u16::from(b'a'),
                u16::from(b'b'),
                u16::from(b'c'),
                u16::from(b'-'),
                u16::from(b'1'),
            ]),
            "abc-1"
        );
        assert_eq!(string_from_wide_lossy(&[0xd55c, 0xae00]), "한글");
        assert_eq!(string_from_wide_lossy(&[0xd800]), "\u{fffd}");
    }

    #[test]
    fn endpoint_ids_are_not_loaded_when_notifications_are_available() {
        let mut loaded = false;

        let endpoint_ids = endpoint_ids_for_change_detection(true, || {
            loaded = true;
            Err(message_error("endpoint ids unavailable"))
        })
        .unwrap();

        assert!(endpoint_ids.is_empty());
        assert!(!loaded);
    }

    #[test]
    fn endpoint_ids_are_required_when_notifications_are_unavailable() {
        let endpoint_ids =
            endpoint_ids_for_change_detection(false, || Ok(vec!["endpoint-a".to_owned()])).unwrap();

        assert_eq!(endpoint_ids, ["endpoint-a"]);
        assert!(
            endpoint_ids_for_change_detection(false, || Err(message_error(
                "endpoint ids unavailable"
            )))
            .is_err()
        );
    }

    #[test]
    fn audio_session_snapshot_count_validates_the_api_range() {
        for (count, expected) in [(0, Some(0)), (3, Some(3)), (-1, None)] {
            assert_eq!(audio_session_snapshot_count(count).ok(), expected);
        }
    }
}
