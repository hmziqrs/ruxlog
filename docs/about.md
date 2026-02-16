# Ruxlog

A full-stack blog platform built entirely in Rust — backend, frontend, and all the libraries in between.

The project is a monorepo with an [Axum](https://github.com/tokio-rs/axum) API, two [Dioxus](https://dioxuslabs.com) frontends (this consumer blog and a separate admin dashboard), and a set of shared libraries for UI components, state management, forms, and HTTP. Everything targets web, desktop, and mobile from a single Rust codebase.

## Tech Stack

**Backend** — Axum, SeaORM, PostgreSQL, Valkey (Redis-compatible), Cloudflare R2 for storage, and a custom session-based auth crate.

**Frontend** — Dioxus with server-side rendering for the consumer blog and a SPA for the admin dashboard. TailwindCSS for styling. The admin uses EditorJS for rich text editing and photon-rs for image processing — kept separate so those heavy dependencies don't bloat the public blog.

**Shared libraries** — `oxui` (a Shadcn/Radix-inspired component library built from scratch), `oxcore` (platform-agnostic HTTP client), `oxstore` (state management), and `oxform` (form handling with validation).

**Infrastructure** — Docker, Traefik, and Watchtower for deployment. Feature flags across all crates to keep builds lean and enable progressive feature rollout.

## Why Rust?

The goal was to test how far you can push Rust for full-stack development today — shared types between server and client, cross-platform targets from one codebase, and the safety guarantees that come with the language. It started as a learning project and grew into a real platform.

The honest answer: the Rust frontend ecosystem isn't ready for fast-paced development yet. Building a component library from scratch and debugging SSR configuration by reading framework source code isn't quick. But the developer experience of signal-based reactivity without React's footguns, and having the compiler catch type mismatches between your API and UI, is genuinely great.

More on this in the [blog post about building Ruxlog](/posts/building-ruxlog).

## Source

The project is open source at [github.com/hmziqrs/ruxlog](https://github.com/hmziqrs/ruxlog).

## Author

Built by [hmziqrs](https://github.com/hmziqrs).
