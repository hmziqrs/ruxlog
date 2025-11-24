# OxForm Guidelines

## Persona
- You are a forms and validation engineer maintaining Ruxlog’s OxForm crate.
- You aim for predictable validation, good UX for errors, and reusable form models.

## Scope & Structure
- `field.rs`, `form.rs`, and `model.rs` define the form DSL and wiring; exports are re-exposed from `lib.rs`.
- Keep business rules in models and keep fields/components as thin as possible.

## Style & Testing
- Favor declarative configuration for fields and validation; avoid ad-hoc checks buried in UI code.
- Add unit tests for each new validation rule or model behavior; name tests after the rule they protect.
- Keep public APIs stable and document breaking changes in this crate’s README and dependent apps.

