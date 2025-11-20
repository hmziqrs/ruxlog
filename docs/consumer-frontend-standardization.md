# Consumer Frontend Standardization Plan

This document outlines the plan to standardize the `consumer-dioxus` frontend application to match the architecture, patterns, and tooling of the existing `admin-dioxus` application.

## Goal

To establish a consistent development experience and architecture across both frontend applications by replicating the proven structure of `admin-dioxus` in `consumer-dioxus`.

## Current State

- **admin-dioxus**: Full-featured Dioxus application with:
    - Modular structure (`components`, `containers`, `hooks`, `screens`, `utils`)
    - Shared packages (`oxcore`, `oxstore`, `oxform`, `oxui`, `ruxlog-shared`)
    - Routing with guards (`AuthGuardContainer`)
    - Theme management (Dark/Light mode)
    - Tailwind CSS integration
- **consumer-dioxus**: Minimal Dioxus application with a single `main.rs`.

## Standardization Steps

### 1. Dependencies & Configuration

Update `consumer-dioxus/Cargo.toml` to include necessary dependencies and shared packages.

- **Shared Packages**:
    - `oxcore`: HTTP client and core utilities.
    - `oxstore`: State management.
    - `oxform`: Form handling.
    - `oxui`: Shared UI components.
    - `ruxlog-shared`: Shared types and logic.
- **External Crates**:
    - `dioxus`, `dioxus-router`, `dioxus-logger` (if used)
    - `reqwest`, `serde`, `serde_json`
    - `tailwindcss` (via build script or CLI)

### 2. Project Structure

Refactor `consumer-dioxus/src` to match `admin-dioxus`:

```text
src/
├── components/     # Local components specific to consumer
├── containers/     # Layout containers (NavBar, Footer, AuthGuard)
├── hooks/          # Custom hooks
├── screens/        # Page components (Home, Login, etc.)
├── styles/         # CSS/Tailwind styles
├── utils/          # Helper functions
├── config.rs       # App configuration
├── env.rs          # Environment variables
├── main.rs         # Entry point
└── router.rs       # Route definitions
```

### 3. Core Implementation Details

#### A. Entry Point (`main.rs`)
- Initialize `oxcore` HTTP client.
- Set up `dioxus` launch configuration.
- Initialize Theme management (persistence).
- Render `Router` wrapped in necessary providers (Toast, Theme).

#### B. Routing (`router.rs`)
- Define `Route` enum with `#[derive(Routable)]`.
- Implement `NavBarContainer` for common layout.
- **Public Routes** (No Auth Guard):
    - `/`: Home
    - `/posts/:id`: View Post
    - `/login`: Login
    - `/register`: Signup
- **Protected Routes** (Wrapped in `AuthGuardContainer`):
    - `/profile`: User Profile & Settings
    - `/profile/edit`: Edit Profile

#### C. Theme & Styling
- Copy `tailwind.css` setup.
- Implement `persist::get_theme()` logic in `main.rs` to handle dark/light mode preference.
- Ensure `oxui` components render correctly with the theme.

#### D. State Management
- Initialize `oxstore` if global state is needed (e.g., User session, Cart).

### 4. Shared Components (`oxui`)
- Replace any ad-hoc components with `oxui` equivalents (Buttons, Inputs, Cards).
- Ensure `oxui` is correctly linked in `Cargo.toml`.

## Execution Plan

1.  **Setup**: Update `Cargo.toml` and create directory structure.
2.  **Core**: Create `config.rs`, `env.rs`, and basic `main.rs`.
3.  **Routing**: Create `router.rs` and `containers/NavBarContainer.rs`.
4.  **Screens**: Create placeholder screens (`HomeScreen`, etc.).
5.  **Integration**: Connect `oxcore` and `oxui`.
6.  **Verification**: Build and run to ensure parity.

## Verification

- **Build**: `cargo build -p consumer-dioxus` should succeed.
- **Run**: `dx serve` (or equivalent) should launch the app.
- **Visual Check**:
    - Theme switching works.
    - Navigation works.
    - Shared components (`oxui`) look consistent with `admin-dioxus`.
