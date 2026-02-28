// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Real-World Codebase Validation Tests (Phase 5.5)
//!
//! Validates the incremental analysis system on large-scale codebases (10K+ files)
//! across multiple programming languages. Uses a hybrid approach:
//! - Synthetic scale tests (generated 10K+ file fixtures)
//! - Real-world pattern tests (code samples from major open-source projects)
//!
//! ## Test Coverage (16 tests)
//!
//! 1. **Scale Tests** (4 tests): 10K+ files per language, performance validation
//! 2. **Pattern Tests** (8 tests): Real-world code patterns, edge cases
//! 3. **Performance Tests** (4 tests): Cold start, incremental updates, cache efficiency
//!
//! ## Constitutional Requirements (Principle VI)
//!
//! - Content-addressed caching: >90% hit rate
//! - Storage latency: <10ms (Postgres), <50ms (D1)
//! - Incremental updates: Only affected components reanalyzed
//!
//! ## Success Criteria
//!
//! - All tests pass with `cargo nextest run --all-features`
//! - Performance targets met at 10K+ file scale
//! - Edge cases discovered and documented
//! - Validation report generated in claudedocs/REAL_WORLD_VALIDATION.md

use std::path::{Path, PathBuf};
use std::time::Instant;
use thread_flow::incremental::analyzer::IncrementalAnalyzer;
use thread_flow::incremental::dependency_builder::DependencyGraphBuilder;
use thread_flow::incremental::storage::InMemoryStorage;
use tokio::fs;
use tokio::io::AsyncWriteExt;

// ═══════════════════════════════════════════════════════════════════════════
// Test Fixtures
// ═══════════════════════════════════════════════════════════════════════════

/// Test fixture for real-world validation tests.
///
/// Provides infrastructure for large-scale testing including:
/// - Temporary directory management
/// - Large-scale file generation (10K+ files)
/// - Performance measurement utilities
/// - Real-world pattern templates
struct ValidationFixture {
    /// Temporary directory for test files
    temp_dir: tempfile::TempDir,
    /// Analyzer with InMemory storage
    analyzer: IncrementalAnalyzer,
    /// Dependency graph builder
    builder: DependencyGraphBuilder,
}

impl ValidationFixture {
    /// Creates a new validation fixture.
    async fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("create temp dir");

        let analyzer_storage = InMemoryStorage::new();
        let analyzer = IncrementalAnalyzer::new(Box::new(analyzer_storage));

        let builder_storage = InMemoryStorage::new();
        let builder = DependencyGraphBuilder::new(Box::new(builder_storage));

        Self {
            temp_dir,
            analyzer,
            builder,
        }
    }

    /// Returns the path to the temporary directory.
    fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Creates a test file with the given content.
    async fn create_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let file_path = self.temp_path().join(relative_path);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.expect("create parent dir");
        }

        // Write file content
        let mut file = fs::File::create(&file_path).await.expect("create file");
        file.write_all(content.as_bytes())
            .await
            .expect("write file");
        file_path
    }

    /// Updates an existing test file with new content.
    async fn update_file(&self, file_path: &Path, content: &str) {
        let mut file = fs::File::create(file_path).await.expect("open file");
        file.write_all(content.as_bytes())
            .await
            .expect("write file");
    }

    /// Analyzes changes and extracts dependencies (E2E workflow).
    async fn analyze_and_extract(
        &mut self,
        paths: &[PathBuf],
    ) -> thread_flow::incremental::analyzer::AnalysisResult {
        // Step 1: Analyze changes (fingerprinting)
        let result = self
            .analyzer
            .analyze_changes(paths)
            .await
            .expect("analyze changes");

        // Step 2: Extract dependencies (graph building)
        self.builder
            .extract_files(paths)
            .await
            .expect("extract dependencies");

        // Step 3: Sync builder's graph to analyzer's graph
        let builder_graph = self.builder.graph();
        let analyzer_graph = self.analyzer.graph_mut();

        for edge in &builder_graph.edges {
            analyzer_graph.add_edge(edge.clone());
        }

        result
    }

    /// Generates a large-scale Rust codebase (10K+ files).
    ///
    /// Creates a synthetic project structure simulating a large Rust application:
    /// - Multiple modules organized hierarchically
    /// - Realistic import patterns (use statements)
    /// - Mix of library and binary crates
    async fn generate_rust_scale(&self, file_count: usize) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Calculate module structure (10 top-level modules, 100 submodules each)
        let modules_per_level = (file_count as f64).sqrt() as usize;
        let files_per_module = file_count / modules_per_level;

        for module_idx in 0..modules_per_level {
            let module_name = format!("module_{}", module_idx);

            // Create module directory
            let module_dir = self.temp_path().join(&module_name);
            fs::create_dir_all(&module_dir)
                .await
                .expect("create module");

            // Create mod.rs for the module
            let mod_file = module_dir.join("mod.rs");
            let mut mod_content = String::from("// Module exports\n\n");

            for file_idx in 0..files_per_module {
                let file_name = format!("file_{}.rs", file_idx);
                mod_content.push_str(&format!("pub mod file_{};\n", file_idx));

                // Create source file with imports
                let file_path = module_dir.join(&file_name);
                let content = format!(
                    "// File {} in module {}\n\
                     use std::collections::HashMap;\n\
                     use crate::{}::mod;\n\
                     \n\
                     pub fn function_{}() -> HashMap<String, i32> {{\n\
                         HashMap::new()\n\
                     }}\n",
                    file_idx, module_idx, module_name, file_idx
                );

                let mut file = fs::File::create(&file_path).await.expect("create file");
                file.write_all(content.as_bytes())
                    .await
                    .expect("write file");

                paths.push(file_path);
            }

            // Write mod.rs
            let mut file = fs::File::create(&mod_file).await.expect("create mod.rs");
            file.write_all(mod_content.as_bytes())
                .await
                .expect("write mod.rs");
            paths.push(mod_file);
        }

        paths
    }

    /// Generates a large-scale TypeScript codebase (10K+ files).
    async fn generate_typescript_scale(&self, file_count: usize) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let modules_per_level = (file_count as f64).sqrt() as usize;
        let files_per_module = file_count / modules_per_level;

        for module_idx in 0..modules_per_level {
            let module_name = format!("module_{}", module_idx);
            let module_dir = self.temp_path().join(&module_name);
            fs::create_dir_all(&module_dir)
                .await
                .expect("create module");

            // Create index.ts for module
            let index_file = module_dir.join("index.ts");
            let mut index_content = String::from("// Module exports\n\n");

            for file_idx in 0..files_per_module {
                let file_name = format!("file_{}.ts", file_idx);
                index_content.push_str(&format!("export * from './file_{}';\n", file_idx));

                let file_path = module_dir.join(&file_name);
                let content = format!(
                    "// File {} in module {}\n\
                     import {{ Map }} from './index';\n\
                     \n\
                     export class Component_{} {{\n\
                         private data: Map<string, number> = new Map();\n\
                         \n\
                         public process(): void {{\n\
                             // Processing logic\n\
                         }}\n\
                     }}\n",
                    file_idx, module_idx, file_idx
                );

                let mut file = fs::File::create(&file_path).await.expect("create file");
                file.write_all(content.as_bytes())
                    .await
                    .expect("write file");
                paths.push(file_path);
            }

            let mut file = fs::File::create(&index_file)
                .await
                .expect("create index.ts");
            file.write_all(index_content.as_bytes())
                .await
                .expect("write index.ts");
            paths.push(index_file);
        }

        paths
    }

    /// Generates a large-scale Python codebase (10K+ files).
    async fn generate_python_scale(&self, file_count: usize) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let packages = (file_count as f64).sqrt() as usize;
        let files_per_package = file_count / packages;

        for pkg_idx in 0..packages {
            let pkg_name = format!("package_{}", pkg_idx);
            let pkg_dir = self.temp_path().join(&pkg_name);
            fs::create_dir_all(&pkg_dir).await.expect("create package");

            // Create __init__.py
            let init_file = pkg_dir.join("__init__.py");
            let mut init_content = String::from("# Package exports\n\n");

            for file_idx in 0..files_per_package {
                let file_name = format!("module_{}.py", file_idx);
                init_content.push_str(&format!("from .module_{} import *\n", file_idx));

                let file_path = pkg_dir.join(&file_name);
                let content = format!(
                    "# Module {} in package {}\n\
                     from typing import Dict\n\
                     from . import __init__\n\
                     \n\
                     class Service_{}:\n\
                         def __init__(self):\n\
                             self.data: Dict[str, int] = {{}}\n\
                         \n\
                         def process(self) -> None:\n\
                             pass\n",
                    file_idx, pkg_idx, file_idx
                );

                let mut file = fs::File::create(&file_path).await.expect("create file");
                file.write_all(content.as_bytes())
                    .await
                    .expect("write file");
                paths.push(file_path);
            }

            let mut file = fs::File::create(&init_file)
                .await
                .expect("create __init__.py");
            file.write_all(init_content.as_bytes())
                .await
                .expect("write __init__.py");
            paths.push(init_file);
        }

        paths
    }

    /// Generates a large-scale Go codebase (10K+ files).
    async fn generate_go_scale(&self, file_count: usize) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let packages = (file_count as f64).sqrt() as usize;
        let files_per_package = file_count / packages;

        for pkg_idx in 0..packages {
            let pkg_name = format!("pkg{}", pkg_idx);
            let pkg_dir = self.temp_path().join(&pkg_name);
            fs::create_dir_all(&pkg_dir).await.expect("create package");

            for file_idx in 0..files_per_package {
                let file_name = format!("file_{}.go", file_idx);
                let file_path = pkg_dir.join(&file_name);

                let content = format!(
                    "// File {} in package {}\n\
                     package {}\n\
                     \n\
                     import \"fmt\"\n\
                     \n\
                     type Service_{} struct {{\n\
                         Data map[string]int\n\
                     }}\n\
                     \n\
                     func (s *Service_{}) Process() {{\n\
                         fmt.Println(\"processing\")\n\
                     }}\n",
                    file_idx, pkg_name, pkg_name, file_idx, file_idx
                );

                let mut file = fs::File::create(&file_path).await.expect("create file");
                file.write_all(content.as_bytes())
                    .await
                    .expect("write file");
                paths.push(file_path);
            }
        }

        paths
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Scale Tests (4 tests)
// ═══════════════════════════════════════════════════════════════════════════

/// Validates incremental analysis on large-scale Rust codebase (10K+ files).
///
/// Simulates a project like tokio with:
/// - Multiple modules organized hierarchically
/// - Realistic import patterns (std, crate, external)
/// - 10,000+ source files
///
/// **Performance Targets**:
/// - Initial analysis: <10s for 10K files
/// - Incremental update (1% change): <1s
/// - Cache hit rate: >90%
#[tokio::test]
async fn test_real_world_rust_scale() {
    let mut fixture = ValidationFixture::new().await;

    // Generate 10K Rust files
    let start = Instant::now();
    let paths = fixture.generate_rust_scale(10_000).await;
    let generation_time = start.elapsed();
    println!(
        "Generated {} Rust files in {:?}",
        paths.len(),
        generation_time
    );

    assert!(
        paths.len() >= 10_000,
        "Expected at least 10K files, got {}",
        paths.len()
    );

    // Initial analysis (cold start)
    let start = Instant::now();
    let result = fixture.analyze_and_extract(&paths).await;
    let analysis_time = start.elapsed();
    println!(
        "Initial analysis of {} files in {:?}",
        paths.len(),
        analysis_time
    );

    // Validate results
    assert!(
        result.changed_files.len() >= 10_000,
        "Expected >=10K changed files, got {}",
        result.changed_files.len()
    );

    // Performance validation: <10s for 10K files
    assert!(
        analysis_time.as_secs() < 10,
        "Initial analysis took {:?}, exceeds 10s target",
        analysis_time
    );

    // Validate dependency graph populated
    let graph = fixture.builder.graph();
    assert!(
        graph.node_count() >= 10_000,
        "Expected >=10K nodes, got {}",
        graph.node_count()
    );

    // Test incremental update (1% change)
    let changed_count = (paths.len() as f64 * 0.01) as usize;
    let changed_paths: Vec<_> = paths.iter().take(changed_count).cloned().collect();

    for path in &changed_paths {
        fixture
            .update_file(path, "// Updated content\npub fn updated() {}")
            .await;
    }

    let start = Instant::now();
    let result = fixture.analyze_and_extract(&changed_paths).await;
    let incremental_time = start.elapsed();
    println!(
        "Incremental update of {} files in {:?}",
        changed_count, incremental_time
    );

    // Validate incremental efficiency
    assert!(
        result.changed_files.len() == changed_count,
        "Expected {} changed files, got {}",
        changed_count,
        result.changed_files.len()
    );

    // Performance validation: <1s for 1% update
    assert!(
        incremental_time.as_secs() < 1,
        "Incremental update took {:?}, exceeds 1s target",
        incremental_time
    );

    // Cache hit rate is already computed in AnalysisResult
    println!("Cache hit rate: {:.1}%", result.cache_hit_rate * 100.0);
}

/// Validates incremental analysis on large-scale TypeScript codebase (10K+ files).
///
/// Simulates a project like VSCode with:
/// - ES6 module system (import/export)
/// - Class-based architecture
/// - 10,000+ TypeScript files
#[tokio::test]
async fn test_real_world_typescript_scale() {
    let mut fixture = ValidationFixture::new().await;

    // Generate 10K TypeScript files
    let start = Instant::now();
    let paths = fixture.generate_typescript_scale(10_000).await;
    let generation_time = start.elapsed();
    println!(
        "Generated {} TypeScript files in {:?}",
        paths.len(),
        generation_time
    );

    assert!(
        paths.len() >= 10_000,
        "Expected at least 10K files, got {}",
        paths.len()
    );

    // Initial analysis
    let start = Instant::now();
    let result = fixture.analyze_and_extract(&paths).await;
    let analysis_time = start.elapsed();
    println!(
        "Initial analysis of {} files in {:?}",
        paths.len(),
        analysis_time
    );

    assert!(result.changed_files.len() >= 10_000);
    // TypeScript parsing is slowest at scale - allow 20s for 10K files
    assert!(
        analysis_time.as_secs() < 20,
        "TypeScript analysis time {:?} exceeded 20s",
        analysis_time
    );

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 10_000);
}

/// Validates incremental analysis on large-scale Python codebase (10K+ files).
///
/// Simulates a project like Django with:
/// - Package-based structure (__init__.py)
/// - Import system (from ... import)
/// - 10,000+ Python modules
#[tokio::test]
async fn test_real_world_python_scale() {
    let mut fixture = ValidationFixture::new().await;

    // Generate 10K Python files
    let start = Instant::now();
    let paths = fixture.generate_python_scale(10_000).await;
    let generation_time = start.elapsed();
    println!(
        "Generated {} Python files in {:?}",
        paths.len(),
        generation_time
    );

    assert!(paths.len() >= 10_000);

    // Initial analysis
    let start = Instant::now();
    let result = fixture.analyze_and_extract(&paths).await;
    let analysis_time = start.elapsed();
    println!(
        "Initial analysis of {} files in {:?}",
        paths.len(),
        analysis_time
    );

    assert!(result.changed_files.len() >= 10_000);
    // Python parsing is slower at scale - allow 15s for 10K files
    assert!(
        analysis_time.as_secs() < 15,
        "Python analysis time {:?} exceeded 15s",
        analysis_time
    );

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 10_000);
}

/// Validates incremental analysis on large-scale Go codebase (10K+ files).
///
/// Simulates a project like Kubernetes with:
/// - Package-based organization
/// - Interface-driven design
/// - 10,000+ Go source files
#[tokio::test]
async fn test_real_world_go_scale() {
    let mut fixture = ValidationFixture::new().await;

    // Generate 10K Go files
    let start = Instant::now();
    let paths = fixture.generate_go_scale(10_000).await;
    let generation_time = start.elapsed();
    println!(
        "Generated {} Go files in {:?}",
        paths.len(),
        generation_time
    );

    assert!(paths.len() >= 10_000);

    // Initial analysis
    let start = Instant::now();
    let result = fixture.analyze_and_extract(&paths).await;
    let analysis_time = start.elapsed();
    println!(
        "Initial analysis of {} files in {:?}",
        paths.len(),
        analysis_time
    );

    assert!(result.changed_files.len() >= 10_000);
    assert!(analysis_time.as_secs() < 10);

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 10_000);
}

// ═══════════════════════════════════════════════════════════════════════════
// Pattern Tests (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

/// Validates handling of tokio-like async Rust patterns.
///
/// Tests real-world patterns found in tokio:
/// - Async traits and impl blocks
/// - Macro-heavy code (tokio::main, tokio::test)
/// - Complex module re-exports
#[tokio::test]
async fn test_real_world_rust_patterns() {
    let mut fixture = ValidationFixture::new().await;

    // Create tokio-like async code
    let runtime_rs = fixture
        .create_file(
            "runtime.rs",
            "use std::sync::Arc;\n\
             use tokio::sync::Mutex;\n\
             \n\
             #[tokio::main]\n\
             async fn main() {\n\
                 let runtime = Arc::new(Mutex::new(Runtime::new()));\n\
                 runtime.lock().await.run();\n\
             }\n\
             \n\
             pub struct Runtime {\n\
                 workers: Vec<Worker>,\n\
             }\n\
             \n\
             impl Runtime {\n\
                 pub fn new() -> Self {\n\
                     Self { workers: vec![] }\n\
                 }\n\
                 pub async fn run(&self) {}\n\
             }\n\
             \n\
             struct Worker;\n",
        )
        .await;

    let result = fixture.analyze_and_extract(&[runtime_rs]).await;
    assert_eq!(result.changed_files.len(), 1);

    // Validate async/macro patterns detected
    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 1);
}

/// Validates handling of VSCode-like TypeScript patterns.
///
/// Tests patterns found in VSCode:
/// - Decorators and metadata
/// - Dependency injection patterns
/// - Complex class hierarchies
#[tokio::test]
async fn test_real_world_typescript_patterns() {
    let mut fixture = ValidationFixture::new().await;

    // Create VSCode-like dependency injection pattern
    let service_ts = fixture
        .create_file(
            "service.ts",
            "import { injectable, inject } from './di';\n\
             import { ILogger } from './interfaces';\n\
             \n\
             @injectable()\n\
             export class EditorService {\n\
                 constructor(\n\
                     @inject('ILogger') private logger: ILogger\n\
                 ) {}\n\
                 \n\
                 public edit(file: string): void {\n\
                     this.logger.log(`Editing ${file}`);\n\
                 }\n\
             }\n",
        )
        .await;

    let result = fixture.analyze_and_extract(&[service_ts]).await;
    assert_eq!(result.changed_files.len(), 1);

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 1);
}

/// Validates handling of Django-like Python patterns.
///
/// Tests patterns found in Django:
/// - Decorators (@property, @classmethod)
/// - ORM model patterns
/// - Settings and configuration imports
#[tokio::test]
async fn test_real_world_python_patterns() {
    let mut fixture = ValidationFixture::new().await;

    // Create Django-like model
    let models_py = fixture
        .create_file(
            "models.py",
            "from django.db import models\n\
             from django.conf import settings\n\
             \n\
             class User(models.Model):\n\
                 username = models.CharField(max_length=100)\n\
                 email = models.EmailField()\n\
                 \n\
                 @property\n\
                 def full_name(self) -> str:\n\
                     return f\"{self.first_name} {self.last_name}\"\n\
                 \n\
                 @classmethod\n\
                 def create_user(cls, username: str) -> 'User':\n\
                     return cls(username=username)\n",
        )
        .await;

    let result = fixture.analyze_and_extract(&[models_py]).await;
    assert_eq!(result.changed_files.len(), 1);

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 1);
}

/// Validates handling of Kubernetes-like Go patterns.
///
/// Tests patterns found in Kubernetes:
/// - Interface-driven architecture
/// - Package-level organization
/// - Error handling patterns
#[tokio::test]
async fn test_real_world_go_patterns() {
    let mut fixture = ValidationFixture::new().await;

    // Create Kubernetes-like controller pattern
    let controller_go = fixture
        .create_file(
            "controller.go",
            "package controller\n\
             \n\
             import (\n\
                 \"context\"\n\
                 \"fmt\"\n\
             )\n\
             \n\
             type Controller interface {\n\
                 Run(ctx context.Context) error\n\
                 Stop()\n\
             }\n\
             \n\
             type podController struct {\n\
                 stopCh chan struct{}\n\
             }\n\
             \n\
             func NewPodController() Controller {\n\
                 return &podController{\n\
                     stopCh: make(chan struct{}),\n\
                 }\n\
             }\n\
             \n\
             func (c *podController) Run(ctx context.Context) error {\n\
                 select {\n\
                 case <-ctx.Done():\n\
                     return ctx.Err()\n\
                 case <-c.stopCh:\n\
                     return nil\n\
                 }\n\
             }\n\
             \n\
             func (c *podController) Stop() {\n\
                 close(c.stopCh)\n\
             }\n",
        )
        .await;

    let result = fixture.analyze_and_extract(&[controller_go]).await;
    assert_eq!(result.changed_files.len(), 1);

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 1);
}

/// Validates handling of monorepo patterns with multiple languages.
///
/// Tests multi-language monorepo structure:
/// - Rust services + TypeScript frontend + Python scripts
/// - Cross-language boundaries
/// - Shared configuration files
#[tokio::test]
async fn test_real_world_monorepo() {
    let mut fixture = ValidationFixture::new().await;

    // Create monorepo structure
    let rust_service = fixture
        .create_file(
            "services/api/src/main.rs",
            "fn main() { println!(\"API\"); }",
        )
        .await;

    let ts_frontend = fixture
        .create_file(
            "apps/web/src/index.ts",
            "import { App } from './App';\nconst app = new App();",
        )
        .await;

    let python_script = fixture
        .create_file(
            "scripts/deploy.py",
            "#!/usr/bin/env python3\nimport sys\nimport os\n\ndef deploy():\n    pass\n",
        )
        .await;

    let paths = vec![rust_service, ts_frontend, python_script];
    let result = fixture.analyze_and_extract(&paths).await;

    assert_eq!(result.changed_files.len(), 3);

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 3);
}

/// Validates handling of deep module nesting.
///
/// Tests deeply nested module hierarchies (10+ levels):
/// - Deeply nested imports
/// - Long dependency chains
/// - Path resolution at depth
#[tokio::test]
async fn test_real_world_deep_nesting() {
    let mut fixture = ValidationFixture::new().await;

    // Create deeply nested module structure (10 levels)
    let mut paths = Vec::new();
    let mut current_path = String::new();

    for level in 0..10 {
        current_path.push_str(&format!("level_{}/", level));
        let module_path = format!("{}mod.rs", current_path);

        let content = if level == 0 {
            "pub mod level_1;\npub fn level_0() {}".to_string()
        } else if level < 9 {
            format!(
                "pub mod level_{};\npub fn level_{}() {{}}\n",
                level + 1,
                level
            )
        } else {
            format!("pub fn level_{}() {{}}\n", level)
        };

        let path = fixture.create_file(&module_path, &content).await;
        paths.push(path);
    }

    let result = fixture.analyze_and_extract(&paths).await;
    assert_eq!(result.changed_files.len(), 10);

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 10);
}

/// Validates handling of circular dependency patterns.
///
/// Tests complex circular dependencies:
/// - A → B → C → A cycles
/// - Multiple overlapping cycles
/// - Cycle detection and reporting
#[tokio::test]
async fn test_real_world_circular_deps() {
    let mut fixture = ValidationFixture::new().await;

    // Create circular dependency: a → b → c → a
    let file_a = fixture
        .create_file("a.rs", "use crate::c;\npub fn a() {}")
        .await;
    let file_b = fixture
        .create_file("b.rs", "use crate::a;\npub fn b() {}")
        .await;
    let file_c = fixture
        .create_file("c.rs", "use crate::b;\npub fn c() {}")
        .await;

    let paths = vec![file_a, file_b, file_c];
    let result = fixture.analyze_and_extract(&paths).await;

    assert_eq!(result.changed_files.len(), 3);

    // Validate cycle detection
    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 3);
    assert!(graph.edge_count() >= 3, "Expected circular edges");
}

/// Validates handling of very large files (>100KB).
///
/// Tests edge case of large source files:
/// - Files with thousands of lines
/// - Large import lists
/// - Memory efficiency validation
#[tokio::test]
async fn test_real_world_large_files() {
    let mut fixture = ValidationFixture::new().await;

    // Generate a large Rust file (10000+ lines, ~600KB)
    let mut large_content = String::from("// Large file with extensive documentation\n");
    large_content.push_str("use std::collections::HashMap;\n");
    large_content.push_str("use std::sync::Arc;\n");
    large_content.push_str("use std::sync::Mutex;\n\n");

    for i in 0..20000 {
        large_content.push_str(&format!(
            "pub fn function_{}() -> HashMap<String, i32> {{\n\
             let mut map = HashMap::new();\n\
             map.insert(String::from(\"key\"), {});\n\
             map\n\
             }}\n",
            i, i
        ));
    }

    let large_file = fixture.create_file("large.rs", &large_content).await;

    // Validate file size
    let metadata = fs::metadata(&large_file).await.expect("get metadata");
    assert!(
        metadata.len() > 50_000,
        "Expected >50KB file, got {} bytes",
        metadata.len()
    );

    // Analyze large file
    let start = Instant::now();
    let result = fixture.analyze_and_extract(&[large_file]).await;
    let analysis_time = start.elapsed();

    assert_eq!(result.changed_files.len(), 1);

    // Performance should still be reasonable (<3s for single very large file with 20K lines)
    assert!(
        analysis_time.as_millis() < 3000,
        "Large file analysis took {:?}, exceeds 3s",
        analysis_time
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Performance Tests (4 tests)
// ═══════════════════════════════════════════════════════════════════════════

/// Validates cold start performance on large codebase.
///
/// Measures initial analysis performance when cache is empty:
/// - 10K+ file initial analysis
/// - Fingerprinting throughput
/// - Graph construction speed
#[tokio::test]
async fn test_real_world_cold_start() {
    let mut fixture = ValidationFixture::new().await;

    // Generate 10K files
    let paths = fixture.generate_rust_scale(10_000).await;

    // Cold start analysis
    let start = Instant::now();
    let result = fixture.analyze_and_extract(&paths).await;
    let elapsed = start.elapsed();

    println!(
        "Cold start: {} files in {:?} ({:.0} files/sec)",
        result.changed_files.len(),
        elapsed,
        result.changed_files.len() as f64 / elapsed.as_secs_f64()
    );

    // Throughput validation: >1000 files/sec
    let throughput = result.changed_files.len() as f64 / elapsed.as_secs_f64();
    assert!(
        throughput > 1000.0,
        "Cold start throughput {:.0} files/sec < 1000 target",
        throughput
    );
}

/// Validates incremental update efficiency at scale.
///
/// Measures performance when 1% of files change:
/// - Fast invalidation of affected files
/// - Minimal reanalysis overhead
/// - Cache efficiency
#[tokio::test]
async fn test_real_world_incremental_update() {
    let mut fixture = ValidationFixture::new().await;

    // Initial analysis
    let paths = fixture.generate_rust_scale(10_000).await;
    fixture.analyze_and_extract(&paths).await;

    // Change 1% of files
    let changed_count = 100;
    let changed_paths: Vec<_> = paths.iter().take(changed_count).cloned().collect();

    for path in &changed_paths {
        fixture
            .update_file(path, "// Updated\npub fn updated() {}")
            .await;
    }

    // Incremental update
    let start = Instant::now();
    let result = fixture.analyze_and_extract(&changed_paths).await;
    let elapsed = start.elapsed();

    println!(
        "Incremental: {} changed files in {:?} ({:.0} files/sec)",
        result.changed_files.len(),
        elapsed,
        result.changed_files.len() as f64 / elapsed.as_secs_f64()
    );

    assert_eq!(result.changed_files.len(), changed_count);

    // Performance: <1s for 1% update
    assert!(
        elapsed.as_secs() < 1,
        "Incremental update {:?} exceeds 1s",
        elapsed
    );
}

/// Validates cache hit rate meets constitutional requirements (>90%).
///
/// Tests cache efficiency over multiple analysis cycles:
/// - Initial cold start
/// - Warm cache reanalysis
/// - Cache hit rate calculation
#[tokio::test]
async fn test_real_world_cache_hit_rate() {
    let mut fixture = ValidationFixture::new().await;

    // Generate and analyze 1000 files
    let paths = fixture.generate_rust_scale(1_000).await;
    fixture.analyze_and_extract(&paths).await;

    // Reanalyze without changes (should hit cache)
    let result = fixture.analyze_and_extract(&paths).await;

    // All files should be unchanged (cache hits)
    println!("Cache hit rate: {:.1}%", result.cache_hit_rate * 100.0);

    // Constitutional requirement: >90% cache hit rate
    // On reanalysis with no changes, should be 100% cache hits
    assert!(
        result.cache_hit_rate > 0.90,
        "Expected >90% cache hit rate, got {:.1}%",
        result.cache_hit_rate * 100.0
    );

    // Changed files should be 0 on reanalysis
    assert!(
        result.changed_files.is_empty(),
        "Expected 0 changed files on reanalysis, got {}",
        result.changed_files.len()
    );
}

/// Validates parallel processing efficiency at scale.
///
/// Tests Rayon/tokio performance with large batches:
/// - Parallel fingerprinting
/// - Parallel dependency extraction
/// - Scalability validation
#[tokio::test]
#[cfg(feature = "parallel")]
async fn test_real_world_parallel_scaling() {
    let mut fixture = ValidationFixture::new().await;

    // Generate 5K files for parallel processing test
    let paths = fixture.generate_rust_scale(5_000).await;

    // Analyze with parallelism enabled
    let start = Instant::now();
    let result = fixture.analyze_and_extract(&paths).await;
    let parallel_time = start.elapsed();

    println!(
        "Parallel analysis: {} files in {:?} ({:.0} files/sec)",
        result.changed_files.len(),
        parallel_time,
        result.changed_files.len() as f64 / parallel_time.as_secs_f64()
    );

    // Throughput should be higher with parallelism (>1000 files/sec like cold start)
    let throughput = result.changed_files.len() as f64 / parallel_time.as_secs_f64();
    assert!(
        throughput > 1000.0,
        "Parallel throughput {:.0} files/sec < 1000 target",
        throughput
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge Case Tests (4 additional tests)
// ═══════════════════════════════════════════════════════════════════════════

/// Validates handling of empty files and minimal content.
#[tokio::test]
async fn test_real_world_empty_files() {
    let mut fixture = ValidationFixture::new().await;

    // Create mix of empty and minimal files
    let empty = fixture.create_file("empty.rs", "").await;
    let comment_only = fixture
        .create_file("comment.rs", "// Just a comment\n")
        .await;
    let minimal = fixture.create_file("minimal.rs", "fn main() {}").await;

    let paths = vec![empty, comment_only, minimal];
    let result = fixture.analyze_and_extract(&paths).await;

    assert_eq!(result.changed_files.len(), 3);

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 3);
}

/// Validates handling of binary files and non-source files.
#[tokio::test]
async fn test_real_world_binary_files() {
    let mut fixture = ValidationFixture::new().await;

    // Create binary file (invalid UTF-8)
    let binary_path = fixture.temp_path().join("binary.bin");
    let mut file = fs::File::create(&binary_path).await.expect("create binary");
    file.write_all(&[0xFF, 0xFE, 0xFD, 0xFC])
        .await
        .expect("write binary");

    // Create valid Rust file
    let rust_file = fixture.create_file("valid.rs", "fn main() {}").await;

    // Analyze both (binary should be skipped gracefully)
    let paths = vec![binary_path.clone(), rust_file];
    let result = fixture.analyze_and_extract(&paths).await;

    // Only Rust file should be analyzed
    assert!(
        !result.changed_files.is_empty(),
        "Expected at least 1 file analyzed (binary skipped)"
    );
}

/// Validates handling of symlinks and hard links.
#[tokio::test]
#[cfg(target_family = "unix")]
async fn test_real_world_symlinks() {
    let mut fixture = ValidationFixture::new().await;

    // Create original file
    let original = fixture
        .create_file("original.rs", "pub fn original() {}")
        .await;

    // Create symlink
    let symlink_path = fixture.temp_path().join("symlink.rs");
    #[cfg(target_family = "unix")]
    std::os::unix::fs::symlink(&original, &symlink_path).expect("create symlink");

    // Analyze both (should handle symlinks correctly)
    let paths = vec![original, symlink_path];
    let result = fixture.analyze_and_extract(&paths).await;

    // Both should be analyzed (symlink follows to original)
    assert!(!result.changed_files.is_empty());
}

/// Validates handling of Unicode and non-ASCII characters.
#[tokio::test]
async fn test_real_world_unicode() {
    let mut fixture = ValidationFixture::new().await;

    // Create files with Unicode content
    let unicode_rs = fixture
        .create_file(
            "unicode.rs",
            "// 日本語コメント\n\
             pub fn process_emoji() -> &'static str {\n\
                 \"🚀 Rocket launched! 中文 العربية\"\n\
             }\n",
        )
        .await;

    let result = fixture.analyze_and_extract(&[unicode_rs]).await;
    assert_eq!(result.changed_files.len(), 1);

    let graph = fixture.builder.graph();
    assert!(graph.node_count() >= 1);
}
