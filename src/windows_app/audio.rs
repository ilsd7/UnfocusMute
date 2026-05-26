use crate::config::{TargetProcess, compare_target_identity};
use crate::engine::{AudioSessionKey, MutePlanner, TargetMatchKind, TargetMatcher};
use crate::windows_app::error::{Context, Result, message_error};
use crate::windows_app::process::{self, ProcessInfo, ProcessRefreshOutcome};
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashSet;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::{
    DEVICE_STATE, DEVICE_STATE_ACTIVE, EDataFlow, ERole, IAudioSessionControl2,
    IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, IMMNotificationClient,
    IMMNotificationClient_Impl, ISimpleAudioVolume, MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree};
use windows::core::{Interface, PCWSTR, PWSTR, implement};
use windows_core::BOOL;

const EXPECTED_MANAGED_SESSION_COUNT: usize = 8;

pub struct AudioController {
    enumerator: IMMDeviceEnumerator,
    managers: Vec<IAudioSessionManager2>,
    endpoint_ids: Vec<String>,
    endpoint_notification: Option<EndpointNotification>,
}

pub struct MuteApplyResult {
    pub had_failures: bool,
}

pub struct PlannedMuteApplyResult {
    pub had_failures: bool,
    pub target_updates: Vec<TargetMuteStateUpdate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetMuteStateUpdate {
    pub process_name: String,
    pub pid: Option<u32>,
    pub muted: bool,
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
    ) -> ProcessRefreshOutcome {
        process::replace_processes(processes, scratch, |next| {
            self.collect_session_processes(next).is_ok()
        })
    }

    fn collect_session_processes(&self, processes: &mut Vec<ProcessInfo>) -> Result<()> {
        self.visit_sessions_matching(
            false,
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
        &self,
        session_keys: &mut HashSet<AudioSessionKey>,
    ) -> Result<MuteApplyResult> {
        if session_keys.is_empty() {
            return Ok(MuteApplyResult {
                had_failures: false,
            });
        }

        let failed_sessions = {
            let lookup = ManagedSessionLookup::new(session_keys);
            let mut failed_sessions = None;
            self.visit_sessions_matching(
                false,
                |pid| lookup.may_include_pid(pid),
                |visit| match visit {
                    SessionVisit::Resolved(session) => {
                        if let Err(key) = apply_unmute_to_session(&session, session_keys, &lookup) {
                            failed_sessions
                                .get_or_insert_with(|| HashSet::with_capacity(session_keys.len()))
                                .insert(key);
                        }
                    }
                    SessionVisit::Unresolved(session) => {
                        if let Err(keys) =
                            apply_unmute_to_unresolved_session(&session, session_keys)
                        {
                            failed_sessions
                                .get_or_insert_with(|| HashSet::with_capacity(session_keys.len()))
                                .extend(keys);
                        }
                    }
                    SessionVisit::Unreadable => {
                        failed_sessions
                            .get_or_insert_with(|| session_keys.iter().cloned().collect());
                    }
                },
            )?;
            failed_sessions
        };

        let had_failures = failed_sessions.is_some();
        if let Some(failed_sessions) = failed_sessions {
            session_keys.retain(|key| failed_sessions.contains(key));
        } else {
            session_keys.clear();
        }
        Ok(MuteApplyResult { had_failures })
    }

    pub fn apply_mute_plan(
        &self,
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
        let mut apply_result = PlanApplyResult::new(managed_muted_sessions.len());
        {
            let lookup = ManagedSessionLookup::new(managed_muted_sessions);
            let target_lookup = ManagedTargetLookup::new(targets);
            let needs_all_session_process_names = matcher.needs_all_session_process_names();
            let mut preserve_existing_managed_sessions = false;
            self.visit_sessions_matching(
                needs_all_session_process_names,
                |pid| {
                    needs_all_session_process_names
                        || matcher.has_pid_target(pid)
                        || lookup.may_include_pid(pid)
                        || target_lookup.may_include_pid(pid)
                },
                |visit| match visit {
                    SessionVisit::Resolved(session) => {
                        apply_plan_to_session(
                            &session,
                            &planner,
                            managed_muted_sessions,
                            &lookup,
                            &target_lookup,
                            &mut apply_result,
                        );
                    }
                    SessionVisit::Unresolved(session) => {
                        if lookup.may_include_pid(session.pid) {
                            apply_result
                                .keep_active_sessions_for_pid(session.pid, managed_muted_sessions);
                            apply_result.had_failures = true;
                        } else if unresolved_unmanaged_pid_is_failure(
                            needs_all_session_process_names,
                            matcher,
                            session.pid,
                        ) {
                            apply_result.had_failures = true;
                        }
                    }
                    SessionVisit::Unreadable => {
                        apply_result.had_failures = true;
                        preserve_existing_managed_sessions = true;
                    }
                },
            )?;
            if preserve_existing_managed_sessions {
                apply_result.keep_active_sessions(managed_muted_sessions);
            }
        }
        managed_muted_sessions.clear();
        managed_muted_sessions.extend(apply_result.active_managed_sessions);

        Ok(PlannedMuteApplyResult {
            had_failures: apply_result.had_failures,
            target_updates: apply_result.target_updates,
        })
    }

    fn visit_sessions_matching(
        &self,
        prefer_process_snapshot: bool,
        mut include_pid: impl FnMut(u32) -> bool,
        mut visit: impl FnMut(SessionVisit<'_>),
    ) -> Result<()> {
        unsafe {
            let mut process_names = if prefer_process_snapshot {
                process::ProcessNameResolver::snapshot_first()
            } else {
                process::ProcessNameResolver::new()
            };

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

impl AudioSessionControl<'_> {
    fn key(&self) -> AudioSessionKey {
        AudioSessionKey::from_normalized(self.pid, self.process_name.to_owned(), unsafe {
            session_instance_id(&self.control)
        })
    }

    fn volume(&self) -> Option<ISimpleAudioVolume> {
        self.control.cast().ok()
    }
}

impl UnresolvedAudioSessionControl {
    fn volume(&self) -> Option<ISimpleAudioVolume> {
        self.control.cast().ok()
    }

    fn instance_id(&self) -> Option<String> {
        unsafe { session_instance_id(&self.control) }
    }
}

fn session_muted(volume: &ISimpleAudioVolume) -> windows::core::Result<bool> {
    unsafe { volume.GetMute() }.map(BOOL::as_bool)
}

struct PlanApplyResult {
    active_managed_sessions: Vec<AudioSessionKey>,
    target_updates: Vec<TargetMuteStateUpdate>,
    had_failures: bool,
}

impl PlanApplyResult {
    fn new(managed_session_count: usize) -> Self {
        Self {
            active_managed_sessions: Vec::with_capacity(managed_session_count),
            target_updates: Vec::new(),
            had_failures: false,
        }
    }

    fn keep_active_session(
        &mut self,
        session: &AudioSessionControl<'_>,
        key: Option<AudioSessionKey>,
    ) {
        if self.active_managed_sessions.capacity() == 0 {
            self.active_managed_sessions
                .reserve(EXPECTED_MANAGED_SESSION_COUNT);
        }
        self.active_managed_sessions
            .push(key.unwrap_or_else(|| session.key()));
    }

    fn keep_active_sessions_for_pid(&mut self, pid: u32, session_keys: &HashSet<AudioSessionKey>) {
        for key in session_keys.iter().filter(|key| key.pid == pid) {
            if self.active_managed_sessions.capacity() == 0 {
                self.active_managed_sessions
                    .reserve(EXPECTED_MANAGED_SESSION_COUNT);
            }
            self.active_managed_sessions.push(key.clone());
        }
    }

    fn keep_active_sessions(&mut self, session_keys: &HashSet<AudioSessionKey>) {
        if self.active_managed_sessions.capacity() == 0 {
            self.active_managed_sessions
                .reserve(session_keys.len().max(EXPECTED_MANAGED_SESSION_COUNT));
        }
        self.active_managed_sessions
            .extend(session_keys.iter().cloned());
    }

    fn set_target_state(&mut self, identity: TargetMuteIdentity<'_>, muted: bool) {
        match self
            .target_updates
            .binary_search_by(|update| compare_target_update_identity(update, identity))
        {
            Ok(index) => self.target_updates[index].muted |= muted,
            Err(index) => self.target_updates.insert(
                index,
                TargetMuteStateUpdate {
                    process_name: identity.process_name.to_owned(),
                    pid: identity.pid,
                    muted,
                },
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TargetMuteIdentity<'a> {
    process_name: &'a str,
    pid: Option<u32>,
}

struct ManagedTargetLookup<'a> {
    identities: Vec<TargetMuteIdentity<'a>>,
    pid_lookup: PidPrefilter,
}

impl<'a> ManagedTargetLookup<'a> {
    fn new(targets: &'a [TargetProcess]) -> Self {
        let mut identities = Vec::new();
        let mut pid_lookup = PidPrefilter::None;

        for target in targets.iter().filter(|target| target.managed_muted) {
            identities.push(TargetMuteIdentity {
                process_name: target.name.as_str(),
                pid: target.pid,
            });
            if let Some(pid) = target.pid {
                pid_lookup.push_pid(pid);
            } else {
                pid_lookup = PidPrefilter::Any;
            }
        }
        if identities.len() > 1 {
            identities.sort_unstable_by(|left, right| {
                compare_target_identity_key(*left, right.process_name, right.pid)
            });
            identities.dedup();
        }

        Self {
            identities,
            pid_lookup: pid_lookup.finalized(),
        }
    }

    fn may_include_pid(&self, pid: u32) -> bool {
        self.pid_lookup.may_include_pid(pid)
    }

    #[cfg(test)]
    fn has_match(&self, process_name: &str, pid: u32) -> bool {
        self.matching_sessions(process_name, pid).has_match()
    }

    fn matching_sessions<'b>(
        &'b self,
        process_name: &'b str,
        pid: u32,
    ) -> TargetIdentityMatches<'a> {
        TargetIdentityMatches {
            process_target: self.find_identity(process_name, None),
            pid_target: self.find_identity(process_name, Some(pid)),
        }
    }

    fn find_identity(
        &self,
        process_name: &str,
        pid: Option<u32>,
    ) -> Option<TargetMuteIdentity<'a>> {
        match self.identities.as_slice() {
            [] => None,
            [identity] => {
                (identity.process_name == process_name && identity.pid == pid).then_some(*identity)
            }
            identities => identities
                .binary_search_by(|identity| {
                    compare_target_identity_key(*identity, process_name, pid)
                })
                .ok()
                .map(|index| identities[index]),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum PidPrefilter {
    Any,
    None,
    One(u32),
    Many(Vec<u32>),
}

impl PidPrefilter {
    fn new(any_process_name_target: bool, mut pids: Vec<u32>) -> Self {
        if any_process_name_target {
            return Self::Any;
        }
        if pids.len() < 2 {
            return match pids.pop() {
                Some(pid) => Self::One(pid),
                None => Self::None,
            };
        }

        pids.sort_unstable();
        pids.dedup();
        match pids.as_slice() {
            [] => Self::None,
            [pid] => Self::One(*pid),
            _ => Self::Many(pids),
        }
    }

    fn push_pid(&mut self, pid: u32) {
        match self {
            Self::Any => {}
            Self::None => *self = Self::One(pid),
            Self::One(existing) if *existing == pid => {}
            Self::One(existing) => {
                *self = Self::Many(vec![*existing, pid]);
            }
            Self::Many(pids) => pids.push(pid),
        }
    }

    fn finalized(self) -> Self {
        match self {
            Self::Many(pids) => Self::new(false, pids),
            other => other,
        }
    }

    fn may_include_pid(&self, pid: u32) -> bool {
        match self {
            Self::Any => true,
            Self::None => false,
            Self::One(target_pid) => *target_pid == pid,
            Self::Many(pids) => pids.binary_search(&pid).is_ok(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TargetIdentityMatches<'a> {
    process_target: Option<TargetMuteIdentity<'a>>,
    pid_target: Option<TargetMuteIdentity<'a>>,
}

impl TargetIdentityMatches<'_> {
    fn has_match(self) -> bool {
        self.process_target.is_some() || self.pid_target.is_some()
    }
}

impl<'a> Iterator for TargetIdentityMatches<'a> {
    type Item = TargetMuteIdentity<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.process_target
            .take()
            .or_else(|| self.pid_target.take())
    }
}

fn compare_target_identity_key(
    identity: TargetMuteIdentity<'_>,
    process_name: &str,
    pid: Option<u32>,
) -> CmpOrdering {
    compare_target_identity(identity.process_name, identity.pid, process_name, pid)
}

fn compare_target_update_identity(
    update: &TargetMuteStateUpdate,
    identity: TargetMuteIdentity<'_>,
) -> CmpOrdering {
    compare_target_identity(
        update.process_name.as_str(),
        update.pid,
        identity.process_name,
        identity.pid,
    )
}

struct ManagedSessionLookup<'a> {
    identities: Vec<(u32, &'a str)>,
}

impl<'a> ManagedSessionLookup<'a> {
    fn new(session_keys: &'a HashSet<AudioSessionKey>) -> Self {
        if session_keys.is_empty() {
            return Self {
                identities: Vec::new(),
            };
        }

        let mut identities = Vec::with_capacity(session_keys.len());
        for key in session_keys {
            identities.push((key.pid, key.process_name.as_str()));
        }
        identities.sort_unstable_by(compare_session_identity_entry);
        identities.dedup();
        Self { identities }
    }

    fn may_include_pid(&self, pid: u32) -> bool {
        self.identities
            .binary_search_by(|(entry_pid, _)| entry_pid.cmp(&pid))
            .is_ok()
    }

    fn may_include(&self, pid: u32, process_name: &str) -> bool {
        self.identities
            .binary_search_by(|entry| compare_session_identity_key(*entry, pid, process_name))
            .is_ok()
    }
}

fn compare_session_identity_entry(left: &(u32, &str), right: &(u32, &str)) -> CmpOrdering {
    compare_session_identity_key(*left, right.0, right.1)
}

fn compare_session_identity_key(entry: (u32, &str), pid: u32, process_name: &str) -> CmpOrdering {
    entry.0.cmp(&pid).then_with(|| entry.1.cmp(process_name))
}

fn apply_plan_to_session(
    session: &AudioSessionControl<'_>,
    planner: &MutePlanner<'_>,
    managed_muted_sessions: &HashSet<AudioSessionKey>,
    lookup: &ManagedSessionLookup<'_>,
    target_lookup: &ManagedTargetLookup<'_>,
    result: &mut PlanApplyResult,
) {
    let match_kind = planner.match_kind(session.process_name, session.pid);
    let target_matches = target_lookup.matching_sessions(session.process_name, session.pid);
    let has_managed_target = target_matches.has_match();
    let target_identity =
        target_identity_for_session(match_kind, session.process_name, session.pid);
    let mut key = None;
    let managed_session = if lookup.may_include(session.pid, session.process_name) {
        let session_key = session.key();
        let managed = managed_muted_sessions.contains(&session_key);
        key = Some(session_key);
        managed
    } else {
        false
    };
    // 앱 재시작 후에는 정확한 오디오 세션 키가 메모리에 남아 있지 않으므로,
    // 타깃 단위 복원 상태가 음소거 해제 판단에 쓰이는 유일한 단서다.
    // 저장된 타깃 상태를 실제로 해제할지는 아래 `allow_unmuted_target_update`가 제한한다.
    let managed = managed_session || has_managed_target;
    let session_is_foreground = planner.session_is_foreground(session.process_name, session.pid);
    let allow_unmuted_target_update =
        planner.can_clear_managed_target_state(match_kind, session.pid, session_is_foreground);

    if match_kind.is_none() && !managed {
        return;
    }

    let Some(desired_mute) =
        planner.desired_mute_with_foreground(match_kind, managed, session_is_foreground)
    else {
        return;
    };
    let Some(volume) = session.volume() else {
        result.had_failures = true;
        if managed_session {
            result.keep_active_session(session, key);
            set_target_states_for_session(result, target_matches, target_identity, true, true);
        } else if has_managed_target {
            set_target_states_for_session(result, target_matches, target_identity, true, true);
        }
        return;
    };
    let mute = match session_muted(&volume) {
        Ok(muted) => {
            if desired_mute == muted {
                if managed {
                    if desired_mute {
                        result.keep_active_session(session, key);
                    }
                    set_target_states_for_session(
                        result,
                        target_matches,
                        target_identity,
                        desired_mute,
                        allow_unmuted_target_update,
                    );
                }
                return;
            }
            desired_mute
        }
        Err(_) => desired_mute,
    };

    if unsafe { volume.SetMute(mute, std::ptr::null()) }.is_err() {
        result.had_failures = true;
        if managed {
            result.keep_active_session(session, key);
            set_target_states_for_session(result, target_matches, target_identity, true, true);
        }
        return;
    }

    if mute {
        result.keep_active_session(session, key);
    }
    set_target_states_for_session(
        result,
        target_matches,
        target_identity,
        mute,
        allow_unmuted_target_update,
    );
}

fn target_identity_for_session<'a>(
    match_kind: Option<TargetMatchKind>,
    process_name: &'a str,
    pid: u32,
) -> Option<TargetMuteIdentity<'a>> {
    match match_kind {
        Some(TargetMatchKind::ProcessName) => Some(TargetMuteIdentity {
            process_name,
            pid: None,
        }),
        Some(TargetMatchKind::Pid) => Some(TargetMuteIdentity {
            process_name,
            pid: Some(pid),
        }),
        None => None,
    }
}

fn set_target_states_for_session(
    result: &mut PlanApplyResult,
    target_matches: TargetIdentityMatches<'_>,
    fallback_identity: Option<TargetMuteIdentity<'_>>,
    muted: bool,
    allow_unmuted_update: bool,
) {
    if !muted && !allow_unmuted_update {
        return;
    }

    let mut updated_persisted_target = false;
    for identity in target_matches {
        result.set_target_state(identity, muted);
        updated_persisted_target = true;
    }

    if !updated_persisted_target && let Some(identity) = fallback_identity {
        result.set_target_state(identity, muted);
    }
}

fn apply_unmute_to_session(
    session: &AudioSessionControl<'_>,
    session_keys: &HashSet<AudioSessionKey>,
    lookup: &ManagedSessionLookup<'_>,
) -> std::result::Result<(), AudioSessionKey> {
    if !lookup.may_include(session.pid, session.process_name) {
        return Ok(());
    }

    let key = session.key();
    if !session_keys.contains(&key) {
        return Ok(());
    }
    let Some(volume) = session.volume() else {
        return Err(key);
    };
    let Ok(muted) = session_muted(&volume) else {
        return Err(key);
    };
    if !muted {
        return Ok(());
    }
    if unsafe { volume.SetMute(false, std::ptr::null()) }.is_err() {
        return Err(key);
    }
    Ok(())
}

fn apply_unmute_to_unresolved_session(
    session: &UnresolvedAudioSessionControl,
    session_keys: &HashSet<AudioSessionKey>,
) -> std::result::Result<Vec<AudioSessionKey>, Vec<AudioSessionKey>> {
    let instance_id = session.instance_id();
    let matching_sessions =
        matching_unresolved_session_keys(session_keys, session.pid, instance_id.as_deref());
    if matching_sessions.is_empty() {
        return Ok(matching_sessions);
    }

    let Some(volume) = session.volume() else {
        return Err(matching_sessions);
    };
    let Ok(muted) = session_muted(&volume) else {
        return Err(matching_sessions);
    };
    if muted && unsafe { volume.SetMute(false, std::ptr::null()) }.is_err() {
        return Err(matching_sessions);
    }
    Ok(matching_sessions)
}

fn matching_unresolved_session_keys(
    session_keys: &HashSet<AudioSessionKey>,
    pid: u32,
    instance_id: Option<&str>,
) -> Vec<AudioSessionKey> {
    let mut pid_match_count = 0;
    let mut fallback_match = None;
    let mut matches = Vec::new();
    for key in session_keys.iter().filter(|key| key.pid == pid) {
        pid_match_count += 1;
        fallback_match = Some(key);
        if key.instance_id.as_deref() == instance_id {
            matches.push(key.clone());
        }
    }

    if matches.is_empty()
        && instance_id.is_none()
        && pid_match_count == 1
        && let Some(key) = fallback_match
    {
        matches.push(key.clone());
    }
    matches
}

fn unresolved_unmanaged_pid_is_failure(
    needs_all_session_process_names: bool,
    matcher: &TargetMatcher,
    pid: u32,
) -> bool {
    !needs_all_session_process_names || matcher.has_pid_target(pid)
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
    fn unresolved_session_skips_ambiguous_pid_fallback() {
        let first =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("a".to_owned()));
        let second =
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), Some("b".to_owned()));
        let session_keys = HashSet::from([first, second]);

        assert!(matching_unresolved_session_keys(&session_keys, 7, None).is_empty());
    }

    #[test]
    fn managed_session_lookup_prefilters_by_pid() {
        let session_keys = (1..=EXPECTED_MANAGED_SESSION_COUNT + 1)
            .map(|pid| AudioSessionKey::from_normalized(pid as u32, format!("app{pid}.exe"), None))
            .collect::<HashSet<_>>();
        let lookup = ManagedSessionLookup::new(&session_keys);

        assert!(lookup.may_include_pid(3));
        assert!(lookup.may_include(3, "app3.exe"));
        assert!(!lookup.may_include(3, "other.exe"));
        assert!(!lookup.may_include_pid(99));
    }

    #[test]
    fn managed_session_lookup_deduplicates_repeated_identities() {
        let session_keys = (1..=EXPECTED_MANAGED_SESSION_COUNT + 1)
            .map(|index| {
                AudioSessionKey::from_normalized(
                    7,
                    "game.exe".to_owned(),
                    Some(format!("session-{index}")),
                )
            })
            .collect::<HashSet<_>>();
        let lookup = ManagedSessionLookup::new(&session_keys);

        assert_eq!(lookup.identities, vec![(7, "game.exe")]);
        assert!(lookup.may_include_pid(7));
    }

    #[test]
    fn small_managed_session_lookup_matches_pid_and_name() {
        let session_keys = HashSet::from([
            AudioSessionKey::from_normalized(7, "game.exe".to_owned(), None),
            AudioSessionKey::from_normalized(8, "chat.exe".to_owned(), None),
            AudioSessionKey::from_normalized(9, "music.exe".to_owned(), None),
        ]);
        let lookup = ManagedSessionLookup::new(&session_keys);

        assert!(lookup.may_include_pid(7));
        assert!(lookup.may_include(7, "game.exe"));
        assert!(!lookup.may_include(7, "other.exe"));
        assert!(!lookup.may_include(42, "game.exe"));
    }

    #[test]
    fn single_managed_session_lookup_matches_pid_and_name() {
        let session_keys = HashSet::from([AudioSessionKey::from_normalized(
            7,
            "game.exe".to_owned(),
            None,
        )]);
        let lookup = ManagedSessionLookup::new(&session_keys);

        assert!(lookup.may_include_pid(7));
        assert!(lookup.may_include(7, "game.exe"));
        assert!(!lookup.may_include(7, "other.exe"));
        assert!(!lookup.may_include(8, "game.exe"));
    }

    #[test]
    fn managed_target_lookup_keeps_persisted_targets_even_when_disabled() {
        let mut target = crate::config::TargetProcess::new("game.exe").unwrap();
        target.enabled = false;
        target.managed_muted = true;
        let targets = [target];
        let lookup = ManagedTargetLookup::new(&targets);

        assert!(lookup.may_include_pid(42));
        assert!(lookup.has_match("game.exe", 42));
    }

    #[test]
    fn managed_target_lookup_prefilters_pid_targets() {
        let mut managed = crate::config::TargetProcess::for_pid("game.exe", 42).unwrap();
        managed.managed_muted = true;
        let mut inactive = crate::config::TargetProcess::for_pid("chat.exe", 7).unwrap();
        inactive.managed_muted = false;
        let targets = [inactive, managed];
        let lookup = ManagedTargetLookup::new(&targets);

        assert!(lookup.may_include_pid(42));
        assert!(!lookup.may_include_pid(7));
        assert!(lookup.has_match("game.exe", 42));
        assert!(!lookup.has_match("other.exe", 42));
    }

    #[test]
    fn pid_prefilter_uses_small_variants() {
        assert_eq!(PidPrefilter::new(false, Vec::new()), PidPrefilter::None);
        assert_eq!(PidPrefilter::new(false, vec![42]), PidPrefilter::One(42));
        assert_eq!(
            PidPrefilter::new(false, vec![42, 42]),
            PidPrefilter::One(42)
        );
        assert!(PidPrefilter::new(true, Vec::new()).may_include_pid(7));
        assert!(!PidPrefilter::new(false, Vec::new()).may_include_pid(7));
        assert!(PidPrefilter::new(false, vec![7]).may_include_pid(7));
        assert!(!PidPrefilter::new(false, vec![7]).may_include_pid(8));
    }

    #[test]
    fn managed_process_name_targets_require_process_names_for_all_pids() {
        let mut target = crate::config::TargetProcess::new("game.exe").unwrap();
        target.managed_muted = true;
        let targets = [target];
        let lookup = ManagedTargetLookup::new(&targets);

        assert!(lookup.may_include_pid(42));
        assert!(lookup.may_include_pid(7));
        assert!(lookup.has_match("game.exe", 42));
        assert!(!lookup.has_match("chat.exe", 42));
    }

    #[test]
    fn managed_target_lookup_returns_all_matching_persisted_targets() {
        let mut process_target = crate::config::TargetProcess::new("game.exe").unwrap();
        process_target.managed_muted = true;
        let mut pid_target = crate::config::TargetProcess::for_pid("game.exe", 42).unwrap();
        pid_target.managed_muted = true;
        let targets = [process_target, pid_target];
        let lookup = ManagedTargetLookup::new(&targets);

        let matches = lookup.matching_sessions("game.exe", 42).collect::<Vec<_>>();

        assert_eq!(
            matches,
            vec![
                TargetMuteIdentity {
                    process_name: "game.exe",
                    pid: None,
                },
                TargetMuteIdentity {
                    process_name: "game.exe",
                    pid: Some(42),
                },
            ]
        );
    }

    #[test]
    fn managed_target_lookup_sorts_and_deduplicates_persisted_targets() {
        let mut process_target = crate::config::TargetProcess::new("game.exe").unwrap();
        process_target.managed_muted = true;
        let mut duplicate_process_target = process_target.clone();
        duplicate_process_target.enabled = false;
        let mut pid_target = crate::config::TargetProcess::for_pid("game.exe", 42).unwrap();
        pid_target.managed_muted = true;
        let targets = [pid_target, duplicate_process_target, process_target];
        let lookup = ManagedTargetLookup::new(&targets);

        let matches = lookup.matching_sessions("game.exe", 42).collect::<Vec<_>>();

        assert_eq!(
            matches,
            vec![
                TargetMuteIdentity {
                    process_name: "game.exe",
                    pid: None,
                },
                TargetMuteIdentity {
                    process_name: "game.exe",
                    pid: Some(42),
                },
            ]
        );
    }

    #[test]
    fn target_mute_updates_prefer_muted_when_sessions_disagree() {
        let mut result = PlanApplyResult::new(0);

        result.set_target_state(
            TargetMuteIdentity {
                process_name: "game.exe",
                pid: None,
            },
            false,
        );
        result.set_target_state(
            TargetMuteIdentity {
                process_name: "game.exe",
                pid: None,
            },
            true,
        );

        assert_eq!(
            result.target_updates,
            vec![TargetMuteStateUpdate {
                process_name: "game.exe".to_owned(),
                pid: None,
                muted: true,
            }]
        );
    }

    #[test]
    fn target_mute_updates_stay_sorted_for_binary_lookup() {
        let mut result = PlanApplyResult::new(0);

        result.set_target_state(
            TargetMuteIdentity {
                process_name: "game.exe",
                pid: None,
            },
            false,
        );
        result.set_target_state(
            TargetMuteIdentity {
                process_name: "chat.exe",
                pid: Some(7),
            },
            true,
        );
        result.set_target_state(
            TargetMuteIdentity {
                process_name: "game.exe",
                pid: None,
            },
            true,
        );

        assert_eq!(
            result.target_updates,
            vec![
                TargetMuteStateUpdate {
                    process_name: "chat.exe".to_owned(),
                    pid: Some(7),
                    muted: true,
                },
                TargetMuteStateUpdate {
                    process_name: "game.exe".to_owned(),
                    pid: None,
                    muted: true,
                },
            ]
        );
    }

    #[test]
    fn non_foreground_same_name_session_does_not_clear_persisted_target_mute() {
        let mut target = crate::config::TargetProcess::new("game.exe").unwrap();
        target.managed_muted = true;
        let targets = [target];
        let lookup = ManagedTargetLookup::new(&targets);
        let mut result = PlanApplyResult::new(0);

        set_target_states_for_session(
            &mut result,
            lookup.matching_sessions("game.exe", 10),
            Some(TargetMuteIdentity {
                process_name: "game.exe",
                pid: None,
            }),
            false,
            false,
        );

        assert!(result.target_updates.is_empty());
    }

    #[test]
    fn foreground_session_can_clear_persisted_target_mute() {
        let mut target = crate::config::TargetProcess::new("game.exe").unwrap();
        target.managed_muted = true;
        let targets = [target];
        let lookup = ManagedTargetLookup::new(&targets);
        let mut result = PlanApplyResult::new(0);

        set_target_states_for_session(
            &mut result,
            lookup.matching_sessions("game.exe", 20),
            Some(TargetMuteIdentity {
                process_name: "game.exe",
                pid: None,
            }),
            false,
            true,
        );

        assert_eq!(
            result.target_updates,
            vec![TargetMuteStateUpdate {
                process_name: "game.exe".to_owned(),
                pid: None,
                muted: false,
            }]
        );
    }

    #[test]
    fn unresolved_pid_only_filter_failures_reuse_prefilter_decision() {
        let matcher =
            TargetMatcher::new(&[crate::config::TargetProcess::for_pid("game.exe", 7).unwrap()]);

        assert!(unresolved_unmanaged_pid_is_failure(false, &matcher, 42));
    }

    #[test]
    fn unresolved_all_pid_scan_failures_still_require_pid_target() {
        let matcher = TargetMatcher::new(&[crate::config::TargetProcess::new("game.exe").unwrap()]);

        assert!(!unresolved_unmanaged_pid_is_failure(true, &matcher, 42));
    }

    #[test]
    fn wide_lossy_string_uses_ascii_fast_path() {
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
    }

    #[test]
    fn wide_lossy_string_preserves_unicode_and_replacement_behavior() {
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
    fn audio_session_snapshot_count_accepts_non_negative_counts() {
        assert_eq!(audio_session_snapshot_count(0).unwrap(), 0);
        assert_eq!(audio_session_snapshot_count(3).unwrap(), 3);
    }

    #[test]
    fn audio_session_snapshot_count_rejects_negative_counts() {
        assert!(audio_session_snapshot_count(-1).is_err());
    }
}
