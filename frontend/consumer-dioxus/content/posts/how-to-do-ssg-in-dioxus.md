---
title: "How to Do SSG in Dioxus"
slug: "how-to-do-ssg-in-dioxus"
excerpt: "A practical pattern for pre-rendering public routes in Dioxus with static demo content."
published_at: "2026-02-20T09:00:00Z"
updated_at: "2026-02-20T09:30:00Z"
author:
  name: "Hmziq"
  email: "hmziq@example.com"
category:
  name: "Dioxus"
  slug: "dioxus"
  color: "#f97316"
tags:
  - name: "Dioxus"
    slug: "dioxus"
    color: "#f97316"
  - name: "SSG"
    slug: "ssg"
    color: "#3b82f6"
  - name: "Fullstack"
    slug: "fullstack"
    color: "#14b8a6"
featured_image:
  file_url: "/assets/logo.png"
  width: 512
  height: 512
---
# SSG in Dioxus (and yes, "SSJ" usually means SSG)

Static Site Generation in Dioxus means pages are rendered at build time, then served as plain HTML, CSS, and JS.

## Core pieces

1. Keep a `static_routes` server function that returns every route you want pre-rendered.
2. Add dynamic routes from your local markdown index (`/posts/{slug}`, `/tags/{slug}`, `/categories/{slug}`).
3. Build with `--ssg` and fullstack features enabled.

```rust
#[server(endpoint = "static_routes", output = server_fn::codec::Json)]
pub async fn static_routes() -> Result<Vec<String>, ServerFnError> {
    Ok(vec!["/".to_string(), "/about".to_string()])
}
```

## Build command shape

Use your demo feature so routes are generated from local content and not backend APIs:

```bash
dx build --platform web --release --fullstack --ssg --features "server demo-static-content"
```

## Why this works well

- Fast first paint on public pages
- Cheap static hosting deployment
- Deterministic output from versioned markdown content

