<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Thread Semantic Classification: Specification & Implementation Plan

**Version**: 1.0.0
**Date**: 2026-02-22
**Status**: Proposed
**Crate**: `thread-definitions` (new crate within workspace)

---

## Executive Summary

This document specifies the Rust port of CodeWeaver's semantic classification system into
Thread as the `thread-definitions` crate. The system classifies tree-sitter AST node types
into 22 language-agnostic semantic categories using a **purely declarative, data-driven
approach** validated at 99.7% accuracy across 27 languages.

The architecture replaces CodeWeaver's 7,200-line Python implementation (7 modules, 3
classification systems, pickle caches, Pydantic models) with an estimated ~1,800 lines of
Rust code backed by static data files. No classifier logic lives in code — classification is
table lookup over declarative rules.

### Key Metrics from Validation

| Metric | Value |
|--------|-------|
| Languages validated | 27 |
| Languages at 100.0% accuracy | 25/27 |
| Overall accuracy | 99.7% |
| Universal rules baseline | 82.6% (zero language-specific data) |
| Average override cost | ~41 lines TOML per language |
| Total override lines | 1,063 across 26 files |

---

## 1. Architecture Overview

### 1.1 Design Principles

1. **Data over code.** Every classification decision lives in a data file, not in a method.
   The classifier is a lookup engine, not a decision engine.

2. **One lookup path, not seven.** A single priority-ordered pipeline:
   `override → file_detection → token_purpose → universal_exact → universal_majority →
   category → name_heuristic → unclassified`.

3. **Declarative language support.** Adding a new language = providing `node-types.json` +
   a small TOML override file (~41 lines average). No Rust code changes.

4. **Thread-native integration.** Uses `thread-language::SupportLang` for language identity.
   Lives alongside `thread-ast-engine` and `thread-flow` as a peer crate.

5. **Zero runtime dependencies on Python.** All data files are pre-generated JSON/TOML
   shipped with the crate. No pickle, no Pydantic, no Python runtime.

### 1.2 Crate Position in Workspace

```
thread/
├── crates/
│   ├── ast-engine/          # Tree-sitter parsing, pattern matching (existing)
│   ├── definitions/         # NEW: Semantic classification (this spec)
│   ├── flow/                # Dataflow pipelines (existing)
│   ├── language/            # Language parsers & SupportLang enum (existing)
│   ├── rule-engine/         # Rule-based analysis (existing)
│   ├── services/            # Service layer (existing)
│   ├── utils/               # Hashers, SIMD, etc. (existing)
│   └── wasm/                # WASM target (existing)
```

### 1.3 Dependency Graph

```
thread-definitions
├── serde + serde_json       (workspace)
├── toml                     (new dep, for TOML override parsing)
├── thread-language          (for SupportLang enum only, default-features = false)
└── thiserror                (workspace)
```

**Important**: `thread-definitions` does NOT depend on `thread-ast-engine` or `tree-sitter`.
It classifies node types by *name*, not by parsing actual AST nodes. This keeps it
lightweight and usable without linking any tree-sitter parsers.

---

## 2. Data Files

All classification data ships as static files embedded via `include_str!` or loaded at
initialization from a `data/` directory within the crate.

### 2.1 File Inventory

```
crates/definitions/
├── src/
│   ├── lib.rs               # Public API
│   ├── types.rs             # SemanticClass, ImportanceRank, TokenPurpose, etc.
│   ├── classifier.rs        # Classification pipeline (lookup engine)
│   ├── rules.rs             # Rule loading and representation
│   ├── scoring.rs           # ImportanceScores and AgentTask scoring
│   └── error.rs             # Error types
├── data/
│   ├── universal_rules.json # 2,444 exact + 21 majority cross-language rules
│   ├── categories.json      # 55 category → SemanticClass mappings
│   ├── scoring.json         # ImportanceScores per class + AgentTask profiles
│   └── overrides/
│       ├── SPEC.toml        # Override format documentation
│       ├── bash.toml         # 13 overrides
│       ├── c.toml            # 57 overrides
│       ├── cpp.toml          # 105 overrides
│       ├── csharp.toml       # 90 overrides
│       ├── css.toml          # 44 overrides
│       ├── elixir.toml       # 20 overrides
│       ├── go.toml           # 25 overrides
│       ├── haskell.toml      # 85 overrides
│       ├── hcl.toml          # 2 overrides
│       ├── html.toml         # 10 overrides
│       ├── java.toml         # 54 overrides
│       ├── javascript.toml   # 16 overrides
│       ├── json.toml         # 1 override
│       ├── jsx.toml          # 14 overrides
│       ├── kotlin.toml       # 22 overrides
│       ├── lua.toml          # 12 overrides
│       ├── nix.toml          # 16 overrides
│       ├── php.toml          # 46 overrides
│       ├── python.toml       # 43 overrides
│       ├── ruby.toml         # ~40 overrides
│       ├── rust.toml         # ~45 overrides
│       ├── scala.toml        # 71 overrides
│       ├── solidity.toml     # 63 overrides
│       ├── swift.toml        # 112 overrides
│       ├── tsx.toml          # 14 overrides
│       ├── typescript.toml   # 13 overrides
│       └── yaml.toml         # 0 overrides (empty/absent)
└── tests/
    ├── classification_tests.rs  # Round-trip accuracy tests
    └── holdout_tests.rs         # Holdout evaluation port
```

### 2.2 Data Format: `universal_rules.json`

```json
{
  "exact": {
    "function_definition": "definition_callable",
    "class_definition": "definition_type",
    "import_statement": "boundary_module",
    "if_statement": "flow_branching",
    "+": "operation_operator",
    "identifier": "syntax_identifier"
  },
  "majority": {
    "method": "syntax_identifier",
    "block": "syntax_punctuation"
  }
}
```

- **exact**: Name appears in every language that has it, always with the same class (2,444 entries)
- **majority**: Name appears in multiple languages, ≥75% agree on class (21 entries)

### 2.3 Data Format: TOML Overrides

```toml
# go.toml — 25 overrides for Go-specific node types

[overrides]
const_spec                = "definition_data"
short_var_declaration     = "definition_data"
type_declaration          = "definition_data"
import_spec               = "boundary_module"
communication_case        = "flow_branching"
range_clause              = "flow_iteration"

[token_overrides]
# Token names that need non-default classification

[doc_comment_tokens]
comment                   = true    # Promotes to documentation_structured
```

Three sections, all optional:
- `[overrides]`: composite thing name → SemanticClass
- `[token_overrides]`: token name → SemanticClass (overrides token_purpose detection)
- `[doc_comment_tokens]`: token name → `true` (promotes comment tokens to `documentation_structured`)

### 2.4 Data Format: `categories.json`

```json
{
  "mapping": {
    "expression": "operation_operator",
    "statement": "flow_branching",
    "declaration": "definition_data",
    "type": "definition_type",
    "literal": "syntax_literal",
    "identifier": "syntax_identifier",
    "parameter": "definition_data"
  }
}
```

55 entries mapping tree-sitter grammar category names to SemanticClass values.

### 2.5 Data Format: `scoring.json`

```json
{
  "semantic_classes": {
    "definition_callable": {
      "rank": 1,
      "importance_scores": {
        "discovery": 0.95,
        "comprehension": 0.92,
        "modification": 0.85,
        "debugging": 0.85,
        "documentation": 0.92
      }
    }
  },
  "agent_tasks": {
    "debug": {
      "profile": { "discovery": 0.2, "comprehension": 0.3, "modification": 0.1, "debugging": 0.35, "documentation": 0.05 }
    },
    "implement": {
      "profile": { "discovery": 0.3, "comprehension": 0.3, "modification": 0.2, "debugging": 0.1, "documentation": 0.1 }
    }
  }
}
```

---

## 3. Type Definitions

### 3.1 `SemanticClass` — The Core Enum

```rust
/// Language-agnostic semantic categories for AST node types.
///
/// 22 categories organized into 5 importance tiers. Every tree-sitter
/// node type in every supported language maps to exactly one of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticClass {
    // Tier 1: Primary Definitions
    FileThing,
    DefinitionCallable,
    DefinitionType,
    DefinitionData,
    DefinitionTest,

    // Tier 2: Behavioral Contracts
    BoundaryModule,
    BoundaryError,
    BoundaryResource,
    DocumentationStructured,

    // Tier 3: Control Flow & Logic
    FlowBranching,
    FlowIteration,
    FlowControl,
    FlowAsync,

    // Tier 4: Operations & Expressions
    OperationInvocation,
    OperationData,
    OperationOperator,
    ExpressionAnonymous,

    // Tier 5: Syntax & References
    SyntaxKeyword,
    SyntaxIdentifier,
    SyntaxLiteral,
    SyntaxAnnotation,
    SyntaxPunctuation,
}
```

### 3.2 `ImportanceRank`

```rust
/// Importance tiers from highest (1) to lowest (5).
/// Used for coarse-grained filtering before fine-grained scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ImportanceRank {
    PrimaryDefinitions = 1,
    BehavioralContracts = 2,
    ControlFlowLogic = 3,
    OperationsExpressions = 4,
    SyntaxReferences = 5,
}

impl SemanticClass {
    /// Get the importance rank for this classification.
    pub const fn rank(&self) -> ImportanceRank { /* match table */ }
}
```

### 3.3 `TokenPurpose` — Leaf Node Classification

```rust
/// Purpose of a leaf (token) node in the AST.
/// Determined from the node-types.json `named` flag and name patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenPurpose {
    Operator,
    Keyword,
    Literal,
    Punctuation,
    Comment,
    Identifier,
}

impl TokenPurpose {
    /// Map to default SemanticClass (before overrides).
    pub const fn default_class(&self) -> SemanticClass {
        match self {
            Self::Operator => SemanticClass::OperationOperator,
            Self::Keyword => SemanticClass::SyntaxKeyword,
            Self::Literal => SemanticClass::SyntaxLiteral,
            Self::Punctuation => SemanticClass::SyntaxPunctuation,
            Self::Comment => SemanticClass::SyntaxAnnotation,
            Self::Identifier => SemanticClass::SyntaxIdentifier,
        }
    }
}
```

### 3.4 `Classification` — Result Type

```rust
/// The result of classifying a single AST node type.
#[derive(Debug, Clone, PartialEq)]
pub struct Classification {
    /// The semantic class assigned.
    pub class: SemanticClass,
    /// Importance rank (derived from class).
    pub rank: ImportanceRank,
    /// How confident we are.
    pub confidence: Confidence,
    /// Which rule tier produced this classification.
    pub method: ClassificationMethod,
}

/// Classification confidence level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// Language-specific override or exact universal match.
    High,
    /// Category mapping or majority universal match.
    Medium,
    /// Name heuristic or fallback.
    Low,
}

/// Which stage of the classification pipeline produced the result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassificationMethod {
    Override,
    FileDetection,
    TokenPurpose,
    UniversalExact,
    UniversalMajority,
    Category,
    NameHeuristic,
    Unclassified,
}
```

### 3.5 `ImportanceScores` — Multi-Dimensional Scoring

```rust
/// Multi-dimensional importance scoring for AI assistant contexts.
/// Each dimension is a weight in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ImportanceScores {
    pub discovery: f32,
    pub comprehension: f32,
    pub modification: f32,
    pub debugging: f32,
    pub documentation: f32,
}

/// Agent task types that influence scoring weights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTask {
    Debug,
    Implement,
    Refactor,
    Review,
    Search,
    Test,
    Document,
    LocalEdit,
    Default,
}

impl ImportanceScores {
    /// Compute weighted score for a given agent task.
    pub fn for_task(&self, task: AgentTask, profiles: &TaskProfiles) -> f32 {
        let p = profiles.get(task);
        self.discovery * p.discovery
            + self.comprehension * p.comprehension
            + self.modification * p.modification
            + self.debugging * p.debugging
            + self.documentation * p.documentation
    }
}
```

---

## 4. Classification Pipeline

### 4.1 The Classifier

```rust
/// Pre-loaded classification rules for all languages.
/// Constructed once at initialization, then used for O(1) lookups.
pub struct Classifier {
    /// name → SemanticClass (unanimous across all languages)
    universal_exact: HashMap<Box<str>, SemanticClass>,
    /// name → SemanticClass (≥75% agreement)
    universal_majority: HashMap<Box<str>, SemanticClass>,
    /// category name → SemanticClass
    category_map: HashMap<Box<str>, SemanticClass>,
    /// Per-language overrides: lang → (name → SemanticClass)
    overrides: HashMap<SupportLang, LanguageOverrides>,
    /// Scoring data
    scoring: ScoringData,
}

struct LanguageOverrides {
    /// Composite and token name → class
    overrides: HashMap<Box<str>, SemanticClass>,
    /// Token names promoted to documentation_structured
    doc_comment_tokens: HashSet<Box<str>>,
}
```

### 4.2 The Classification Function

```rust
impl Classifier {
    /// Classify a single AST node type.
    ///
    /// # Arguments
    /// - `name`: The tree-sitter node type name (e.g., "function_definition")
    /// - `lang`: The programming language
    /// - `kind`: Whether this is a token (leaf) or composite (branch) node
    /// - `purpose`: For tokens, the detected purpose (operator/keyword/etc.)
    /// - `is_file`: Whether this is the root file node
    /// - `categories`: Grammar categories this node belongs to
    pub fn classify(
        &self,
        name: &str,
        lang: SupportLang,
        kind: NodeKind,
        purpose: Option<TokenPurpose>,
        is_file: bool,
        categories: &[&str],
    ) -> Classification {
        // 1. Language-specific override (highest priority)
        if let Some(overrides) = self.overrides.get(&lang) {
            // Check doc_comment_tokens first (token-specific)
            if kind == NodeKind::Token && overrides.doc_comment_tokens.contains(name) {
                return Classification::high(
                    SemanticClass::DocumentationStructured,
                    ClassificationMethod::Override,
                );
            }
            if let Some(&class) = overrides.overrides.get(name) {
                return Classification::high(class, ClassificationMethod::Override);
            }
        }

        // 2. File thing detection
        if is_file && kind == NodeKind::Composite {
            return Classification::high(
                SemanticClass::FileThing,
                ClassificationMethod::FileDetection,
            );
        }

        // 3. Token purpose (most reliable for leaf nodes)
        if kind == NodeKind::Token {
            if let Some(purpose) = purpose {
                return Classification::high(
                    purpose.default_class(),
                    ClassificationMethod::TokenPurpose,
                );
            }
        }

        // 4. Universal exact match
        if let Some(&class) = self.universal_exact.get(name) {
            return Classification::high(class, ClassificationMethod::UniversalExact);
        }

        // 5. Universal majority match
        if let Some(&class) = self.universal_majority.get(name) {
            return Classification::medium(class, ClassificationMethod::UniversalMajority);
        }

        // 6. Category-based inference
        for cat in categories {
            if let Some(&class) = self.category_map.get(*cat) {
                return Classification::medium(class, ClassificationMethod::Category);
            }
        }

        // 7. Name heuristic (comment detection)
        let name_lower = name.to_ascii_lowercase();
        if name_lower.contains("comment") {
            let class = if name_lower.contains("line") {
                SemanticClass::SyntaxAnnotation
            } else if name_lower.contains("block") || name_lower.contains("doc") {
                SemanticClass::DocumentationStructured
            } else {
                SemanticClass::SyntaxAnnotation
            };
            return Classification::low(class, ClassificationMethod::NameHeuristic);
        }

        // 8. Unclassified fallback
        Classification::low(
            SemanticClass::SyntaxKeyword,
            ClassificationMethod::Unclassified,
        )
    }
}
```

### 4.3 Pipeline Priority Table

| Priority | Method | Source | Confidence | Description |
|----------|--------|--------|------------|-------------|
| 1 | `Override` | `{lang}.toml` | High | Language-specific TOML override |
| 2 | `FileDetection` | Code | High | Root AST node (is_file flag) |
| 3 | `TokenPurpose` | Code + data | High | Leaf node purpose detection |
| 4 | `UniversalExact` | `universal_rules.json` | High | Unanimous cross-language agreement |
| 5 | `UniversalMajority` | `universal_rules.json` | Medium | ≥75% cross-language agreement |
| 6 | `Category` | `categories.json` | Medium | Grammar category mapping |
| 7 | `NameHeuristic` | Code | Low | Substring pattern matching |
| 8 | `Unclassified` | Code | Low | Fallback |

---

## 5. Public API

### 5.1 Core API Surface

```rust
// === Types ===
pub enum SemanticClass { /* 22 variants */ }
pub enum ImportanceRank { /* 5 variants */ }
pub enum TokenPurpose { /* 6 variants */ }
pub enum AgentTask { /* 9 variants */ }
pub enum NodeKind { Token, Composite }
pub enum Confidence { High, Medium, Low }
pub enum ClassificationMethod { /* 8 variants */ }
pub struct Classification { /* class, rank, confidence, method */ }
pub struct ImportanceScores { /* 5 f32 fields */ }
pub struct Classifier { /* opaque */ }

// === Construction ===
impl Classifier {
    /// Load classifier with embedded data (default).
    /// Uses include_str! for universal rules + overrides.
    pub fn new() -> Result<Self, ClassifierError>;

    /// Load classifier with custom data directory (for testing/extension).
    pub fn from_directory(path: &Path) -> Result<Self, ClassifierError>;
}

// === Classification ===
impl Classifier {
    /// Classify a single AST node type.
    pub fn classify(
        &self,
        name: &str,
        lang: SupportLang,
        kind: NodeKind,
        purpose: Option<TokenPurpose>,
        is_file: bool,
        categories: &[&str],
    ) -> Classification;

    /// Convenience: classify with minimal info (name + language + kind).
    pub fn classify_simple(
        &self,
        name: &str,
        lang: SupportLang,
        kind: NodeKind,
    ) -> Classification;
}

// === Scoring ===
impl Classifier {
    /// Get importance scores for a semantic class.
    pub fn importance_scores(&self, class: SemanticClass) -> ImportanceScores;

    /// Compute task-weighted score.
    pub fn task_score(&self, class: SemanticClass, task: AgentTask) -> f32;
}

// === Introspection ===
impl Classifier {
    /// List all languages with loaded overrides.
    pub fn languages_with_overrides(&self) -> Vec<SupportLang>;

    /// Get override count for a language.
    pub fn override_count(&self, lang: SupportLang) -> usize;

    /// Get stats about loaded rules.
    pub fn stats(&self) -> ClassifierStats;
}
```

### 5.2 Type Count Summary

| Category | Count |
|----------|-------|
| Public enums | 7 |
| Public structs | 3 |
| Public methods | 9 |
| **Total public API surface** | **19 items** |

Compare to CodeWeaver's ~40+ public types across 8 modules.

---

## 6. Integration Points

### 6.1 With `thread-language`

```rust
// thread-definitions depends on thread-language for SupportLang only
use thread_language::SupportLang;

// Language identity flows from thread-language → thread-definitions
let class = classifier.classify("function_definition", SupportLang::Python, NodeKind::Composite, None, false, &[]);
```

### 6.2 With `thread-flow` (Future)

```rust
// thread-flow will depend on thread-definitions for semantic enrichment
use thread_definitions::{Classifier, SemanticClass, AgentTask};

// In a flow pipeline: parse → extract → classify → score → emit
let classifier = Classifier::new()?;

for node_type in parsed_file.node_types() {
    let classification = classifier.classify(
        node_type.name,
        lang,
        node_type.kind.into(),
        node_type.purpose,
        node_type.is_root,
        &node_type.categories,
    );

    // Use for context pack ranking
    let score = classifier.task_score(classification.class, AgentTask::Debug);
}
```

### 6.3 With `thread-ast-engine` (Future)

The classifier does NOT depend on tree-sitter or thread-ast-engine. However, consumers
that have both can bridge them:

```rust
// In thread-services or thread-flow:
use tree_sitter::Node;
use thread_definitions::{Classifier, NodeKind, TokenPurpose};

fn classify_tree_sitter_node(node: &Node, lang: SupportLang, classifier: &Classifier) -> Classification {
    let kind = if node.child_count() == 0 { NodeKind::Token } else { NodeKind::Composite };
    let purpose = if kind == NodeKind::Token {
        Some(detect_token_purpose(node.kind()))
    } else {
        None
    };
    classifier.classify(node.kind(), lang, kind, purpose, node.parent().is_none(), &[])
}
```

---

## 7. Cargo Configuration

### 7.1 `Cargo.toml`

```toml
[package]
name = "thread-definitions"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
description = "Semantic classification of AST node types for Thread. Provides language-agnostic SemanticClass assignment, importance ranking, and task-weighted scoring."
readme = "README.md"
repository.workspace = true
license = "AGPL-3.0-or-later AND MIT"
keywords = ["ast", "classification", "semantic", "tree-sitter"]
categories = ["development-tools", "parsing"]
include.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
thread-language = { workspace = true, default-features = false }
toml = "0.8"

[dev-dependencies]
insta = "1.41"  # snapshot testing for classification outputs

[features]
default = []
# Include all embedded data (universal rules + all language overrides)
embedded-data = []

[lints]
workspace = true
```

### 7.2 Workspace Integration

Add to `thread/Cargo.toml`:

```toml
[workspace]
members = [
  "crates/ast-engine",
  "crates/definitions",    # NEW
  "crates/flow",
  # ...
]

[workspace.dependencies]
thread-definitions = { path = "crates/definitions", default-features = false }
```

---

## 8. Implementation Plan

### Phase 1: Core Types & Data Loading (Week 1)

**Goal**: Define all types, load and parse data files, pass `cargo test`.

| Task | Est. Lines | Description |
|------|-----------|-------------|
| `types.rs` | ~200 | `SemanticClass`, `ImportanceRank`, `TokenPurpose`, `Confidence`, `ClassificationMethod`, `NodeKind`, `Classification` enums/structs with serde derives |
| `error.rs` | ~40 | `ClassifierError` enum with `thiserror` |
| `rules.rs` | ~250 | Deserialize `universal_rules.json`, `categories.json`, TOML overrides into `HashMap`s |
| `scoring.rs` | ~120 | `ImportanceScores`, `AgentTask`, task profile loading from `scoring.json` |
| `lib.rs` | ~50 | Re-exports, module declarations |
| Data files | — | Copy from CodeWeaver's `data/classifications/` |
| Unit tests | ~200 | Parse all data files, verify counts, round-trip serde |

**Milestone**: All data files parse successfully. Type system compiles.

### Phase 2: Classification Pipeline (Week 2)

**Goal**: Implement the 8-stage lookup pipeline, achieve 99.7% accuracy against ground truth.

| Task | Est. Lines | Description |
|------|-----------|-------------|
| `classifier.rs` | ~300 | `Classifier` struct, `new()`, `classify()`, `classify_simple()` |
| Integration tests | ~400 | Port holdout evaluation as a Rust test. Load all 27 language JSON files, classify every thing, compare against ground truth. Assert ≥99% accuracy. |
| Snapshot tests | ~100 | `insta` snapshots for classification outputs of representative node types |

**Milestone**: `cargo test` passes with 99.7% accuracy across all 27 languages.

### Phase 3: Scoring & API Polish (Week 3)

**Goal**: Scoring system works, public API is clean, documentation complete.

| Task | Est. Lines | Description |
|------|-----------|-------------|
| Scoring integration | ~80 | Wire `ImportanceScores` lookup and `AgentTask` scoring into `Classifier` |
| `ClassifierStats` | ~60 | Introspection: rule counts, override counts, language list |
| Documentation | ~200 | Rustdoc for all public items, module-level docs, examples |
| Benchmarks | ~100 | `criterion` bench: classify 5,000 node types, measure throughput |

**Milestone**: Complete, documented, benchmarked crate ready for integration.

### Phase 4: Thread Integration (Week 4)

**Goal**: Wire into `thread-flow` and `thread-services`.

| Task | Description |
|------|-------------|
| Add `thread-definitions` dependency to `thread-flow` | Feature-gated |
| Create classification step in flow pipeline | Between parse and symbol extraction |
| Update `thread-services` to expose classification via API | MCP/CLI integration |
| End-to-end test | Parse a real file → classify all nodes → verify output |

### Estimated Totals

| Metric | Estimate |
|--------|----------|
| Rust source lines | ~1,800 |
| Test lines | ~700 |
| Data files (JSON + TOML) | ~1,100 lines of overrides + ~5,000 lines of JSON |
| New dependencies | 1 (`toml`) |
| Compilation impact | Minimal — no tree-sitter parsers linked |
| Timeline | 4 weeks |

---

## 9. Testing Strategy

### 9.1 Ground Truth Tests

The primary correctness test is a Rust port of `holdout-evaluation.py`:

```rust
#[test]
fn test_classification_accuracy_all_languages() {
    let classifier = Classifier::new().unwrap();
    let mut total_correct = 0;
    let mut total_things = 0;

    for lang_file in std::fs::read_dir("../../codeweaver-data/classifications/").unwrap() {
        let data: LanguageData = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let lang = parse_lang(&data.language);

        for (name, entry) in data.tokens.iter().chain(data.composites.iter()) {
            let expected = &entry.classification;
            let result = classifier.classify(name, lang, entry.kind(), entry.purpose(), entry.is_file, &entry.categories);
            if result.class.as_str() == expected {
                total_correct += 1;
            }
            total_things += 1;
        }
    }

    let accuracy = total_correct as f64 / total_things as f64 * 100.0;
    assert!(accuracy >= 99.0, "Accuracy {accuracy:.1}% below threshold");
}
```

### 9.2 Holdout Tests

Test the "just add a grammar" hypothesis by excluding each language from universal rules:

```rust
#[test]
fn test_holdout_evaluation() {
    // For each language, rebuild universal rules excluding it,
    // classify with universal-only, measure baseline accuracy.
    // Verify ≥75% coverage and ≥80% accuracy without overrides.
    // Verify ≥99% with overrides.
}
```

### 9.3 Snapshot Tests

```rust
#[test]
fn test_python_classifications_snapshot() {
    let classifier = Classifier::new().unwrap();
    let key_types = ["function_definition", "class_definition", "import_statement",
                     "if_statement", "for_statement", "identifier", "+", "string"];
    let results: Vec<_> = key_types.iter()
        .map(|name| (name, classifier.classify_simple(name, SupportLang::Python, NodeKind::Composite)))
        .collect();
    insta::assert_debug_snapshot!(results);
}
```

### 9.4 Property Tests

```rust
#[test]
fn test_every_class_has_rank() {
    for class in SemanticClass::all() {
        let rank = class.rank();
        assert!((1..=5).contains(&(rank as u8)));
    }
}

#[test]
fn test_every_class_has_scores() {
    let classifier = Classifier::new().unwrap();
    for class in SemanticClass::all() {
        let scores = classifier.importance_scores(class);
        assert!(scores.discovery >= 0.0 && scores.discovery <= 1.0);
        // ... etc
    }
}
```

---

## 10. Performance Characteristics

### 10.1 Expected Performance

| Operation | Expected | Notes |
|-----------|----------|-------|
| `Classifier::new()` | ~5ms | Parse JSON + TOML, build HashMaps |
| `classify()` per call | ~50ns | HashMap lookup (1-3 lookups typical) |
| Classify all 5,899 things | ~300μs | All 27 languages |
| Memory footprint | ~2MB | All rules + overrides loaded |

### 10.2 Optimization Notes

- Universal exact rules (2,444 entries) are the hot path — HashMap with `Box<str>` keys
  avoids allocation on lookup.
- Override maps are small (avg 41 entries) — could use `BTreeMap` or sorted `Vec` for
  cache-friendly scanning, but `HashMap` is fine at this scale.
- No regex in the hot path. The name heuristic step uses `str::contains`, not regex.
- `Classifier` is `Send + Sync` — can be shared across threads with `Arc<Classifier>`.

---

## 11. Migration Path from CodeWeaver

### 11.1 Data Migration

| CodeWeaver File | Thread File | Action |
|-----------------|-------------|--------|
| `_universal_rules.json` | `data/universal_rules.json` | Copy, strip `description` and count fields |
| `_categories.json` | `data/categories.json` | Copy, extract `mapping` field only |
| `_scoring.json` | `data/scoring.json` | Copy as-is |
| `overrides/*.toml` | `data/overrides/*.toml` | Copy as-is |
| `{lang}.json` (27 files) | Test fixtures only | Used for ground-truth validation, not shipped |

### 11.2 Code Replacement Map

| CodeWeaver Module | Lines | Thread Replacement | Lines |
|-------------------|-------|--------------------|-------|
| `grammar.py` | 1,508 | Not ported (grammar model not needed for classification) | 0 |
| `token_patterns.py` | 1,354 | `TokenPurpose` enum + data | ~30 |
| `classifier.py` | 1,116 | `classifier.rs` | ~300 |
| `classifications.py` | 1,173 | `types.rs` + `scoring.rs` | ~320 |
| `types.py` | 562 | Not needed (no node-types.json parsing at runtime) | 0 |
| `registry.py` | 476 | `HashMap`s in `Classifier` | ~50 |
| `node_type_parser.py` | 921 | Not needed (classification from data, not grammar parsing) | 0 |
| `scoring.py` | 108 | `scoring.rs` | ~120 |
| **Total** | **~7,218** | | **~820** |

**Reduction**: ~88% less code for the same classification accuracy.

### 11.3 What Is NOT Ported

1. **Grammar model** (`grammar.py`): The `Thing/CompositeThing/Token/Category/Connection`
   type hierarchy. Thread doesn't need to parse `node-types.json` at runtime — all
   classification decisions are pre-baked into the data files.

2. **Node type parser** (`node_type_parser.py`): Same reason — no runtime grammar parsing.

3. **Registry** (`registry.py`): Replaced by simple `HashMap`s.

4. **AST-grep integration** (`ast_grep.py`): Thread uses tree-sitter directly.

5. **Pickle cache**: Replaced by JSON/TOML data files.

6. **Pydantic models**: Replaced by plain Rust structs with serde derives.

These components may be ported later if Thread needs runtime grammar analysis (e.g., for
the "auto-classify new languages" feature). That would be a separate crate
(`thread-grammars`) built on top of `thread-definitions`.

---

## 12. Future Extensions

### 12.1 Auto-Classification for New Languages

The "just add a grammar" pipeline for languages not yet covered:

1. Parse `node-types.json` → extract thing names, categories, token purposes
2. Classify using universal rules only (expect ~80% accuracy)
3. Run audit mode → report unclassified/uncertain items
4. Human writes ~30-50 TOML override entries
5. Validate: expect ~100% accuracy

This requires porting `node_type_parser.py` — estimated as a separate `thread-grammars`
crate (~500 lines of Rust).

### 12.2 C-Family Override Generalization

C, C++, and C# share many preprocessor directive tokens (`#define`, `#include`, `#ifdef`,
etc.) that could be factored into a shared `c-family.toml` base file, with language-specific
files inheriting from it. This would reduce the combined ~252 override lines to ~180.
Not necessary for correctness but a nice DX improvement.

### 12.3 Context Pack Integration

When Thread's context pack system is ready, `thread-definitions` provides the scoring
backbone:

```rust
// Rank definitions in a context pack by task relevance
let mut pack: Vec<(Definition, f32)> = definitions
    .iter()
    .map(|def| {
        let score = classifier.task_score(def.semantic_class, AgentTask::Debug);
        (def.clone(), score)
    })
    .collect();
pack.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
```

---

## Appendix A: SemanticClass ↔ ImportanceRank Mapping

| Rank | Value | Classifications |
|------|-------|----------------|
| PRIMARY_DEFINITIONS | 1 | `file_thing`, `definition_callable`, `definition_type`, `definition_data`, `definition_test` |
| BEHAVIORAL_CONTRACTS | 2 | `boundary_module`, `boundary_error`, `boundary_resource`, `documentation_structured` |
| CONTROL_FLOW_LOGIC | 3 | `flow_branching`, `flow_iteration`, `flow_control`, `flow_async` |
| OPERATIONS_EXPRESSIONS | 4 | `operation_invocation`, `operation_data`, `operation_operator`, `expression_anonymous` |
| SYNTAX_REFERENCES | 5 | `syntax_keyword`, `syntax_identifier`, `syntax_literal`, `syntax_annotation`, `syntax_punctuation` |

## Appendix B: Override Cost by Language

| Language | Override Lines | Baseline Accuracy | Final Accuracy |
|----------|---------------|-------------------|----------------|
| yaml | 0 | 100.0% | 100.0% |
| json | 1 | 100.0% | 100.0% |
| hcl | 2 | 97.0% | 100.0% |
| html | 10 | 63.0% | 100.0% |
| lua | 12 | 81.4% | 100.0% |
| bash | 13 | 87.0% | 100.0% |
| typescript | 13 | 89.9% | 100.0% |
| jsx | 14 | 92.3% | 100.0% |
| tsx | 14 | 89.2% | 100.0% |
| javascript | 16 | 90.2% | 100.0% |
| nix | 16 | 84.5% | 100.0% |
| elixir | 20 | 84.3% | 100.0% |
| kotlin | 22 | 90.2% | 100.0% |
| go | 25 | 86.3% | 100.0% |
| ruby | ~40 | 75.2% | 97.5% |
| python | 43 | 76.2% | 100.0% |
| css | 44 | 57.7% | 100.0% |
| rust | ~45 | 73.5% | 95.6% |
| php | 46 | 75.9% | 100.0% |
| java | 54 | 77.9% | 100.0% |
| c | 57 | 72.5% | 100.0% |
| solidity | 63 | 76.2% | 100.0% |
| scala | 71 | 69.4% | 100.0% |
| haskell | 85 | 67.8% | 100.0% |
| csharp | 90 | 73.2% | 100.0% |
| cpp | 105 | 70.7% | 100.0% |
| swift | 112 | 67.6% | 100.0% |

**Note**: Ruby (97.5%) and Rust (95.6%) are the two not at 100% — their overrides were
hand-written in an earlier round. Regenerating with the automated pipeline would bring
them to 100%.
