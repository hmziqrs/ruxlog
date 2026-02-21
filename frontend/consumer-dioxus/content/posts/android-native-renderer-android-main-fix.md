---
title: "Android Native Renderer Fix: Resolving android_main"
slug: "android-native-renderer-android-main-fix"
excerpt: "How we fixed the Android native renderer release failure caused by a missing android_main entrypoint."
published_at: "2026-02-21T12:00:00Z"
updated_at: "2026-02-21T12:00:00Z"
author:
  name: "Hmziq"
  email: "hmziq@example.com"
category:
  name: "Builds"
  slug: "builds"
  color: "#0ea5e9"
tags:
  - name: "Android"
    slug: "android"
    color: "#10b981"
  - name: "Native Renderer"
    slug: "native-renderer"
    color: "#22c55e"
  - name: "Dioxus"
    slug: "dioxus"
    color: "#f97316"
  - name: "Release"
    slug: "release"
    color: "#06b6d4"
featured_image:
  file_url: "/assets/logo.png"
  width: 512
  height: 512
---
# Android native release fix log

We fixed the Android native renderer release failure:

- `undefined symbol: android_main`

## What was happening

Native Android renderer builds (`--renderer native`) were failing at link time because the app did not export an Android-native entrypoint expected by the Android activity glue path.

## Root cause

For this renderer path, the native runtime needs:

1. An exported `android_main(app: AndroidApp)` entrypoint.
2. The Android app handle registered before creating the winit/blitz event loop.

Without those, linking/runtime setup for native Android is incomplete.

## The fix we applied

In `consumer-dioxus`:

- Added Android-only dependencies:
  - `android-activity`
  - `blitz-shell`
- Added `#[no_mangle] fn android_main(app: android_activity::AndroidApp)`.
- Called `blitz_shell::set_android_app(app)` inside `android_main`.
- Reused shared launch logic via `run_client()` to keep behavior consistent with desktop/mobile non-web builds.

Minimal shape of the fix:

```rust
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    blitz_shell::set_android_app(app);
    run_client();
}
```

## Validation results

After the patch, these release builds with demo content passed:

- Android native ARM64:
  - `dx build --platform android --renderer native --release --no-default-features --features "mobile basic demo-static-content"`
- Android native x86_64:
  - `dx build --platform android --renderer native --release --target x86_64-linux-android --no-default-features --features "mobile basic demo-static-content"`
- Android webview demo build also still passed.
- Desktop native demo build still passed.

## Outcome

Android native renderer release builds are now unblocked for both ARM64 and x86_64 in demo static content mode.
