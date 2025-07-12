// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use ag_service_tree_sitter::{Language, LanguageExt};
use thread_utils::FastMap;

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
    pub lang: &'static (dyn Language + LanguageExt),
    pub docs: FastMap<SymbolU32, ag_service_core::StrDoc>,
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
