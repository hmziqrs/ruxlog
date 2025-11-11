//! Keyboard shortcuts module for the rich text editor.
//!
//! Provides a flexible system for registering and handling keyboard shortcuts.

use dioxus::prelude::*;
use std::collections::HashMap;

/// Represents a keyboard shortcut combination.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Shortcut {
    pub fn new(key: String) -> Self {
        Self {
            key,
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }

    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn meta(mut self) -> Self {
        self.meta = true;
        self
    }

    /// Check if the shortcut matches the given keyboard event.
    pub fn matches(&self, evt: &Event<KeyboardData>) -> bool {
        let key_matches = match evt.key() {
            Key::Character(ch) => ch.to_lowercase() == self.key.to_lowercase(),
            _ => format!("{:?}", evt.key()).to_lowercase() == self.key.to_lowercase(),
        };

        let data = evt.data();
        key_matches
            && data.modifiers().ctrl() == self.ctrl
            && data.modifiers().shift() == self.shift
            && data.modifiers().alt() == self.alt
            && data.modifiers().meta() == self.meta
    }
}

/// Action to perform when a shortcut is triggered.
#[derive(Debug, Clone, PartialEq)]
pub enum ShortcutAction {
    ToggleBold,
    ToggleItalic,
    ToggleUnderline,
    ToggleStrike,
    ToggleCode,
    InsertLink,
    SetHeading(u8),
    SetParagraph,
    SetQuote,
    SetCodeBlock,
    InsertBulletList,
    InsertOrderedList,
    InsertTaskList,
    Undo,
    Redo,
    Save,
    Find,
    /// Move current block up
    MoveBlockUp,
    /// Move current block down
    MoveBlockDown,
}

impl ShortcutAction {
    /// Get a human-readable description of the action.
    pub fn description(&self) -> &str {
        match self {
            Self::ToggleBold => "Toggle bold",
            Self::ToggleItalic => "Toggle italic",
            Self::ToggleUnderline => "Toggle underline",
            Self::ToggleStrike => "Toggle strikethrough",
            Self::ToggleCode => "Toggle inline code",
            Self::InsertLink => "Insert link",
            Self::SetHeading(level) => match level {
                1 => "Set heading 1",
                2 => "Set heading 2",
                3 => "Set heading 3",
                4 => "Set heading 4",
                5 => "Set heading 5",
                6 => "Set heading 6",
                _ => "Set heading",
            },
            Self::SetParagraph => "Set paragraph",
            Self::SetQuote => "Set quote",
            Self::SetCodeBlock => "Set code block",
            Self::InsertBulletList => "Insert bullet list",
            Self::InsertOrderedList => "Insert numbered list",
            Self::InsertTaskList => "Insert task list",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Save => "Save",
            Self::Find => "Find",
            Self::MoveBlockUp => "Move block up",
            Self::MoveBlockDown => "Move block down",
        }
    }
}

/// Keyboard shortcuts registry.
pub struct ShortcutRegistry {
    shortcuts: HashMap<Shortcut, ShortcutAction>,
}

impl ShortcutRegistry {
    /// Create a new registry with default shortcuts.
    pub fn with_defaults() -> Self {
        let mut registry = Self {
            shortcuts: HashMap::new(),
        };

        // Text formatting shortcuts
        registry.register(
            Shortcut::new("b".to_string()).ctrl(),
            ShortcutAction::ToggleBold,
        );
        registry.register(
            Shortcut::new("i".to_string()).ctrl(),
            ShortcutAction::ToggleItalic,
        );
        registry.register(
            Shortcut::new("u".to_string()).ctrl(),
            ShortcutAction::ToggleUnderline,
        );
        registry.register(
            Shortcut::new("d".to_string()).ctrl().shift(),
            ShortcutAction::ToggleStrike,
        );
        registry.register(
            Shortcut::new("e".to_string()).ctrl(),
            ShortcutAction::ToggleCode,
        );

        // Link shortcut
        registry.register(
            Shortcut::new("k".to_string()).ctrl(),
            ShortcutAction::InsertLink,
        );

        // Heading shortcuts (Ctrl+Alt+1 through Ctrl+Alt+6)
        for level in 1..=6 {
            registry.register(
                Shortcut::new(level.to_string()).ctrl().alt(),
                ShortcutAction::SetHeading(level),
            );
        }

        // Paragraph shortcut
        registry.register(
            Shortcut::new("0".to_string()).ctrl().alt(),
            ShortcutAction::SetParagraph,
        );

        // Quote shortcut
        registry.register(
            Shortcut::new("q".to_string()).ctrl().shift(),
            ShortcutAction::SetQuote,
        );

        // Code block shortcut
        registry.register(
            Shortcut::new("c".to_string()).ctrl().alt(),
            ShortcutAction::SetCodeBlock,
        );

        // List shortcuts
        registry.register(
            Shortcut::new("l".to_string()).ctrl().shift(),
            ShortcutAction::InsertBulletList,
        );
        registry.register(
            Shortcut::new("o".to_string()).ctrl().shift(),
            ShortcutAction::InsertOrderedList,
        );
        registry.register(
            Shortcut::new("t".to_string()).ctrl().shift(),
            ShortcutAction::InsertTaskList,
        );

        // Undo/Redo
        registry.register(Shortcut::new("z".to_string()).ctrl(), ShortcutAction::Undo);
        registry.register(
            Shortcut::new("z".to_string()).ctrl().shift(),
            ShortcutAction::Redo,
        );
        registry.register(Shortcut::new("y".to_string()).ctrl(), ShortcutAction::Redo);

        // Save
        registry.register(Shortcut::new("s".to_string()).ctrl(), ShortcutAction::Save);

        // Find
        registry.register(Shortcut::new("f".to_string()).ctrl(), ShortcutAction::Find);

        // Block reordering
        registry.register(
            Shortcut::new("ArrowUp".to_string()).alt(),
            ShortcutAction::MoveBlockUp,
        );
        registry.register(
            Shortcut::new("ArrowDown".to_string()).alt(),
            ShortcutAction::MoveBlockDown,
        );

        registry
    }

    /// Register a new shortcut.
    pub fn register(&mut self, shortcut: Shortcut, action: ShortcutAction) {
        self.shortcuts.insert(shortcut, action);
    }

    /// Unregister a shortcut.
    pub fn unregister(&mut self, shortcut: &Shortcut) {
        self.shortcuts.remove(shortcut);
    }

    /// Find the action for a keyboard event.
    pub fn find_action(&self, evt: &Event<KeyboardData>) -> Option<ShortcutAction> {
        self.shortcuts
            .iter()
            .find(|(shortcut, _)| shortcut.matches(evt))
            .map(|(_, action)| action.clone())
    }

    /// Get all registered shortcuts.
    pub fn get_all(&self) -> Vec<(Shortcut, ShortcutAction)> {
        self.shortcuts
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Format a shortcut as a human-readable string.
pub fn format_shortcut(shortcut: &Shortcut) -> String {
    let mut parts = Vec::new();

    if shortcut.ctrl {
        parts.push("Ctrl".to_string());
    }
    if shortcut.alt {
        parts.push("Alt".to_string());
    }
    if shortcut.shift {
        parts.push("Shift".to_string());
    }
    if shortcut.meta {
        parts.push("Cmd".to_string());
    }

    parts.push(shortcut.key.to_uppercase());

    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_creation() {
        let shortcut = Shortcut::new("b".to_string()).ctrl();
        assert_eq!(shortcut.key, "b");
        assert!(shortcut.ctrl);
        assert!(!shortcut.shift);
        assert!(!shortcut.alt);
        assert!(!shortcut.meta);
    }

    #[test]
    fn test_shortcut_chaining() {
        let shortcut = Shortcut::new("z".to_string()).ctrl().shift();
        assert!(shortcut.ctrl);
        assert!(shortcut.shift);
    }

    #[test]
    fn test_format_shortcut() {
        let shortcut = Shortcut::new("b".to_string()).ctrl();
        assert_eq!(format_shortcut(&shortcut), "Ctrl+B");

        let shortcut = Shortcut::new("1".to_string()).ctrl().alt();
        assert_eq!(format_shortcut(&shortcut), "Ctrl+Alt+1");
    }

    #[test]
    fn test_registry_defaults() {
        let registry = ShortcutRegistry::with_defaults();
        assert!(!registry.shortcuts.is_empty());
    }

    #[test]
    fn test_registry_register_unregister() {
        let mut registry = ShortcutRegistry::with_defaults();
        let count_before = registry.shortcuts.len();

        let custom_shortcut = Shortcut::new("x".to_string()).ctrl().alt();
        registry.register(custom_shortcut.clone(), ShortcutAction::ToggleBold);
        assert_eq!(registry.shortcuts.len(), count_before + 1);

        registry.unregister(&custom_shortcut);
        assert_eq!(registry.shortcuts.len(), count_before);
    }
}
