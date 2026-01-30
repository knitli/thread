// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Comprehensive tests for Go dependency extraction.
//!
//! Validates tree-sitter query-based extraction of Go import statements
//! for populating the incremental update DependencyGraph.
//!
//! ## Coverage
//!
//! - Single import statements
//! - Import blocks (grouped imports)
//! - Aliased imports
//! - Dot imports
//! - Blank imports (side-effect only)
//! - CGo imports (`import "C"`)
//! - Standard library imports
//! - External module imports
//! - go.mod module path resolution
//! - Vendor directory imports
//! - Edge cases (empty file, no imports, comments)
//! - DependencyEdge construction

use std::path::{Path, PathBuf};
use thread_flow::incremental::DependencyType;
use thread_flow::incremental::extractors::go::GoDependencyExtractor;

// =============================================================================
// Test Helpers
// =============================================================================

/// Create an extractor with a mock go.mod module path.
fn extractor_with_module(module_path: &str) -> GoDependencyExtractor {
    GoDependencyExtractor::new(Some(module_path.to_string()))
}

/// Create an extractor without go.mod awareness.
fn extractor_no_module() -> GoDependencyExtractor {
    GoDependencyExtractor::new(None)
}

// =============================================================================
// Single Import Tests
// =============================================================================

#[test]
fn test_single_import_statement() {
    let source = r#"package main

import "fmt"

func main() {
    fmt.Println("hello")
}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].import_path, "fmt");
    assert!(imports[0].alias.is_none());
    assert!(!imports[0].is_dot_import);
    assert!(!imports[0].is_blank_import);
}

#[test]
fn test_single_import_with_subdirectory() {
    let source = r#"package main

import "net/http"

func main() {}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].import_path, "net/http");
}

// =============================================================================
// Import Block Tests
// =============================================================================

#[test]
fn test_import_block() {
    let source = r#"package main

import (
    "fmt"
    "os"
    "strings"
)

func main() {}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 3);

    let paths: Vec<&str> = imports.iter().map(|i| i.import_path.as_str()).collect();
    assert!(paths.contains(&"fmt"));
    assert!(paths.contains(&"os"));
    assert!(paths.contains(&"strings"));
}

#[test]
fn test_multiple_import_blocks() {
    let source = r#"package main

import (
    "fmt"
)

import (
    "os"
)

func main() {}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 2);
    let paths: Vec<&str> = imports.iter().map(|i| i.import_path.as_str()).collect();
    assert!(paths.contains(&"fmt"));
    assert!(paths.contains(&"os"));
}

// =============================================================================
// Aliased Import Tests
// =============================================================================

#[test]
fn test_aliased_import() {
    let source = r#"package main

import f "fmt"

func main() {
    f.Println("hello")
}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].import_path, "fmt");
    assert_eq!(imports[0].alias.as_deref(), Some("f"));
    assert!(!imports[0].is_dot_import);
    assert!(!imports[0].is_blank_import);
}

#[test]
fn test_aliased_import_in_block() {
    let source = r#"package main

import (
    f "fmt"
    nethttp "net/http"
)

func main() {}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 2);

    let fmt_import = imports.iter().find(|i| i.import_path == "fmt").unwrap();
    assert_eq!(fmt_import.alias.as_deref(), Some("f"));

    let http_import = imports
        .iter()
        .find(|i| i.import_path == "net/http")
        .unwrap();
    assert_eq!(http_import.alias.as_deref(), Some("nethttp"));
}

// =============================================================================
// Dot Import Tests
// =============================================================================

#[test]
fn test_dot_import() {
    let source = r#"package main

import . "fmt"

func main() {
    Println("hello")
}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].import_path, "fmt");
    assert!(imports[0].is_dot_import);
    assert!(imports[0].alias.is_none());
}

// =============================================================================
// Blank Import Tests
// =============================================================================

#[test]
fn test_blank_import() {
    let source = r#"package main

import _ "database/sql/driver"

func main() {}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].import_path, "database/sql/driver");
    assert!(imports[0].is_blank_import);
    assert!(!imports[0].is_dot_import);
}

// =============================================================================
// CGo Import Tests
// =============================================================================

#[test]
fn test_cgo_import() {
    let source = r#"package main

// #include <stdio.h>
import "C"

import "fmt"

func main() {}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    // Should extract both "C" and "fmt"
    assert_eq!(imports.len(), 2);

    let c_import = imports.iter().find(|i| i.import_path == "C").unwrap();
    assert_eq!(c_import.import_path, "C");

    let fmt_import = imports.iter().find(|i| i.import_path == "fmt").unwrap();
    assert_eq!(fmt_import.import_path, "fmt");
}

// =============================================================================
// External Module Import Tests
// =============================================================================

#[test]
fn test_external_module_import() {
    let source = r#"package main

import (
    "fmt"
    "github.com/user/repo/pkg"
    "golang.org/x/sync/errgroup"
)

func main() {}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 3);
    let paths: Vec<&str> = imports.iter().map(|i| i.import_path.as_str()).collect();
    assert!(paths.contains(&"fmt"));
    assert!(paths.contains(&"github.com/user/repo/pkg"));
    assert!(paths.contains(&"golang.org/x/sync/errgroup"));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_file() {
    let source = "";
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("empty.go"))
        .expect("extraction should succeed on empty file");

    assert!(imports.is_empty());
}

#[test]
fn test_no_imports() {
    let source = r#"package main

func main() {
    println("hello")
}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert!(imports.is_empty());
}

#[test]
fn test_commented_import_not_extracted() {
    let source = r#"package main

// import "fmt"

/* import "os" */

func main() {}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert!(imports.is_empty());
}

// =============================================================================
// Mixed Import Styles
// =============================================================================

#[test]
fn test_mixed_import_styles() {
    let source = r#"package main

import (
    "fmt"
    "os"
    f "flag"
    . "math"
    _ "image/png"
)

func main() {}
"#;
    let extractor = extractor_no_module();
    let imports = extractor
        .extract_imports(source, Path::new("main.go"))
        .expect("extraction should succeed");

    assert_eq!(imports.len(), 5);

    let fmt_import = imports.iter().find(|i| i.import_path == "fmt").unwrap();
    assert!(fmt_import.alias.is_none());
    assert!(!fmt_import.is_dot_import);
    assert!(!fmt_import.is_blank_import);

    let os_import = imports.iter().find(|i| i.import_path == "os").unwrap();
    assert!(os_import.alias.is_none());

    let flag_import = imports.iter().find(|i| i.import_path == "flag").unwrap();
    assert_eq!(flag_import.alias.as_deref(), Some("f"));

    let math_import = imports.iter().find(|i| i.import_path == "math").unwrap();
    assert!(math_import.is_dot_import);

    let png_import = imports
        .iter()
        .find(|i| i.import_path == "image/png")
        .unwrap();
    assert!(png_import.is_blank_import);
}

// =============================================================================
// Import Path Resolution Tests
// =============================================================================

#[test]
fn test_resolve_standard_library_import() {
    let extractor = extractor_no_module();
    let result = extractor.resolve_import_path(Path::new("main.go"), "fmt");

    // Standard library imports cannot be resolved to local paths
    assert!(result.is_err() || result.unwrap() == PathBuf::from("GOROOT/src/fmt"));
}

#[test]
fn test_resolve_module_internal_import() {
    let extractor = extractor_with_module("github.com/user/myproject");
    let result = extractor.resolve_import_path(
        Path::new("cmd/main.go"),
        "github.com/user/myproject/internal/utils",
    );

    // Should resolve to a local path relative to module root
    let resolved = result.expect("module-internal import should resolve");
    assert_eq!(resolved, PathBuf::from("internal/utils"));
}

#[test]
fn test_resolve_external_import() {
    let extractor = extractor_with_module("github.com/user/myproject");
    let result = extractor.resolve_import_path(Path::new("main.go"), "github.com/other/repo/pkg");

    // External imports cannot be resolved to local paths
    assert!(result.is_err());
}

// =============================================================================
// DependencyEdge Construction Tests
// =============================================================================

#[test]
fn test_to_dependency_edges() {
    let source = r#"package main

import (
    "fmt"
    "github.com/user/myproject/internal/utils"
)

func main() {}
"#;
    let extractor = extractor_with_module("github.com/user/myproject");
    let file_path = Path::new("cmd/main.go");
    let edges = extractor
        .extract_dependency_edges(source, file_path)
        .expect("edge extraction should succeed");

    // Only module-internal imports produce edges (external/stdlib do not)
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].from, PathBuf::from("cmd/main.go"));
    assert_eq!(edges[0].to, PathBuf::from("internal/utils"));
    assert_eq!(edges[0].dep_type, DependencyType::Import);
}

// =============================================================================
// Vendor Directory Tests
// =============================================================================

#[test]
fn test_resolve_vendor_import() {
    let extractor =
        GoDependencyExtractor::with_vendor(Some("github.com/user/myproject".to_string()), true);
    let result = extractor.resolve_import_path(Path::new("main.go"), "github.com/dep/pkg");

    // With vendor mode, external imports resolve to vendor directory
    let resolved = result.expect("vendor import should resolve");
    assert_eq!(resolved, PathBuf::from("vendor/github.com/dep/pkg"));
}
