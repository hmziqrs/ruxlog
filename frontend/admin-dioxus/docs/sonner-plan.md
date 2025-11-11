# Sonner for Dioxus — Implementation Plan

This document is the canonical, step-by-step plan to build a Sonner-equivalent toast system for Dioxus. It is designed for an AI agent or a developer to follow deterministically. Each phase is small, independently testable, and includes acceptance criteria and checklists. The implementation should track Sonner’s API and behavior as captured in `docs/sonner.ref` while following idioms and patterns from the existing Dioxus `src/components/toast.rs`.

Sources of truth:
- React/TS reference: `docs/sonner.ref` (contains assets, hooks, index, types)
- Current Dioxus toast: `src/components/toast.rs` (provider, stacking, portal, timers, a11y)

Primary goal:
- Ship a feature-parity, production-grade Sonner clone in Dioxus with a clean API, robust a11y, and smooth UX.

Non-goals (for v1 unless specifically included in a phase below):
- Animations identical to CSS keyframes of Sonner. We will match behavior and feel; pixel-perfect parity is not required.
- Fancy devtools or analytics.


## Progress Update — 2025-08-17T01:26:00+05:00

- Phase 9 Promise-based toasts implemented:
  - Added `PromiseConfig` in `src/components/sonner/types.rs`.
  - Added `SonnerToasts.promise(...)` in `src/components/sonner/state.rs` to insert a loading toast and update to success/error on resolution.
  - Ensured timers/durations behave correctly: `src/components/sonner/toast.rs` initializes timers from `props.duration_ms` on start/update; provider `add_toast` skips assigning default duration to `Loading` toasts so they persist until resolved.
- Demo: added “Promise Success” and “Promise Error” buttons in `src/screens/sonner_demo.rs` that simulate async success/failure via `dioxus_time::sleep`.
- Build: `cargo check` passes.
 - Callbacks: `on_auto_close` fires on timer end; new `on_dismiss` added and invoked by provider on any removal (manual close, swipe, or auto-close), including after promise transitions.

## Progress Update — 2025-08-16T14:05:00+05:00

- Mirrored top/bottom stacking animations in `src/components/sonner/toaster.rs`:
  - Top positions now animate via `top` only; bottom positions via `bottom` only.
  - Top visible positioning mirrors bottom logic: `top = visible_height - (dist_from_slice_top + height_i)` so new toasts appear at the top and push older ones downward.
  - Overflow stacks (older items) use scale/opacity with pointer-events disabled and correct z-index.
- Transition parity:
  - All branches include `transform 200ms ease, opacity 200ms ease, top|bottom 200ms ease`.
  - Ensures smooth reflow when items shift and when overflow fades/scales.
- Stability fixes:
  - Stable keys for toasts (`key: toast.id`) to avoid remounts that cancel transitions.
- Stacking/measurement:
  - Container height computed from visible slice only.
  - Offsets computed forward for top clusters and reverse-accumulated for bottom clusters.
- Positions/offsets:
  - Desktop/mobile offsets supported; center positions span horizontally with item self-centering.

Next:
- Add entrance/exit transitions and delayed removal after out animation.
- Centralize transition timing/easing as shared constants.
- Implement dismissible/closeButton parity and `onDismiss`.
- Begin swipe/drag to dismiss (Phase 7).


## Progress Update — 2025-08-15T13:59:55+05:00

- Implemented Phase 4 stacking behaviors:
  - Enforced `visibleToasts` limit with stacking of overflow toasts (scale/opacity, z-index, pointer-events: none).
  - `expand` shows all toasts without limiting/clamping.
  - Container height computed from visible toasts only.
  - Positions Top/Bottom flows respected for offsets; full positional offsets remain in Phase 5.
- Added smooth layout transitions for position/opacity changes during reflow (movement transitions). Entrance/exit transitions still TODO.
- Cleaned `heights` state on toast dismiss/delete to avoid stale entries.
- Updated this plan’s Phase 4 deliverables and acceptance to reflect completion.

Next:
- Implement entrance/exit transitions for mount/unmount.
- Start Phase 5: positions and viewport offsets (desktop/mobile).


## Progress Update — 2025-08-13T19:23:35+05:00

- Created `src/components/sonner/` with `mod.rs`, `types.rs`, `state.rs`.
- Implemented core types and defaults in `types.rs` (`ToastType`, `Position`, `SwipeDirection`, `Action`, `ToastIcons`, `ToastClassNames`, `ToastOptions`, `ToastT`, `ToasterProps`, `Offset`, constants).
- Added `use_sonner()` handle and context contracts in `state.rs` (callbacks and ID generator; provider to come in Phase 2).
- Exported module via `src/components/mod.rs` (`pub mod sonner;`).
- Verified compilation via `cargo check`.

Next: Phase 3 — implement timers, auto-dismiss, and pause-on-hover/hidden.


## High-level Architecture

- A "Toaster" provider (context + portal) manages a list of toasts, their lifecycle, stacking and positioning.
- An imperative handle (hook) to push/dismiss/update toasts from anywhere.
- Layout/interaction logic: measure heights, compute offsets/gaps, pause timers on hover/focus/interaction, drag/swipe to dismiss, and auto-dismiss.
- Theming/styling: expose rich colors, invert, unstyled, classNames; leverage existing design tokens/classes used by `src/components/toast.rs`.
- Accessibility: region and roles (alert/alertdialog), labels, focus management, keyboard shortcuts, RTL.


## Proposed File Structure (new)

- `src/components/sonner/`
  - `mod.rs`
  - `types.rs` (Rust equivalents of Sonner types: ToastTypes, ToastT, ToasterProps, Action, etc.)
  - `state.rs` (toast list, signals, height tracking, timers)
  - `toaster.rs` (provider component + portal + stacking container)
  - `toast.rs` (single toast view and interactions)
  - `icons.rs` (Success, Info, Warning, Error, Close, Loader)
  - `gestures.rs` (swipe/drag helpers; optional feature gate if needed)
  - `styles.md` (notes on classes/variables we rely on; we reuse existing utility classes like `text-foreground` from the codebase)

We will reuse or be guided by:
- `src/components/toast.rs` for: provider pattern, portal usage, focus region, timer pattern (`use_timeout`), class utilities.
- `super::portal_v2::{use_portal, PortalIn, PortalOut}`.
- `hooks::use_unique_id`.

Note: If we prefer a single-file component for v1, we can integrate into `src/components/sonner_toast.rs` and expand later. The plan assumes a small module for clarity.


## Parity Matrix (Sonner → Dioxus mapping)

- Toast types: `success`, `info`, `warning`, `error`, `loading`
  - Rust enum: `ToastType::{Success, Info, Warning, Error, Loading}`
- Core toast fields (`ToastT` in Sonner):
  - id, title, description, type, icon, jsx (custom node), richColors, invert, closeButton, dismissible, duration, delete, action, cancel, onDismiss, onAutoClose, promise, styles/classNames, position, testId → Rust struct fields with Option/flags
- Toaster props (`ToasterProps` in Sonner):
  - id, invert, theme(light/dark/system), position, hotkey, richColors, expand, duration, gap, visibleToasts, closeButton, toastOptions (defaults), className/style, offset/mobileOffset, dir, swipeDirections, icons, containerAriaLabel → Rust props with appropriate types and defaults
- Behavior:
  - Auto-dismiss with timer; pause on hover, interacting, and when `document.hidden` → Dioxus: timers via `dioxus_time`, pause/resume on hover/focus and when page hidden via `document` events.
  - Stacking: maximum visible toasts, gap, measuring heights → compute offsets, update on content change, animate in/out.
  - Swipe/drag to dismiss (threshold ~45px) → pointer events with velocity/thresholds.
  - Promise toasts: loading → success/error state transitions.
  - Multiple toasters by id.
  - RTL and positioning.


## Acceptance Criteria (global)

- API remains stable and ergonomic, with parity to Sonner where it makes sense for Rust/Dioxus.
- Provider renders in a portal, is accessible (`role`, `aria-*`, focus region), and supports keyboard interactions.
- Timers pause on hover/focus and when page hidden; resume when appropriate.
- Stacking and positions behave as expected across screen sizes; mobile offsets applied at narrow widths.
- Swipe to dismiss works with mouse/touch; threshold prevents accidental dismissals.
- Promise flows update the same toast id; no flicker or jumps.
- Theming and className overrides are honored; unstyled/invert work.


## Phase 0 — Discovery and Spec (Read-only)

Checklist:
- [ ] Extract from `docs/sonner.ref` all props and types (`types.tsx` block) and list in `types.rs` draft.
- [ ] Extract behaviors from `index.tsx` (visibleToasts, viewport offsets, timer logic, pause-on-hover, doc-hidden logic, measuring strategy, swipe, expand-by-default, rich colors)
- [ ] Extract assets and loader from `assets.tsx` → map to `icons.rs` in Dioxus.
- [ ] Map each Sonner feature to the Dioxus equivalent; flag risky/unknown items.

Exit criteria:
- [ ] This plan reflects all Sonner props/behaviors with Dioxus-native equivalents.


## Phase 1 — API and Types Scaffold (Compilable, no UI yet)

Deliverables:
- `src/components/sonner/types.rs`
  - [x] `ToastType` enum with `Success, Info, Warning, Error, Loading`.
  - [x] `Position` enum: `TopLeft, TopRight, BottomLeft, BottomRight, TopCenter, BottomCenter`.
  - [x] `SwipeDirection` enum: `Top, Right, Bottom, Left`.
  - [x] `Action` struct (label: Node/Text, on_click callback, optional style).
  - [x] `ToastIcons` struct (optional custom icons per type + loading + close).
  - [x] `ToastClassNames` struct mirroring Sonner `classNames` shape (toast, description, icon, content, title, action/cancel buttons, per-type overrides).
  - [x] `ToastOptions` for per-toast overrides (duration, closeButton, className, classNames, style, descriptionClassName, cancelButtonStyle, actionButtonStyle, toasterId, etc.).
  - [x] `ToastT` struct modeling runtime toast (id, type, title, description, icon, durations, flags, action/cancel, promise state, position, styles, classNames, testId, etc.).
  - [x] `ToasterProps` equivalent.
  - [x] `Offset` type: either number/string or per-side struct.

- `src/components/sonner/state.rs`
  - [ ] `Heights` tracking type: `{ toast_id, height, position }`.
  - [ ] Signals/State for: toast list, heights list, interacting flag, expanded flag, default props.
  - [ ] Events to add/update/dismiss/delete toasts.

- `src/components/sonner/mod.rs`
  - [x] Re-exports and basic module wiring.

Acceptance:
- [x] Compiles. No rendering yet.

Integration test:
- [ ] Add a temporary example compilation unit that instantiates `ToasterProps` defaults.


## Phase 2 — Toaster Provider + Portal (Basic render, no animations)

Deliverables:
- `src/components/sonner/toaster.rs`
  - [x] `SonnerToaster` component:
    - [x] Context provider with signals for toasts, heights, interacting, defaults.
    - [x] Portal via `use_portal`, `PortalIn/Out` (reusing `src/components/toast.rs` pattern).
    - [x] Container `role="region"`, `aria-label` from `props.containerAriaLabel` or computed count.
    - [x] `dir` attribute support (`ltr`, `rtl`, `auto`).
    - [x] Positioning based on `props.position`.
    - [x] Visible list (no measuring/stacking yet): render each toast using `toast.rs` component.

- `src/components/sonner/toast.rs`
  - [x] Minimal `ToastView` with title, optional description, close button.
  - [x] `aria-labelledby/aria-describedby` wiring via unique ids (`use_unique_id`).
  - [x] Close button behavior calls provider’s dismiss.

- Hook/handle:
  - [x] `use_sonner()` returns a `SonnerToasts` handle with `.show`, `.success`, `.error`, `.warning`, `.info`, `.loading` convenience methods.

Acceptance:
- [x] Mounting `SonnerToaster` and calling `use_sonner().success(...)` renders a toast.

Integration test:
- [x] Add a demo route/screen with two buttons to trigger `success` and `error` (Route: `/demo/sonner`).


## Phase 3 — Timers, Auto-dismiss, Pause on Hover/Interacting/Hidden

Deliverables:
- [x] Per-toast `duration` with provider-level default (via `ToasterProps::duration_ms` fallback in `SonnerToaster`).
- [x] Pause/resume timer on hover and on focus inside the toast (`src/components/sonner/toast.rs`).
- [x] Global `interacting` flag toggled while pointer is inside the list (`src/components/sonner/toaster.rs`).
- [x] `document.hidden` handling: pause all timers while hidden (visibilitychange listener wired via `dioxus::document::eval`).
- [x] `onAutoClose` callback when a toast closes automatically. Exposed via `ToastOptions::on_auto_close` and invoked by `SonnerToast` when timers expire.

Acceptance:
- [x] If user hovers over toasts, timers pause; on leaving, resume.
- [x] Switching tabs pauses timers; returning resumes.

Integration test:
- [x] Demo duration set to 2s; verify pause/resume via `/demo/sonner`.


## Phase 4 — Stacking, Heights, Gap, VisibleToasts, Expand

Deliverables:
- [x] Measure toast heights on mount and after content updates; update `heights` state.
- [x] Compute per-toast offset from prior visible toasts using `gap` (default 14) and measured heights.
- [x] Enforce `visibleToasts` (default 3) per Sonner behavior; others stack beneath.
- [x] `expand`: when true, skip clipping/limit so content shows fully.
- [ ] Basic in/out transitions using existing class utilities (match `src/components/toast.rs` approach).

Acceptance:
- [x] Only `visibleToasts` are fully visible; others are stacked with scale/opacity/offset.
- [x] Toggling `expand` shows full height of each toast.

Integration test:
- [ ] Render 5+ toasts; visually confirm stacking and offsets.
## Phase 5 — Positions and Offsets (Desktop/Mobile)

Deliverables:
 - [x] Support positions: `top-left`, `top-right`, `bottom-left`, `bottom-right`, `top-center`, `bottom-center`.
 - [x] Implement viewport offsets: `offset` and `mobileOffset` similar to Sonner (`24px` desktop, `16px` mobile by default).
 - [x] Simple mobile detection by viewport width (configurable threshold) for `mobileOffset`.

Acceptance:
 - [x] Each position works; offsets applied correctly.
 - [x] Mobile viewport uses `mobileOffset` values.

Integration test:
 - [x] Add a demo that cycles through positions; add a control for offset/mobileOffset.
- [ ] `dismissible` flag per toast (default true) to allow/deny manual dismissal.
- [ ] `closeButton` global (toaster props) and per-toast overrides.
- [ ] Provider-level `remove` after an out animation (delay ~200ms like `src/components/toast.rs`).
- [ ] `onDismiss` callback when a toast is dismissed manually.

Acceptance:
- [ ] Clicking close dismisses the toast (if dismissible) and fires `onDismiss`.

Integration test:
- [ ] Trigger non-dismissible toast and confirm close button is hidden/disabled.


## Phase 7 — Swipe/Drag to Dismiss

Deliverables:
- [x] Pointer/touch events on the individual toast item; track drag distance (velocity optional/TBD).
- [x] Dismiss if exceeding threshold ~45px (configurable) in an allowed direction based on current position.
- [x] Animate back to original position if threshold not met.

Acceptance:
- [ ] Dragging a toast along the configured direction dismisses it after threshold, otherwise snaps back.

Integration test:
- [ ] Manual QA on desktop and mobile devices/emulators.


## Phase 8 — Icons, Loader, and Custom Icon Overrides

Deliverables:
- [x] `icons.rs` with Success, Info, Warning, Error, Close, and a Loader made of bars/spinner similar to Sonner.
- [x] `ToastIcons` prop to override icons globally; per-toast custom icon supported (`toast.icon`).
- [x] Loading toasts show loader (and block interactions if desired).

Acceptance:
- [x] Icons appear per type; loader appears for loading toasts.
- [x] Per-toast and global overrides work.

Integration test:
- [x] Demo controls to swap icon set and override a single toast’s icon.


## Phase 9 — Promise-based Toasts

Deliverables:
- [x] API: `toasts.promise(future, { loading, success, error, ...options })`.
- [x] Internals: insert `loading` toast; when future resolves, update the same toast id to `success` or `error` (merge updates; keep duration semantics like Sonner).
- [x] Optional: support `onAutoClose`/`onDismiss` on transitions as appropriate.

Acceptance:
- [x] A sample async task shows loading → success; failing task shows loading → error; timings map to options/defaults.

Integration test:
- [x] Demo page with a “simulate success” and “simulate failure” button.


## Phase 10 — Theming, Rich Colors, Invert, Unstyled, ClassNames

Deliverables:
- [ ] `theme` prop: `light | dark | system` (system reads `prefers-color-scheme`).
- [ ] `richColors`, `invert`, `unstyled` support.
- [ ] `className` and `classNames` support: merge class strings and apply per parts (container, toast, description, action/cancel, icon, etc.).
- [ ] Ensure compatibility with existing design tokens used in `src/components/toast.rs` (`text-foreground`, `border-border`, etc.).

Acceptance:
- [ ] Visual parity modes work; developers can override classnames.

Integration test:
- [ ] Demo toggles for theme, invert, richColors, unstyled.


## Phase 11 — Multiple Toasters and Toaster IDs

Deliverables:
- [ ] Support multiple independent `SonnerToaster` instances via `id`/`toasterId`.
- [ ] Only toasts addressed to a given `toasterId` render in that provider.

Acceptance:
- [ ] Two providers on the same page can render independent queues.

Integration test:
- [ ] Demo with two regions (top-right, bottom-left) using different IDs.


## Phase 12 — Accessibility and Keyboard

Deliverables:
- [ ] Container as focusable region (`role="region"`, aria-label with toast count) similar to existing provider.
- [ ] Toasts use `role="alertdialog"`, `aria-labelledby`, `aria-describedby` as in `src/components/toast.rs`.
- [ ] Provide a keyboard shortcut to focus the region (e.g., F6 or configurable `hotkey`).
- [ ] Tab cycle and screen reader announcements verified.

Acceptance:
- [ ] Region focus works; SR announces new toasts and dismissals.

Integration test:
- [ ] Keyboard and SR testing across platforms.


## Phase 13 — Documentation, Examples, and Migration Guide

Deliverables:
- [ ] README section with usage examples (mount provider, call hooks, options).
- [ ] Example showcase page composing all features toggles for QA.
- [ ] Migration notes from existing `src/components/toast.rs` to `sonner` (or how they coexist).

Acceptance:
- [ ] Docs render correctly and are sufficient for adoption.


## Detailed Task Breakdown (checklist)

- [x] Create `src/components/sonner/` module with `mod.rs` and placeholder files.
- [x] Implement `types.rs` enumerations and structs (see Phase 1).
- [ ] Implement `state.rs` signals and events:
  - [ ] `add_toast`, `update_toast`, `dismiss_toast`, `delete_toast`
  - [ ] `set_interacting(bool)`, `set_expanded(bool)`
  - [ ] heights add/update/remove
- [ ] Implement provider `toaster.rs`:
  - [ ] Context value struct (signals, defaults, callbacks)
  - [ ] Portal and positioned container
  - [ ] Visible list rendering with keys
- [ ] Implement `toast.rs` view with close button and a11y wiring
- [ ] Implement timers with pause/resume
- [ ] Implement `document.hidden` handling
- [ ] Implement measuring and stacking
- [ ] Implement positions and offsets
- [ ] Implement dismissible, close button logic, and onDismiss/onAutoClose
- [x] Implement swipe-to-dismiss
- [x] Implement icons and loader
- [x] Implement promise API
- [ ] Implement theming, classNames, overrides
- [ ] Implement multiple toasters (id)
- [ ] Write documentation and example page


## API Sketch (Rust/Dioxus)

Example usage for app developers:

```rust
// In app root
use crate::components::sonner::toaster::SonnerToaster;

fn App(cx: Scope) -> Element {
    rsx! {
        SonnerToaster { // props shown as example
            id: Some("main".into()),
            position: Some(Position::BottomRight),
            rich_colors: true,
            visible_toasts: 3,
            gap: 14,
            close_button: true,
            // ...
        }
        AppRoutes {}
    }
}
```

```rust
// In any component
use crate::components::sonner::use_sonner;

fn SomeComponent(cx: Scope) -> Element {
    let toasts = use_sonner();

    let on_click = move |_| {
        toasts.success("Saved".to_string(), ToastOptions::default());
    };

    rsx! { button { onclick: on_click, "Notify" } }
}
```

```rust
// Promise-based example (pseudo)
let fut = async move {
    do_work().await
};
toasts.promise(fut, PromiseOptions::new()
    .loading("Loading...")
    .success(|res| format!("Done: {}", res))
    .error(|e| format!("Failed: {}", e))
);
```


## Configuration Defaults (align with Sonner)

- TOAST_LIFETIME: 4000ms (unless overridden)
- VISIBLE_TOASTS_AMOUNT: 3
- VIEWPORT_OFFSET: 24px; MOBILE_VIEWPORT_OFFSET: 16px
- TOAST_WIDTH: ~356px (for layout/drag thresholds)
- GAP: 14px
- SWIPE_THRESHOLD: ~45px


## Styling Guidance

- Reuse the same utility classes already in `src/components/toast.rs` for consistency:
  - Examples: `text-foreground`, `text-muted-foreground`, `border-border`, etc.
- CSS variables for color roles (success/error/warning/info) should mirror existing approach.
- Allow `className` and fine-grained `classNames` overrides, merging user-provided classes.
- Provide an `unstyled` mode where we output minimal markup and no utility classes.


## Accessibility Checklist

- [ ] Region: focusable, labeled with toast count or `containerAriaLabel`.
- [ ] Toast: `role="alertdialog"`, has `aria-labelledby`/`aria-describedby`.
- [ ] Close button has `aria-label` and is reachable via keyboard.
- [ ] Keyboard shortcut (`hotkey`) focuses the region.
- [ ] Live region behavior: ensure AT announces changes without traps.
- [ ] RTL: set `dir` appropriately; verify swipe directions adapt.


## QA Scenarios

- [ ] Burst of 10 toasts: no layout jank; visibleToasts respected; non-visible stacked neatly.
- [ ] Hover and move away: timers pause/resume.
- [ ] Switch browser tabs: timers paused while hidden; resume when visible.
- [ ] Swipe a toast slightly (< threshold): snaps back.
- [ ] Swipe decisively (> threshold): dismisses.
- [ ] Dismiss via close button: fires onDismiss; focus remains manageable.
- [ ] Promise flow success and failure paths.
- [ ] Theming toggles; richColors/invert/unstyled permutations.
- [ ] Multiple toasters on one page, addressed by id.


## Migration Notes (from existing `src/components/toast.rs`)

- The current provider already implements: portal, timer close, basic stacking/offset, a11y wiring, focus region, and close animation delay.
- Migrate consumers gradually:
  1) Keep both providers active side-by-side (different IDs and positions).
  2) Replace calls to `use_toast()` with `use_sonner()` progressively.
  3) Once parity is proven, remove or alias legacy API to new Sonner module.
- Consider offering a thin adapter: implement `Toasts` on top of new Sonner engine to minimize churn.


## Risks and Mitigations

- Swipe gestures across platforms: implement threshold + inertia; allow opt-out via `swipeDirections`.
- Measuring heights in Dioxus: ensure `use_layout_effect` equivalent with `onmounted`/`onupdated` to recompute.
- Timer pause/resume edge cases: centralize pause state; reconcile per-toast timers with global interacting flag.
- Multiple providers: ensure toast IDs don’t collide (use atomic counters + (optional) scope prefix by provider id).


## Rollout Plan

- Land per phase behind feature flags where helpful.
- Provide a standalone demo page and storybook-like examples if available.
- Capture regression screenshots across themes/positions.


## Definition of Done

- All phases completed with acceptance checks met.
- Demo page validates all behaviors.
- Documentation is complete, and migration plan executed or documented.
- Code passes CI and basic linting.


— End of Plan —
