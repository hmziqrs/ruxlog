---
title: "Getting Started with Dioxus SSG"
slug: "getting-started-dioxus-ssg"
excerpt: "A quick walkthrough for static generation in a Dioxus consumer app demo."
published_at: "2026-02-18T12:00:00Z"
author:
  name: "Hmziq"
  email: "hmziq@example.com"
category:
  name: "Dioxus"
  slug: "dioxus"
  color: "#f97316"
tags:
  - name: "Rust"
    slug: "rust"
    color: "#f59e0b"
  - name: "SSG"
    slug: "ssg"
    color: "#3b82f6"
featured_image:
  file_url: "/assets/logo.png"
  width: 512
  height: 512
---
# Dioxus SSG Demo

This demo post is loaded from markdown frontmatter and rendered as static content.

## Why this setup?

- Build-time rendering for public routes
- Works on static hosting like GitHub Pages and Cloudflare Pages
- Keeps API-backed mode available for normal development

```rust
fn main() {
    dioxus::launch(App);
}
```
