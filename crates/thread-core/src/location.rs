

// knitli-core/src/location.rs
use serde::{Deserialize, Serialize};

/// Source location with line and column information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl SourceLocation {
    pub fn new(
        file_path: String,
        start_line: usize,
        end_line: usize,
        start_column: usize,
        end_column: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            file_path,
            start_line,
            end_line,
            start_column,
            end_column,
            start_byte,
            end_byte,
        }
    }

    /// Create from tree-sitter node
    pub fn from_node(node: &tree_sitter::Node, file_path: String, source: &str) -> Self {
        let start_point = node.start_position();
        let end_point = node.end_position();

        Self::new(
            file_path,
            start_point.row + 1, // Convert to 1-based
            end_point.row + 1,
            start_point.column + 1,
            end_point.column + 1,
            node.start_byte(),
            node.end_byte(),
        )
    }

    /// Check if this location contains another location
    pub fn contains(&self, other: &SourceLocation) -> bool {
        self.file_path == other.file_path
            && self.start_byte <= other.start_byte
            && self.end_byte >= other.end_byte
    }

    /// Get the line range as a string (e.g., "42-67")
    pub fn line_range(&self) -> String {
        if self.start_line == self.end_line {
            self.start_line.to_string()
        } else {
            format!("{}-{}", self.start_line, self.end_line)
        }
    }
}
