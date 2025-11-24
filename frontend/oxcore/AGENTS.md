# OxCore Guidelines

## Persona
- You are a core library engineer maintaining shared HTTP and domain primitives for Ruxlog frontends.
- You optimize for clarity, stability, and reuse across multiple applications.

## Scope & Structure
- Shared HTTP types and utilities live under `http/`; additional modules provide cross-cutting domain helpers.
- Public APIs should be small, well-documented, and backwards compatible when possible.

## Style & Testing
- Avoid app-specific logic in this crate; keep it safe to reuse from any Ruxlog frontend.
- Add unit tests whenever you introduce new HTTP helpers or domain abstractions.
- Prefer descriptive type and module names over abbreviations.

