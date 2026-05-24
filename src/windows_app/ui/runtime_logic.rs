use super::constants::PROCESS_REFRESH_STALE_INTERVAL;
use crate::config::TargetProcess;
use std::time::Instant;

pub(super) const MANAGED_MUTE_FOREGROUND_RETRY_INTERVAL_MS: u32 = 100;
pub(super) const MANAGED_MUTE_FOREGROUND_RETRY_TICKS: u8 = 20;

pub(super) fn cached_foreground_process_name(
    cache: Option<&(u32, Option<String>)>,
    foreground_pid: Option<u32>,
) -> Option<&str> {
    let pid = foreground_pid?;
    let (cached_pid, name) = cache?;
    (*cached_pid == pid).then_some(name.as_deref()).flatten()
}

pub(super) fn foreground_process_cache_needs_refresh(
    cache: Option<&(u32, Option<String>)>,
    pid: u32,
) -> bool {
    !matches!(cache, Some((cached_pid, Some(_))) if *cached_pid == pid)
}

pub(super) fn initial_process_refresh_attempt() -> Instant {
    Instant::now() - PROCESS_REFRESH_STALE_INTERVAL
}

pub(super) fn process_refresh_is_stale(last_process_refresh_attempt: Instant) -> bool {
    last_process_refresh_attempt.elapsed() >= PROCESS_REFRESH_STALE_INTERVAL
}

pub(super) fn initial_managed_mute_fast_retry_count(targets: &[TargetProcess]) -> u8 {
    if targets.iter().any(|target| target.managed_muted) {
        MANAGED_MUTE_FOREGROUND_RETRY_TICKS
    } else {
        0
    }
}

pub(super) fn should_start_managed_mute_fast_retry_after_audio_update(
    previous_has_managed_mutes: bool,
    next_has_managed_mutes: bool,
    previous_managed_session_count: usize,
    next_managed_session_count: usize,
) -> bool {
    next_has_managed_mutes
        && (!previous_has_managed_mutes
            || next_managed_session_count > previous_managed_session_count)
}

fn audio_fallback_timer_needed(
    paused: bool,
    target_matcher_empty: bool,
    has_managed_mutes: bool,
) -> bool {
    has_managed_mutes || (!paused && !target_matcher_empty)
}

pub(super) fn desired_audio_fallback_timer_interval_ms(
    paused: bool,
    target_matcher_empty: bool,
    has_managed_mutes: bool,
    managed_mute_fast_retry_remaining: u8,
    polling_interval_ms: u64,
) -> Option<u32> {
    if !audio_fallback_timer_needed(paused, target_matcher_empty, has_managed_mutes) {
        return None;
    }
    if has_managed_mutes && managed_mute_fast_retry_remaining > 0 {
        return Some(MANAGED_MUTE_FOREGROUND_RETRY_INTERVAL_MS);
    }
    Some(polling_interval_ms as u32)
}

pub(super) fn audio_fallback_timer_matches_desired(
    desired_interval: Option<u32>,
    active_interval: Option<u32>,
) -> bool {
    desired_interval == active_interval
}

pub(super) fn replace_text_if_changed(current: &mut String, next: &mut String) -> bool {
    if current == next {
        next.clear();
        false
    } else {
        std::mem::swap(current, next);
        next.clear();
        true
    }
}

pub(super) fn should_hide_to_tray(tray_added: bool) -> bool {
    tray_added
}

pub(super) fn should_retry_tray_icon_before_hide(tray_added: bool) -> bool {
    !tray_added
}

pub(super) fn should_release_idle_audio_while_paused(
    paused: bool,
    has_managed_mutes: bool,
) -> bool {
    paused && !has_managed_mutes
}

pub(super) fn should_reset_audio_controller_after_update(had_failures: bool) -> bool {
    had_failures
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_text_is_cleared_from_scratch_buffer() {
        let mut current = String::from("game.exe");
        let mut next = String::from("game.exe");

        assert!(!replace_text_if_changed(&mut current, &mut next));
        assert_eq!(current, "game.exe");
        assert!(next.is_empty());
    }

    #[test]
    fn changed_text_replaces_current_and_clears_scratch_buffer() {
        let mut current = String::from("game.exe");
        let mut next = String::from("chat.exe");

        assert!(replace_text_if_changed(&mut current, &mut next));
        assert_eq!(current, "chat.exe");
        assert!(next.is_empty());
    }

    #[test]
    fn foreground_process_cache_reuses_successful_lookup() {
        let cache = Some((42, Some("game.exe".to_owned())));

        assert!(!foreground_process_cache_needs_refresh(cache.as_ref(), 42));
    }

    #[test]
    fn foreground_process_cache_retries_failed_lookup() {
        let cache = Some((42, None));

        assert!(foreground_process_cache_needs_refresh(cache.as_ref(), 42));
    }

    #[test]
    fn initial_process_refresh_is_due_immediately() {
        assert!(process_refresh_is_stale(initial_process_refresh_attempt()));
    }

    #[test]
    fn process_refresh_is_needed_after_stale_interval() {
        assert!(process_refresh_is_stale(
            Instant::now() - PROCESS_REFRESH_STALE_INTERVAL
        ));
    }

    #[test]
    fn process_refresh_is_throttled_after_recent_attempt() {
        assert!(!process_refresh_is_stale(Instant::now()));
    }

    #[test]
    fn managed_mute_fast_retry_starts_when_persisted_target_exists() {
        let mut target = TargetProcess::new("game.exe").unwrap();
        target.managed_muted = true;

        assert_eq!(
            initial_managed_mute_fast_retry_count(&[target]),
            MANAGED_MUTE_FOREGROUND_RETRY_TICKS
        );
    }

    #[test]
    fn managed_mute_fast_retry_restarts_for_new_managed_audio_session() {
        assert!(should_start_managed_mute_fast_retry_after_audio_update(
            true, true, 0, 1
        ));
        assert!(should_start_managed_mute_fast_retry_after_audio_update(
            false, true, 0, 0
        ));
        assert!(!should_start_managed_mute_fast_retry_after_audio_update(
            true, true, 1, 1
        ));
        assert!(!should_start_managed_mute_fast_retry_after_audio_update(
            true, false, 1, 0
        ));
    }

    #[test]
    fn managed_mute_fast_retry_uses_short_audio_interval_only_temporarily() {
        assert_eq!(
            desired_audio_fallback_timer_interval_ms(true, true, true, 1, 3_000),
            Some(MANAGED_MUTE_FOREGROUND_RETRY_INTERVAL_MS)
        );
        assert_eq!(
            desired_audio_fallback_timer_interval_ms(true, true, true, 0, 3_000),
            Some(3_000)
        );
        assert_eq!(
            desired_audio_fallback_timer_interval_ms(false, false, false, 1, 3_000),
            Some(3_000)
        );
        assert_eq!(
            desired_audio_fallback_timer_interval_ms(true, true, false, 1, 3_000),
            None
        );
    }

    #[test]
    fn audio_fallback_timer_is_ready_only_at_desired_interval() {
        assert!(audio_fallback_timer_matches_desired(
            Some(3_000),
            Some(3_000)
        ));
        assert!(audio_fallback_timer_matches_desired(None, None));
        assert!(!audio_fallback_timer_matches_desired(
            Some(3_000),
            Some(MANAGED_MUTE_FOREGROUND_RETRY_INTERVAL_MS)
        ));
        assert!(!audio_fallback_timer_matches_desired(Some(3_000), None));
        assert!(!audio_fallback_timer_matches_desired(None, Some(3_000)));
    }

    #[test]
    fn window_hides_only_when_tray_icon_is_available() {
        assert!(should_hide_to_tray(true));
        assert!(!should_hide_to_tray(false));
    }

    #[test]
    fn missing_tray_icon_is_retried_before_hiding() {
        assert!(should_retry_tray_icon_before_hide(false));
        assert!(!should_retry_tray_icon_before_hide(true));
    }

    #[test]
    fn paused_idle_audio_is_released_only_after_mutes_are_restored() {
        assert!(should_release_idle_audio_while_paused(true, false));
        assert!(!should_release_idle_audio_while_paused(true, true));
        assert!(!should_release_idle_audio_while_paused(false, false));
    }

    #[test]
    fn audio_controller_is_reset_after_failed_update() {
        assert!(should_reset_audio_controller_after_update(true));
        assert!(!should_reset_audio_controller_after_update(false));
    }
}
