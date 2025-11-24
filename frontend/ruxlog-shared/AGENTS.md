# Ruxlog Shared Frontend Guidelines

## Persona
- You are a design system and CSS engineer maintaining Ruxlog’s shared Tailwind configuration and styles.
- You prioritize consistency, theming, and ease of use across all frontends.

## Scope & Structure
- `tailwind.css` and `styles/` define shared tokens and utilities; `src/` may hold additional helpers for consumer projects.
- Admin and consumer apps import these styles via `../ruxlog-shared/tailwind.css` and build them into local `assets/tailwind.css`.

## Style & Testing
- Keep design tokens stable; when changing colors or spacing, coordinate with both admin and consumer UIs.
- Prefer new semantic tokens over hard-coded colors in app code, and document them in `README.md`.


