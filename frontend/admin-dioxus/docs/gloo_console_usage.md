# Dioxus Logger Usage Documentation

## Overview
This document tracks all dioxus logger usage throughout the codebase for debugging and browser console logging. Previously used gloo-console, now migrated to dioxus::logger::tracing.

## Dependency
- **Dioxus**: `dioxus = { version = "0.7.1" }` (includes logger)

## Import Statements

### Full Module Imports
- `src/store/image_editor/actions.rs:5` - `use dioxus::logger::tracing;`
- `src/store/media/actions.rs:5` - `use dioxus::logger::tracing;`

### Specific Item Import
- Components: `use dioxus::{logger::tracing, prelude::*};`

## Usage by Category

### Store Actions

#### Image Editor Actions - `src/store/image_editor/actions.rs`
- **Usage**: Editor lifecycle, operations (crop/resize/rotate/compress), error handling
- **Types**: `tracing::debug!`, `tracing::warn!`, `tracing::error!`

#### Media Actions - `src/store/media/actions.rs`
- **Usage**: File upload process, form data preparation, HTTP requests, response handling
- **Types**: `tracing::debug!`, `tracing::error!`

### Components
- **Usage**: Form processing, file handling, UI interactions
- **Types**: `tracing::debug!`, `tracing::warn!`, `tracing::error!`

## Usage Patterns

### Debug Logging
```rust
tracing::debug!("[Component::action] Context: {}", data);
```
- Used for tracking component lifecycle
- State changes and operation progress
- Data flow debugging

### Error Handling
```rust
tracing::error!("[Component::action] Error: {}", &error);
```
- Exception and error reporting
- Failed operations logging
- Debug context for failures

### Warning Messages
```rust
tracing::warn!("[Component::action] Warning: {}", context);
```
- Non-critical issues
- Expected error conditions
- Performance warnings

## Integration Notes

### Browser Console Output
- All dioxus logger calls output to browser dev tools console via tracing
- Structured logging format with component/action context
- WASM-compiled Rust values converted to JavaScript

### Performance Considerations
- Logging in media upload and image editor operations
- Consider log levels for production builds

## Migration History
- Migrated from gloo-console to dioxus::logger::tracing
- Better integrated with Dioxus ecosystem
- Provides consistent logging across components and shared code

## Total Summary
- **Files with usage**: Components and shared stores
- **Import statements**: `use dioxus::{logger::tracing, prelude::*};` or `use dioxus::logger::tracing;`
- **Total macro calls**: ~100+
- **Primary categories**: Store actions, UI components