# Building a Full-Stack Blog Platform in Rust: Lessons from Ruxlog

I built a complete blog platform in Rust — backend, frontend, everything. Two Dioxus apps, an Axum API, a suite of shared libraries, and targets for web, desktop, and mobile. Here's what I learned, what worked, what didn't, and why I'm both pausing and not giving up on the Rust frontend ecosystem.

## Why Rust for Everything?

The idea was straightforward: build a blog platform entirely in Rust to push the boundaries of what's possible with the current ecosystem. Not just the backend (where Rust is already proven), but the frontend too — using Dioxus for both the public-facing consumer blog and the admin dashboard.

I wanted to test full cross-platform capabilities. One language, one type system, shared domain models between server and client, and binaries for web, desktop, and Android from the same codebase.

It started as a boilerplate project to learn Rust and have a solid starting point for future work. But boilerplates feel abstract — it's easier to implement features when they solve a real problem. So it became a blog, and the scope grew from there.

## The Stack

**Backend**: Axum + SeaORM + PostgreSQL. Session-based auth backed by Valkey (Redis-compatible). S3-compatible object storage (RustFS for local dev, Cloudflare R2 for production). SMTP email via lettre. OpenTelemetry for observability.

**Frontend**: Two separate Dioxus apps. The consumer blog uses server-side rendering for SEO — dynamic meta tags, Open Graph, JSON-LD structured data. The admin dashboard is a pure SPA with EditorJS for rich text editing, photon-rs for image processing, and analytics charts. They're separate because bundling code editors and image processors into the public blog would be absurd.

**Shared code**: A set of libraries that both apps consume — `oxcore` for platform-agnostic HTTP (reqwest on native, gloo-net on WASM), `oxstore` for state management, `oxform` for validated forms, and `oxui`, a Shadcn/Radix-inspired component library I built from scratch because nothing like it existed in the Dioxus ecosystem.

**Feature flags everywhere**: The codebase is heavily feature-gated. The backend's default `basic` feature gives you a minimal blog API. Flip on `full` and you get comments, newsletters, analytics, OAuth, ACL, route blocking, and a data seeding system. Same pattern on the frontend — the consumer defaults to a read-only public blog, but feature flags progressively enable auth, profiles, comments, and engagement features.

## What Worked Well

**Axum is excellent.** The backend was genuinely enjoyable to build. Axum's middleware composition, type-safe extractors, and the broader tower ecosystem make for a clean, performant API server. SeaORM handled database operations well enough, and the custom `rux-auth` crate I built on top of tower-sessions gave me exactly the authentication layer I needed — session-based, role-based, composable.

**Shared types are a superpower.** Having the same Rust structs for API responses on both server and client eliminates an entire class of bugs. No more hoping your TypeScript types match your Go structs. The compiler tells you when they don't.

**Dioxus's mental model is refreshing.** Coming from React, not having to do mental gymnastics about `useState` on the server is genuinely nice. The signal-based reactivity is clean, and the hybrid approach for data fetching between server and client components just works once you understand it. Server functions for SSR data fetching, `use_server_cached` for hydration-safe state — it's well-thought-out.

**Feature flags made iteration possible.** When you're one person building a full-stack platform, being able to ship a minimal blog while having comments, analytics, and user management already implemented but behind flags is a practical approach. Ship the MVP, enable features when they're polished.

## Where Things Got Painful

**The ecosystem gap is real.** This is the honest part. Building `oxui` from scratch — an entire component library — just to have basic UI primitives is not fast-paced development. In React/Vue/Svelte land, you pick from dozens of mature component libraries and get moving. In Dioxus, you're implementing accordion animations and combobox keyboard navigation yourself.

**Documentation and examples were a recurring blocker.** Configuring SSR/SSG required digging into the Dioxus core repository, filtering examples for similar patterns, and occasionally landing on Google page 3 to find a relevant article or YouTube video. This isn't a criticism of the Dioxus team — they're doing incredible work with limited resources — but it's the reality of an early ecosystem. Basic things that Next.js or Astro handle elegantly required significant investigation.

**AI-assisted development was a double-edged sword.** I used Claude Code, Codex, and other AI tools throughout the project. They made planning and implementing new features deceptively easy — to the point where the codebase bloated with fully built features I didn't actually need yet. This deserves its own deep dive, so I'm writing a separate article on how AI-driven development can lead to bloated planning and premature feature implementation. Stay tuned for that one.

**Cross-platform ambitions met reality.** The original plan included native Firebase Analytics, Crashlytics, and push notifications via Rust FFI. I scrapped it. I'm not fluent enough in Rust to write reliable native interop, and vibe-coding it would create more problems than it solved. The goal thinned out to: release a basic read-only blog, but at least ship binaries for desktop and Android.

## The Codebase in Numbers

- **30+ database entities** — users, posts, categories, tags, media (with variants and usage tracking), comments, likes, views, sessions, bans, series, revisions, newsletters, ACL, and more.
- **15+ API modules** — all versioned, most feature-gated, with a layered middleware stack (request tracking, CORS, auth, CSRF, compression, tracing).
- **18+ oxui components** — built from scratch, Shadcn-inspired, covering everything from buttons and cards to dialogs, comboboxes, and data tables.
- **5 shared libraries** — oxcore, oxstore, oxform, oxui, ruxlog-shared — plus custom Dioxus SDK extensions for geolocation, notifications, storage, and window utilities.
- **4 target platforms** — web, desktop, mobile, and server (SSR).

## Will I Stop Using Rust and Dioxus?

No and yes.

No, I won't stop using Rust. It remains my go-to for backend work, and I regularly solve problems in it for the challenge and satisfaction. Axum is production-ready and a joy to work with.

But I'm pausing on building frontend-heavy projects with Dioxus — for now. The ecosystem needs time. Properly maintained UI libraries would be a massive indicator of readiness. Firebase/analytics/push notification packages that just work. Documentation that covers common patterns without requiring source-code archaeology.

In practical terms, for fast-paced development cycles today, mature tools like Next.js, Astro, TanStack, React Native, Flutter, and Tauri provide a dramatically better developer experience. Even in terms of raw performance, Bun with TypeScript gets you near-Go performance without Rust's complexity.

That said, this was my third Dioxus project. I've had genuine fun with every one. The framework's approach to reactivity, its server/client model, and the promise of what it's becoming are compelling.

## What I'm Watching

**Dioxus Native Renderer.** This is the one I'm most excited about. A native GPU-rendered UI — not a webview wrapper like Tauri or Electron — would be a game-changer for desktop applications. Nothing beats the feel of native rendering, and it's why I still prefer Flutter for mobile (GPU rendering across all platforms). If Dioxus achieves this with a mature component ecosystem, it could genuinely replace Electron for a class of desktop apps.

**The broader Rust UI ecosystem.** Between Dioxus, Bevy (for games), and projects like Iced and egui, Rust is slowly building a real frontend story. It's not there yet for production fast-paced work, but the trajectory is clear.

## Takeaways

1. **Rust backend is production-ready.** Axum, SeaORM, tower — the ecosystem is mature and performant. No reservations here.

2. **Rust frontend is promising but not fast-paced-ready.** You'll spend significant time building infrastructure that other ecosystems provide out of the box.

3. **Shared types between server and client are worth the investment.** The safety guarantees alone justify the approach, even if the frontend framework is still maturing.

4. **Feature flags are essential for solo developers.** Ship small, enable progressively. Don't let scope creep block your MVP.

5. **AI tools need a human owner.** They're accelerators, not autopilots. More on this in an upcoming article about bloated AI-driven development.

6. **Know when to pause.** Using immature tools isn't failure — it's gathering data. I'll revisit Dioxus when the ecosystem catches up to the framework's ambition.

Ruxlog is open source at [github.com/hmziqrs/ruxlog](https://github.com/hmziqrs/ruxlog). The backend is solid, the frontend is functional, and the shared libraries might save someone else from building an accordion component from scratch in Rust.

---

*Written by [hmziqrs](https://github.com/hmziqrs)*
