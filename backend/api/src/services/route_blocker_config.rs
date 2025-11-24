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
