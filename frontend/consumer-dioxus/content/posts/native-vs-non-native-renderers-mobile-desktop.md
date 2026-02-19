---
title: "Native vs Non-Native Renderers for Desktop and Mobile"
slug: "native-vs-non-native-renderers-mobile-desktop"
excerpt: "How to think about Dioxus renderer choices across desktop and mobile targets."
published_at: "2026-02-20T11:00:00Z"
updated_at: "2026-02-20T11:40:00Z"
author:
  name: "Hmziq"
  email: "hmziq@example.com"
category:
  name: "Renderers"
  slug: "renderers"
  color: "#22c55e"
tags:
  - name: "Dioxus"
    slug: "dioxus"
    color: "#f97316"
  - name: "Mobile"
    slug: "mobile"
    color: "#10b981"
  - name: "Desktop"
    slug: "desktop"
    color: "#06b6d4"
  - name: "Renderer"
    slug: "renderer"
    color: "#a855f7"
featured_image:
  file_url: "/assets/logo.png"
  width: 512
  height: 512
---
# Letting Native and Non-Native Renderers Work Together

You can keep one Dioxus component tree while targeting different renderer modes on desktop and mobile.

## Non-native path (common today)

- `dioxus/web` for browser apps
- `dioxus/desktop` and `dioxus/mobile` for native app shells with webview-backed UI

This gives strong cross-platform reuse and fast feature delivery.

## Native path

Dioxus also has a native renderer track (`dioxus-native`) for direct native rendering, separate from the webview path.

## Practical strategy

1. Share routes, components, and state logic across all targets.
2. Keep feature flags explicit (`web`, `desktop`, `mobile`, `server`).
3. Use target-specific modules only for platform APIs (filesystem, camera, notifications).

```toml
[features]
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
mobile = ["dioxus/mobile"]
server = ["dioxus/server"]
```

This lets your public content model stay consistent while each renderer target gets the right runtime.

