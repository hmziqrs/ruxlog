# Building a Full-Stack Blog Platform in Rust: Lessons from Ruxlog

I built a blog platform in Rust. Backend, frontend, all of it. Two Dioxus apps, an Axum API, bunch of shared libraries, and targets for web, desktop and mobile. This is what I learned, what worked, what didn't, and why I'm pausing but not giving up on the Rust frontend ecosystem.

Also worth mentioning upfront, this article was written with help from AI. I wrote the raw thoughts and direction, AI helped structure and polish it. Same way I built most of this project honestly.

## Why Rust for Everything?

It started with a simple idea: build a production-ready project first, then extract a boilerplate from it. The other way around — building a boilerplate first and then a project on top — felt counterproductive. A boilerplate designed in isolation ends up with features a real project might never need. Ironically, that's exactly what happened anyway. The project grew, and I ended up disabling a lot of features behind flags for the actual blog.

The project itself is just a blog. Backend in Axum, frontend in Dioxus for both the consumer blog and the admin dashboard. I also wanted to see how the same frontend code would behave across desktop, mobile, and web — not as a cross-platform stress test, but just to get a sense of where things stand with Dioxus today.

The two frontends share components and state management stores between them, not business logic between server and client. There's no shared domain layer across that boundary — the admin and consumer apps just reuse the same UI building blocks and data stores.

## The Stack

Backend is Axum with SeaORM and PostgreSQL. Frontend is two separate Dioxus apps — a consumer blog with SSR for SEO, and an admin SPA with a post editor, image editing, and analytics. The two apps are separate because I didn't want to bloat the public blog with editor and image processing dependencies. Both apps share a set of libraries for HTTP requests, form handling, state management, and a Shadcn-inspired component library.

The whole codebase is heavily feature-gated. This wasn't planned from the start. It became necessary because I let AI loose and ended up with a ton of features I didn't need for the initial release. Email notifications, sessions, file storage — all implemented but some disabled for now. Feature flags let me ship a minimal blog without having to rip all that code out. I'll go deeper into this in a future article about how AI development can lead to bloated planning and premature feature implementation.

## What Worked Well

**Axum is great.** The backend was genuinely fun to build. Middleware composition, type-safe extractors, the whole tower ecosystem. It just works well together. SeaORM was good enough for database stuff, and the custom `rux-auth` crate I built on top of tower-sessions gave me the auth layer I wanted. Session-based, role-based, composable.

**Shared types between apps.** Having the same Rust structs for API responses in both frontend apps removes a whole category of bugs. No more hoping your TypeScript types match what the server actually sends. The compiler just tells you when something is off.

**Dioxus's mental model.** Coming from React, not having to think about whether `useState` is running on the server or client is really nice. Signal-based reactivity feels clean. The hybrid data fetching approach with server functions and `use_server_cached` for hydration-safe state makes sense once you get it.

**Feature flags saved the project.** When you're one person building a full-stack platform, being able to ship a minimal blog while having a bunch of other stuff already built but turned off is really practical. Ship the MVP, turn on features when they're ready.

## Where Things Got Painful

**The ecosystem gap.** This is the real honest part. I had to build `oxui` from scratch just to have basic UI components. In React or Vue or Svelte you just pick a component library and move on. In Dioxus you're implementing accordion animations and combobox keyboard navigation yourself. That's not fast.

**Figuring things out took longer than it should have.** Not because the libraries aren't capable — they are. But finding out *how* to use them often meant reading source code instead of examples. I started with Diesel for the ORM but hit a wall with dynamic queries. I needed filters, sorting, and pagination on the same query, but Diesel's boxed queries can't be cloned, so you can't count total results and fetch a page from the same dynamic query. SeaORM handled all of that without friction. For validation I tried garde first, but it conflicted with Axum's state extractor — combining both on a handler just broke. Switched to validator, and eventually wrote custom extractors so validation errors came back in a consistent, structured format the frontend could actually parse, instead of plain strings. Same story with Dioxus SSR — setting up server-side rendering meant digging through source code, filtering examples for patterns that might work, and sometimes ending up on Google page 3 looking for some random article or YouTube video. Every single one of these could have been solved faster if proper examples existed. Instead I spent time digging through source code, trying combinations, and figuring things out by trial and error. None of this is a criticism of any team or maintainer — the libraries genuinely work well once you know how to use them. But this is a pattern across the Rust ecosystem, not just the projects I used. Look at Iced or GPUI — both are impressive frameworks with real potential, but the documentation and examples aren't there yet. Iced especially, the lack of examples for even basic patterns is one of the main reasons people hesitate to adopt it. And it creates a compounding problem: if developers can't easily learn how a framework works, they're not going to build libraries, crates, and utilities on top of it either. The ecosystem can't grow without that foundation. And with AI-assisted development being so popular now, this problem gets worse. There are a growing number of new Rust libraries and crates that just don't work reliably — published fast, not tested well. AI tools also can't help you much when the docs and examples don't exist in their training data. I tried vibe coding a simple app with GPUI entirely through AI and it just didn't work. The models didn't have enough context about the framework to generate anything usable.

**AI development was a double-edged sword.** I used Claude Code, Codex, and other tools throughout the project. They made it so easy to plan and build features that I ended up with way more than I needed. OTEL auth, comments, user profiles, reports, banning — all fully built, none needed for launch. This topic deserves its own article so I'm writing one separately. Stay tuned.

**Cross-platform ambitions didn't survive contact with reality.** Original plan was to include native Firebase Analytics, Crashlytics, push notifications through Rust FFI. I dropped all of it. I'm not fluent enough in Rust to write solid native interop, and vibe coding that kind of stuff would just create problems. Goal became simpler: release a basic read-only blog, but at least provide binaries for desktop and Android.

## Will I Stop Using Rust and Dioxus?

No and yes.

No I won't stop using Rust. It's still my go-to for backend work. I solve HackerRank problems in Rust for fun. Axum is production-ready and I enjoy working with it.

But I'm pausing frontend projects with Dioxus for now. The ecosystem needs time. Having properly maintained UI libraries would be a big sign that things are ready. Firebase, analytics, push notification packages that just work out of the box. Documentation that covers common stuff without needing you to read the framework source code.

For fast-paced development right now, tools like Next.js, Astro, TanStack, React Native, Flutter, and Tauri are just way ahead in terms of developer experience. Even performance wise, Bun with TypeScript gets you close to Go performance without the complexity of Rust.

That said this was my third Dioxus project and I had fun with all of them. The reactivity model, the server/client approach, what the framework is becoming — it's genuinely compelling.

## What I'm Watching

**Dioxus Native Renderer.** This is what I'm most excited about. A native GPU-rendered UI, not a webview wrapper like Tauri or Electron. That would be huge for desktop apps. Nothing beats the feel of native rendering and that's why I still prefer Flutter for mobile, it uses GPU rendering across all platforms. If Dioxus gets this right with a good component ecosystem it could seriously replace Electron for a lot of use cases.

**The broader Rust UI ecosystem.** Between Dioxus, Bevy for games, and stuff like Iced and egui, Rust is slowly building a real frontend story. Not ready for production fast-paced work yet but the direction is clear.

## Takeaways

1. **Rust backend is production-ready.** Axum, SeaORM, tower — mature and performant. No reservations.

2. **Rust frontend is promising but not there yet.** You'll spend a lot of time building stuff that other ecosystems give you for free.

3. **Shared types between apps are worth it.** The safety you get from the compiler makes up for the extra effort.

4. **Feature flags are a lifesaver for solo devs.** Ship small, turn things on when they're ready. Don't let scope creep block your release.

5. **AI tools need a human driving.** They're accelerators not autopilots. More on this in an upcoming article about how AI bloats projects.

6. **Know when to pause.** Using early tools isn't failure, it's collecting data. I'll come back to Dioxus when the ecosystem catches up to what the framework is trying to be.

Ruxlog is open source at [github.com/hmziqrs/ruxlog](https://github.com/hmziqrs/ruxlog). Backend is solid, frontend works, and the shared libraries might save someone else from having to build an accordion component from scratch in Rust.

---

*Written by [hmziqrs](https://github.com/hmziqrs) with help from AI for structuring and editing.*
