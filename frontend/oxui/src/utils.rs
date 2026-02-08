use std::time::Duration;

/// Cross-platform async sleep that works on both server (tokio) and WASM (gloo-timers)
#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
    gloo_timers::future::TimeoutFuture::new(duration.as_millis() as u32).await;
}
