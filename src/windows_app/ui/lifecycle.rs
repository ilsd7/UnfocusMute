use super::*;

impl AppWindow {
    pub(super) unsafe fn on_create(&mut self, hwnd: HWND) -> Result<()> {
        self.hwnd = hwnd;
        apply_window_theme(hwnd);
        self.icons.apply_to(hwnd);

        unsafe {
            self.create_controls()?;
        }
        self.apply_process_filter();
        self.refresh_text();
        if self.runtime.is_active() {
            self.reset_timers();
            self.tick();
        }
        Ok(())
    }

    pub(super) fn activate_replacement_runtime(&mut self, snapshot: Option<HandoffSnapshot>) {
        // Legacy predecessors cannot transfer their in-memory pause state.
        let (paused, managed_sessions, ownership_incomplete, target_mute_states) = snapshot
            .map(|snapshot| {
                (
                    snapshot.paused,
                    snapshot.managed_sessions,
                    snapshot.ownership_incomplete,
                    snapshot.target_mute_states,
                )
            })
            .unwrap_or_default();
        if !self
            .runtime
            .activate(paused, managed_sessions, ownership_incomplete)
        {
            return;
        }
        self.last_status = None;
        let reload = self.config_store.reload_now();
        self.apply_config_reload_without_startup_sync(reload);
        // A failed runtime-state save can leave the old process newer than disk. Apply only its
        // internal mute flags, and only to targets that still exist after the reload.
        let mute_state_changed = target_mute_states.is_some_and(|states| {
            handoff::apply_target_mute_states(&mut self.config.targets, &states)
        });
        if mute_state_changed {
            self.runtime.mark_runtime_mute_state_dirty();
        }
        let startup_sync = sync_startup_setting(&mut self.config);
        self.issues.merge(startup_sync.issues);
        if let Some(detail) = startup_sync.issue_detail {
            self.issue_diagnostics
                .set(StatusIssue::StartupUpdateFailed, detail);
        }
        if startup_sync.config_changed {
            self.save_config();
        } else if mute_state_changed {
            self.save_runtime_mute_state();
        }
        self.reset_timers();
        self.tick();
    }

    pub(super) fn accept_handoff_snapshot(&mut self, source: HWND, bytes: &[u8]) -> bool {
        if self.runtime.is_active() {
            return false;
        }
        let Some(snapshot) = HandoffSnapshot::decode(bytes) else {
            return false;
        };
        self.handoff.accept(source, snapshot)
    }

    pub(super) fn take_handoff_snapshot(&mut self) -> Option<HandoffSnapshot> {
        self.handoff.take_received()
    }

    pub(super) fn prepare_handoff_snapshot(&mut self) -> Option<Vec<u8>> {
        if !self.handoff.prepare() {
            return None;
        }
        if !self.runtime.suspend() {
            let _ = self.handoff.leave_prepared();
            return None;
        }
        self.stop_runtime_work();

        let dirty_targets = self
            .runtime
            .runtime_mute_state_dirty()
            .then_some(self.config.targets.as_slice());
        let snapshot = HandoffSnapshot::capture(
            self.runtime.is_paused(),
            self.runtime.managed_sessions(),
            dirty_targets,
        );
        match snapshot.encode() {
            Some(bytes) => Some(bytes),
            None => {
                self.abort_handoff();
                None
            }
        }
    }

    pub(super) fn abort_handoff(&mut self) {
        if !self.handoff.leave_prepared() {
            return;
        }
        if self.runtime.resume() {
            self.reset_timers();
            self.tick();
        }
    }

    pub(super) fn prepare_for_destroy(&mut self) -> Option<MessageDialogRequest> {
        self.prepare_for_shutdown();
        let restore_issue = self.restore_managed_mutes();
        let save_issue = (self.runtime.runtime_mute_state_dirty()
            && !self.save_runtime_mute_state())
        .then_some(StatusIssue::ConfigSaveFailed);
        let exit_issue = restore_issue.or(save_issue);
        let warning = exit_issue.map(|issue| {
            let detail = self.issue_diagnostics.detail(issue);
            let mut request = self.issue_message_request(issue, detail);
            if matches!(
                issue,
                StatusIssue::AudioUnavailable | StatusIssue::AudioUpdateFailed
            ) {
                request.explanation = Some(self.strings.exit_restore_failed_explanation.to_owned());
                request.diagnostic_code = Some("exit-restore-failed");
            }
            request
        });
        self.remove_tray_icon();
        warning
    }

    pub(super) fn finish_handoff(&mut self) -> bool {
        if !self.handoff.leave_prepared() {
            return false;
        }
        self.remove_tray_icon();
        true
    }

    fn prepare_for_shutdown(&mut self) {
        self.runtime.deactivate();
        self.stop_runtime_work();
        self.save_window_placement();
    }

    fn stop_runtime_work(&mut self) {
        self.runtime.foreground_hook = None;
        self.clear_timer(CONFIG_RELOAD_TIMER_ID);
        self.runtime.config_reload_timer_ready = false;
        self.clear_audio_fallback_timer();
    }
}
