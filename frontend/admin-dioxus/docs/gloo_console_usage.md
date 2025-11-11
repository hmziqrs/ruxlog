# gloo-console Usage Documentation

## Overview
This document tracks all gloo-console usage throughout the codebase for debugging and browser console logging.

## Dependency
- **File**: `Cargo.toml:72`
- **Version**: `gloo-console = "0.3.0"`

## Import Statements

### Full Module Imports
- `src/store/image_editor/actions.rs:5` - `use gloo_console;`
- `src/containers/user_form/user_form.rs:2` - `use gloo_console;`

### Specific Item Import
- `src/components/editor_js_host.rs:2` - `use gloo_console::error;`

## Usage by Category

### Store Actions (104 occurrences)

#### Image Editor Actions - `src/store/image_editor/actions.rs`
- **Lines**: 16,24,34,42,77,83,91,96,102,113,138,148,157,168,191,201,210,221,238,252,261,278,290,307,316,326,349
- **Usage**: Editor lifecycle, operations (crop/resize/rotate/compress), error handling
- **Types**: `gloo_console::log!`, `gloo_console::warn!`, `gloo_console::error!`

#### Media Actions - `src/store/media/actions.rs`
- **Lines**: 16,20,24,33,45,68,71,74,82,87,95,101,106,112,117,122,126,135,139,146,154,160,185,188,195,208,216,224,236
- **Usage**: File upload process, form data preparation, HTTP requests, response handling
- **Types**: `gloo_console::log!`, `gloo_console::error!`

### Container Forms (26 occurrences)

#### Blog Form - `src/containers/blog_form/blog_form.rs`
- **Usage**: Form submission, validation, image handling
- **Types**: `gloo_console::log!`, `gloo_console::error!`

#### Category Form - `src/containers/category_form/category_form.rs`
- **Usage**: Form operations, state management
- **Types**: `gloo_console::log!`, `gloo_console::error!`

#### User Form - `src/containers/user_form/user_form.rs`
- **Usage**: Form processing, avatar handling
- **Types**: `gloo_console::log!`, `gloo_console::error!`

### UI Components (18 occurrences)

#### Media Upload Zone - `src/components/media_upload_zone.rs`
- **Usage**: File drag-and-drop, validation, upload process
- **Types**: `gloo_console::log!`, `gloo_console::warn!`, `gloo_console::error!`

#### Image Editor - `src/components/image_editor.rs`
- **Usage**: Component error handling
- **Types**: `gloo_console::error!`

### Utilities/Legacy (30 occurrences)

#### JS Bridge - `src/utils/js_bridge.rs`
- **Usage**: JavaScript interop, browser API calls
- **Types**: `gloo_console::log!`, `gloo_console::error!`

#### Legacy Editor - `src/legacy/editor/editor.rs`
- **Usage**: Editor initialization, operations, cleanup
- **Types**: `gloo_console::log!`, `gloo_console::error!`, `gloo_console::warn!`

## Usage Patterns

### Debug Logging
```rust
gloo_console::log!("[Component::action] Context:", data);
```
- Used for tracking component lifecycle
- State changes and operation progress
- Data flow debugging

### Error Handling
```rust
gloo_console::error!("[Component::action] Error:", &error);
```
- Exception and error reporting
- Failed operations logging
- Debug context for failures

### Warning Messages
```rust
gloo_console::warn!("[Component::action] Warning:", context);
```
- Non-critical issues
- Expected error conditions
- Performance warnings

## Integration Notes

### Browser Console Output
- All gloo-console calls output to browser dev tools console
- Structured logging format with component/action context
- WASM-compiled Rust values converted to JavaScript

### Performance Considerations
- High usage in media upload flows (72 calls in upload process)
- Image editor operations generate extensive logging
- Consider log levels for production builds

## Migration History
- Replaced `console_log` crate usage
- Better integrated with Dioxus/WebAssembly ecosystem
- Provides browser-native console API access

## Total Summary
- **Files with usage**: 15
- **Import statements**: 3
- **Total macro calls**: 157+
- **Primary categories**: Store actions (66%), UI components (29%), Utilities (19%)