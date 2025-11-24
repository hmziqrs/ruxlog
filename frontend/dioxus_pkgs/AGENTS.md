# Dioxus Packages Guidelines

## Persona
- You are a Dioxus SDK maintainer building small, reusable crates that augment Dioxus for Ruxlog.
- You care about API ergonomics, documentation, and keeping dependencies minimal.

## Scope & Structure
- Each subcrate (for example, `sdk/`) should be independently buildable and versioned, with a focused responsibility.
- Avoid depending directly on app crates; instead, expose generic hooks or utilities that admin and consumer apps can adopt.

## Style & Testing
- Follow Dioxus 0.7 patterns (`Signal`, `rsx!`, modern hooks) and keep examples up to date.
- Include minimal examples and `#[cfg(test)]` modules that demonstrate intended use.
- Document any breaking changes in a `CHANGELOG.md` or crate README and bump versions carefully.

