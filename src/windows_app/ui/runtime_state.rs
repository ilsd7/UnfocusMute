use super::ForegroundEventHook;
use super::runtime_logic::{
    cached_foreground_process_name, foreground_process_cache_needs_refresh,
};
use crate::config::TargetProcess;
use crate::engine::{AudioSessionKey, ManagedSessionLookup, TargetMatcher};
use crate::windows_app::audio::{AudioController, PlannedMuteApplyResult};
use crate::windows_app::error::Result;
use crate::windows_app::process;
use std::collections::HashSet;

/// Runtime-only state that changes while monitoring foreground windows and audio sessions.
///
/// Keeping this separate from the window/view fields makes it harder to accidentally treat
/// transient audio ownership as presentation state or user configuration.
pub(super) struct RuntimeCoordinator {
    pub(super) audio: Option<AudioController>,
    pub(super) foreground_hook: Option<ForegroundEventHook>,
    foreground_process_name_cache: Option<(u32, Option<String>)>,
    muted_by_app: HashSet<AudioSessionKey>,
    last_target_muted: Vec<bool>,
    muted_target_count: usize,
    runtime_mute_state_dirty: bool,
    paused: bool,
    active: bool,
    pub(super) config_reload_timer_ready: bool,
    pub(super) audio_fallback_timer_interval_ms: Option<u32>,
    managed_mute_foreground_retry_pending: bool,
}

impl RuntimeCoordinator {
    pub(super) fn new(active: bool, persisted_recovery_pending: bool) -> Self {
        Self {
            audio: None,
            foreground_hook: None,
            foreground_process_name_cache: None,
            muted_by_app: HashSet::new(),
            last_target_muted: Vec::new(),
            muted_target_count: 0,
            runtime_mute_state_dirty: false,
            paused: false,
            active,
            config_reload_timer_ready: false,
            audio_fallback_timer_interval_ms: None,
            managed_mute_foreground_retry_pending: persisted_recovery_pending,
        }
    }

    pub(super) fn is_active(&self) -> bool {
        self.active
    }

    pub(super) fn activate(
        &mut self,
        paused: bool,
        managed_sessions: HashSet<AudioSessionKey>,
    ) -> bool {
        if self.active {
            return false;
        }
        self.active = true;
        self.paused = paused;
        self.muted_by_app = managed_sessions;
        true
    }

    pub(super) fn suspend(&mut self) -> bool {
        if !self.active {
            return false;
        }
        self.active = false;
        true
    }

    pub(super) fn resume(&mut self) -> bool {
        if self.active {
            return false;
        }
        self.active = true;
        true
    }

    pub(super) fn deactivate(&mut self) {
        self.active = false;
        self.managed_mute_foreground_retry_pending = false;
    }

    pub(super) fn is_paused(&self) -> bool {
        self.paused
    }

    pub(super) fn toggle_paused(&mut self) -> bool {
        self.paused = !self.paused;
        self.paused
    }

    pub(super) fn managed_sessions(&self) -> &HashSet<AudioSessionKey> {
        &self.muted_by_app
    }

    pub(super) fn managed_sessions_mut(&mut self) -> &mut HashSet<AudioSessionKey> {
        &mut self.muted_by_app
    }

    pub(super) fn has_managed_mutes(&self) -> bool {
        !self.muted_by_app.is_empty()
    }

    pub(super) fn mark_runtime_mute_state_dirty(&mut self) {
        self.runtime_mute_state_dirty = true;
    }

    pub(super) fn mark_runtime_mute_state_saved(&mut self) {
        self.runtime_mute_state_dirty = false;
    }

    pub(super) fn runtime_mute_state_dirty(&self) -> bool {
        self.runtime_mute_state_dirty
    }

    pub(super) fn apply_mute_plan(
        &mut self,
        matcher: &TargetMatcher,
        foreground_pid: Option<u32>,
        needs_foreground_process_name: bool,
        targets: &[TargetProcess],
    ) -> Option<Result<PlannedMuteApplyResult>> {
        let audio = self.audio.as_mut()?;
        let foreground_process_name = needs_foreground_process_name
            .then(|| {
                cached_foreground_process_name(
                    self.foreground_process_name_cache.as_ref(),
                    foreground_pid,
                )
            })
            .flatten();
        Some(audio.apply_mute_plan(
            matcher,
            foreground_pid,
            foreground_process_name,
            &mut self.muted_by_app,
            targets,
        ))
    }

    pub(super) fn update_foreground_process_name(&mut self, foreground_pid: Option<u32>) {
        let Some(pid) = foreground_pid else {
            self.foreground_process_name_cache = None;
            return;
        };
        if foreground_process_cache_needs_refresh(self.foreground_process_name_cache.as_ref(), pid)
        {
            self.foreground_process_name_cache = Some((pid, process::process_name(pid)));
        }
    }

    pub(super) fn clear_foreground_process_name(&mut self) {
        self.foreground_process_name_cache = None;
    }

    pub(super) fn muted_target_count(&self) -> usize {
        self.muted_target_count
    }

    pub(super) fn target_muted(&self, index: usize, target: &TargetProcess) -> bool {
        self.last_target_muted
            .get(index)
            .copied()
            .unwrap_or_else(|| {
                ManagedSessionLookup::new(&self.muted_by_app).target_has_managed_mute(target)
            })
    }

    pub(super) fn refresh_target_mute_snapshot(&mut self, targets: &[TargetProcess]) {
        self.last_target_muted.clear();
        self.last_target_muted.reserve(targets.len());
        let lookup = ManagedSessionLookup::new(&self.muted_by_app);
        self.muted_target_count = 0;
        for target in targets {
            let muted = lookup.target_has_managed_mute(target);
            self.muted_target_count += usize::from(muted);
            self.last_target_muted.push(muted);
        }
    }

    pub(super) fn sync_target_mute_indicators(&mut self, targets: &[TargetProcess]) -> bool {
        if targets.is_empty() {
            self.muted_target_count = 0;
            let changed = !self.last_target_muted.is_empty();
            self.last_target_muted.clear();
            return changed;
        }

        let mut changed = self.last_target_muted.len() != targets.len();
        self.last_target_muted.resize(targets.len(), false);
        let lookup = ManagedSessionLookup::new(&self.muted_by_app);
        self.muted_target_count = 0;
        for (muted, target) in self.last_target_muted.iter_mut().zip(targets) {
            let next = lookup.target_has_managed_mute(target);
            self.muted_target_count += usize::from(next);
            if *muted != next {
                *muted = next;
                changed = true;
            }
        }
        changed
    }

    pub(super) fn schedule_managed_mute_foreground_retry(&mut self, has_managed_mutes: bool) {
        self.managed_mute_foreground_retry_pending = has_managed_mutes;
    }

    pub(super) fn consume_managed_mute_foreground_retry(&mut self) {
        self.managed_mute_foreground_retry_pending = false;
    }

    pub(super) fn cancel_managed_mute_foreground_retry_if_idle(&mut self, has_managed_mutes: bool) {
        if !has_managed_mutes {
            self.managed_mute_foreground_retry_pending = false;
        }
    }

    pub(super) fn managed_mute_foreground_retry_pending(&self) -> bool {
        self.managed_mute_foreground_retry_pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_adopts_transferred_state_only_while_inactive() {
        let mut runtime = RuntimeCoordinator::new(false, false);

        assert!(!runtime.is_active());
        let managed_session =
            AudioSessionKey::new(42, "game.exe", Some("session-a".to_owned())).unwrap();
        assert!(runtime.activate(true, HashSet::from([managed_session.clone()])));
        assert!(runtime.is_active());
        assert!(runtime.is_paused());
        assert!(runtime.managed_sessions().contains(&managed_session));
        assert!(!runtime.activate(false, HashSet::new()));
        assert!(runtime.is_paused());
    }

    #[test]
    fn suspension_can_be_rolled_back_without_changing_runtime_state() {
        let mut runtime = RuntimeCoordinator::new(true, false);
        let managed_session =
            AudioSessionKey::new(42, "game.exe", Some("session-a".to_owned())).unwrap();
        runtime
            .managed_sessions_mut()
            .insert(managed_session.clone());
        runtime.toggle_paused();
        runtime.schedule_managed_mute_foreground_retry(true);

        assert!(runtime.suspend());
        assert!(!runtime.is_active());
        assert!(!runtime.suspend());
        assert!(runtime.resume());
        assert!(runtime.is_active());
        assert!(runtime.is_paused());
        assert!(runtime.managed_sessions().contains(&managed_session));
        assert!(runtime.managed_mute_foreground_retry_pending());
        assert!(!runtime.resume());
    }

    #[test]
    fn deactivation_stops_runtime_work_and_pending_retry() {
        let mut runtime = RuntimeCoordinator::new(true, false);
        runtime.schedule_managed_mute_foreground_retry(true);

        runtime.deactivate();

        assert!(!runtime.is_active());
        assert!(!runtime.managed_mute_foreground_retry_pending());
    }

    #[test]
    fn target_indicators_include_exact_session_ownership() {
        let mut runtime = RuntimeCoordinator::new(true, false);
        let target = TargetProcess::new("game.exe").unwrap();
        runtime
            .managed_sessions_mut()
            .insert(AudioSessionKey::new(42, "game.exe", Some("session-a".to_owned())).unwrap());

        runtime.refresh_target_mute_snapshot(std::slice::from_ref(&target));

        assert!(runtime.has_managed_mutes());
        assert_eq!(runtime.muted_target_count(), 1);
        assert!(runtime.target_muted(0, &target));
    }

    #[test]
    fn target_indicators_include_persisted_recovery_state() {
        let mut runtime = RuntimeCoordinator::new(true, true);
        let mut target = TargetProcess::new("game.exe").unwrap();
        target.managed_muted = true;

        runtime.refresh_target_mute_snapshot(std::slice::from_ref(&target));

        assert_eq!(runtime.muted_target_count(), 1);
        assert!(runtime.target_muted(0, &target));
    }

    #[test]
    fn foreground_retry_is_scheduled_only_while_ownership_remains() {
        let mut runtime = RuntimeCoordinator::new(true, false);
        runtime.schedule_managed_mute_foreground_retry(false);
        assert!(!runtime.managed_mute_foreground_retry_pending());

        runtime
            .managed_sessions_mut()
            .insert(AudioSessionKey::new(42, "game.exe", Some("session-a".to_owned())).unwrap());
        runtime.schedule_managed_mute_foreground_retry(true);
        assert!(runtime.managed_mute_foreground_retry_pending());

        runtime.consume_managed_mute_foreground_retry();
        assert!(!runtime.managed_mute_foreground_retry_pending());

        runtime.schedule_managed_mute_foreground_retry(true);

        runtime.managed_sessions_mut().clear();
        runtime.cancel_managed_mute_foreground_retry_if_idle(false);

        assert!(!runtime.managed_mute_foreground_retry_pending());
    }

    #[test]
    fn persisted_recovery_gets_one_delayed_startup_retry() {
        let runtime = RuntimeCoordinator::new(true, true);

        assert!(runtime.managed_mute_foreground_retry_pending());
    }

    #[test]
    fn runtime_mute_state_stays_dirty_until_a_save_succeeds() {
        let mut runtime = RuntimeCoordinator::new(true, false);

        assert!(!runtime.runtime_mute_state_dirty());
        runtime.mark_runtime_mute_state_dirty();
        assert!(runtime.runtime_mute_state_dirty());

        // A failed attempt deliberately has no state transition.
        assert!(runtime.runtime_mute_state_dirty());

        runtime.mark_runtime_mute_state_saved();
        assert!(!runtime.runtime_mute_state_dirty());
    }
}
