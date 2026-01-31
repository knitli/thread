// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the Rust dependency extractor.
//!
//! Validates tree-sitter-based extraction of `use` declarations and `pub use`
//! re-exports from Rust source files. Tests follow TDD methodology per
//! Constitutional Principle III.
//!
//! Coverage targets (15+ tests):
//! - Simple imports: `use std::collections::HashMap;`
//! - Nested imports: `use std::collections::{HashMap, HashSet};`
//! - Glob/wildcard imports: `use module::*;`
//! - Aliased imports: `use std::io::Result as IoResult;`
//! - Crate-relative: `use crate::core::Engine;`
//! - Super-relative: `use super::utils;`
//! - Self-relative: `use self::types::Config;`
//! - Multiple imports in one file
//! - Deeply nested path: `use a::b::c::d::E;`
//! - Nested with alias: `use std::collections::{HashMap as Map, HashSet};`
//! - pub use re-exports
//! - pub(crate) use
//! - pub use wildcard
//! - pub use nested
//! - Module path resolution
//! - Edge cases: empty source, no imports

use std::path::Path;
use thread_flow::incremental::extractors::rust::{RustDependencyExtractor, Visibility};

// =============================================================================
// Import Extraction Tests
// =============================================================================

#[test]
fn test_simple_import() {
    let extractor = RustDependencyExtractor::new();
    let source = "use std::collections::HashMap;";
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "std::collections");
    assert_eq!(imports[0].symbols, vec!["HashMap"]);
    assert!(!imports[0].is_wildcard);
}

#[test]
fn test_nested_import() {
    let extractor = RustDependencyExtractor::new();
    let source = "use std::collections::{HashMap, HashSet};";
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "std::collections");
    assert!(imports[0].symbols.contains(&"HashMap".to_string()));
    assert!(imports[0].symbols.contains(&"HashSet".to_string()));
    assert_eq!(imports[0].symbols.len(), 2);
    assert!(!imports[0].is_wildcard);
}

#[test]
fn test_wildcard_import() {
    let extractor = RustDependencyExtractor::new();
    let source = "use std::collections::*;";
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "std::collections");
    assert!(imports[0].is_wildcard);
    assert!(imports[0].symbols.is_empty());
}

#[test]
fn test_aliased_import() {
    let extractor = RustDependencyExtractor::new();
    let source = "use std::io::Result as IoResult;";
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "std::io");
    assert_eq!(imports[0].symbols, vec!["Result"]);
    assert_eq!(
        imports[0].aliases,
        vec![("Result".to_string(), "IoResult".to_string())]
    );
}

#[test]
fn test_crate_relative_import() {
    let extractor = RustDependencyExtractor::new();
    let source = "use crate::core::Engine;";
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "crate::core");
    assert_eq!(imports[0].symbols, vec!["Engine"]);
}

#[test]
fn test_super_import() {
    let extractor = RustDependencyExtractor::new();
    let source = "use super::utils;";
    let imports = extractor
        .extract_imports(source, Path::new("src/sub/mod.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "super");
    assert_eq!(imports[0].symbols, vec!["utils"]);
}

#[test]
fn test_self_import() {
    let extractor = RustDependencyExtractor::new();
    let source = "use self::types::Config;";
    let imports = extractor
        .extract_imports(source, Path::new("src/lib.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "self::types");
    assert_eq!(imports[0].symbols, vec!["Config"]);
}

#[test]
fn test_multiple_imports() {
    let extractor = RustDependencyExtractor::new();
    let source = r#"
use std::collections::HashMap;
use std::io::Read;
use crate::config::Settings;
"#;
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();

    assert_eq!(imports.len(), 3);
    assert_eq!(imports[0].module_path, "std::collections");
    assert_eq!(imports[0].symbols, vec!["HashMap"]);
    assert_eq!(imports[1].module_path, "std::io");
    assert_eq!(imports[1].symbols, vec!["Read"]);
    assert_eq!(imports[2].module_path, "crate::config");
    assert_eq!(imports[2].symbols, vec!["Settings"]);
}

#[test]
fn test_deeply_nested_import() {
    let extractor = RustDependencyExtractor::new();
    let source = "use a::b::c::d::E;";
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "a::b::c::d");
    assert_eq!(imports[0].symbols, vec!["E"]);
}

#[test]
fn test_nested_with_alias() {
    let extractor = RustDependencyExtractor::new();
    let source = "use std::collections::{HashMap as Map, HashSet};";
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "std::collections");
    assert!(imports[0].symbols.contains(&"HashMap".to_string()));
    assert!(imports[0].symbols.contains(&"HashSet".to_string()));
    assert_eq!(
        imports[0].aliases,
        vec![("HashMap".to_string(), "Map".to_string())]
    );
}

// =============================================================================
// Export (pub use) Extraction Tests
// =============================================================================

#[test]
fn test_pub_use_reexport() {
    let extractor = RustDependencyExtractor::new();
    let source = "pub use types::Config;";
    let exports = extractor
        .extract_exports(source, Path::new("src/lib.rs"))
        .unwrap();

    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].symbol_name, "Config");
    assert_eq!(exports[0].module_path, "types");
    assert_eq!(exports[0].visibility, Visibility::Public);
}

#[test]
fn test_pub_crate_use() {
    let extractor = RustDependencyExtractor::new();
    let source = "pub(crate) use internal::Helper;";
    let exports = extractor
        .extract_exports(source, Path::new("src/lib.rs"))
        .unwrap();

    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].symbol_name, "Helper");
    assert_eq!(exports[0].module_path, "internal");
    assert_eq!(exports[0].visibility, Visibility::Crate);
}

#[test]
fn test_pub_use_wildcard() {
    let extractor = RustDependencyExtractor::new();
    let source = "pub use module::*;";
    let exports = extractor
        .extract_exports(source, Path::new("src/lib.rs"))
        .unwrap();

    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].symbol_name, "*");
    assert_eq!(exports[0].module_path, "module");
    assert_eq!(exports[0].visibility, Visibility::Public);
}

#[test]
fn test_pub_use_nested() {
    let extractor = RustDependencyExtractor::new();
    let source = "pub use types::{Config, Settings};";
    let exports = extractor
        .extract_exports(source, Path::new("src/lib.rs"))
        .unwrap();

    assert_eq!(exports.len(), 2);
    assert!(exports.iter().any(|e| e.symbol_name == "Config"));
    assert!(exports.iter().any(|e| e.symbol_name == "Settings"));
    assert!(exports.iter().all(|e| e.module_path == "types"));
    assert!(exports.iter().all(|e| e.visibility == Visibility::Public));
}

// =============================================================================
// Module Path Resolution Tests
// =============================================================================

#[test]
fn test_resolve_crate_path() {
    let extractor = RustDependencyExtractor::new();
    let resolved = extractor
        .resolve_module_path(Path::new("src/handlers/auth.rs"), "crate::config")
        .unwrap();

    // crate:: resolves from the src/ root
    assert_eq!(resolved, Path::new("src/config.rs"));
}

#[test]
fn test_resolve_super_path() {
    let extractor = RustDependencyExtractor::new();
    let resolved = extractor
        .resolve_module_path(Path::new("src/handlers/auth.rs"), "super::utils")
        .unwrap();

    // super:: resolves to parent module
    assert_eq!(resolved, Path::new("src/handlers/utils.rs"));
}

#[test]
fn test_resolve_self_path() {
    let extractor = RustDependencyExtractor::new();
    let resolved = extractor
        .resolve_module_path(Path::new("src/handlers/mod.rs"), "self::auth")
        .unwrap();

    // self:: resolves to sibling in same module directory
    assert_eq!(resolved, Path::new("src/handlers/auth.rs"));
}

#[test]
fn test_resolve_external_crate_returns_error() {
    let extractor = RustDependencyExtractor::new();
    let result = extractor.resolve_module_path(Path::new("src/main.rs"), "std::collections");

    // External crate paths cannot be resolved to local files
    assert!(result.is_err());
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_empty_source() {
    let extractor = RustDependencyExtractor::new();
    let imports = extractor
        .extract_imports("", Path::new("src/main.rs"))
        .unwrap();
    assert!(imports.is_empty());
}

#[test]
fn test_no_imports() {
    let extractor = RustDependencyExtractor::new();
    let source = r#"
fn main() {
    println!("Hello, world!");
}
"#;
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();
    assert!(imports.is_empty());
}

#[test]
fn test_bare_module_import() {
    // `use some_crate;` -- imports just the module, no specific symbol
    let extractor = RustDependencyExtractor::new();
    let source = "use serde;";
    let imports = extractor
        .extract_imports(source, Path::new("src/main.rs"))
        .unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "serde");
    assert!(imports[0].symbols.is_empty());
    assert!(!imports[0].is_wildcard);
}
