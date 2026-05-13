use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use tokio::sync::Notify;

const DEFAULT_SYNC_INTERVAL_SECS: u64 = 60 * 30; // 30 minutes
const MIN_SYNC_INTERVAL_SECS: u64 = 60; // 1 minute
const MAX_SYNC_INTERVAL_SECS: u64 = 60 * 60 * 24; // 24 hours

lazy_static! {
    static ref SYNC_INTERVAL_SECS: AtomicU64 = AtomicU64::new(DEFAULT_SYNC_INTERVAL_SECS);
    static ref SYNC_PAUSED: AtomicBool = AtomicBool::new(false);
    static ref FORCE_SYNC: AtomicBool = AtomicBool::new(false);
    static ref SYNC_NOTIFY: Notify = Notify::new();
    static ref LAST_SYNC_AT: RwLock<Option<DateTime<Utc>>> = RwLock::new(None);
    static ref NEXT_SYNC_AT: RwLock<Option<DateTime<Utc>>> = RwLock::new(None);
    static ref SYNC_RUNNING: AtomicBool = AtomicBool::new(false);
}

pub fn get_sync_interval_secs() -> u64 {
    SYNC_INTERVAL_SECS.load(Ordering::Relaxed)
}

pub fn set_sync_interval_secs(secs: u64) {
    let clamped = secs.clamp(MIN_SYNC_INTERVAL_SECS, MAX_SYNC_INTERVAL_SECS);
    SYNC_INTERVAL_SECS.store(clamped, Ordering::Relaxed);
    SYNC_NOTIFY.notify_waiters();
}

pub fn pause_sync() {
    SYNC_PAUSED.store(true, Ordering::Relaxed);
    SYNC_NOTIFY.notify_waiters();
}

pub fn resume_sync() {
    let was_paused = SYNC_PAUSED.swap(false, Ordering::Relaxed);
    if was_paused {
        FORCE_SYNC.store(true, Ordering::Relaxed);
    }
    SYNC_NOTIFY.notify_waiters();
}

pub fn is_paused() -> bool {
    SYNC_PAUSED.load(Ordering::Relaxed)
}

pub fn request_immediate_sync() {
    FORCE_SYNC.store(true, Ordering::Relaxed);
    SYNC_NOTIFY.notify_waiters();
}

pub fn take_force_sync_flag() -> bool {
    FORCE_SYNC.swap(false, Ordering::Relaxed)
}

pub fn notifier() -> &'static Notify {
    &SYNC_NOTIFY
}

pub fn set_last_sync_at(timestamp: DateTime<Utc>) {
    if let Ok(mut last) = LAST_SYNC_AT.write() {
        *last = Some(timestamp);
    }
}

pub fn get_last_sync_at() -> Option<DateTime<Utc>> {
    LAST_SYNC_AT.read().ok().and_then(|guard| *guard)
}

pub fn set_next_sync_at(timestamp: DateTime<Utc>) {
    if let Ok(mut next) = NEXT_SYNC_AT.write() {
        *next = Some(timestamp);
    }
}

pub fn get_next_sync_at() -> Option<DateTime<Utc>> {
    NEXT_SYNC_AT.read().ok().and_then(|guard| *guard)
}

pub fn set_sync_running(running: bool) {
    SYNC_RUNNING.store(running, Ordering::Relaxed);
}

pub fn is_sync_running() -> bool {
    SYNC_RUNNING.load(Ordering::Relaxed)
}

pub fn calculate_next_sync() -> DateTime<Utc> {
    let interval = get_sync_interval_secs();
    Utc::now() + chrono::Duration::seconds(interval as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_default_sync_interval() {
        // Reset to known state
        set_sync_interval_secs(30 * 60);
        assert_eq!(get_sync_interval_secs(), 30 * 60);
    }

    #[test]
    fn test_set_sync_interval_clamps_minimum() {
        set_sync_interval_secs(10);
        assert_eq!(get_sync_interval_secs(), 60);
    }

    #[test]
    fn test_set_sync_interval_clamps_maximum() {
        set_sync_interval_secs(200_000);
        assert_eq!(get_sync_interval_secs(), 86_400);
    }

    #[test]
    fn test_set_sync_interval_exact_bounds() {
        set_sync_interval_secs(60);
        assert_eq!(get_sync_interval_secs(), 60);

        set_sync_interval_secs(86_400);
        assert_eq!(get_sync_interval_secs(), 86_400);
    }

    #[test]
    fn test_pause_and_resume_sync() {
        // Ensure we start unpaused
        resume_sync();

        assert!(!is_paused());
        pause_sync();
        assert!(is_paused());
        pause_sync();
        assert!(is_paused());
        resume_sync();
        assert!(!is_paused());
    }

    #[test]
    fn test_resume_sync_sets_force_flag() {
        // Clear force flag first, then ensure paused state
        take_force_sync_flag();
        pause_sync();
        assert!(is_paused());
        resume_sync();
        assert!(!is_paused());
        // After resume from paused state, force flag should be set
        assert!(take_force_sync_flag());
        // Taking it again should return false (atomic swap)
        assert!(!take_force_sync_flag());
    }

    #[test]
    fn test_request_immediate_sync_and_take() {
        take_force_sync_flag();
        assert!(!take_force_sync_flag());

        request_immediate_sync();
        assert!(take_force_sync_flag());
        assert!(!take_force_sync_flag());
    }

    #[test]
    fn test_force_sync_is_atomic_swap() {
        request_immediate_sync();
        request_immediate_sync();
        // Even if called twice, one take clears it
        assert!(take_force_sync_flag());
        assert!(!take_force_sync_flag());
    }

    #[test]
    fn test_set_and_get_last_sync_at() {
        let ts = Utc::now();
        set_last_sync_at(ts);
        let retrieved = get_last_sync_at().expect("should have timestamp");
        assert_eq!(retrieved, ts);
    }

    #[test]
    fn test_last_sync_at_overwrites() {
        let ts1 = Utc::now() - Duration::days(1);
        let ts2 = Utc::now();
        set_last_sync_at(ts1);
        assert_eq!(get_last_sync_at().unwrap(), ts1);
        set_last_sync_at(ts2);
        assert_eq!(get_last_sync_at().unwrap(), ts2);
    }

    #[test]
    fn test_set_and_get_next_sync_at() {
        let ts = Utc::now() + Duration::hours(1);
        set_next_sync_at(ts);
        let retrieved = get_next_sync_at().expect("should have timestamp");
        assert_eq!(retrieved, ts);
    }

    #[test]
    fn test_sync_running_flag() {
        set_sync_running(false);
        assert!(!is_sync_running());

        set_sync_running(true);
        assert!(is_sync_running());

        set_sync_running(false);
        assert!(!is_sync_running());
    }

    // Restore defaults after tests
    #[test]
    fn test_restore_defaults() {
        set_sync_interval_secs(30 * 60);
        resume_sync();
        take_force_sync_flag();
        set_sync_running(false);
    }
}
