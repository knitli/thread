<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Phase 0 Implementation Plan: Service Abstraction Layer

## Executive Summary

**Objective**: Create a clean abstraction layer for thread-services that isolates ast-grep behind swappable service interfaces, enabling commercial extensions and improved testability.

**Timeline**: 3 weeks (21 days)
**Risk Level**: Low-Medium
**Success Criteria**: All existing functionality works through service layer with <5% performance regression

## Problem Statement

### Current State

- Direct dependency on ast-grep API: `Language::Tsx.ast_grep(content)`
- Tight coupling makes testing, swapping implementations, and commercial extensions difficult
- No clear boundaries between public and commercial features

### Target State

- Clean service traits abstract parsing and analysis operations
- ast-grep wrapped behind implementation that can be swapped for testing or alternatives
- Clear commercial extension points through feature-gated traits
- Maintainable service boundaries with contract testing

## Architecture Design

### Core Service Traits

```rust
// thread-services/src/traits/parser.rs
#[async_trait::async_trait]
pub trait CodeParser: Send + Sync {
    async fn parse(&self, content: &str, language: SupportedLanguage)
        -> Result<ParsedCode, ParseError>;
    fn supported_languages(&self) -> &[SupportedLanguage];
    fn capabilities(&self) -> ParserCapabilities;
}

// thread-services/src/traits/analyzer.rs
pub trait CodeAnalyzer: Send + Sync {
    fn find_pattern(&self, code: &ParsedCode, pattern: &str)
        -> Result<Vec<Match>, AnalysisError>;
    fn find_all_patterns(&self, code: &ParsedCode, pattern: &str)
        -> Result<Vec<Match>, AnalysisError>;
    fn replace_pattern(&self, code: &mut ParsedCode, pattern: &str, replacement: &str)
        -> Result<(), AnalysisError>;
}

// thread-services/src/traits/storage.rs (Commercial Boundary)
pub trait StorageService: Send + Sync {
    fn store_analysis(&self, result: &AnalysisResult) -> Result<(), StorageError>;
    fn load_cached_analysis(&self, key: &CacheKey) -> Option<AnalysisResult>;
}
```

### Language-Agnostic Data Structures

```rust
// thread-services/src/types.rs
pub struct ParsedCode {
    source: String,
    language: SupportedLanguage,
    internal: Box<dyn Any + Send + Sync>, // Hidden ast-engine types
}

pub struct Match {
    text: String,
    range: Range,
    meta_vars: HashMap<String, String>,
    internal: Box<dyn Any + Send + Sync>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Rust, JavaScript, TypeScript, Python, Go, Java,
    Cpp, CSharp, PHP, Ruby, Swift, Kotlin, Scala,
    // Tier 3 languages
    Bash, CSS, HTML, JSON, YAML, Lua, Elixir, Haskell,
}
```

### AST-Grep Service Implementation

```rust
// thread-services/src/implementations/ast_grep.rs
pub struct AstGrepParser {
    // Wraps thread-ast-engine internally
}

impl CodeParser for AstGrepParser {
    async fn parse(&self, content: &str, language: SupportedLanguage)
        -> Result<ParsedCode, ParseError> {
        let ast_lang = language.to_ast_engine_language();
        let ast = ast_lang.ast_grep(content);
        Ok(ParsedCode::new(content, language, Box::new(ast)))
    }
}

pub struct AstGrepAnalyzer;

impl CodeAnalyzer for AstGrepAnalyzer {
    fn find_pattern(&self, code: &ParsedCode, pattern: &str)
        -> Result<Vec<Match>, AnalysisError> {
        let ast = code.extract_ast_engine_type()?;
        let matches = ast.root().find_all(pattern);
        Ok(matches.into_iter().map(|m| Match::from_ast_engine(m)).collect())
    }
}
```

### Enhanced Feature Flags

```toml
# thread-services/Cargo.toml
[features]
default = ["in-memory-only"]

# Core service implementations
in-memory-only = []
ast-grep-backend = ["thread-ast-engine", "thread-language"]

# Commercial service boundaries
persistence-traits = ["serde"]  # Trait definitions only
storage-implementations = ["persistence-traits"]  # Commercial only

# Intelligence service boundaries
intelligence-traits = []  # Public trait definitions
intelligence-implementations = ["intelligence-traits"]  # Commercial only

# Plugin system
plugins = ["libloading"]
extensions = ["plugins"]
```

## Directory Structure

```plaintext
thread-services/
├── src/
│   ├── lib.rs                    # Public API exports
│   ├── types.rs                  # 🆕 Language-agnostic data structures
│   ├── error.rs                  # 🆕 Enhanced error types
│   ├── traits/                   # 🆕 Core service trait definitions
│   │   ├── mod.rs
│   │   ├── parser.rs            # CodeParser trait
│   │   ├── analyzer.rs          # CodeAnalyzer trait
│   │   ├── storage.rs           # StorageService trait (commercial boundary)
│   │   └── context.rs           # ExecutionContext (existing)
│   ├── implementations/          # 🆕 Service implementations
│   │   ├── mod.rs
│   │   ├── ast_grep.rs          # AstGrepParser + AstGrepAnalyzer
│   │   ├── memory_only.rs       # In-memory implementations for testing
│   │   └── composite.rs         # Combined service orchestration
│   └── testing/                 # 🆕 Test utilities and mocks
│       ├── mod.rs
│       ├── mock_parser.rs
│       └── mock_analyzer.rs
├── tests/
│   ├── contract_tests.rs        # 🆕 Service boundary compliance tests
│   └── integration_tests.rs     # 🆕 End-to-end workflow tests
└── Cargo.toml                   # Enhanced with new feature flags
```

## Implementation Timeline

### Week 1: Foundation (Days 1-5)

#### Days 1-2: Data Structures & Type System

**Deliverables**:

- `src/types.rs` - Complete implementation
- `src/error.rs` - Enhanced error handling
- Basic conversion utilities between service types and ast-engine types

**Key Tasks**:

- Design ParsedCode wrapper with type erasure for ast-engine types
- Implement SupportedLanguage enum with conversion methods
- Create Match structure with meta-variable support
- Define Range, ParserCapabilities, and other supporting types

#### Days 3-4: Core Service Traits

**Deliverables**:

- `src/traits/parser.rs` - CodeParser trait with documentation
- `src/traits/analyzer.rs` - CodeAnalyzer trait with documentation
- `src/traits/storage.rs` - StorageService trait (commercial boundary)

**Key Tasks**:

- Define async CodeParser interface supporting all current ast-grep functionality
- Design CodeAnalyzer trait covering pattern matching and replacement
- Create StorageService trait with clear commercial boundaries
- Write comprehensive trait documentation with usage examples

#### Day 5: Feature Flags & Project Structure

**Deliverables**:

- Enhanced `Cargo.toml` with granular feature control
- `src/lib.rs` with conditional compilation for features
- Basic project structure setup

**Key Tasks**:

- Implement feature-gated compilation for commercial boundaries
- Set up conditional exports based on enabled features
- Create foundation for implementations/ directory

### Week 2: Implementation (Days 6-10)

#### Days 6-8: AST-Grep Service Implementation

**Deliverables**:

- `src/implementations/ast_grep.rs` - Complete wrapper implementation
- Working service layer that abstracts all ast-grep functionality

**Key Tasks**:

- Implement AstGrepParser wrapping thread-ast-engine Language API
- Implement AstGrepAnalyzer wrapping pattern matching and replacement
- Create conversion utilities between service types and ast-engine types
- Handle all existing ast-grep functionality through service interface
- Performance optimization with `#[inline]` where appropriate

#### Days 9-10: Testing Infrastructure & Mock Implementations

**Deliverables**:

- `src/implementations/memory_only.rs` - In-memory testing implementations
- `src/testing/` - Complete mock and testing utilities
- `src/implementations/composite.rs` - Service orchestration

**Key Tasks**:

- Create MockParser and MockAnalyzer for deterministic testing
- Implement MemoryOnlyStorage for test scenarios
- Build CompositeService that coordinates multiple service implementations
- Create testing utilities for service validation

### Week 3: Validation & Integration (Days 11-15)

#### Days 11-12: Contract Testing Suite

**Deliverables**:

- `tests/contract_tests.rs` - Comprehensive service boundary validation
- Property-based tests ensuring all implementations follow contracts

**Key Tasks**:

- Implement property-based tests for service trait contracts
- Create test suite validating commercial boundary enforcement
- Test that all service implementations behave consistently
- Validate feature flag behavior and conditional compilation

#### Days 13-14: Integration Testing & Migration

**Deliverables**:

- `tests/integration_tests.rs` - End-to-end workflow validation
- Migration guide for existing code using ast-engine directly

**Key Tasks**:

- Test complete analysis workflows through service layer
- Validate existing thread-ast-engine functionality works unchanged
- Create examples showing migration from direct ast-engine usage
- Test interaction with existing thread-rule-engine and thread-language

#### Day 15: Performance Validation & Documentation

**Deliverables**:

- Performance benchmarks showing <5% regression
- Complete API documentation
- Phase 0 implementation ready for Phase 1

**Key Tasks**:

- Benchmark critical paths (parsing, pattern matching, replacement)
- Optimize any performance bottlenecks identified
- Generate comprehensive API documentation
- Validate Phase 0 success criteria

## Risk Assessment & Mitigation

### High Risk: Performance Regression

**Risk**: Service trait indirection adds overhead to critical paths
**Mitigation**:

- Use `#[inline]` aggressively on hot paths
- Benchmark early and often
- Design zero-cost abstractions where possible
- Consider compile-time specialization for critical operations

### Medium Risk: Type System Complexity

**Risk**: Wrapping ast-engine types in generic containers becomes unwieldy
**Mitigation**:

- Use type erasure judiciously with `Box<dyn Any>`
- Provide helper functions for common type conversions
- Keep internal type complexity hidden from public APIs
- Consider newtype wrappers for frequently used combinations

### Medium Risk: Feature Flag Dependencies

**Risk**: Complex dependency management between commercial and public features
**Mitigation**:

- Start with minimal feature set and add complexity incrementally
- Use clear naming conventions for feature boundaries
- Test all feature combinations in CI
- Document feature dependencies clearly

### Low Risk: Testing Coverage Gaps

**Risk**: Service abstraction makes it harder to test implementation details
**Mitigation**:

- Focus on testing behavior through service interfaces
- Use property-based testing for trait contracts
- Maintain unit tests for critical implementation details
- Create comprehensive integration test suite

## Success Criteria

### Functional Requirements

- ✅ All existing thread-ast-engine functionality accessible through service layer
- ✅ Mock implementations can be swapped in for testing scenarios
- ✅ Commercial boundaries clearly enforced by feature flags
- ✅ Service traits support all current ast-grep operations (find, replace, etc.)

### Non-Functional Requirements

- ✅ Performance regression < 5% for critical parsing and analysis operations
- ✅ Memory usage increase < 10% due to abstraction overhead
- ✅ Compilation time increase < 15% with full feature set
- ✅ API surface area remains manageable and well-documented

### Quality Requirements

- ✅ 100% test coverage for service trait implementations
- ✅ Property-based tests validate all service contracts
- ✅ Integration tests cover complete analysis workflows
- ✅ Documentation covers migration path from direct ast-engine usage

## Commercial Extension Points

### Clear Service Boundaries

The service architecture creates natural extension points:

```rust
// Public: Available in open source
pub trait CodeParser { /* ... */ }
pub trait CodeAnalyzer { /* ... */ }

// Commercial: Feature-gated trait definitions
#[cfg(feature = "persistence-traits")]
pub trait StorageService { /* ... */ }

#[cfg(feature = "intelligence-traits")]
pub trait IntelligenceService { /* ... */ }

// Commercial: Implementation-only (not published)
struct EnterpriseStorageService { /* proprietary */ }
struct IntelligenceAnalyzer { /* proprietary */ }
```

### Plugin Architecture Foundation

```rust
pub trait ServicePlugin: Send + Sync {
    fn enhance_parser(&self, parser: Box<dyn CodeParser>) -> Box<dyn CodeParser>;
    fn enhance_analyzer(&self, analyzer: Box<dyn CodeAnalyzer>) -> Box<dyn CodeAnalyzer>;
}

// Commercial plugins can be dynamically loaded
#[cfg(feature = "commercial")]
fn load_commercial_plugins() {
    register_plugin(SecurityAnalysisPlugin::new());
    register_plugin(PerformanceAnalysisPlugin::new());
}
```

## Migration Strategy

### Backwards Compatibility

- Keep existing thread-ast-engine APIs unchanged during Phase 0
- New code can adopt service traits incrementally
- Existing code continues working without modification

### Adoption Path

```rust
// Current direct usage (continues to work)
let ast = Language::Tsx.ast_grep(content);
ast.replace("var $NAME = $VALUE", "let $NAME = $VALUE")?;

// New service-based usage
let parser = AstGrepParser::new();
let analyzer = AstGrepAnalyzer::new();
let parsed = parser.parse(content, SupportedLanguage::TypeScript).await?;
analyzer.replace_pattern(&mut parsed, "var $NAME = $VALUE", "let $NAME = $VALUE")?;
```

### Gradual Migration

1. **Phase 0**: Service layer implemented, existing code unchanged
2. **Phase 1**: New features use service layer exclusively
3. **Phase 2**: Migrate existing code to service layer incrementally
4. **Phase 3**: Deprecate direct ast-engine usage in public APIs

## Dependencies

### New Dependencies

```toml
async-trait = "0.1"  # For async trait definitions
libloading = { version = "0.8", optional = true }  # For plugin system
```

### Enhanced Existing Dependencies

- `thread-ast-engine`: Enhanced with service implementation wrapper
- `thread-language`: Extended with SupportedLanguage conversion utilities
- `thread-utils`: Additional utilities for type conversion and performance

## Testing Strategy

### Contract Testing

Property-based tests ensuring all service implementations follow the same contracts:

```rust
#[test]
fn all_parsers_follow_contract() {
    let parsers: Vec<Box<dyn CodeParser>> = vec![
        Box::new(AstGrepParser::new()),
        Box::new(MockParser::new()),
    ];

    for parser in parsers {
        // Test that all parsers behave consistently
        property_test_parser_contract(parser);
    }
}
```

### Integration Testing

End-to-end workflows through service layer:

```rust
#[test]
fn complete_analysis_workflow() {
    let parser = AstGrepParser::new();
    let analyzer = AstGrepAnalyzer::new();

    // Test complete workflow: parse → analyze → transform
    let result = run_complete_workflow(&parser, &analyzer, test_code);
    assert_eq!(result.expected_output());
}
```

### Performance Benchmarking

Continuous performance monitoring:

```rust
#[bench]
fn benchmark_service_vs_direct_parsing(b: &mut Bencher) {
    // Compare service layer performance vs direct ast-engine usage
}
```

## Conclusion

This Phase 0 implementation creates a solid foundation for the Thread architecture evolution. By abstracting ast-grep behind clean service interfaces, we achieve:

1. **Improved Testability**: Mock implementations enable comprehensive testing
2. **Commercial Extensibility**: Clear boundaries for proprietary features
3. **Implementation Flexibility**: Can swap parsers/analyzers without breaking changes
4. **Maintainable Architecture**: Service boundaries enforce clean separation of concerns

The 3-week timeline provides sufficient buffer for challenges while maintaining aggressive progress toward the Thread 2.0 vision. Success in Phase 0 directly enables the intelligence features and commercial extensions planned for subsequent phases.
