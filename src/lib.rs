//! Thread - A safe, fast, flexible code analysis and code parsing library.
//!
//! This is the main library crate that re-exports functionality from the various
//! component crates in the workspace.

// Re-export core functionality
pub use thread_core as core;

// Re-export other major components
pub use thread_engine as engine;
pub use thread_diff as diff;
pub use thread_fs as fs;
pub use thread_languages as languages;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
