//! Core data models, formatting utilities, and import/export logic for KeyStats.

#![warn(missing_docs)]

/// Compact number and distance formatting with K/M suffixes.
pub mod format;
/// JSON export/import for stats backup and transfer.
pub mod import_export;
/// Data structures for daily statistics, rates, permissions, and settings.
pub mod model;

pub use import_export::*;
pub use model::*;
