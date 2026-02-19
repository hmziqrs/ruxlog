---
title: "Demo SSG Debug References We Used"
slug: "demo-ssg-debug-references"
excerpt: "A reference log of docs, code paths, and evidence used to fix the consumer demo static flow."
published_at: "2026-02-20T12:00:00Z"
updated_at: "2026-02-20T12:20:00Z"
author:
  name: "Hmziq"
  email: "hmziq@example.com"
category:
  name: "Engineering"
  slug: "engineering"
  color: "#0ea5e9"
tags:
  - name: "Dioxus"
    slug: "dioxus"
    color: "#f97316"
  - name: "Debugging"
    slug: "debugging"
    color: "#ef4444"
  - name: "SSG"
    slug: "ssg"
    color: "#3b82f6"
featured_image:
  file_url: "/assets/logo.png"
  width: 512
  height: 512
---
# References used during the fix

This note tracks the exact references used while stabilizing the consumer demo static flow.

## Context7 / Dioxus docs

- Static route generation for SSG:
  `docs-src/0.7/src/essentials/fullstack/static_site_generation.md`
- Feature-gated server/client split:
  `docs-src/0.7/src/tutorial/backend.md`
- Native/fullstack server function model:
  `docs-src/0.7/src/essentials/fullstack/native.md`

## Repository code paths that mattered

- `frontend/consumer-dioxus/src/server_fns.rs`
- `frontend/consumer-dioxus/src/demo_content/mod.rs`
- `frontend/consumer-dioxus/src/main.rs`
- `frontend/oxcore/Cargo.toml`

## Runtime evidence used

- Android `adb logcat` traces
- APK library inspection
- emulator screenshots + UI hierarchy dumps
- dependency tracing with `cargo tree`

These references gave us a repeatable way to move from crash reports to root cause and then to verified fixes.

