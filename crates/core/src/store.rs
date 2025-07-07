use string_interner::{StringInterner, SymbolU32};
use crate::fastmap::FastMap;

pub enum FileType {
    /// A file that is a source code file.
    Source,
    /// A file that is a test file.
    Test,
    /// A file that is a configuration file.
    Config,
    /// A file that is a documentation file.
    Documentation,
    /// A file that is a data file.
    Data,
}

pub struct File {
    pub path: std::path::Path,
    pub file_type: FileType,
    pub lang: &(impl crate::ast_grep::language::Language + crate::ast_grep::language::LanguageExt + 'static),
    pub docs: FastMap<SymbolU32, crate::ast_grep::tree_sitter::StrDoc>,
    pub content: Option<ropey::Rope>,
}


/// A simple key-value store using a fast map.
pub struct Store {
    map: FastMap<SymbolU32, String>,
}

pub trait StoreExt {
    /// Get a value from the store by its key.
    fn get(&self, key: &str) -> Option<&String>;

    /// Insert a key-value pair into the store.
    fn insert(&mut self, key: &str, value: String) -> Option<String>;
}
