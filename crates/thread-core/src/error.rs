// thread-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThreadError {
    #[error("Parser error: {0}")]
    ParseError(String),

    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    #[error("File too large: {size_mb}MB exceeds limit of {limit_mb}MB")]
    FileTooLarge { size_mb: usize, limit_mb: usize },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ThreadError>;

// thread-core/src/hash.rs
use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// Content-addressable hash for deduplication
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(String);

impl ContentHash {
    /// Create hash from content string
    pub fn from_content(content: &str) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(content.as_bytes());
        Self(hasher.finalize().to_hex().to_string())
    }

    /// Create hash from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(bytes);
        Self(hasher.finalize().to_hex().to_string())
    }

    /// Get the hash as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
