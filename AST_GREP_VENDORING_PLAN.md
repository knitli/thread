# ast-grep CLI Vendoring Plan: Service-Oriented Library Architecture

## Executive Summary

This document outlines a comprehensive plan for vendoring the ast-grep CLI crate as a service-oriented library component. The architecture extracts core functionality from CLI assumptions and transforms it into individual functions (`scan_files()`, `search_pattern()`, `apply_fixes()`) with configurable input/output adapters, making it suitable for diverse environments including CLI, Cloudflare Workers, HTTP APIs, and CI/CD systems.

## Table of Contents

1. [Current CLI Architecture Analysis](#current-cli-architecture-analysis)
2. [Proposed Library Architecture](#proposed-library-architecture)
3. [Core Functionality Extraction](#core-functionality-extraction)
4. [Public API Design](#public-api-design)
5. [Configuration System](#configuration-system)
6. [Error Handling Strategy](#error-handling-strategy)
7. [Async/Await Support](#asyncawait-support)
8. [Tower Service Integration](#tower-service-integration)
9. [Testing Strategy](#testing-strategy)
10. [Migration Path](#migration-path)
11. [Implementation Phases](#implementation-phases)

## Current CLI Architecture Analysis

### Key Components Identified

The CLI crate (`ast-grep/crates/cli/`) contains these core abstractions that need extraction:

#### 1. Worker System (`utils/worker.rs`)
- **Purpose**: Multiple-producer-single-consumer pattern for processing
- **Key Traits**: `Worker`, `PathWorker`, `StdInWorker`
- **CLI Dependencies**: File system assumptions, threading model
- **Extract**: Processing logic without file I/O coupling

#### 2. Print System (`print/mod.rs`)
- **Purpose**: Output formatting with `Printer` and `PrintProcessor` traits
- **Key Components**: `ColoredPrinter`, `JSONPrinter`, `InteractivePrinter`
- **CLI Dependencies**: Terminal assumptions, file path coupling
- **Extract**: Format generation without output destination coupling

#### 3. Input Processing (`utils/mod.rs`)
- **Purpose**: File discovery and filtering
- **Key Functions**: `filter_file_rule()`, `filter_file_pattern()`
- **CLI Dependencies**: File system walker, ignore files
- **Extract**: Content processing logic without file system assumptions

#### 4. Configuration Management (`config.rs`)
- **Purpose**: Project config discovery and rule loading
- **Key Components**: `ProjectConfig`, rule directory walking
- **CLI Dependencies**: File system discovery, sgconfig.yml assumptions
- **Extract**: Configuration logic with pluggable sources

#### 5. Core Operations
- **Scanning** (`scan.rs`): Rule-based analysis via `ScanWithConfig` and `ScanStdin`
- **Pattern Matching** (`run.rs`): Pattern-based search via `RunWithSpecificLang` and `RunWithInferredLang`
- **Fixing** (`print/mod.rs`): Code replacement via `Diff` generation

### Dependencies to Remove

- `clap` argument parsing structures (`ScanArg`, `RunArg`)
- File system discovery (`WalkParallel`, `ignore` crate usage)
- Direct printer coupling and terminal assumptions
- Project config file system assumptions
- Threading model assumptions (`Worker` trait constraints)

## Proposed Library Architecture

### 1. Trait-Based Adapter System

```rust
// Input Sources
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    type Item: Send + Sync;
    type Error: std::error::Error + Send + Sync;

    async fn read_items(&self) -> Result<Vec<Self::Item>, Self::Error>;
}

#[async_trait]
pub trait ContentProvider: Send + Sync {
    async fn get_content(&self, identifier: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_language(&self, identifier: &str) -> Option<SgLang>;
}

// Output Destinations
#[async_trait]
pub trait SinkAdapter<T>: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    async fn write_results(&self, results: Vec<T>) -> Result<(), Self::Error>;
}

// Configuration Sources
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    async fn get_rules(&self) -> Result<RuleCollection<SgLang>, Box<dyn std::error::Error>>;
    async fn get_project_config(&self) -> Result<ProjectSettings, Box<dyn std::error::Error>>;
}
```

### 2. Core Library Functions

```rust
// Individual operation functions
pub async fn scan_files<S, C, O>(
    source: S,
    config: C,
    output: O,
    options: ScanOptions,
) -> Result<ScanResults, AstGrepError>
where
    S: SourceAdapter + ContentProvider,
    C: ConfigProvider,
    O: SinkAdapter<ScanMatch>,

pub async fn search_pattern<S, O>(
    source: S,
    pattern: &str,
    language: Option<SgLang>,
    output: O,
    options: SearchOptions,
) -> Result<SearchResults, AstGrepError>
where
    S: SourceAdapter + ContentProvider,
    O: SinkAdapter<PatternMatch>,

pub async fn apply_fixes<S, O>(
    source: S,
    fixes: Vec<FixInstruction>,
    output: O,
    options: FixOptions,
) -> Result<FixResults, AstGrepError>
where
    S: SourceAdapter + ContentProvider,
    O: SinkAdapter<FixedContent>,
```

### 3. Adapter Implementations

```rust
// File System Adapter
pub struct FileSystemSource {
    paths: Vec<PathBuf>,
    globs: Vec<String>,
    ignore_config: IgnoreConfig,
}

// HTTP API Adapter
pub struct HttpSource {
    client: reqwest::Client,
    endpoints: Vec<String>,
}

// In-Memory Adapter
pub struct MemorySource {
    items: Vec<(String, String, Option<SgLang>)>, // id, content, lang
}

// Cloudflare Workers Adapter
pub struct CloudflareSource {
    kv_namespace: String,
    // KV store access, etc.
}

// Standard Output Adapter
pub struct StdoutSink<T> {
    formatter: Box<dyn Formatter<T>>,
}

// HTTP Response Adapter
pub struct HttpResponseSink<T> {
    response_builder: ResponseBuilder<T>,
}
```

## Core Functionality Extraction

### Scanning Functionality (`scan.rs` → `ScanEngine`)

**Extract from `ScanWithConfig`:**
```rust
pub struct ScanEngine<C: ConfigProvider> {
    config_provider: C,
    unused_suppression_rule: RuleConfig<SgLang>,
}

impl<C: ConfigProvider> ScanEngine<C> {
    pub async fn scan_content(
        &self,
        content: &str,
        path: &str,
        language: SgLang,
        options: &ScanOptions,
    ) -> Result<Vec<ScanMatch>, AstGrepError> {
        // Extract logic from produce_item without file I/O
        let grep = language.ast_grep(content);
        let rules = self.config_provider.get_rules_for_lang(language).await?;

        let mut combined = CombinedScan::new(rules.iter().collect());
        combined.set_unused_suppression_rule(&self.unused_suppression_rule);

        let scanned = combined.scan(&grep, options.interactive);
        Ok(self.process_scan_results(scanned, path))
    }
}
```

**CLI Dependencies Removed:**
- `ScanArg` command line arguments
- `ProjectConfig` file-based discovery
- Direct `Printer` coupling
- `Worker` trait threading assumptions

### Pattern Matching Functionality (`run.rs` → `SearchEngine`)

**Extract from `RunWithSpecificLang`/`RunWithInferredLang`:**
```rust
pub struct SearchEngine {
    strictness: Option<MatchStrictness>,
}

impl SearchEngine {
    pub async fn search_content(
        &self,
        content: &str,
        pattern: &str,
        language: SgLang,
        selector: Option<&str>,
        rewrite: Option<&str>,
    ) -> Result<Vec<PatternMatch>, AstGrepError> {
        // Extract from build_pattern + match_one_file
        let pattern = if let Some(sel) = selector {
            Pattern::contextual(pattern, sel, language)
        } else {
            Pattern::try_new(pattern, language)
        }?;

        if let Some(strictness) = &self.strictness {
            pattern = pattern.with_strictness(strictness.clone());
        }

        let grep = language.ast_grep(content);
        let root = grep.root();
        let matches = root.find_all(&pattern).collect::<Vec<_>>();

        // Process rewrite if provided
        let results = if let Some(rewrite_str) = rewrite {
            let fixer = Fixer::from_str(rewrite_str, &language)?;
            self.generate_diffs(matches, &pattern, &fixer)
        } else {
            self.generate_matches(matches)
        };

        Ok(results)
    }
}
```

**CLI Dependencies Removed:**
- `RunArg` command line arguments
- `SgLang::from_path()` file system assumptions
- `Worker` trait threading
- Direct printer coupling

### Fixing Functionality (`print/mod.rs` + `fixer.rs` → `FixEngine`)

**Extract from `Diff` generation:**
```rust
pub struct FixEngine {
    dry_run: bool,
}

impl FixEngine {
    pub async fn apply_fixes(
        &self,
        content: &str,
        language: SgLang,
        fix_instructions: &[FixInstruction],
    ) -> Result<FixResult, AstGrepError> {
        let grep = language.ast_grep(content);
        let mut all_diffs = Vec::new();

        for instruction in fix_instructions {
            let pattern = Pattern::try_new(&instruction.pattern, language)?;
            let fixer = Fixer::from_str(&instruction.replacement, &language)?;

            let matches = grep.root().find_all(&pattern);
            for node_match in matches {
                let diff = Diff::generate(node_match, &pattern, &fixer);
                all_diffs.push(diff);
            }
        }

        if self.dry_run {
            Ok(FixResult::DryRun(all_diffs))
        } else {
            let modified_content = self.apply_diffs_to_content(content, all_diffs)?;
            Ok(FixResult::Applied(modified_content))
        }
    }
}
```

**CLI Dependencies Removed:**
- `PrintProcessor` trait coupling
- Path-based processing assumptions

## Public API Design

### Main Library Interface

```rust
pub struct AstGrepLibrary<S, C, O>
where
    S: SourceAdapter + ContentProvider,
    C: ConfigProvider,
    O: SinkAdapter<ScanMatch> + SinkAdapter<PatternMatch> + SinkAdapter<FixResult>,
{
    source: S,
    config: C,
    output: O,
    scan_engine: ScanEngine<C>,
    search_engine: SearchEngine,
    fix_engine: FixEngine,
}

impl<S, C, O> AstGrepLibrary<S, C, O>
where
    S: SourceAdapter + ContentProvider + Clone,
    C: ConfigProvider + Clone,
    O: SinkAdapter<ScanMatch> + SinkAdapter<PatternMatch> + SinkAdapter<FixResult> + Clone,
{
    pub fn new(source: S, config: C, output: O) -> Self {
        Self {
            source: source.clone(),
            config: config.clone(),
            output: output.clone(),
            scan_engine: ScanEngine::new(config.clone()),
            search_engine: SearchEngine::new(),
            fix_engine: FixEngine::new(),
        }
    }

    // Individual operation functions
    pub async fn scan_files(&self, options: ScanOptions) -> Result<ScanResults, AstGrepError> {
        let items = self.source.read_items().await?;
        let mut all_results = Vec::new();

        for item in items {
            let content = self.source.get_content(&item.identifier).await?;
            let language = self.source.get_language(&item.identifier)
                .await
                .ok_or_else(|| AstGrepError::Language("Cannot determine language".into()))?;

            let matches = self.scan_engine
                .scan_content(&content, &item.identifier, language, &options)
                .await?;

            all_results.extend(matches);
        }

        self.output.write_results(all_results.clone()).await?;
        Ok(ScanResults { matches: all_results })
    }

    pub async fn search_pattern(
        &self,
        pattern: &str,
        language: Option<SgLang>,
        options: SearchOptions,
    ) -> Result<SearchResults, AstGrepError> {
        let items = self.source.read_items().await?;
        let mut all_results = Vec::new();

        for item in items {
            let content = self.source.get_content(&item.identifier).await?;
            let lang = if let Some(l) = language {
                l
            } else {
                self.source.get_language(&item.identifier)
                    .await
                    .ok_or_else(|| AstGrepError::Language("Cannot infer language".into()))?
            };

            let matches = self.search_engine
                .search_content(&content, pattern, lang, options.selector.as_deref(), None)
                .await?;

            all_results.extend(matches);
        }

        self.output.write_results(all_results.clone()).await?;
        Ok(SearchResults { matches: all_results })
    }

    pub async fn apply_fixes(
        &self,
        fixes: Vec<FixInstruction>,
        options: FixOptions,
    ) -> Result<FixResults, AstGrepError> {
        let items = self.source.read_items().await?;
        let mut all_results = Vec::new();

        for item in items {
            let content = self.source.get_content(&item.identifier).await?;
            let language = self.source.get_language(&item.identifier)
                .await
                .ok_or_else(|| AstGrepError::Language("Cannot determine language".into()))?;

            let result = self.fix_engine
                .apply_fixes(&content, language, &fixes)
                .await?;

            all_results.push(FixedItem {
                identifier: item.identifier.clone(),
                result,
            });
        }

        self.output.write_results(all_results.clone()).await?;
        Ok(FixResults { items: all_results })
    }
}
```

### Option Structures

```rust
#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub severity_filter: Option<Severity>,
    pub rule_filter: Option<Regex>,
    pub context: (u16, u16), // before, after
    pub include_metadata: bool,
    pub interactive: bool,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub strictness: Option<MatchStrictness>,
    pub selector: Option<String>,
    pub context: (u16, u16),
}

#[derive(Debug, Clone)]
pub struct FixOptions {
    pub dry_run: bool,
    pub interactive: bool,
}
```

## Configuration System

### Abstract Configuration Provider

```rust
#[derive(Debug, Clone)]
pub struct ProjectSettings {
    pub custom_languages: HashMap<String, CustomLang>,
    pub language_globs: Option<LanguageGlobs>,
    pub language_injections: Vec<SerializableInjection>,
    pub rule_overrides: RuleOverwrite,
}

// Various config provider implementations
pub struct FileConfigProvider {
    config_path: Option<PathBuf>,
    rule_dirs: Vec<PathBuf>,
    util_dirs: Option<Vec<PathBuf>>,
}

pub struct InMemoryConfigProvider {
    rules: RuleCollection<SgLang>,
    settings: ProjectSettings,
    global_rules: GlobalRules,
}

pub struct HttpConfigProvider {
    client: reqwest::Client,
    config_endpoints: ConfigEndpoints,
}

pub struct DatabaseConfigProvider {
    connection: DatabaseConnection,
    cache: Arc<RwLock<ConfigCache>>,
}

#[derive(Debug, Clone)]
pub struct ConfigEndpoints {
    pub rules_url: String,
    pub settings_url: String,
    pub global_rules_url: String,
}
```

## Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum AstGrepError {
    #[error("Source adapter error: {source}")]
    Source {
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
    },

    #[error("Configuration error: {source}")]
    Config {
        source: Box<dyn std::error::Error + Send + Sync>,
        config_type: ConfigType,
    },

    #[error("Language processing error: {message}")]
    Language {
        message: String,
        language: Option<SgLang>,
        content_sample: Option<String>,
    },

    #[error("Pattern parsing error: {message}")]
    Pattern {
        message: String,
        pattern: String,
        language: SgLang,
    },

    #[error("Fix application error: {message}")]
    Fix {
        message: String,
        fix_instruction: String,
        original_content: String,
    },

    #[error("Output adapter error: {source}")]
    Output {
        source: Box<dyn std::error::Error + Send + Sync>,
        output_type: String,
    },

    #[error("Async runtime error: {message}")]
    Runtime {
        message: String,
    },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: String,
    },
}

#[derive(Debug, Clone)]
pub enum ConfigType {
    Rules,
    ProjectSettings,
    GlobalRules,
    LanguageSettings,
}

// Error context helpers
impl AstGrepError {
    pub fn source_error(err: impl std::error::Error + Send + Sync + 'static, context: &str) -> Self {
        Self::Source {
            source: Box::new(err),
            context: context.to_string(),
        }
    }

    pub fn config_error(err: impl std::error::Error + Send + Sync + 'static, config_type: ConfigType) -> Self {
        Self::Config {
            source: Box::new(err),
            config_type,
        }
    }

    pub fn language_error(message: &str, language: Option<SgLang>) -> Self {
        Self::Language {
            message: message.to_string(),
            language,
            content_sample: None,
        }
    }
}

pub type AstGrepResult<T> = Result<T, AstGrepError>;
```

## Async/Await Support

### Environment-Specific Runtime Configuration

```rust
#[derive(Debug, Clone)]
pub enum RuntimeEnvironment {
    Tokio,           // Standard async runtime
    CloudflareWorkers, // CF Workers runtime
    WasmWorkers,     // WASM-based workers
    SingleThreaded,  // Synchronous fallback
}

#[async_trait]
pub trait AsyncRuntime: Send + Sync {
    async fn spawn_task<F, T>(&self, future: F) -> Result<T, AstGrepError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;

    async fn timeout<F, T>(&self, duration: Duration, future: F) -> Result<T, AstGrepError>
    where
        F: Future<Output = T> + Send,
        T: Send;

    fn block_on<F, T>(&self, future: F) -> Result<T, AstGrepError>
    where
        F: Future<Output = T>;
}

// Cloudflare Workers specific runtime
pub struct CloudflareRuntime;

#[async_trait]
impl AsyncRuntime for CloudflareRuntime {
    async fn spawn_task<F, T>(&self, future: F) -> Result<T, AstGrepError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // CF Workers don't support spawning tasks in the traditional sense
        // Execute immediately
        Ok(future.await)
    }

    async fn timeout<F, T>(&self, duration: Duration, future: F) -> Result<T, AstGrepError>
    where
        F: Future<Output = T> + Send,
        T: Send,
    {
        // Use CF Workers timeout mechanisms
        Ok(future.await)
    }

    fn block_on<F, T>(&self, future: F) -> Result<T, AstGrepError>
    where
        F: Future<Output = T>,
    {
        Err(AstGrepError::Runtime {
            message: "block_on not supported in Cloudflare Workers".to_string(),
        })
    }
}

// Library with runtime configuration
pub struct AstGrepLibraryBuilder<S, C, O> {
    source: Option<S>,
    config: Option<C>,
    output: Option<O>,
    runtime: Box<dyn AsyncRuntime>,
    environment: RuntimeEnvironment,
}

impl<S, C, O> AstGrepLibraryBuilder<S, C, O> {
    pub fn new() -> Self {
        Self {
            source: None,
            config: None,
            output: None,
            runtime: Box::new(TokioRuntime::default()),
            environment: RuntimeEnvironment::Tokio,
        }
    }

    pub fn with_cloudflare_runtime(mut self) -> Self {
        self.runtime = Box::new(CloudflareRuntime);
        self.environment = RuntimeEnvironment::CloudflareWorkers;
        self
    }
}
```

## Tower Service Integration

### Request/Response Types

```rust
#[derive(Debug, Clone)]
pub struct ScanRequest {
    pub options: ScanOptions,
    pub items: Option<Vec<SourceItem>>, // Override source items if provided
}

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub pattern: String,
    pub language: Option<SgLang>,
    pub options: SearchOptions,
    pub items: Option<Vec<SourceItem>>,
}

#[derive(Debug, Clone)]
pub struct FixRequest {
    pub fixes: Vec<FixInstruction>,
    pub options: FixOptions,
    pub items: Option<Vec<SourceItem>>,
}
```

### Tower Service Implementations

```rust
pub struct AstGrepScanService<S, C, O> {
    library: Arc<AstGrepLibrary<S, C, O>>,
}

#[async_trait]
impl<S, C, O> tower::Service<ScanRequest> for AstGrepScanService<S, C, O>
where
    S: SourceAdapter + ContentProvider + Clone + 'static,
    C: ConfigProvider + Clone + 'static,
    O: SinkAdapter<ScanMatch> + Clone + 'static,
{
    type Response = ScanResults;
    type Error = AstGrepError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ScanRequest) -> Self::Future {
        let library = self.library.clone();

        Box::pin(async move {
            library.scan_files(req.options).await
        })
    }
}

// Service composition and middleware
pub struct AstGrepServiceStack<S, C, O> {
    scan_service: AstGrepScanService<S, C, O>,
    search_service: AstGrepSearchService<S, C, O>,
    fix_service: AstGrepFixService<S, C, O>,
}

impl<S, C, O> AstGrepServiceStack<S, C, O>
where
    S: SourceAdapter + ContentProvider + Clone + 'static,
    C: ConfigProvider + Clone + 'static,
    O: SinkAdapter<ScanMatch> + SinkAdapter<PatternMatch> + SinkAdapter<FixResult> + Clone + 'static,
{
    pub fn new(library: AstGrepLibrary<S, C, O>) -> Self {
        let library = Arc::new(library);

        Self {
            scan_service: AstGrepScanService { library: library.clone() },
            search_service: AstGrepSearchService { library: library.clone() },
            fix_service: AstGrepFixService { library },
        }
    }

    // Add middleware layers
    pub fn with_timeout(self, timeout: Duration) -> TimeoutLayer<Self> {
        TimeoutLayer::new(timeout, self)
    }

    pub fn with_rate_limiting(self, rate: u64) -> RateLimitLayer<Self> {
        RateLimitLayer::new(rate, self)
    }

    pub fn with_metrics(self) -> MetricsLayer<Self> {
        MetricsLayer::new(self)
    }
}
```

### Router for Multiple Request Types

```rust
pub enum AstGrepRequest {
    Scan(ScanRequest),
    Search(SearchRequest),
    Fix(FixRequest),
}

pub enum AstGrepResponse {
    Scan(ScanResults),
    Search(SearchResults),
    Fix(FixResults),
}

pub struct AstGrepRouter<S, C, O> {
    services: AstGrepServiceStack<S, C, O>,
}

#[async_trait]
impl<S, C, O> tower::Service<AstGrepRequest> for AstGrepRouter<S, C, O>
where
    S: SourceAdapter + ContentProvider + Clone + 'static,
    C: ConfigProvider + Clone + 'static,
    O: SinkAdapter<ScanMatch> + SinkAdapter<PatternMatch> + SinkAdapter<FixResult> + Clone + 'static,
{
    type Response = AstGrepResponse;
    type Error = AstGrepError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, req: AstGrepRequest) -> Self::Future {
        let mut services = self.services.clone();

        Box::pin(async move {
            match req {
                AstGrepRequest::Scan(scan_req) => {
                    let result = services.scan_service.call(scan_req).await?;
                    Ok(AstGrepResponse::Scan(result))
                }
                AstGrepRequest::Search(search_req) => {
                    let result = services.search_service.call(search_req).await?;
                    Ok(AstGrepResponse::Search(result))
                }
                AstGrepRequest::Fix(fix_req) => {
                    let result = services.fix_service.call(fix_req).await?;
                    Ok(AstGrepResponse::Fix(result))
                }
            }
        })
    }
}
```

## Testing Strategy

### Test Infrastructure

```rust
#[cfg(test)]
mod test_infrastructure {
    use super::*;

    // Mock adapters for testing
    pub struct MockSourceAdapter {
        pub items: Vec<SourceItem>,
        pub content_map: HashMap<String, String>,
        pub language_map: HashMap<String, SgLang>,
        pub should_fail: bool,
    }

    #[async_trait]
    impl SourceAdapter for MockSourceAdapter {
        type Item = SourceItem;
        type Error = Box<dyn std::error::Error + Send + Sync>;

        async fn read_items(&self) -> Result<Vec<Self::Item>, Self::Error> {
            if self.should_fail {
                return Err("Mock source failure".into());
            }
            Ok(self.items.clone())
        }
    }

    #[async_trait]
    impl ContentProvider for MockSourceAdapter {
        async fn get_content(&self, identifier: &str) -> Result<String, Box<dyn std::error::Error>> {
            self.content_map.get(identifier)
                .cloned()
                .ok_or_else(|| format!("Content not found: {}", identifier).into())
        }

        async fn get_language(&self, identifier: &str) -> Option<SgLang> {
            self.language_map.get(identifier).cloned()
        }
    }
}

// Component-specific tests
#[cfg(test)]
mod scan_engine_tests {
    use super::*;
    use test_infrastructure::*;

    #[tokio::test]
    async fn test_scan_engine_basic_functionality() {
        let config = MockConfigProvider {
            rules: create_test_rules(),
            settings: ProjectSettings::default(),
            should_fail: false,
        };

        let engine = ScanEngine::new(config);
        let content = "function test() { console.log('hello'); }";

        let results = engine
            .scan_content(content, "test.js", SgLang::JavaScript, &ScanOptions::default())
            .await
            .unwrap();

        assert!(!results.is_empty());
    }
}

// Integration tests
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_full_library_integration() {
        let source = MockSourceAdapter {
            items: vec![
                SourceItem { identifier: "test.js".to_string() },
                SourceItem { identifier: "test.rs".to_string() },
            ],
            content_map: [
                ("test.js".to_string(), "console.log('test')".to_string()),
                ("test.rs".to_string(), "fn main() { println!(\"test\"); }".to_string()),
            ].into_iter().collect(),
            language_map: [
                ("test.js".to_string(), SgLang::JavaScript),
                ("test.rs".to_string(), SgLang::Rust),
            ].into_iter().collect(),
            should_fail: false,
        };

        let library = AstGrepLibrary::new(source, config, output);
        let results = library.scan_files(ScanOptions::default()).await.unwrap();
        assert!(!results.matches.is_empty());
    }
}

// Property-based testing
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_search_engine_never_panics(
            pattern in "\\PC{0,50}",
            content in "\\PC{0,200}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let engine = SearchEngine::new();
                let _ = engine.search_content(
                    &content,
                    &pattern,
                    SgLang::JavaScript,
                    None,
                    None
                ).await;
                // Should not panic regardless of input
            });
        }
    }
}
```

## Migration Path

### Compatibility Wrapper

```rust
pub mod migration {
    use super::*;

    // Compatibility wrapper for existing CLI code
    pub struct CliCompatibilityWrapper<S, C, O> {
        library: AstGrepLibrary<S, C, O>,
    }

    impl<S, C, O> CliCompatibilityWrapper<S, C, O>
    where
        S: SourceAdapter + ContentProvider + Clone,
        C: ConfigProvider + Clone,
        O: SinkAdapter<ScanMatch> + SinkAdapter<PatternMatch> + SinkAdapter<FixResult> + Clone,
    {
        // Migrate existing scan command
        pub async fn run_scan_command(
            &self,
            scan_arg: ScanArg,
        ) -> Result<(), AstGrepError> {
            let options = ScanOptions {
                severity_filter: None, // Extract from scan_arg.overwrite
                rule_filter: scan_arg.overwrite.filter.clone(),
                context: scan_arg.context.get(),
                include_metadata: scan_arg.include_metadata,
            };

            self.library.scan_files(options).await?;
            Ok(())
        }

        // Migrate existing run command
        pub async fn run_pattern_command(
            &self,
            run_arg: RunArg,
        ) -> Result<(), AstGrepError> {
            let options = SearchOptions {
                strictness: run_arg.strictness.map(|s| s.0),
                selector: run_arg.selector,
                context: run_arg.context.get(),
            };

            self.library.search_pattern(
                &run_arg.pattern,
                run_arg.lang,
                options,
            ).await?;
            Ok(())
        }
    }
}
```

### Step-by-Step Migration Guide

1. **Replace CLI Commands with Library Functions:**

   Before (CLI):
   ```bash
   sg scan --rule my-rule.yml src/
   ```

   After (Library):
   ```rust
   let source = FileSystemSource::new(vec!["src/".into()])?;
   let config = FileConfigProvider::from_rule_file("my-rule.yml")?;
   let output = StandardOutputSink::colored();

   let library = AstGrepLibrary::new(source, config, output);
   library.scan_files(ScanOptions::default()).await?;
   ```

2. **Replace Pattern Matching:**

   Before (CLI):
   ```bash
   sg -p "console.log($MSG)" -l js src/
   ```

   After (Library):
   ```rust
   library.search_pattern(
       "console.log($MSG)",
       Some(SgLang::JavaScript),
       SearchOptions::default()
   ).await?;
   ```

3. **Replace Fix Operations:**

   Before (CLI):
   ```bash
   sg -p "console.log($MSG)" -r "logger.info($MSG)" -U src/
   ```

   After (Library):
   ```rust
   let fixes = vec![FixInstruction {
       pattern: "console.log($MSG)".to_string(),
       replacement: "logger.info($MSG)".to_string(),
   }];

   library.apply_fixes(fixes, FixOptions { dry_run: false, interactive: false }).await?;
   ```

## Implementation Phases

### Phase 1: Core Engines (Weeks 1-2)
- Extract `ScanEngine` from `scan.rs`
- Extract `SearchEngine` from `run.rs`
- Extract `FixEngine` from print/diff logic
- Basic error handling implementation
- Unit tests for core engines

### Phase 2: Basic Adapters (Weeks 3-4)
- Implement `FileSystemSource` and `FileConfigProvider`
- Implement `MemorySource` and `InMemoryConfigProvider`
- Implement `StdoutSink` and basic formatters
- Integration tests with file system adapters

### Phase 3: Tower Services (Weeks 5-6)
- Implement Tower service traits for all engines
- Create service stack and router
- Add basic middleware (timeout, logging)
- Service integration tests

### Phase 4: Advanced Adapters (Weeks 7-8)
- Implement `HttpSource` and `HttpResponseSink`
- Implement `CloudflareSource` for CF Workers
- Async runtime abstraction
- Environment-specific optimizations

### Phase 5: Production Features (Weeks 9-10)
- Advanced middleware (rate limiting, metrics)
- Comprehensive error handling and recovery
- Performance optimizations
- Documentation and migration guides

## Environment Support Matrix

| Environment | Input Source | Config Source | Output Destination |
|-------------|--------------|---------------|-------------------|
| **CLI** | File system | sgconfig.yml | Terminal/Files |
| **CF Workers** | HTTP/KV store | API/Database | HTTP Response |
| **CI/CD** | Git/File system | API/Config files | API/Reports |
| **Web Service** | HTTP requests | Database | JSON API |
| **WASM** | In-memory | Embedded config | Return values |

## Conclusion

This architecture successfully extracts ast-grep's powerful AST-based code analysis capabilities from CLI assumptions and transforms them into a service-oriented library suitable for diverse environments. The design provides:

- **Complete separation** from CLI dependencies
- **Individual functions** as requested (`scan_files`, `search_pattern`, `apply_fixes`)
- **Flexible I/O** via trait-based adapters
- **Environment agnostic** async support
- **Tower ecosystem** integration with middleware
- **Robust error handling** for library usage
- **Comprehensive testing** strategy
- **Clear migration path** from CLI to library

The vendored library maintains all the sophisticated parsing, pattern matching, and code transformation functionality while being perfectly suited for Tower-based service ecosystems and diverse deployment environments including Cloudflare Workers, HTTP APIs, and CI/CD systems.
