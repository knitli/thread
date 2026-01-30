// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the Python dependency extractor.
//!
//! Tests are organized by import pattern category:
//! - Absolute imports (`import X`)
//! - From imports (`from X import Y`)
//! - Relative imports (`from .X import Y`)
//! - Wildcard imports (`from X import *`)
//! - Aliased imports (`import X as Y`)
//! - Multiple imports per statement
//! - Package resolution (`__init__.py` awareness)
//! - Edge cases (empty files, syntax errors, mixed patterns)
//!
//! Written TDD-first: all tests written before implementation.

use std::path::Path;
use thread_flow::incremental::extractors::python::{ImportInfo, PythonDependencyExtractor};

// ─── Helper ─────────────────────────────────────────────────────────────────

fn extract(source: &str) -> Vec<ImportInfo> {
    let extractor = PythonDependencyExtractor::new();
    extractor
        .extract_imports(source, Path::new("test.py"))
        .expect("extraction should succeed")
}

// ─── 1. Absolute Imports ────────────────────────────────────────────────────

#[test]
fn test_simple_import() {
    let imports = extract("import os");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "os");
    assert!(imports[0].symbols.is_empty());
    assert!(!imports[0].is_wildcard);
    assert_eq!(imports[0].relative_level, 0);
}

#[test]
fn test_dotted_import() {
    let imports = extract("import os.path");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "os.path");
    assert!(imports[0].symbols.is_empty());
    assert_eq!(imports[0].relative_level, 0);
}

#[test]
fn test_multiple_modules_in_single_import() {
    // `import os, sys` produces two separate import infos
    let imports = extract("import os, sys");
    assert_eq!(imports.len(), 2);

    let paths: Vec<&str> = imports.iter().map(|i| i.module_path.as_str()).collect();
    assert!(paths.contains(&"os"));
    assert!(paths.contains(&"sys"));
}

// ─── 2. From Imports ────────────────────────────────────────────────────────

#[test]
fn test_from_import_single_symbol() {
    let imports = extract("from os import path");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "os");
    assert_eq!(imports[0].symbols, vec!["path"]);
    assert!(!imports[0].is_wildcard);
    assert_eq!(imports[0].relative_level, 0);
}

#[test]
fn test_from_import_multiple_symbols() {
    let imports = extract("from os.path import join, exists, isdir");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "os.path");
    assert_eq!(imports[0].symbols, vec!["join", "exists", "isdir"]);
}

#[test]
fn test_from_import_parenthesized() {
    let source = "from os.path import (\n    join,\n    exists,\n    isdir,\n)";
    let imports = extract(source);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "os.path");
    assert_eq!(imports[0].symbols.len(), 3);
    assert!(imports[0].symbols.contains(&"join".to_string()));
    assert!(imports[0].symbols.contains(&"exists".to_string()));
    assert!(imports[0].symbols.contains(&"isdir".to_string()));
}

// ─── 3. Relative Imports ────────────────────────────────────────────────────

#[test]
fn test_relative_import_single_dot() {
    let imports = extract("from .utils import helper");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "utils");
    assert_eq!(imports[0].symbols, vec!["helper"]);
    assert_eq!(imports[0].relative_level, 1);
}

#[test]
fn test_relative_import_double_dot() {
    let imports = extract("from ..core import Engine");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "core");
    assert_eq!(imports[0].symbols, vec!["Engine"]);
    assert_eq!(imports[0].relative_level, 2);
}

#[test]
fn test_relative_import_triple_dot() {
    let imports = extract("from ...base.config import Settings");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "base.config");
    assert_eq!(imports[0].symbols, vec!["Settings"]);
    assert_eq!(imports[0].relative_level, 3);
}

#[test]
fn test_relative_import_dot_only() {
    // `from . import something` - no module name, just dots
    let imports = extract("from . import something");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "");
    assert_eq!(imports[0].symbols, vec!["something"]);
    assert_eq!(imports[0].relative_level, 1);
}

// ─── 4. Wildcard Imports ────────────────────────────────────────────────────

#[test]
fn test_wildcard_import() {
    let imports = extract("from module import *");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "module");
    assert!(imports[0].is_wildcard);
    assert_eq!(imports[0].relative_level, 0);
}

#[test]
fn test_relative_wildcard_import() {
    let imports = extract("from .subpackage import *");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "subpackage");
    assert!(imports[0].is_wildcard);
    assert_eq!(imports[0].relative_level, 1);
}

// ─── 5. Aliased Imports ─────────────────────────────────────────────────────

#[test]
fn test_aliased_import() {
    let imports = extract("import numpy as np");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "numpy");
    assert_eq!(
        imports[0].aliases,
        vec![("numpy".to_string(), "np".to_string())]
    );
}

#[test]
fn test_from_import_with_alias() {
    let imports = extract("from os import path as ospath");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "os");
    assert_eq!(imports[0].symbols, vec!["path"]);
    assert_eq!(
        imports[0].aliases,
        vec![("path".to_string(), "ospath".to_string())]
    );
}

// ─── 6. Multiple Imports in File ────────────────────────────────────────────

#[test]
fn test_multiple_import_statements() {
    let source = "\
import os
import sys
from pathlib import Path
from collections import OrderedDict, defaultdict
from .utils import helper
";
    let imports = extract(source);
    assert_eq!(imports.len(), 5);

    // Verify each import is present
    let modules: Vec<&str> = imports.iter().map(|i| i.module_path.as_str()).collect();
    assert!(modules.contains(&"os"));
    assert!(modules.contains(&"sys"));
    assert!(modules.contains(&"pathlib"));
    assert!(modules.contains(&"collections"));
    assert!(modules.contains(&"utils"));
}

// ─── 7. Module Path Resolution ──────────────────────────────────────────────

#[test]
fn test_resolve_absolute_module_path() {
    let extractor = PythonDependencyExtractor::new();
    let source_file = Path::new("/project/src/main.py");
    let resolved = extractor
        .resolve_module_path(source_file, "os.path", 0)
        .unwrap();

    // Absolute imports resolve to the module's dotted path converted to path separators
    // e.g., "os.path" -> "os/path.py" (or "os/path/__init__.py")
    let resolved_str = resolved.to_string_lossy();
    assert!(
        resolved_str.ends_with("os/path.py") || resolved_str.ends_with("os/path/__init__.py"),
        "Expected os/path.py or os/path/__init__.py, got: {}",
        resolved_str
    );
}

#[test]
fn test_resolve_relative_module_single_dot() {
    let extractor = PythonDependencyExtractor::new();
    let source_file = Path::new("/project/src/package/main.py");
    let resolved = extractor
        .resolve_module_path(source_file, "utils", 1)
        .unwrap();

    // `.utils` from `/project/src/package/main.py` -> `/project/src/package/utils.py`
    assert_eq!(resolved, Path::new("/project/src/package/utils.py"));
}

#[test]
fn test_resolve_relative_module_double_dot() {
    let extractor = PythonDependencyExtractor::new();
    let source_file = Path::new("/project/src/package/sub/main.py");
    let resolved = extractor
        .resolve_module_path(source_file, "core", 2)
        .unwrap();

    // `..core` from `/project/src/package/sub/main.py` -> `/project/src/package/core.py`
    assert_eq!(resolved, Path::new("/project/src/package/core.py"));
}

#[test]
fn test_resolve_relative_module_dot_only() {
    let extractor = PythonDependencyExtractor::new();
    let source_file = Path::new("/project/src/package/main.py");
    let resolved = extractor.resolve_module_path(source_file, "", 1).unwrap();

    // `from . import X` resolves to the package __init__.py
    assert_eq!(resolved, Path::new("/project/src/package/__init__.py"));
}

// ─── 8. Edge Cases ──────────────────────────────────────────────────────────

#[test]
fn test_empty_source() {
    let imports = extract("");
    assert!(imports.is_empty());
}

#[test]
fn test_no_imports() {
    let source = "\
x = 1
def foo():
    return x + 2
";
    let imports = extract(source);
    assert!(imports.is_empty());
}

#[test]
fn test_import_inside_function() {
    // Conditional/lazy imports inside functions should still be extracted
    let source = "\
def load_numpy():
    import numpy as np
    return np
";
    let imports = extract(source);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "numpy");
}

#[test]
fn test_import_inside_try_except() {
    let source = "\
try:
    import ujson as json
except ImportError:
    import json
";
    let imports = extract(source);
    assert_eq!(imports.len(), 2);
}

#[test]
fn test_commented_import_not_extracted() {
    let source = "\
# import os
import sys
";
    let imports = extract(source);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "sys");
}

#[test]
fn test_string_import_not_extracted() {
    // Import inside a string literal should NOT be extracted
    let source = r#"
code = "import os"
import sys
"#;
    let imports = extract(source);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "sys");
}

#[test]
fn test_deeply_dotted_module() {
    let imports = extract("from a.b.c.d.e import f");
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module_path, "a.b.c.d.e");
    assert_eq!(imports[0].symbols, vec!["f"]);
}
