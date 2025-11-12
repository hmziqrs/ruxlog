#!/usr/bin/env python3
"""
Migration script to update imports from crate::ui and crate::store
to oxui and ruxlog-shared respectively.
"""

import os
import re
from pathlib import Path

# Mapping of old import patterns to new ones
UI_REPLACEMENTS = [
    # Shadcn components
    (r"use crate::ui::shadcn::", r"use oxui::shadcn::"),
    # Radix components
    (r"use crate::ui::radix::", r"use oxui::radix::"),
    # Custom components
    (r"use crate::ui::custom::", r"use oxui::custom::"),
    # Generic components (moved to oxui)
    (
        r"use crate::ui::components::animated_grid",
        r"use oxui::components::animated_grid",
    ),
    (
        r"use crate::ui::components::confirm_dialog",
        r"use oxui::components::confirm_dialog",
    ),
    (r"use crate::ui::components::error", r"use oxui::components::error"),
    (r"use crate::ui::components::form", r"use oxui::components::form"),
    (
        r"use crate::ui::components::loading_overlay",
        r"use oxui::components::loading_overlay",
    ),
    (r"use crate::ui::components::portal_v2", r"use oxui::components::portal_v2"),
]

# Domain-specific components (moved to ruxlog-shared)
DOMAIN_REPLACEMENTS = [
    (r"use crate::ui::components::tag", r"use ruxlog_shared::components::tag"),
    (
        r"use crate::ui::components::user_avatar",
        r"use ruxlog_shared::components::user_avatar",
    ),
]

# Store imports
STORE_REPLACEMENTS = [
    (r"use crate::store::", r"use ruxlog_shared::store::"),
    (r"from crate::store", r"from ruxlog_shared::store"),
]


def process_file(file_path):
    """Process a single Rust file and update imports."""
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()

        original_content = content
        changes_made = []

        # Apply UI replacements
        for old_pattern, new_pattern in UI_REPLACEMENTS:
            if re.search(old_pattern, content):
                content = re.sub(old_pattern, new_pattern, content)
                changes_made.append(f"  {old_pattern} -> {new_pattern}")

        # Apply domain-specific replacements
        for old_pattern, new_pattern in DOMAIN_REPLACEMENTS:
            if re.search(old_pattern, content):
                content = re.sub(old_pattern, new_pattern, content)
                changes_made.append(f"  {old_pattern} -> {new_pattern}")

        # Apply store replacements
        for old_pattern, new_pattern in STORE_REPLACEMENTS:
            if re.search(old_pattern, content):
                content = re.sub(old_pattern, new_pattern, content)
                changes_made.append(f"  {old_pattern} -> {new_pattern}")

        # Write back if changes were made
        if content != original_content:
            with open(file_path, "w", encoding="utf-8") as f:
                f.write(content)
            print(f"✓ Updated: {file_path}")
            for change in changes_made:
                print(change)
            return True

        return False

    except Exception as e:
        print(f"✗ Error processing {file_path}: {e}")
        return False


def find_rust_files(root_dir):
    """Find all Rust files in the directory tree."""
    rust_files = []
    for root, dirs, files in os.walk(root_dir):
        # Skip hidden directories and target directories
        dirs[:] = [d for d in dirs if not d.startswith(".") and d != "target"]

        for file in files:
            if file.endswith(".rs") and not file.startswith("."):
                rust_files.append(os.path.join(root, file))

    return rust_files


def main():
    """Main migration function."""
    # Get the admin-dioxus src directory
    script_dir = Path(__file__).parent
    src_dir = script_dir / "admin-dioxus" / "src"

    if not src_dir.exists():
        print(f"Error: Source directory not found: {src_dir}")
        return

    print("=" * 60)
    print("Starting Import Migration")
    print("=" * 60)
    print(f"Scanning: {src_dir}")
    print()

    # Find all Rust files
    rust_files = find_rust_files(src_dir)
    print(f"Found {len(rust_files)} Rust files")
    print()

    # Process each file
    updated_count = 0
    for file_path in rust_files:
        if process_file(file_path):
            updated_count += 1
            print()

    print("=" * 60)
    print(f"Migration Complete!")
    print(f"Updated {updated_count} out of {len(rust_files)} files")
    print("=" * 60)
    print()
    print("Next steps:")
    print("1. Delete old directories:")
    print("   rm -rf frontend/admin-dioxus/src/ui")
    print("   rm -rf frontend/admin-dioxus/src/store")
    print()
    print("2. Run cargo check:")
    print("   cd frontend/admin-dioxus && cargo check")
    print()


if __name__ == "__main__":
    main()
