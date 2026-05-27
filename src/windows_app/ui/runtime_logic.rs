use super::constants::PROCESS_REFRESH_STALE_INTERVAL;
use crate::config::TargetProcess;
use std::time::Instant;

pub(super) const MANAGED_MUTE_FOREGROUND_RETRY_INTERVAL_MS: u32 = 300;
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

pub(super) fn desired_audio_fallback_timer_interval_ms(
    paused: bool,
    target_matcher_empty: bool,
    has_managed_mutes: bool,
    managed_mute_fast_retry_remaining: u8,
    polling_interval_ms: u64,
) -> Option<u32> {
    if !has_managed_mutes && (paused || target_matcher_empty) {
        return None;
    }
    if has_managed_mutes && managed_mute_fast_retry_remaining > 0 {
        return Some(MANAGED_MUTE_FOREGROUND_RETRY_INTERVAL_MS);
    }
    Some(polling_interval_ms as u32)
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
}
