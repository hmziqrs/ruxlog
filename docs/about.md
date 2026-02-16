# Ruxlog

A full-stack blog platform built entirely in Rust — backend, frontend, and all the libraries in between.

The project is a monorepo with an [Axum](https://github.com/tokio-rs/axum) API, two [Dioxus](https://dioxuslabs.com) frontends (this consumer blog and a separate admin dashboard), and a set of shared libraries for UI components, state management, forms, and HTTP. Everything targets web, desktop, and mobile from a single Rust codebase.

## Tech Stack

**Backend** — Axum, SeaORM, PostgreSQL, Valkey (Redis-compatible), Cloudflare R2 for storage, and a custom session-based auth crate.

**Frontend** — Dioxus with server-side rendering for the consumer blog and a SPA for the admin dashboard. TailwindCSS for styling. The admin uses EditorJS for rich text editing and photon-rs for image processing — kept separate so those heavy dependencies don't bloat the public blog.

**Shared libraries** — `oxui` (a Shadcn/Radix-inspired component library built from scratch), `oxcore` (platform-agnostic HTTP client), `oxstore` (state management), and `oxform` (form handling with validation).

**Infrastructure** — Docker, Traefik, and Watchtower for deployment. Feature flags across all crates to keep builds lean and enable progressive feature rollout.

## Source

Open source at [github.com/hmziqrs/ruxlog](https://github.com/hmziqrs/ruxlog).

## Author

Built by [hmziqrs](https://github.com/hmziqrs).
