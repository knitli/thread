// SPDX-FileCopyrightText: 2026 Knitli Inc.
// SPDX-FileCopyrightText: 2026 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for TypeScript/JavaScript dependency extraction.
//!
//! Tests tree-sitter query-based extraction for ES6 imports, CommonJS requires,
//! and export declarations. All tests follow TDD principles: written first,
//! approved, then implementation created to make them pass.

use std::path::PathBuf;
use thread_flow::incremental::extractors::typescript::{ExportType, TypeScriptDependencyExtractor};

// Helper function to create test file paths
fn test_path(name: &str) -> PathBuf {
    PathBuf::from(format!("test_data/{}", name))
}

/// Test ES6 default import: `import React from 'react'`
#[test]
fn test_es6_default_import() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "import React from 'react';";
    let file_path = test_path("default_import.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "react");
    assert_eq!(import.default_import, Some("React".to_string()));
    assert!(import.symbols.is_empty());
    assert!(import.namespace_import.is_none());
    assert!(!import.is_dynamic);
}

/// Test ES6 single named import: `import { useState } from 'react'`
#[test]
fn test_es6_single_named_import() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "import { useState } from 'react';";
    let file_path = test_path("named_import.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "react");
    assert_eq!(import.symbols.len(), 1);
    assert_eq!(import.symbols[0].imported_name, "useState");
    assert_eq!(import.symbols[0].local_name, "useState");
    assert!(import.default_import.is_none());
}

/// Test ES6 multiple named imports: `import { useState, useEffect } from 'react'`
#[test]
fn test_es6_multiple_named_imports() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "import { useState, useEffect, useCallback } from 'react';";
    let file_path = test_path("multiple_named.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "react");
    assert_eq!(import.symbols.len(), 3);

    let names: Vec<&str> = import
        .symbols
        .iter()
        .map(|s| s.imported_name.as_str())
        .collect();
    assert!(names.contains(&"useState"));
    assert!(names.contains(&"useEffect"));
    assert!(names.contains(&"useCallback"));
}

/// Test ES6 aliased import: `import { useState as useStateHook } from 'react'`
#[test]
fn test_es6_aliased_import() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "import { useState as useStateHook } from 'react';";
    let file_path = test_path("aliased_import.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.symbols.len(), 1);
    assert_eq!(import.symbols[0].imported_name, "useState");
    assert_eq!(import.symbols[0].local_name, "useStateHook");
}

/// Test ES6 namespace import: `import * as fs from 'fs'`
#[test]
fn test_es6_namespace_import() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "import * as fs from 'fs';";
    let file_path = test_path("namespace_import.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "fs");
    assert_eq!(import.namespace_import, Some("fs".to_string()));
    assert!(import.symbols.is_empty());
    assert!(import.default_import.is_none());
}

/// Test ES6 mixed import: `import React, { useState } from 'react'`
#[test]
fn test_es6_mixed_import() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "import React, { useState, useEffect } from 'react';";
    let file_path = test_path("mixed_import.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "react");
    assert_eq!(import.default_import, Some("React".to_string()));
    assert_eq!(import.symbols.len(), 2);
    assert_eq!(import.symbols[0].imported_name, "useState");
    assert_eq!(import.symbols[1].imported_name, "useEffect");
}

/// Test ES6 side-effect import: `import 'module'`
#[test]
fn test_es6_side_effect_import() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "import './polyfills';";
    let file_path = test_path("side_effect.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "./polyfills");
    assert!(import.default_import.is_none());
    assert!(import.symbols.is_empty());
    assert!(import.namespace_import.is_none());
}

/// Test CommonJS require: `const express = require('express')`
#[test]
fn test_commonjs_require() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "const express = require('express');";
    let file_path = test_path("commonjs_require.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "express");
    assert_eq!(import.default_import, Some("express".to_string()));
    assert!(!import.is_dynamic);
}

/// Test CommonJS destructured require: `const { Router } = require('express')`
#[test]
fn test_commonjs_destructured_require() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "const { Router, json } = require('express');";
    let file_path = test_path("destructured_require.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "express");
    assert_eq!(import.symbols.len(), 2);
    assert_eq!(import.symbols[0].imported_name, "Router");
    assert_eq!(import.symbols[1].imported_name, "json");
}

/// Test dynamic import: `import('module')`
#[test]
fn test_dynamic_import() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = r#"
        async function loadModule() {
            const module = await import('./module');
        }
    "#;
    let file_path = test_path("dynamic_import.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "./module");
    assert!(import.is_dynamic);
}

/// Test TypeScript type-only import: `import type { User } from './types'`
#[test]
fn test_typescript_type_import() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "import type { User, Post } from './types';";
    let file_path = test_path("type_import.ts");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.module_specifier, "./types");
    assert_eq!(import.symbols.len(), 2);
    // Type-only imports should be marked in some way (future enhancement)
}

/// Test ES6 default export: `export default function() {}`
#[test]
fn test_es6_default_export() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "export default function handler() {}";
    let file_path = test_path("default_export.js");

    let exports = extractor
        .extract_exports(source, &file_path)
        .expect("Failed to extract exports");

    assert_eq!(exports.len(), 1);
    let export = &exports[0];
    assert!(export.is_default);
    assert_eq!(export.export_type, ExportType::Default);
}

/// Test ES6 named export: `export const X = 1`
#[test]
fn test_es6_named_export() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "export const API_URL = 'https://api.example.com';";
    let file_path = test_path("named_export.js");

    let exports = extractor
        .extract_exports(source, &file_path)
        .expect("Failed to extract exports");

    assert_eq!(exports.len(), 1);
    let export = &exports[0];
    assert_eq!(export.symbol_name, "API_URL");
    assert!(!export.is_default);
    assert_eq!(export.export_type, ExportType::Named);
}

/// Test ES6 named exports with curly braces: `export { X, Y }`
#[test]
fn test_es6_named_exports_list() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "export { useState, useEffect, useCallback };";
    let file_path = test_path("export_list.js");

    let exports = extractor
        .extract_exports(source, &file_path)
        .expect("Failed to extract exports");

    assert_eq!(exports.len(), 3);
    let names: Vec<&str> = exports.iter().map(|e| e.symbol_name.as_str()).collect();
    assert!(names.contains(&"useState"));
    assert!(names.contains(&"useEffect"));
    assert!(names.contains(&"useCallback"));
}

/// Test ES6 re-export: `export * from './other'`
#[test]
fn test_es6_namespace_reexport() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "export * from './utils';";
    let file_path = test_path("reexport.js");

    let exports = extractor
        .extract_exports(source, &file_path)
        .expect("Failed to extract exports");

    assert_eq!(exports.len(), 1);
    let export = &exports[0];
    assert_eq!(export.export_type, ExportType::NamespaceReexport);
    // The module specifier should be accessible somehow for re-exports
}

/// Test ES6 named re-export: `export { X } from './other'`
#[test]
fn test_es6_named_reexport() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "export { useState, useEffect } from 'react';";
    let file_path = test_path("named_reexport.js");

    let exports = extractor
        .extract_exports(source, &file_path)
        .expect("Failed to extract exports");

    assert_eq!(exports.len(), 2);
    assert_eq!(exports[0].symbol_name, "useState");
    assert_eq!(exports[1].symbol_name, "useEffect");
    assert_eq!(exports[0].export_type, ExportType::NamedReexport);
}

/// Test relative path resolution: `./utils` → actual file path
#[test]
fn test_relative_path_resolution() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source_file = PathBuf::from("src/components/Button.tsx");
    let module_specifier = "./utils";

    let resolved = extractor
        .resolve_module_path(&source_file, module_specifier)
        .expect("Failed to resolve module path");

    // Should resolve to src/components/utils.ts or src/components/utils/index.ts
    assert!(
        resolved.to_str().unwrap().contains("src/components/utils")
            || resolved.to_str().unwrap().contains("src/components/utils")
    );
}

/// Test node_modules resolution: `react` → node_modules/react
#[test]
fn test_node_modules_resolution() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source_file = PathBuf::from("src/App.tsx");
    let module_specifier = "react";

    let resolved = extractor
        .resolve_module_path(&source_file, module_specifier)
        .expect("Failed to resolve module path");

    // Should resolve to node_modules/react/index.js or similar
    assert!(resolved.to_str().unwrap().contains("node_modules/react"));
}

/// Test parent directory import: `../utils` → correct resolution
#[test]
fn test_parent_directory_import() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source_file = PathBuf::from("src/components/Button.tsx");
    let module_specifier = "../utils/helpers";

    let resolved = extractor
        .resolve_module_path(&source_file, module_specifier)
        .expect("Failed to resolve module path");

    // Should resolve to src/utils/helpers
    assert!(resolved.to_str().unwrap().contains("src/utils/helpers"));
}

/// Test multiple imports in single file
#[test]
fn test_multiple_imports_per_file() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = r#"
        import React from 'react';
        import { useState, useEffect } from 'react';
        import axios from 'axios';
        const express = require('express');
    "#;
    let file_path = test_path("multiple_imports.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 4);

    // First import: default React
    assert_eq!(imports[0].module_specifier, "react");
    assert_eq!(imports[0].default_import, Some("React".to_string()));

    // Second import: named from react
    assert_eq!(imports[1].module_specifier, "react");
    assert_eq!(imports[1].symbols.len(), 2);

    // Third import: axios
    assert_eq!(imports[2].module_specifier, "axios");

    // Fourth import: CommonJS require
    assert_eq!(imports[3].module_specifier, "express");
}

/// Test barrel file (index.ts re-exporting multiple modules)
#[test]
fn test_barrel_file_pattern() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = r#"
        export * from './Button';
        export * from './Input';
        export * from './Select';
        export { default as Modal } from './Modal';
    "#;
    let file_path = test_path("index.ts");

    let exports = extractor
        .extract_exports(source, &file_path)
        .expect("Failed to extract exports");

    // Should have 4 export statements (3 namespace re-exports + 1 named re-export)
    assert!(exports.len() >= 4);
}

/// Test imports with comments
#[test]
fn test_imports_with_comments() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = r#"
        // Import React
        import React from 'react';
        /* Multi-line comment
           about useState */
        import { useState } from 'react';
    "#;
    let file_path = test_path("commented_imports.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].module_specifier, "react");
    assert_eq!(imports[1].module_specifier, "react");
}

/// Test mixed ESM and CommonJS (valid in some environments)
#[test]
fn test_mixed_esm_commonjs() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = r#"
        import express from 'express';
        const bodyParser = require('body-parser');
        import { Router } from 'express';
    "#;
    let file_path = test_path("mixed_modules.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");

    assert_eq!(imports.len(), 3);

    // Should correctly identify both ESM and CommonJS patterns
    let esm_count = imports.iter().filter(|i| !i.is_dynamic).count();
    assert_eq!(esm_count, 3); // All imports extracted (CommonJS treated as import)
}

/// Test empty file (no imports or exports)
#[test]
fn test_empty_file() {
    let extractor = TypeScriptDependencyExtractor::new();
    let source = "";
    let file_path = test_path("empty.js");

    let imports = extractor
        .extract_imports(source, &file_path)
        .expect("Failed to extract imports");
    let exports = extractor
        .extract_exports(source, &file_path)
        .expect("Failed to extract exports");

    assert!(imports.is_empty());
    assert!(exports.is_empty());
}

/// Test performance: extract from large file (<5ms target)
#[test]
fn test_extraction_performance() {
    let extractor = TypeScriptDependencyExtractor::new();

    // Generate a file with 100 imports
    let mut source = String::new();
    for i in 0..100 {
        source.push_str(&format!("import module{} from 'module{}';\n", i, i));
    }

    let file_path = test_path("large_file.js");

    let start = std::time::Instant::now();
    let imports = extractor
        .extract_imports(&source, &file_path)
        .expect("Failed to extract imports");
    let duration = start.elapsed();

    assert_eq!(imports.len(), 100);
    assert!(
        duration.as_millis() < 5,
        "Extraction took {}ms, expected <5ms",
        duration.as_millis()
    );
}
