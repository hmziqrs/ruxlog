---
title: "Demo SSG Blockers and How We Fixed Them"
slug: "demo-ssg-blockers-and-fixes"
excerpt: "Postmortem of the key blockers in mobile/static demo mode and the concrete fixes applied."
published_at: "2026-02-20T12:30:00Z"
updated_at: "2026-02-20T13:00:00Z"
author:
  name: "Hmziq"
  email: "hmziq@example.com"
category:
  name: "Postmortem"
  slug: "postmortem"
  color: "#f59e0b"
tags:
  - name: "Android"
    slug: "android"
    color: "#10b981"
  - name: "Debugging"
    slug: "debugging"
    color: "#ef4444"
  - name: "SSR"
    slug: "ssr"
    color: "#6366f1"
  - name: "SSG"
    slug: "ssg"
    color: "#3b82f6"
featured_image:
  file_url: "/assets/logo.png"
  width: 512
  height: 512
---
# Main blockers from this run

## Blocker 1: app crash on Android startup

Observed failure:

- `UnsatisfiedLinkError: dlopen failed: library "libssl.so" not found`

Root cause:

- `reqwest` native TLS path pulled OpenSSL into the mobile graph.

Fix:

- switched native `reqwest` to rustls-only in `frontend/oxcore/Cargo.toml`
- regenerated lockfile so `hyper-tls` path was removed
- verified with `cargo tree -i openssl-sys` (no matches for Android target)

## Blocker 2: perceived blank app screen

Observed behavior:

- initial screenshot looked white/empty

Root cause:

- render timing and capture timing mismatch; app was alive and WebView content loaded after initial frame

Fix:

- validated with delayed screenshot and UI hierarchy dumps
- confirmed routes + content nodes were present

## Blocker 3: route validation noise on emulator

Observed behavior:

- navigation back key could jump out of app task into prior app state

Fix:

- switched smoke flow to in-app tap navigation and explicit re-entry to app task
- captured deterministic evidence for home, post, category/tag pages

The result was a stable demo mode with no backend dependency errors in runtime logs.

