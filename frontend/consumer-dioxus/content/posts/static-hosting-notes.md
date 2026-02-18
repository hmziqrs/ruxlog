---
title: "Static Hosting Notes"
slug: "static-hosting-notes"
excerpt: "Notes for deploying a pre-rendered Dioxus site to static infrastructure."
published_at: "2026-02-19T09:30:00Z"
updated_at: "2026-02-19T10:00:00Z"
author:
  name: "Hmziq"
  email: "hmziq@example.com"
category:
  name: "Deploy"
  slug: "deploy"
  color: "#0ea5e9"
tags:
  - name: "SSG"
    slug: "ssg"
    color: "#3b82f6"
  - name: "Cloudflare"
    slug: "cloudflare"
    color: "#f97316"
featured_image:
  file_url: "/assets/logo.png"
  width: 512
  height: 512
---
# Hosting checklist

Static hosting works well when generated HTML, JS, and assets are all present in `dist/`.

1. Build with SSG enabled.
2. Upload the output to your static provider.
3. Verify deep links like `/posts/static-hosting-notes` resolve correctly.
