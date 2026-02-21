---
title: "Release Builds with Demo Static Content: What We Solved"
slug: "release-builds-demo-static-content-status"
excerpt: "Status log for cross-platform release builds, demo feature flags, bundle IDs, and Android setup outcomes."
published_at: "2026-02-21T10:00:00Z"
updated_at: "2026-02-21T10:00:00Z"
author:
  name: "Hmziq"
  email: "hmziq@example.com"
category:
  name: "Builds"
  slug: "builds"
  color: "#0ea5e9"
tags:
  - name: "CI"
    slug: "ci"
    color: "#22c55e"
  - name: "Android"
    slug: "android"
    color: "#10b981"
  - name: "Desktop"
    slug: "desktop"
    color: "#06b6d4"
  - name: "Dioxus"
    slug: "dioxus"
    color: "#f97316"
featured_image:
  file_url: "/assets/logo.png"
  width: 512
  height: 512
---
# Release build progress log

This is the current status of what we already solved for release builds and demo content behavior.

## 1) GitHub Actions release matrix is in place

The release workflow now covers:

- Desktop Linux (`x86_64`, `arm64`)
- Desktop Windows (`x86_64`, `arm64`)
- Android (`x86_64`, `arm64`)
- Renderer variants for desktop/mobile: `webview` and `native`
- Platform-specific artifacts plus universal aggregation step

## 2) Demo feature flags are enforced for release

Consumer release builds now run with demo static content enabled and defaults disabled:

- Desktop: `--no-default-features --features "desktop basic demo-static-content"`
- Mobile: `--no-default-features --features "mobile basic demo-static-content"`

This applies to release CI behavior and local verification commands.

## 3) Renderer-specific bundle IDs are handled

Renderer-specific bundle identifiers are set during CI, so each app can have distinct IDs for webview vs native renderers.

Pattern used:

- `com.hmziq.ruxlog.admin.webview` / `com.hmziq.ruxlog.admin.native`
- `com.hmziq.ruxlog.consumer.webview` / `com.hmziq.ruxlog.consumer.native`

## 4) What we verified locally

### Passed

- Consumer desktop release (webview) with demo static content
- Consumer desktop release (native renderer) with demo static content
- Consumer Android webview release build after setting Java from Android Studio

### Environment fix that unblocked Android webview

Using Android Studio JDK resolved the Java runtime failure:

```bash
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export PATH="$JAVA_HOME/bin:$PATH"
```

## 5) Current remaining blocker

Consumer Android native renderer release still fails at link stage with:

- `undefined symbol: android_main`

So the remaining work is native-Android renderer entrypoint/linkage configuration, not demo flags and not keystore.

## 6) Secrets status

For unsigned CI build artifacts, no additional Android signing secret is required yet.

Keystore/signing secrets are needed later only when producing signed distributable release packages.
