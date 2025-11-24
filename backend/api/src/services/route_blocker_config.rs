use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

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
