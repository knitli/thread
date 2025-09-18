# Thread Architecture 2.0: Evolved Design Plan

Based on my analysis of your original PLAN.md vision and the current codebase, here's a comprehensive new architecture that evolves from where you are to where you want to be.

## Executive Summary

**Current State:** Excellent ast-grep integration foundation with performance optimizations
**Target State:** Intelligent code analysis engine bridging AI expectations with reality

**Mission Alignment:** Making tools that bridge the gap between what people expect from AI and what AI can provide, with focus on accessibility and making difficult things simple and powerful.

**Key Gaps:**
1. Missing AI context intelligence layer for "exquisite context" generation
2. Missing real-time intelligence features (conflict prediction, sprint automation)
3. Missing human-AI bridge architecture
4. Underspecified service boundaries for commercial extensions

## Architecture Principles

1. **🔒 Abstraction-First:** Isolate ast-grep behind clean interfaces
2. **📊 Graph-Centric:** petgraph as single source of truth for code relationships
3. **🧠 Intelligence-Driven:** AI context optimization and human-AI bridge capabilities
4. **🧩 Modular Design:** Granular feature flags, plugin architecture
5. **⚡ Performance-First:** SIMD optimizations, content-addressable storage
6. **🏗️ Extensible Core:** Commercial services build on public foundation
7. **🌐 Tiered Deployment:** Library → CLI → Limited WASM → Private Services
8. **♿ Accessibility-First:** Making complex analysis simple and powerful for all users

## Proposed Crate Structure

### Core Thread Engine (New)

```plaintext
thread-core/                   # 🆕 Main analysis engine
├── src/
│   ├── graph/                # petgraph-based code graph
│   │   ├── builder.rs        # Build graphs from parsed code
│   │   ├── query.rs          # Graph traversal and analysis
│   │   ├── relationships.rs  # Call graphs, imports, dependencies
│   │   └── algorithms.rs     # Analysis algorithms
│   ├── analysis/             # High-level analysis operations
│   │   ├── context.rs        # AI context generation
│   │   ├── dependencies.rs   # Dependency analysis
│   │   └── metrics.rs        # Code metrics and insights
│   ├── incremental/          # Incremental update system
│   └── lib.rs
└── features: ["graph", "analysis", "incremental", "metrics"]

thread-store/                  # 🆕 Content-addressable storage
├── src/
│   ├── content.rs            # Content-addressable storage
│   ├── dedup.rs              # Deduplication algorithms
│   ├── cache.rs              # Analysis result caching
│   ├── memory_map.rs         # Large file handling
│   └── lib.rs
└── features: ["memory-map", "compression", "persistence"]

thread-intelligence/           # 🆕 AI-Human bridge layer
├── src/
│   ├── context/              # AI context generation strategies
│   │   ├── relevance.rs      # Relevance scoring algorithms
│   │   ├── optimization.rs   # Token budget optimization
│   │   ├── adaptation.rs     # Model-specific adaptation
│   │   └── bridge.rs         # Human-AI interface
│   ├── explanation/          # Human-readable explanations
│   │   ├── generator.rs      # Explanation generation
│   │   ├── templates.rs      # Output templates
│   │   └── accessibility.rs  # Accessibility features
│   ├── prediction/           # Real-time intelligence
│   │   ├── conflicts.rs      # Conflict prediction
│   │   ├── sprint.rs         # Sprint automation
│   │   └── evolution.rs      # Change analysis
│   ├── monitoring/           # Performance monitoring
│   │   ├── metrics.rs        # Performance metrics
│   │   ├── alerts.rs         # Alerting system
│   │   └── regression.rs     # Regression detection
│   └── lib.rs
└── features: ["context-intelligence", "prediction", "monitoring", "explanation"]

thread-cli/                    # 🆕 Command-line interface
├── src/
│   ├── commands/             # CLI command implementations
│   ├── config.rs             # Configuration management
│   ├── output.rs             # Output formatting
│   └── main.rs
└── features: ["progress", "json-output", "config-files"]
```

### Enhanced Service Layer

```plaintext
thread-services/               # 🔄 Enhanced abstraction layer
├── src/
│   ├── traits/               # Core service traits
│   │   ├── parser.rs         # Abstract parsing interface
│   │   ├── analyzer.rs       # Abstract analysis interface
│   │   ├── store.rs          # Abstract storage interface
│   │   ├── persistence.rs    # Persistence service boundaries
│   │   ├── intelligence.rs   # Intelligence service interface
│   │   ├── monitoring.rs     # Performance monitoring interface
│   │   └── context.rs        # Execution context (existing)
│   ├── implementations/      # Service implementations
│   │   ├── ast_grep.rs       # ast-grep service implementation
│   │   ├── petgraph.rs       # petgraph service implementation
│   │   ├── memory_only.rs    # In-memory implementations (public)
│   │   └── composite.rs      # Combined service orchestration
│   ├── plugins/              # Plugin system foundation
│   └── lib.rs
└── features: ["ast-grep", "plugins", "extensions", "persistence-traits", "intelligence-traits"]
```

### WASM Enhancement

```plaintext
thread-wasm/                   # 🔄 Comprehensive WASM API
├── src/
│   ├── api/                  # JavaScript-friendly API
│   ├── bindings.rs           # WASM bindings
│   ├── limits.rs             # Rate limiting for public API
│   └── lib.rs
└── features: ["full-api", "limited-api", "rate-limiting"]
```

### Existing Crates (Preserved)

```plaintext
thread-ast-engine/             # ✅ Keep as implementation detail
thread-rule-engine/            # ✅ Keep as implementation detail
thread-language/               # ✅ Keep as implementation detail
thread-utils/                  # ✅ Enhanced with new utilities
```

## Abstraction Strategy: Isolating ast-grep

### Current Problem

```rust
// Direct dependency on ast-grep API
use thread_ast_engine::AstGrep;
let ast = Language::Tsx.ast_grep(content);  // Tied to ast-grep
```

### Proposed Solution: Service Traits

```rust
// thread-services/src/traits/parser.rs
#[async_trait::async_trait]
pub trait CodeParser: Send + Sync {
    async fn parse(&self, content: &str, language: SupportedLanguage)
        -> Result<ParsedCode, ParseError>;

    fn supported_languages(&self) -> &[SupportedLanguage];
    fn capabilities(&self) -> ParserCapabilities;
}

pub trait CodeAnalyzer: Send + Sync {
    fn find_functions(&self, code: &ParsedCode) -> Result<Vec<Function>, AnalysisError>;
    fn find_calls(&self, code: &ParsedCode) -> Result<Vec<FunctionCall>, AnalysisError>;
    fn find_imports(&self, code: &ParsedCode) -> Result<Vec<Import>, AnalysisError>;
}

// thread-services/src/implementations/ast_grep.rs
pub struct AstGrepParser {
    // Wraps thread-ast-engine internally
}

impl CodeParser for AstGrepParser {
    async fn parse(&self, content: &str, language: SupportedLanguage)
        -> Result<ParsedCode, ParseError> {
        // Implementation using thread-ast-engine
        // But external API is clean and swappable
    }
}
```

### Benefits of This Approach

1. **🔒 Implementation Isolation:** Can swap out ast-grep without breaking APIs
2. **🧪 Testability:** Easy to mock parsers for testing
3. **🔌 Extensibility:** Add new parsers (tree-sitter direct, custom, etc.)
4. **📦 Feature Flags:** Enable/disable specific parser implementations

## Core Thread Engine Design

### Graph-Centric Architecture

```rust
// thread-core/src/graph/mod.rs
use petgraph::{Graph, NodeIndex, EdgeIndex};

pub struct CodeGraph {
    graph: Graph<CodeNode, CodeEdge>,
    // Fast lookups
    name_to_node: HashMap<String, NodeIndex>,
    file_to_nodes: HashMap<PathBuf, Vec<NodeIndex>>,
    // Content addressing
    content_store: Arc<ContentStore>,
}

#[derive(Debug, Clone)]
pub struct CodeNode {
    pub id: ContentHash,           // Content-addressable ID
    pub kind: NodeKind,            // Function, Class, Variable, etc.
    pub name: String,              // Symbol name
    pub location: SourceLocation,  // File, line, column
    pub metadata: NodeMetadata,    // Language-specific data
}

#[derive(Debug, Clone)]
pub enum CodeEdge {
    Calls { call_site: SourceLocation },
    Imports { import_path: String },
    Inherits { inheritance_type: InheritanceType },
    References { ref_type: ReferenceType },
    Contains { scope: ScopeType },
}
```

### Analysis Capabilities

```rust
// thread-core/src/analysis/context.rs
impl CodeGraph {
    /// Generate AI-friendly context for a symbol
    pub fn generate_context(&self, symbol: &str, opts: ContextOptions)
        -> Result<AiContext, AnalysisError> {
        let mut context = AiContext::new();

        // Find the symbol
        let node = self.find_symbol(symbol)?;
        context.add_primary(node);

        // Add dependencies if requested
        if opts.include_dependencies {
            let deps = self.get_dependencies(node, opts.max_depth)?;
            context.add_dependencies(deps);
        }

        // Add callers if requested
        if opts.include_callers {
            let callers = self.get_callers(node, opts.max_depth)?;
            context.add_callers(callers);
        }

        Ok(context)
    }

    /// Fast graph queries
    pub fn find_functions_calling(&self, target: &str) -> Vec<&CodeNode> { /* ... */ }
    pub fn get_dependency_chain(&self, from: &str, to: &str) -> Option<Vec<&CodeNode>> { /* ... */ }
    pub fn find_circular_dependencies(&self) -> Vec<Vec<&CodeNode>> { /* ... */ }
}
```

## AI Context Intelligence Layer

### Context Generation Engine

```rust
// thread-intelligence/src/context/relevance.rs
pub struct ContextRelevanceEngine {
    scorer: RelevanceScorer,
    optimizer: TokenBudgetOptimizer,
    adapter: ModelAdapter,
}

pub struct AiContext {
    primary_code: Vec<CodeSegment>,
    dependencies: Vec<(CodeSegment, RelevanceScore)>,
    semantic_relationships: Vec<SemanticLink>,
    token_budget: TokenBudget,
    optimization_metadata: OptimizationMetadata,
}

impl ContextRelevanceEngine {
    /// Generate exquisite context optimized for AI consumption
    pub fn generate_context(&self,
        query: &ContextQuery,
        graph: &CodeGraph,
        model_profile: &LlmProfile
    ) -> Result<AiContext, ContextError> {
        // Score relevance of all code segments
        let scored_segments = self.scorer.score_relevance(query, graph)?;

        // Optimize for token budget
        let optimized = self.optimizer.optimize_for_budget(
            scored_segments,
            model_profile.context_window
        )?;

        // Adapt for specific model characteristics
        let adapted = self.adapter.adapt_for_model(optimized, model_profile)?;

        Ok(adapted)
    }

    /// Generate human-readable explanations
    pub fn explain_context(&self, context: &AiContext) -> Result<Explanation, ExplanationError> {
        // Generate accessible explanations for humans
    }
}
```

### Human-AI Bridge Architecture

```rust
// thread-intelligence/src/context/bridge.rs
pub trait ContextBridge {
    /// Optimize context for human consumption
    fn optimize_for_human(&self, context: AiContext) -> HumanReadableContext;

    /// Optimize context for specific LLM models
    fn optimize_for_llm(&self, context: AiContext, model: LlmProfile) -> OptimizedContext;

    /// Generate explanations of analysis results
    fn explain_analysis(&self, result: &AnalysisResult) -> Explanation;

    /// Make complex analysis accessible
    fn simplify_for_accessibility(&self, analysis: ComplexAnalysis) -> AccessibleAnalysis;
}

pub struct LlmProfile {
    pub model_name: String,
    pub context_window: usize,
    pub token_cost_ratio: f64,
    pub strengths: Vec<ModelStrength>,
    pub weaknesses: Vec<ModelWeakness>,
}

pub struct RelevanceScore {
    pub syntactic_relevance: f64,
    pub semantic_relevance: f64,
    pub dependency_importance: f64,
    pub usage_frequency: f64,
    pub change_likelihood: f64,
    pub combined_score: f64,
}
```

## Real-Time Intelligence Features

### Predictive Intelligence Layer

```rust
// thread-intelligence/src/prediction/mod.rs
pub trait IntelligenceService {
    /// Predict potential merge conflicts before they happen
    fn predict_conflicts(&self, changes: &[FileChange]) -> ConflictPrediction;

    /// Analyze sprint velocity and bottlenecks
    fn analyze_sprint_velocity(&self, graph: &CodeGraph) -> VelocityMetrics;

    /// Suggest intelligent refactoring opportunities
    fn suggest_refactoring(&self, complexity: &ComplexityAnalysis) -> RefactoringSuggestions;

    /// Predict impact of proposed changes
    fn predict_change_impact(&self, proposed: &[Change]) -> ImpactAnalysis;

    /// Analyze code evolution patterns
    fn analyze_evolution(&self, history: &GitHistory, graph: &CodeGraph) -> EvolutionInsights;
}

// thread-intelligence/src/prediction/conflicts.rs
pub struct ConflictPredictor {
    graph: Arc<CodeGraph>,
    git_analyzer: GitAnalyzer,
    pattern_matcher: ConflictPatternMatcher,
}

impl ConflictPredictor {
    pub fn predict_merge_conflicts(&self,
        branch_a: &Branch,
        branch_b: &Branch
    ) -> Result<ConflictPrediction, PredictionError> {
        let changes_a = self.git_analyzer.analyze_changes(branch_a)?;
        let changes_b = self.git_analyzer.analyze_changes(branch_b)?;

        // Find overlapping affected code regions
        let overlaps = self.find_code_overlaps(&changes_a, &changes_b)?;

        // Predict conflict likelihood using graph analysis
        let predictions = overlaps.into_iter()
            .map(|overlap| self.predict_conflict_likelihood(overlap))
            .collect();

        Ok(ConflictPrediction::new(predictions))
    }
}
```

### Performance & Monitoring Framework

```rust
// thread-intelligence/src/monitoring/mod.rs
pub trait PerformanceMonitor {
    fn track_analysis_time(&self, operation: &str, duration: Duration);
    fn report_memory_usage(&self, component: &str, bytes: usize);
    fn alert_on_regression(&self, metric: &str, threshold: f64);
    fn collect_context_quality_metrics(&self, context: &AiContext, feedback: &UserFeedback);
}

pub struct IntelligenceMetrics {
    pub context_generation_time: Duration,
    pub relevance_score_accuracy: f64,
    pub token_efficiency_ratio: f64,
    pub user_satisfaction_score: f64,
    pub prediction_accuracy: f64,
}
```

## Service Architecture Boundaries

### Clear Separation of Concerns

```rust
// thread-services/src/traits/persistence.rs
pub trait AnalysisService {
    /// In-memory only - public API
    fn analyze_in_memory(&self, code: &CodeGraph) -> InMemoryResult;
    fn generate_context(&self, query: &ContextQuery) -> AiContext;
}

pub trait PersistenceService {
    /// Database/storage - commercial only
    fn store_analysis(&self, result: &AnalysisResult) -> Result<(), PersistenceError>;
    fn load_cached_analysis(&self, key: &CacheKey) -> Option<AnalysisResult>;
    fn persist_graph(&self, graph: &CodeGraph) -> Result<GraphId, PersistenceError>;
}

pub trait IntelligenceService {
    /// Advanced intelligence - commercial features
    fn predict_conflicts(&self, changes: &[FileChange]) -> ConflictPrediction;
    fn analyze_sprint_metrics(&self, graph: &CodeGraph) -> SprintMetrics;
    fn suggest_optimizations(&self, analysis: &PerformanceAnalysis) -> OptimizationSuggestions;
}
```

## Feature Flag Architecture

### Enhanced Granular Control System

```toml
# thread-core/Cargo.toml

[features]
default = ["graph", "basic-analysis"]

# Core features
graph = ["petgraph", "thread-store/content-addressing"]
analysis = ["graph", "metrics"]
incremental = ["analysis", "thread-store/caching"]

# Intelligence features (new)
context-intelligence = ["analysis", "thread-intelligence/context"]
conflict-prediction = ["intelligence", "thread-intelligence/prediction"]
sprint-automation = ["intelligence", "thread-intelligence/prediction"]
human-ai-bridge = ["context-intelligence", "thread-intelligence/explanation"]

# Analysis features
metrics = ["analysis"]
ai-context = ["analysis", "thread-services/context-generation"]
dependency-analysis = ["graph"]
call-graph = ["graph", "thread-services/ast-grep"]

# Performance features
simd = ["thread-utils/simd"]
parallel = ["rayon"]
memory-mapping = ["thread-store/memory-map"]
performance-monitoring = ["thread-intelligence/monitoring"]

# Persistence features (commercial)
persistence = ["serde", "database-traits"]
postgres-backend = ["persistence", "sqlx/postgres"]
redis-cache = ["persistence", "redis"]

# Extension features
plugins = ["thread-services/plugins"]
custom-languages = ["thread-services/extensions"]
commercial-extensions = ["plugins", "persistence"]

# WASM features
wasm-basic = ["graph"]
wasm-intelligence = ["wasm-basic", "context-intelligence"]
wasm-full = ["wasm-intelligence", "analysis", "ai-context"]
wasm-commercial = ["wasm-full", "conflict-prediction", "sprint-automation"]
```

### Usage Examples

```toml
# Library users can control exactly what they pay for
[dependencies]
thread-core = { version = "0.1", features = ["graph", "ai-context"] }
# Only includes graph building and AI context - no metrics overhead

thread-cli = { version = "0.1", features = ["full"] }
# CLI includes everything

thread-wasm = { version = "0.1", features = ["basic"] }
# Public WASM is limited
```

## Tiered Deployment Strategy

### 1. Public Library & CLI

```rust
// Full-featured, open source
pub use thread_core::{CodeGraph, AnalysisError};
pub use thread_services::{CodeParser, CodeAnalyzer};
pub use thread_store::ContentStore;

// Available features: All open source features
// License: AGPL-3.0-or-later
```

### 2. Limited Public WASM

```rust
// thread-wasm with "limited-api" feature
impl ThreadWasm {
    #[wasm_bindgen]
    pub fn analyze_code(&self, code: &str, language: &str) -> Result<JsValue, JsValue> {
        // Rate limiting: max 100 files
        if self.file_count > 100 {
            return Err("API key required for large projects".into());
        }
        // Basic analysis only
    }
}
```

### 3. Private Commercial Services

```plaintext
// Private crates (not published)
thread-commercial/
├── src/
│   ├── advanced_analytics/   # Proprietary algorithms
│   ├── enterprise_features/  # Commercial-only features
│   ├── cloud_integration/    # Cloudflare Workers optimizations
│   └── billing/              # Usage tracking, API keys
```

## Commercial Extension Points

```rust
// thread-services/src/traits/extension.rs
pub trait CommercialExtension: Send + Sync {
    fn enhance_analysis(&self, graph: &CodeGraph) -> Result<EnhancedAnalysis, Error>;
    fn custom_metrics(&self, graph: &CodeGraph) -> Result<CustomMetrics, Error>;
}

// Private implementation
struct EnterpriseAnalyzer {
    // Proprietary algorithms
}

impl CommercialExtension for EnterpriseAnalyzer {
    fn enhance_analysis(&self, graph: &CodeGraph) -> Result<EnhancedAnalysis, Error> {
        // Advanced analysis only available in commercial version
    }
}
```

## Enhanced Migration Path: Current → Target

### Phase 0: Foundation & Service Architecture

1. **Create abstraction layer:** Enhance thread-services with parser/analyzer traits
2. **Implement service boundaries:** Clear separation of in-memory vs persistence
3. **Implement ast-grep adapter:** Wrap existing ast-grep components behind new interfaces
4. **Enhanced feature flags:** Add intelligence and persistence feature control
5. **Contract testing setup:** Ensure service boundary compliance

### Phase 1: Intelligence Foundation

1. **Design context intelligence architecture:** Create thread-intelligence crate structure
2. **Implement relevance scoring:** Basic algorithms for context relevance
3. **Create human-AI bridge interfaces:** Design accessibility-first APIs
4. **Setup performance monitoring:** Framework for measuring context quality

### Phase 2: Core Engine & Intelligence

1. **Create thread-core:** Implement petgraph-based analysis engine
2. **Create thread-store:** Content-addressable storage with deduplication
3. **Implement context intelligence:** Basic AI context generation with relevance scoring
4. **Integration:** Connect core engine to ast-grep via service layer
5. **Performance monitoring:** Real-time metrics collection

### Phase 3: User Interface & Accessibility (Week 5-7)

1. **Create thread-cli:** Command-line interface with accessibility features
2. **Enhance thread-wasm:** Comprehensive WASM API with rate limiting
3. **Human-AI bridge implementation:** Context optimization for both humans and AI
4. **Documentation:** API docs, accessibility guides, and usage examples
5. **Integration testing:** End-to-end workflow validation

### Phase 4: Intelligence Features (Week 7-9)

1. **Conflict prediction:** Implement merge conflict prediction system
2. **Sprint automation:** Basic sprint metrics and velocity analysis
3. **Evolution analysis:** Change impact prediction and pattern recognition
4. **Quality feedback loops:** Context quality measurement and improvement

### Phase 5: Commercial Preparation (Week 9-10)

1. **Commercial extensions:** Private crates for enhanced intelligence features
2. **Cloudflare optimization:** WASM deployment optimizations
3. **Enterprise features:** Advanced analytics and monitoring
4. **Testing & polish:** Performance testing, edge case handling, accessibility validation

## Extension & Plugin Architecture

### Plugin System Design

```rust
// thread-services/src/plugins/mod.rs
pub trait LanguagePlugin: Send + Sync {
    fn language(&self) -> SupportedLanguage;
    fn parse(&self, content: &str) -> Result<ParsedCode, ParseError>;
    fn analyze(&self, code: &ParsedCode) -> Result<AnalysisResult, AnalysisError>;
}

pub trait AnalysisPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn analyze(&self, graph: &CodeGraph) -> Result<PluginResult, PluginError>;
}

// Plugin registry
pub struct PluginRegistry {
    language_plugins: HashMap<SupportedLanguage, Box<dyn LanguagePlugin>>,
    analysis_plugins: Vec<Box<dyn AnalysisPlugin>>,
}
```

### Commercial Extension Pattern

```rust
// Public plugin interface
#[cfg(feature = "plugins")]
pub fn register_plugin<P: AnalysisPlugin + 'static>(plugin: P) {
    PLUGIN_REGISTRY.write().unwrap().register_analysis(Box::new(plugin));
}

// Private commercial plugins (in separate repo)
struct SecurityAnalysisPlugin { /*proprietary */ }
impl AnalysisPlugin for SecurityAnalysisPlugin { /* ...*/ }

// Dynamically loaded in commercial version
#[cfg(feature = "commercial")]
fn load_commercial_plugins() {
    register_plugin(SecurityAnalysisPlugin::new());
    register_plugin(PerformanceAnalysisPlugin::new());
    // etc.
}
```

## Enhanced Testing Strategy

### Multi-Layer Testing Approach

```rust
// Integration-first testing philosophy
// Test what actually matters and changes outcomes

// Core functionality unit tests
#[cfg(test)]
mod core_tests {
    use super::*;

    #[test]
    fn test_graph_construction_performance() {
        // Performance regression tests
    }

    #[test]
    fn test_context_relevance_scoring() {
        // Algorithm correctness tests
    }
}

// Contract testing for service boundaries
#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn test_parser_service_contract() {
        // Ensure all parser implementations follow contract
    }

    #[test]
    fn test_persistence_service_boundaries() {
        // Verify in-memory vs persistence separation
    }
}

// Property-based testing for graph algorithms
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn graph_operations_preserve_invariants(operations: Vec<GraphOperation>) {
            // Ensure graph operations maintain consistency
        }
    }
}

// End-to-end integration tests
#[cfg(test)]
mod integration_tests {
    #[test]
    fn test_full_analysis_pipeline() {
        // Test complete workflows that users actually use
    }

    #[test]
    fn test_context_generation_quality() {
        // Measure actual context quality for real codebases
    }
}
```

## Performance & Efficiency Design

### Content-Addressable Storage

```rust
// thread-store/src/content.rs
pub struct ContentStore {
    storage: HashMap<ContentHash, Arc<str>>,
    dedup_stats: DedupStats,
}

impl ContentStore {
    pub fn intern(&mut self, content: &str) -> ContentHash {
        let hash = rapidhash::hash(content.as_bytes());
        self.storage.entry(hash)
            .or_insert_with(|| Arc::from(content));
        hash
    }

    // Memory-efficient: same content = same Arc<str>
    pub fn get(&self, hash: ContentHash) -> Option<Arc<str>> {
        self.storage.get(&hash).cloned()
    }
}
```

### Incremental Updates

```rust
// thread-core/src/incremental/mod.rs
pub struct IncrementalAnalyzer {
    graph: CodeGraph,
    file_hashes: HashMap<PathBuf, ContentHash>,
}

impl IncrementalAnalyzer {
    pub fn update_file(&mut self, path: PathBuf, new_content: &str)
        -> Result<UpdateResult, AnalysisError> {
        let new_hash = self.store.intern(new_content);

        if let Some(old_hash) = self.file_hashes.get(&path) {
            if *old_hash == new_hash {
                return Ok(UpdateResult::NoChange);
            }
            // Remove old nodes for this file
            self.graph.remove_file_nodes(&path);
        }

        // Re-analyze only the changed file
        self.analyze_and_add_file(path, new_content)?;
        Ok(UpdateResult::Updated)
    }
}
```

## Conclusion

This architecture preserves your excellent ast-grep foundation while building the core Thread capability on top. It provides clean abstractions, commercial extensibility, and a clear path from where you are to where you want to be.

## Todo Status

- [x] Review PLAN.md to understand original vision and requirements
- [x] Analyze current architecture and identify abstraction opportunities for ast-grep isolation
- [x] Design core thread capability architecture with petgraph for codebase-wide analysis
- [x] Design modular and extensible API architecture with granular feature flags
- [x] Plan tiered WASM deployment strategy (public limited vs private full-featured)
- [x] Design commercial services extension architecture for Cloudflare deployment

## Implementation Recommendations

Start with Phase 1 - enhance the services layer to abstract ast-grep. This gives you immediate benefits:

- Clean separation between interface and implementation
- Testability improvements
- Foundation for commercial extensions

### Key Next Steps

1. Design the service traits in [`thread-services`](crates/services/)
2. Create the [`AstGrepParser`](crates/services/src/lib.rs) implementation
3. Add basic feature flags
4. Start building [`thread-core`](crates/) with petgraph integration

## Future Vision: Extensible Intelligence

### Long-Term Architectural Goals

```rust
// Future: Domain-specific query language for comprehensive intelligence
pub struct ThreadQuery {
    pattern: String, // "functions calling $API where complexity > 10 and change_frequency < 0.1"
    scope: QueryScope,
    optimizations: QueryHints,
    context_requirements: ContextRequirements,
}

// Semantic understanding beyond syntax
pub trait SemanticAnalyzer {
    fn understand_intent(&self, code: &CodeSegment) -> IntentAnalysis;
    fn find_semantic_similarities(&self, segments: &[CodeSegment]) -> SimilarityGraph;
    fn predict_developer_intent(&self, changes: &[Change]) -> IntentPrediction;
}

// Evolution and change intelligence
pub trait EvolutionAnalyzer {
    fn predict_change_impact(&self, proposed: &[Change]) -> ImpactAnalysis;
    fn suggest_migration_path(&self, from: &CodeGraph, to: &DesiredState) -> MigrationPlan;
    fn analyze_technical_debt_evolution(&self, history: &GitHistory) -> DebtEvolution;
}
```

### Accessibility & Human-Centered Design

```rust
// Making complex analysis accessible to all users
pub trait AccessibilityInterface {
    fn generate_audio_descriptions(&self, analysis: &AnalysisResult) -> AudioDescription;
    fn create_high_contrast_visualizations(&self, graph: &CodeGraph) -> AccessibleVisualization;
    fn simplify_technical_language(&self, explanation: &TechnicalExplanation) -> PlainLanguageExplanation;
    fn provide_guided_exploration(&self, codebase: &CodeGraph) -> GuidedTour;
}
```

## Resolved Areas (Based on Clarifications)

### Commercial Protection Strategy
✅ **Licensing Protection**: AGPL with trait-based commercial extensions
- Public interfaces with trait implementations for CLI and limited WASM
- Commercial implementations remain private
- Clear service boundaries prevent unauthorized access

### Persistence Architecture
✅ **Service-Based Architecture**: Feature-gated serialization
- In-memory public API only
- Persistence through commercial service implementations
- Database writers as separate commercial services

### Testing Philosophy Implementation
✅ **Integration-First Testing**:
- Focus on outcomes and user workflows
- Unit tests for core algorithms affecting results
- Contract testing for service boundaries
- Property-based testing for graph algorithms

## Areas Still Needing Clarification

1. **Context Quality Metrics**: How should we measure "exquisite context" quality? What feedback loops will drive improvement?

2. **Performance Benchmarks**: Specific targets for context generation latency and relevance scoring accuracy?

3. **Rate Limiting Strategy**: Public WASM API limits and commercial tier scaling?

4. **Human-AI Bridge Effectiveness**: How do we measure success in bridging expectations vs reality?

5. **Accessibility Standards**: What specific accessibility compliance targets (WCAG levels, etc.)?

6. **Intelligence Feature Accuracy**: Acceptable accuracy thresholds for conflict prediction and sprint automation?
