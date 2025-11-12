# Migration Guide: UI and Store Refactoring

## Overview

We've refactored the codebase to separate generic reusable components from domain-specific code:

- **`oxui`** - Generic Dioxus UI components (shadcn, radix, custom components, forms, etc.)
- **`ruxlog-shared`** - Domain-specific code (store + components that depend on store types)

## Package Structure

```
frontend/
├── oxcore/              # Cross-platform utilities (HTTP, etc.)
├── oxstore/             # Generic state management framework
├── oxform/              # Generic form validation framework
├── oxui/                # NEW: Generic Dioxus UI components
│   ├── shadcn/          # ShadcnUI components
│   ├── radix/           # Radix primitives
│   ├── custom/          # Custom generic components
│   └── components/      # Generic reusable components
│       ├── animated_grid/
│       ├── confirm_dialog.rs
│       ├── error/
│       ├── form/
│       ├── loading_overlay.rs
│       └── portal_v2.rs
├── ruxlog-shared/       # NEW: Domain-specific shared code
│   ├── store/           # Business logic stores
│   │   ├── analytics/
│   │   ├── auth/
│   │   ├── categories/
│   │   ├── image_editor/
│   │   ├── media/
│   │   ├── posts/
│   │   ├── tags/
│   │   └── users/
│   └── components/      # Domain-specific components
│       ├── tag.rs       # Uses store::Tag
│       └── user_avatar.rs  # Uses store::Media
└── admin-dioxus/        # Admin application
    ├── components/      # App-specific components
    ├── screens/         # App screens/pages
    └── ...
```

## Import Changes Required

### Before (Old imports)
```rust
use crate::ui::shadcn::button::{Button, ButtonVariant};
use crate::ui::components::loading_overlay::LoadingOverlay;
use crate::ui::custom::portal::AppPortal;
use crate::store::{use_auth, AuthUser};
use crate::store::media::{Media, use_media};
```

### After (New imports)
```rust
use oxui::shadcn::button::{Button, ButtonVariant};
use oxui::components::loading_overlay::LoadingOverlay;
use oxui::custom::portal::AppPortal;
use ruxlog_shared::{use_auth, AuthUser};
use ruxlog_shared::media::{Media, use_media};
```

## Quick Reference Table

| Old Import Pattern | New Import Pattern |
|-------------------|-------------------|
| `crate::ui::shadcn::*` | `oxui::shadcn::*` |
| `crate::ui::radix::*` | `oxui::radix::*` |
| `crate::ui::custom::*` | `oxui::custom::*` |
| `crate::ui::components::animated_grid::*` | `oxui::components::animated_grid::*` |
| `crate::ui::components::confirm_dialog::*` | `oxui::components::confirm_dialog::*` |
| `crate::ui::components::error::*` | `oxui::components::error::*` |
| `crate::ui::components::form::*` | `oxui::components::form::*` |
| `crate::ui::components::loading_overlay::*` | `oxui::components::loading_overlay::*` |
| `crate::ui::components::portal_v2::*` | `oxui::components::portal_v2::*` |
| `crate::ui::components::tag::*` | `ruxlog_shared::components::tag::*` |
| `crate::ui::components::user_avatar::*` | `ruxlog_shared::components::user_avatar::*` |
| `crate::store::*` | `ruxlog_shared::*` or `ruxlog_shared::store::*` |

## Domain-Specific Components

These components depend on store types and are now in `ruxlog-shared`:

- **`TagBadge`** - Uses `store::Tag`
- **`UserAvatar`** - Uses `store::Media`

## Automated Migration Script

Run this command from the `frontend/admin-dioxus` directory to update all imports:

```bash
# Update ui imports to oxui
find src -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::ui::shadcn::/use oxui::shadcn::/g' \
  -e 's/use crate::ui::radix::/use oxui::radix::/g' \
  -e 's/use crate::ui::custom::/use oxui::custom::/g' \
  -e 's/use crate::ui::components::animated_grid/use oxui::components::animated_grid/g' \
  -e 's/use crate::ui::components::confirm_dialog/use oxui::components::confirm_dialog/g' \
  -e 's/use crate::ui::components::error/use oxui::components::error/g' \
  -e 's/use crate::ui::components::form/use oxui::components::form/g' \
  -e 's/use crate::ui::components::loading_overlay/use oxui::components::loading_overlay/g' \
  -e 's/use crate::ui::components::portal_v2/use oxui::components::portal_v2/g' \
  {} +

# Update domain-specific components
find src -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::ui::components::tag/use ruxlog_shared::components::tag/g' \
  -e 's/use crate::ui::components::user_avatar/use ruxlog_shared::components::user_avatar/g' \
  {} +

# Update store imports
find src -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::store::/use ruxlog_shared::store::/g' \
  -e 's/use crate::store;/use ruxlog_shared::store;/g' \
  {} +
```

For Linux, remove the `''` after `-i`:
```bash
sed -i -e 's/pattern/replacement/g' {} +
```

## Manual Steps

1. **Update `Cargo.toml`** ✅ Already done
   ```toml
   [dependencies]
   oxui = { path = "../oxui" }
   ruxlog-shared = { path = "../ruxlog-shared" }
   ```

2. **Remove old module declarations** ✅ Already done
   - Removed `pub mod store;` from `main.rs`
   - Removed `pub mod ui;` from `main.rs`

3. **Run the migration script** (see above)

4. **Fix any remaining issues**
   ```bash
   cargo check
   ```

5. **Update any qualified paths**
   Some files might use fully qualified paths like:
   ```rust
   crate::store::auth::use_auth()
   ```
   Should become:
   ```rust
   ruxlog_shared::auth::use_auth()
   // or
   use ruxlog_shared::auth::use_auth;
   use_auth()
   ```

## Common Issues

### Issue: "unresolved import"
**Solution**: Check if you're importing from the correct package:
- Generic UI components → `oxui`
- Store/business logic → `ruxlog_shared`
- App-specific code → `crate::*`

### Issue: "no `ui` in the root"
**Solution**: Replace `crate::ui::` with `oxui::`

### Issue: "no `store` in the root"
**Solution**: Replace `crate::store::` with `ruxlog_shared::store::` or `ruxlog_shared::`

### Issue: Module not found in oxui
**Solution**: Check if it's a domain-specific component (TagBadge, UserAvatar) - these are in `ruxlog_shared::components`

## Benefits

1. **Reusability**: Both `oxui` and `ruxlog-shared` can be used in future projects
2. **Clear separation**: Generic vs domain-specific code is clearly separated
3. **Maintainability**: Changes to UI components don't affect business logic
4. **Modularity**: Each package has a single, clear responsibility
5. **Future-proof**: Easy to add new frontends (mobile, etc.) that share the same store and components

## Testing

After migration, ensure:
1. `cargo check` passes in all packages
2. `cargo build` succeeds
3. All features work as expected in the running application
4. No import errors in your IDE

## Rollback

If you need to rollback:
1. `git checkout main` (or your branch before migration)
2. Delete the `oxui` and `ruxlog-shared` directories
3. Restore the original `Cargo.toml` and `main.rs`
