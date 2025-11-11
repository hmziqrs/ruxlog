//! History module for transaction-based undo/redo.
//!
//! This module provides a robust undo/redo system with transaction coalescing,
//! allowing users to undo/redo their edits while maintaining a clean history.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Maximum number of history entries to keep
const MAX_HISTORY_SIZE: usize = 100;

/// Time window for coalescing consecutive typing operations (milliseconds)
const COALESCE_WINDOW_MS: u64 = 1000;

/// Represents a single transaction in the edit history.
#[derive(Debug, Clone)]
pub struct Transaction {
    /// The HTML content before the transaction
    pub before: String,

    /// The HTML content after the transaction
    pub after: String,

    /// Timestamp when the transaction was created
    pub timestamp: Instant,

    /// Type of transaction for coalescing decisions
    pub transaction_type: TransactionType,
}

/// Types of transactions for intelligent coalescing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionType {
    /// Continuous typing (can be coalesced)
    Typing,

    /// Formatting change (bold, italic, etc.)
    Formatting,

    /// Block-level change (heading, list, etc.)
    BlockChange,

    /// Paste operation
    Paste,

    /// Delete/backspace operation
    Delete,

    /// Other operations (explicit, not coalesced)
    Other,
}

/// History manager for undo/redo operations.
pub struct History {
    /// Stack of undo transactions (past)
    undo_stack: VecDeque<Transaction>,

    /// Stack of redo transactions (future)
    redo_stack: VecDeque<Transaction>,

    /// Current document state (HTML)
    current_state: String,

    /// Last transaction timestamp for coalescing
    last_transaction_time: Option<Instant>,

    /// Type of the last transaction
    last_transaction_type: Option<TransactionType>,
}

impl History {
    /// Creates a new history manager with initial state.
    pub fn new(initial_state: String) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            current_state: initial_state,
            last_transaction_time: None,
            last_transaction_type: None,
        }
    }

    /// Pushes a new transaction to the history.
    /// Returns true if a new transaction was added, false if it was coalesced.
    pub fn push(&mut self, new_state: String, transaction_type: TransactionType) -> bool {
        let now = Instant::now();

        // Check if we can coalesce with the last transaction
        if self.can_coalesce(&transaction_type, now) {
            // Coalesce: update the last transaction's "after" state
            if let Some(last_transaction) = self.undo_stack.back_mut() {
                last_transaction.after = new_state.clone();
                last_transaction.timestamp = now;
                self.current_state = new_state;
                self.last_transaction_time = Some(now);
                self.last_transaction_type = Some(transaction_type);
                return false;
            }
        }

        // Create a new transaction
        let transaction = Transaction {
            before: self.current_state.clone(),
            after: new_state.clone(),
            timestamp: now,
            transaction_type: transaction_type.clone(),
        };

        // Add to undo stack
        self.undo_stack.push_back(transaction);

        // Limit history size
        if self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.pop_front();
        }

        // Clear redo stack (new edits invalidate future)
        self.redo_stack.clear();

        // Update current state
        self.current_state = new_state;
        self.last_transaction_time = Some(now);
        self.last_transaction_type = Some(transaction_type);

        true
    }

    /// Checks if the new transaction can be coalesced with the last one.
    fn can_coalesce(&self, transaction_type: &TransactionType, now: Instant) -> bool {
        // Only coalesce typing and delete operations
        if !matches!(transaction_type, TransactionType::Typing | TransactionType::Delete) {
            return false;
        }

        // Check if we have a recent transaction of the same type
        if let (Some(last_time), Some(last_type)) = (&self.last_transaction_time, &self.last_transaction_type) {
            let elapsed = now.duration_since(*last_time);
            let within_window = elapsed < Duration::from_millis(COALESCE_WINDOW_MS);
            let same_type = last_type == transaction_type;

            return within_window && same_type && !self.undo_stack.is_empty();
        }

        false
    }

    /// Performs an undo operation.
    /// Returns the state to restore, or None if nothing to undo.
    pub fn undo(&mut self) -> Option<String> {
        if let Some(transaction) = self.undo_stack.pop_back() {
            // Move to redo stack
            self.redo_stack.push_back(transaction.clone());

            // Limit redo stack size
            if self.redo_stack.len() > MAX_HISTORY_SIZE {
                self.redo_stack.pop_front();
            }

            // Restore the "before" state
            self.current_state = transaction.before.clone();
            self.last_transaction_time = None;
            self.last_transaction_type = None;

            Some(transaction.before)
        } else {
            None
        }
    }

    /// Performs a redo operation.
    /// Returns the state to restore, or None if nothing to redo.
    pub fn redo(&mut self) -> Option<String> {
        if let Some(transaction) = self.redo_stack.pop_back() {
            // Move back to undo stack
            self.undo_stack.push_back(transaction.clone());

            // Restore the "after" state
            self.current_state = transaction.after.clone();
            self.last_transaction_time = Some(transaction.timestamp);
            self.last_transaction_type = Some(transaction.transaction_type);

            Some(transaction.after)
        } else {
            None
        }
    }

    /// Returns true if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns true if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Gets the current state.
    pub fn current_state(&self) -> &str {
        &self.current_state
    }

    /// Clears all history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.last_transaction_time = None;
        self.last_transaction_type = None;
    }

    /// Returns the number of undo operations available.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Returns the number of redo operations available.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_undo_redo() {
        let mut history = History::new("initial".to_string());

        // Make some changes - typing operations will coalesce
        history.push("change1".to_string(), TransactionType::Typing);
        history.push("change2".to_string(), TransactionType::Typing);
        // Formatting is a different type, so it won't coalesce
        history.push("change3".to_string(), TransactionType::Formatting);

        // Should have 2 transactions: 1 coalesced typing, 1 formatting
        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.can_undo(), true);
        assert_eq!(history.can_redo(), false);

        // Undo formatting
        let state = history.undo();
        assert_eq!(state, Some("change2".to_string()));
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 1);

        // Redo formatting
        let state = history.redo();
        assert_eq!(state, Some("change3".to_string()));
        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_coalescing() {
        let mut history = History::new("".to_string());

        // Rapid typing should coalesce
        history.push("a".to_string(), TransactionType::Typing);
        history.push("ab".to_string(), TransactionType::Typing);
        history.push("abc".to_string(), TransactionType::Typing);

        // Should only have 1 transaction (coalesced)
        assert_eq!(history.undo_count(), 1);

        // Undo should restore to initial state
        let state = history.undo();
        assert_eq!(state, Some("".to_string()));
    }

    #[test]
    fn test_new_edit_clears_redo() {
        let mut history = History::new("initial".to_string());

        history.push("change1".to_string(), TransactionType::Typing);
        history.push("change2".to_string(), TransactionType::Formatting);

        // Undo once
        history.undo();
        assert_eq!(history.redo_count(), 1);

        // Make a new edit - should clear redo stack
        history.push("change3".to_string(), TransactionType::Typing);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_max_history_size() {
        let mut history = History::new("initial".to_string());

        // Add more than MAX_HISTORY_SIZE transactions
        for i in 0..MAX_HISTORY_SIZE + 10 {
            history.push(format!("change{}", i), TransactionType::Other);
        }

        // Should not exceed max size
        assert!(history.undo_count() <= MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_formatting_not_coalesced() {
        let mut history = History::new("".to_string());

        history.push("change1".to_string(), TransactionType::Formatting);
        history.push("change2".to_string(), TransactionType::Formatting);

        // Formatting operations should not coalesce
        assert_eq!(history.undo_count(), 2);
    }
}
