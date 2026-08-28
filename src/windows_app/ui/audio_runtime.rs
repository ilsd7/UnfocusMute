use super::*;

impl AppWindow {
    pub(super) fn refresh_audio_session_processes(&mut self) -> ProcessRefreshResult {
        self.reset_audio_if_endpoint_changed();
        if !self.ensure_audio_controller(false) {
            self.set_issue(StatusIssue::AudioUnavailable);
            return ProcessRefreshResult::Failed;
        }

        let Some(audio) = &self.runtime.audio else {
            self.set_issue(StatusIssue::AudioUnavailable);
            return ProcessRefreshResult::Failed;
        };
        let refresh = audio.refresh_session_processes(
            &mut self.running_processes,
            &mut self.running_process_refresh_buffer,
        );
        match refresh.outcome {
            ProcessRefreshOutcome::Changed => {
                self.clear_issue(StatusIssue::AudioUnavailable);
                self.rebuild_process_choices();
                self.apply_process_filter();
                ProcessRefreshResult::Refreshed
            }
            ProcessRefreshOutcome::Unchanged => {
                self.clear_issue(StatusIssue::AudioUnavailable);
                ProcessRefreshResult::Unchanged
            }
            ProcessRefreshOutcome::Failed => {
                self.runtime.audio = None;
                self.set_issue_with_detail(
                    StatusIssue::AudioUnavailable,
                    refresh
                        .failure_detail
                        .unwrap_or_else(|| "refresh audio session process list failed".to_owned()),
                );
                ProcessRefreshResult::Failed
            }
        }
    }

    pub(super) fn timer_tick(&mut self, timer_id: usize) {
        if !self.runtime.is_active() {
            return;
        }
        self.retry_missing_timers();
        match timer_id {
            CONFIG_RELOAD_TIMER_ID => {
                let target_matcher_changed = self.reload_config_if_due().target_matcher_changed;
                self.retry_runtime_mute_state_save();
                if target_matcher_changed {
                    self.tick();
                }
            }
            AUDIO_FALLBACK_TIMER_ID => {
                self.clear_foreground_process_cache();
                self.runtime.consume_managed_mute_foreground_retry();
                self.tick();
            }
            _ => {}
        }
    }

    pub(super) fn foreground_changed(&mut self) {
        if !self.runtime.is_active() {
            return;
        }
        self.clear_foreground_process_cache();
        self.tick();
        let has_managed_mutes = self.has_managed_mutes();
        self.runtime
            .schedule_managed_mute_foreground_retry(has_managed_mutes);
        self.reset_polling_timer();
    }

    pub(super) fn tick(&mut self) {
        self.reload_config_if_due();

        if self.runtime.is_paused() {
            self.runtime.foreground_hook = None;
            self.clear_issue(StatusIssue::ForegroundHookUnavailable);
            let _ = self.restore_managed_mutes();
            self.release_idle_audio_while_paused();
            self.sync_audio_fallback_timer();
            self.update_status();
            return;
        }

        if self.target_matcher.is_empty() && !self.has_managed_mutes() {
            self.runtime.foreground_hook = None;
            self.clear_issue(StatusIssue::ForegroundHookUnavailable);
            self.runtime.audio = None;
            self.clear_audio_issues();
            self.sync_audio_fallback_timer();
            self.update_status();
            return;
        }

        self.sync_audio_fallback_timer();
        self.ensure_foreground_hook();
        self.reset_audio_if_endpoint_changed();

        if !self.ensure_audio_controller(true) {
            return;
        }

        let foreground_pid = process::foreground_pid();
        let needs_foreground_process_name = self.target_matcher.needs_foreground_process_name();
        if needs_foreground_process_name {
            self.runtime.update_foreground_process_name(foreground_pid);
        }

        let Some(apply_result) = self.runtime.apply_mute_plan(
            &self.target_matcher,
            foreground_pid,
            needs_foreground_process_name,
            &self.config.targets,
        ) else {
            return;
        };
        let apply_result = match apply_result {
            Ok(result) => {
                self.clear_issue(StatusIssue::AudioUnavailable);
                result
            }
            Err(error) => {
                self.runtime.audio = None;
                self.set_issue_with_detail(StatusIssue::AudioUnavailable, error.to_string());
                return;
            }
        };

        if self.apply_target_mute_updates(&apply_result.target_updates) {
            self.save_runtime_mute_state();
        }
        let has_managed_mutes = self.has_managed_mutes();
        self.runtime
            .cancel_managed_mute_foreground_retry_if_idle(has_managed_mutes);
        self.apply_audio_update_result(apply_result.had_failures, apply_result.failure_detail);

        self.sync_target_mute_indicators();
        self.sync_audio_fallback_timer();
        self.update_status();
    }

    fn reset_audio_after_endpoint_change(&mut self) {
        if let Some(mut audio) = self.runtime.audio.take() {
            let result = restore_mute_set(&mut audio, self.runtime.managed_sessions_mut());
            self.apply_audio_update_result(result.had_failures, result.failure_detail);
        }
        self.sync_target_mute_indicators();
    }

    fn reset_audio_if_endpoint_changed(&mut self) {
        if self
            .runtime
            .audio
            .as_ref()
            .is_some_and(|audio| audio.take_endpoint_changed())
        {
            self.reset_audio_after_endpoint_change();
        }
    }

    fn clear_foreground_process_cache(&mut self) {
        self.runtime.clear_foreground_process_name();
    }

    pub(super) fn restore_managed_mutes(&mut self) -> Option<StatusIssue> {
        if !self.has_managed_mutes() {
            return None;
        }

        if !self.ensure_audio_controller(true) {
            return Some(StatusIssue::AudioUnavailable);
        }

        let Some(mut audio) = self.runtime.audio.take() else {
            self.set_issue_with_detail(
                StatusIssue::AudioUnavailable,
                "audio controller unavailable while unmuting managed sessions",
            );
            return Some(StatusIssue::AudioUnavailable);
        };
        let restore_matcher = TargetMatcher::default();
        let result = audio.apply_mute_plan(
            &restore_matcher,
            None,
            None,
            self.runtime.managed_sessions_mut(),
            &self.config.targets,
        );
        self.runtime.audio = Some(audio);
        let restore_issue = match result {
            Ok(result) => {
                if self.apply_target_mute_updates(&result.target_updates) {
                    self.save_runtime_mute_state();
                }
                let has_managed_mutes = self.has_managed_mutes();
                self.runtime
                    .cancel_managed_mute_foreground_retry_if_idle(has_managed_mutes);
                let had_failures = result.had_failures;
                self.apply_audio_update_result(had_failures, result.failure_detail);
                had_failures.then_some(StatusIssue::AudioUpdateFailed)
            }
            Err(error) => {
                self.runtime.audio = None;
                self.set_issue_with_detail(StatusIssue::AudioUnavailable, error.to_string());
                Some(StatusIssue::AudioUnavailable)
            }
        };
        self.sync_target_mute_indicators();
        restore_issue
    }

    pub(super) fn restore_target_mute_before_removal(&mut self, target: &TargetProcess) -> bool {
        let target_sessions = session_keys_exclusive_to_target(
            target,
            &self.config.targets,
            self.runtime.managed_sessions(),
        );
        if target_sessions.is_empty() && !target.managed_muted {
            return true;
        }
        let mut managed_sessions = self.runtime.managed_sessions().clone();

        let Some(transition_targets) = removal_restore_targets(target, &self.config.targets) else {
            return false;
        };
        let transition_matcher = TargetMatcher::new(&transition_targets);
        let foreground_pid = process::foreground_pid();
        let foreground_process_name = foreground_pid
            .filter(|_| transition_matcher.needs_foreground_process_name())
            .and_then(process::process_name);

        if !self.ensure_audio_controller(true) {
            return false;
        }

        let Some(mut audio) = self.runtime.audio.take() else {
            return false;
        };
        let result = audio.apply_mute_plan(
            &transition_matcher,
            foreground_pid,
            foreground_process_name.as_deref(),
            &mut managed_sessions,
            &transition_targets,
        );
        self.runtime.audio = Some(audio);
        match result {
            Ok(result) => {
                let target_restore_failed = result.target_unmute_is_uncertain(target)
                    || !session_keys_exclusive_to_target(
                        target,
                        &self.config.targets,
                        &managed_sessions,
                    )
                    .is_empty();
                *self.runtime.managed_sessions_mut() = managed_sessions;
                if self.apply_target_mute_updates(&result.target_updates) {
                    self.save_runtime_mute_state();
                }
                let had_failures = result.had_failures || target_restore_failed;
                let failure_detail = result.failure_detail.or_else(|| {
                    target_restore_failed.then(|| {
                        let mut identity = String::new();
                        target.display_identity_into(&mut identity);
                        format!("mute state could not be cleared before removing {identity}")
                    })
                });
                self.apply_audio_update_result(had_failures, failure_detail);
                !target_restore_failed
            }
            Err(error) => {
                self.runtime.audio = None;
                self.set_issue_with_detail(StatusIssue::AudioUnavailable, error.to_string());
                false
            }
        }
    }

    fn release_idle_audio_while_paused(&mut self) {
        if self.runtime.is_paused() && !self.has_managed_mutes() {
            self.runtime.audio = None;
            self.clear_audio_issues();
        }
    }

    fn ensure_audio_controller(&mut self, report_issue: bool) -> bool {
        if self.runtime.audio.is_some() {
            return true;
        }

        match AudioController::new() {
            Ok(audio) => {
                self.runtime.audio = Some(audio);
                if report_issue {
                    self.clear_issue(StatusIssue::AudioUnavailable);
                }
                true
            }
            Err(error) => {
                self.record_issue_detail(StatusIssue::AudioUnavailable, error.to_string());
                if report_issue {
                    self.set_issue(StatusIssue::AudioUnavailable);
                }
                false
            }
        }
    }

    fn apply_audio_update_result(&mut self, had_failures: bool, failure_detail: Option<String>) {
        if had_failures {
            if let Some(detail) = failure_detail {
                self.set_issue_with_detail(StatusIssue::AudioUpdateFailed, detail);
            } else {
                self.set_issue(StatusIssue::AudioUpdateFailed);
            }
        } else {
            self.clear_issue(StatusIssue::AudioUpdateFailed);
        }
    }

    fn apply_target_mute_updates(&mut self, updates: &[TargetMuteStateUpdate]) -> bool {
        let mut changed = false;
        for update in updates {
            let Some(index) =
                target_index_by_identity(&self.config.targets, &update.process_name, update.pid)
            else {
                continue;
            };
            changed |= self.config.set_target_managed_muted_at(index, update.muted);
        }
        if changed {
            self.runtime.mark_runtime_mute_state_dirty();
        }
        changed
    }

    pub(super) fn has_managed_mutes(&self) -> bool {
        self.runtime.has_managed_mutes()
            || self
                .config
                .targets
                .iter()
                .any(|target| target.managed_muted)
    }

    fn clear_audio_issues(&mut self) {
        let mask = StatusIssue::AudioUnavailable.bit() | StatusIssue::AudioUpdateFailed.bit();
        if self.clear_issue_mask(mask) {
            self.last_status = None;
            self.update_status();
        }
    }

    fn install_foreground_hook(&mut self) {
        match unsafe { ForegroundEventHook::new(self.hwnd) } {
            Ok(hook) => {
                self.runtime.foreground_hook = Some(hook);
                self.clear_issue(StatusIssue::ForegroundHookUnavailable);
            }
            Err(error) => {
                self.runtime.foreground_hook = None;
                self.set_issue_with_detail(
                    StatusIssue::ForegroundHookUnavailable,
                    error.to_string(),
                );
            }
        }
    }

    fn ensure_foreground_hook(&mut self) {
        if self.runtime.foreground_hook.is_none() {
            self.install_foreground_hook();
        }
    }

    pub(super) fn reset_timers(&mut self) {
        let ready = self.set_timer(CONFIG_RELOAD_TIMER_ID, CONFIG_RELOAD_TIMER_INTERVAL_MS);
        self.runtime.config_reload_timer_ready = ready;
        self.reset_polling_timer();
    }

    pub(super) fn reset_polling_timer(&mut self) {
        let desired_interval = self.desired_audio_fallback_timer_interval_ms();
        self.apply_audio_fallback_timer_interval(desired_interval);
        self.update_timer_setup_issue(desired_interval);
    }

    fn retry_missing_timers(&mut self) {
        let mut retried = false;
        let desired_audio_interval = self.desired_audio_fallback_timer_interval_ms();
        if !self.runtime.config_reload_timer_ready {
            let ready = self.set_timer(CONFIG_RELOAD_TIMER_ID, CONFIG_RELOAD_TIMER_INTERVAL_MS);
            self.runtime.config_reload_timer_ready = ready;
            retried = true;
        }
        if desired_audio_interval != self.runtime.audio_fallback_timer_interval_ms {
            self.apply_audio_fallback_timer_interval(desired_audio_interval);
            retried = true;
        }
        if retried {
            self.update_timer_setup_issue(desired_audio_interval);
        }
    }

    pub(super) fn sync_audio_fallback_timer(&mut self) {
        let desired_interval = self.desired_audio_fallback_timer_interval_ms();
        if desired_interval != self.runtime.audio_fallback_timer_interval_ms {
            self.apply_audio_fallback_timer_interval(desired_interval);
        }
        self.update_timer_setup_issue(desired_interval);
    }

    fn desired_audio_fallback_timer_interval_ms(&self) -> Option<u32> {
        desired_audio_fallback_timer_interval_ms(
            self.runtime.is_paused(),
            self.target_matcher.is_empty(),
            self.has_managed_mutes(),
            self.runtime.managed_mute_foreground_retry_pending(),
            self.config.polling_interval_ms,
        )
    }

    fn apply_audio_fallback_timer_interval(&mut self, interval_ms: Option<u32>) {
        match interval_ms {
            Some(interval_ms) => {
                if self.set_timer(AUDIO_FALLBACK_TIMER_ID, interval_ms) {
                    self.runtime.audio_fallback_timer_interval_ms = Some(interval_ms);
                }
            }
            None => self.clear_audio_fallback_timer(),
        }
    }

    pub(super) fn clear_audio_fallback_timer(&mut self) {
        if self.runtime.audio_fallback_timer_interval_ms.is_some() {
            self.clear_timer(AUDIO_FALLBACK_TIMER_ID);
        }
        self.runtime.audio_fallback_timer_interval_ms = None;
    }

    fn set_timer(&mut self, timer_id: usize, interval_ms: u32) -> bool {
        if (unsafe { SetTimer(Some(self.hwnd), timer_id, interval_ms, None) }) != 0 {
            return true;
        }

        self.record_issue_detail(
            StatusIssue::TimerSetupFailed,
            last_win32_error_detail("set timer"),
        );
        false
    }

    pub(super) fn clear_timer(&self, timer_id: usize) {
        unsafe {
            let _ = KillTimer(Some(self.hwnd), timer_id);
        }
    }

    fn update_timer_setup_issue(&mut self, desired_audio_interval: Option<u32>) {
        let audio_fallback_timer_ready =
            desired_audio_interval == self.runtime.audio_fallback_timer_interval_ms;
        if self.runtime.config_reload_timer_ready && audio_fallback_timer_ready {
            self.clear_issue(StatusIssue::TimerSetupFailed);
        } else {
            self.set_issue(StatusIssue::TimerSetupFailed);
        }
    }

    pub(super) fn toggle_pause(&mut self) {
        if self.runtime.toggle_paused() {
            let _ = self.restore_managed_mutes();
            self.runtime.foreground_hook = None;
            self.release_idle_audio_while_paused();
        } else {
            self.tick();
        }
        self.sync_audio_fallback_timer();
        self.update_status();
    }
}
