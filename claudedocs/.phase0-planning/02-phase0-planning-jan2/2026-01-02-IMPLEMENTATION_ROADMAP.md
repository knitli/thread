<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Phase 0 Implementation Roadmap
## 3-4 Week Plan to Completion

**Start Date**: TBD  
**Target Completion**: 3-4 weeks from start  
**Current Status**: 25-30% complete

---

## Week 1: Foundation & Fixes

### Day 1-2: Fix Compilation Errors 🔧

**Goal**: Get services crate compiling

**Tasks**:
1. Fix `crates/services/src/types.rs` type parameter issues
   ```rust
   // Add PhantomData markers
   use std::marker::PhantomData;
   
   pub struct ParsedDocument<D: Doc> {
       pub ast_root: Root<D>,
       pub metadata: DocumentMetadata,
       internal: Box<dyn Any + Send + Sync>,
       _phantom: PhantomData<D>, // FIX: Add this
   }
   ```

2. Fix stub types when ast-grep-backend disabled
   - Option A: Make ast-grep-backend a required feature
   - Option B: Fix stub types to match real signatures

3. Verify workspace builds
   ```bash
   cargo check --workspace --features thread-services/ast-grep-backend,thread-language/all-parsers
   ```

**Success Criteria**:
- ✅ Zero compilation errors in services crate
- ✅ `cargo check --workspace` succeeds
- ✅ All crates compile cleanly

**Estimated Time**: 2 days

---

### Day 3-5: Minimal Implementation 🚀

**Goal**: Create working ast-grep bridge

**Tasks**:

#### 1. Create implementation structure
```bash
mkdir -p crates/services/src/implementations
mkdir -p crates/services/src/testing
mkdir -p crates/services/tests
```

#### 2. Implement AstGrepParser (Day 3)

**File**: `crates/services/src/implementations/ast_grep.rs`

```rust
use thread_ast_engine::{Language as AstLanguage};
use thread_language::SupportLang;
use crate::types::*;
use crate::traits::*;
use crate::error::*;

pub struct AstGrepParser;

impl AstGrepParser {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CodeParser for AstGrepParser {
    async fn parse_content(
        &self,
        content: &str,
        language: SupportLang,
        context: &AnalysisContext,
    ) -> ServiceResult<ParsedDocument<impl Doc>> {
        // 1. Get ast-grep Language instance
        let ast_lang = language.get_ts_language();
        
        // 2. Parse content using ast-grep
        let root = ast_lang.ast_grep(content);
        
        // 3. Compute content hash
        let content_hash = thread_utils::rapidhash::hash(content.as_bytes());
        
        // 4. Create ParsedDocument
        let mut doc = ParsedDocument::new(
            root,
            context.base_directory.join("file.rs"), // TODO: real path
            language,
            content_hash,
        );
        
        // 5. Extract basic metadata (for now just placeholder)
        // TODO: Implement in Week 2
        
        Ok(doc)
    }
    
    // Implement other required methods...
    async fn parse_file(&self, file_path: &Path, context: &AnalysisContext) 
        -> ServiceResult<ParsedDocument<impl Doc>> 
    {
        let content = std::fs::read_to_string(file_path)?;
        let language = self.detect_language(file_path)?;
        self.parse_content(&content, language, context).await
    }
    
    // ... remaining methods
}
```

#### 3. Implement AstGrepAnalyzer (Day 4)

**File**: `crates/services/src/implementations/ast_grep.rs`

```rust
pub struct AstGrepAnalyzer;

#[async_trait]
impl CodeAnalyzer for AstGrepAnalyzer {
    async fn find_pattern<D: Doc>(
        &self,
        document: &ParsedDocument<D>,
        pattern: &str,
        context: &AnalysisContext,
    ) -> ServiceResult<Vec<CodeMatch<'_, D>>> {
        // 1. Get ast-grep root
        let root = document.ast_grep_root();
        
        // 2. Find all matches using ast-grep
        let ast_matches = root.root().find_all(pattern);
        
        // 3. Convert to CodeMatch instances
        let matches = ast_matches
            .map(|node_match| {
                let mut code_match = CodeMatch::new(node_match);
                // TODO: Add cross-file relationships in Week 2
                code_match
            })
            .collect();
        
        Ok(matches)
    }
    
    // Implement other required methods...
}
```

#### 4. Create Mock Implementations (Day 5)

**File**: `crates/services/src/testing/mock_parser.rs`

```rust
pub struct MockParser {
    // Deterministic behavior for testing
    should_fail: bool,
    parse_delay_ms: u64,
}

impl MockParser {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            parse_delay_ms: 0,
        }
    }
    
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl CodeParser for MockParser {
    async fn parse_content(&self, content: &str, language: SupportLang, context: &AnalysisContext)
        -> ServiceResult<ParsedDocument<impl Doc>>
    {
        if self.should_fail {
            return Err(ParseError::InvalidSource { 
                message: "Mock failure".into() 
            }.into());
        }
        
        // Simulate delay
        if self.parse_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.parse_delay_ms)).await;
        }
        
        // Return mock parsed document
        // TODO: Create proper mock
        todo!("Create mock ParsedDocument")
    }
    
    // ... other methods
}
```

#### 5. Add Initial Tests

**File**: `crates/services/tests/contract_tests.rs`

```rust
#[cfg(test)]
mod parser_contract_tests {
    use thread_services::*;
    
    #[tokio::test]
    async fn test_ast_grep_parser_follows_contract() {
        let parser = AstGrepParser::new();
        test_parser_contract(&parser).await;
    }
    
    #[tokio::test]
    async fn test_mock_parser_follows_contract() {
        let parser = MockParser::new();
        test_parser_contract(&parser).await;
    }
    
    async fn test_parser_contract<P: CodeParser>(parser: &P) {
        // Test that parser follows CodeParser contract
        let content = "fn main() {}";
        let lang = SupportLang::Rust;
        let context = AnalysisContext::default();
        
        // Should parse valid content
        let result = parser.parse_content(content, lang, &context).await;
        assert!(result.is_ok());
        
        // TODO: More contract tests
    }
}
```

**Success Criteria**:
- ✅ AstGrepParser compiles and basic parse_content works
- ✅ AstGrepAnalyzer compiles and basic find_pattern works
- ✅ MockParser/MockAnalyzer compile
- ✅ At least one integration test passes
- ✅ `cargo test -p thread-services` runs

**Estimated Time**: 3 days

---

## Week 2: Complete Implementation

### Day 6-8: Metadata Extraction 📊

**Goal**: Implement symbol, import, export extraction

**Tasks**:

#### 1. Symbol Extraction
**File**: `crates/services/src/conversion.rs`

```rust
pub fn extract_symbols<D: Doc>(
    root: &Root<D>,
    language: SupportLang,
) -> HashMap<String, SymbolInfo> {
    let mut symbols = HashMap::new();
    
    match language {
        SupportLang::Rust => {
            // Find function definitions
            for func in root.root().find_all("fn $NAME($$$) { $$$ }") {
                if let Some(name) = func.get_env().get_match("NAME") {
                    symbols.insert(
                        name.text().to_string(),
                        SymbolInfo {
                            name: name.text().to_string(),
                            kind: SymbolKind::Function,
                            position: name.range(),
                            // ...
                        }
                    );
                }
            }
            
            // Find struct definitions
            for struct_def in root.root().find_all("struct $NAME { $$$ }") {
                // Extract struct info
            }
            
            // ... more patterns
        },
        SupportLang::JavaScript | SupportLang::TypeScript => {
            // JavaScript-specific patterns
        },
        // ... other languages
    }
    
    symbols
}
```

#### 2. Import/Export Extraction
```rust
pub fn extract_imports<D: Doc>(
    root: &Root<D>,
    language: SupportLang,
) -> HashMap<String, ImportInfo> {
    // Similar pattern-based extraction
}

pub fn extract_exports<D: Doc>(
    root: &Root<D>,
    language: SupportLang,
) -> HashMap<String, ExportInfo> {
    // Similar pattern-based extraction
}
```

#### 3. Integrate into Parser
```rust
async fn postprocess_document<D: Doc>(
    &self,
    mut document: ParsedDocument<D>,
    context: &AnalysisContext,
) -> ServiceResult<ParsedDocument<D>> {
    // Extract metadata
    let symbols = extract_symbols(document.ast_grep_root(), document.language);
    document.metadata_mut().defined_symbols = symbols;
    
    let imports = extract_imports(document.ast_grep_root(), document.language);
    document.metadata_mut().imported_symbols = imports;
    
    let exports = extract_exports(document.ast_grep_root(), document.language);
    document.metadata_mut().exported_symbols = exports;
    
    Ok(document)
}
```

**Success Criteria**:
- ✅ Can extract functions from Rust code
- ✅ Can extract imports/exports from Rust code
- ✅ Tests verify extraction works correctly
- ✅ At least 2 languages supported (Rust + JavaScript)

**Estimated Time**: 3 days

---

### Day 9-10: Cross-File Analysis 🔗

**Goal**: Implement relationship building

**Tasks**:

#### 1. Cross-File Analyzer
```rust
async fn analyze_cross_file_relationships<D: Doc>(
    &self,
    documents: &[ParsedDocument<D>],
    context: &AnalysisContext,
) -> ServiceResult<Vec<CrossFileRelationship>> {
    let mut relationships = Vec::new();
    
    // Build symbol map across all files
    let mut symbol_locations = HashMap::new();
    for doc in documents {
        for (name, symbol) in &doc.metadata().defined_symbols {
            symbol_locations.insert(name.clone(), doc.file_path.clone());
        }
    }
    
    // Find cross-file references
    for doc in documents {
        // Match imports to definitions
        for (import_name, import_info) in &doc.metadata().imported_symbols {
            if let Some(target_file) = symbol_locations.get(import_name) {
                relationships.push(CrossFileRelationship {
                    kind: RelationshipKind::Imports,
                    source_file: doc.file_path.clone(),
                    target_file: target_file.clone(),
                    source_symbol: import_name.clone(),
                    target_symbol: import_name.clone(),
                    relationship_data: HashMap::new(),
                });
            }
        }
        
        // TODO: Function calls, inheritance, etc.
    }
    
    Ok(relationships)
}
```

**Success Criteria**:
- ✅ Can identify import relationships across files
- ✅ Tests verify relationship building
- ✅ Example workflow demonstrates capability

**Estimated Time**: 2 days

---

## Week 3: Testing & Validation

### Day 11-12: Comprehensive Testing 🧪

**Goal**: Build complete test suite

**Tasks**:

#### 1. Contract Tests
```rust
// Test all implementations follow same contract
#[test]
fn all_parsers_follow_contract() {
    let parsers: Vec<Box<dyn CodeParser>> = vec![
        Box::new(AstGrepParser::new()),
        Box::new(MockParser::new()),
    ];
    
    for parser in parsers {
        test_parser_contract(&parser);
    }
}

fn test_parser_contract(parser: &dyn CodeParser) {
    // Comprehensive contract validation
    test_parse_valid_content(parser);
    test_parse_invalid_content(parser);
    test_language_detection(parser);
    test_capabilities(parser);
}
```

#### 2. Integration Tests
```rust
#[tokio::test]
async fn test_complete_analysis_workflow() {
    let parser = AstGrepParser::new();
    let analyzer = AstGrepAnalyzer::new();
    
    // Parse file
    let doc = parser.parse_file(
        Path::new("test_data/sample.rs"),
        &AnalysisContext::default()
    ).await.unwrap();
    
    // Verify metadata
    assert!(!doc.metadata().defined_symbols.is_empty());
    
    // Find patterns
    let matches = analyzer.find_pattern(
        &doc,
        "fn $NAME($$$) { $$$ }",
        &AnalysisContext::default()
    ).await.unwrap();
    
    assert!(!matches.is_empty());
}
```

#### 3. Property-Based Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_any_valid_rust_code(code in any_valid_rust_code()) {
        let parser = AstGrepParser::new();
        let result = parser.parse_content(
            &code,
            SupportLang::Rust,
            &AnalysisContext::default()
        ).await;
        
        // Should either parse successfully or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}
```

**Success Criteria**:
- ✅ 100% test coverage for service implementations
- ✅ All contract tests pass
- ✅ All integration tests pass
- ✅ Property-based tests provide confidence

**Estimated Time**: 2 days

---

### Day 13-14: Performance Validation ⚡

**Goal**: Verify <5% overhead target

**Tasks**:

#### 1. Create Benchmarks
**File**: `crates/services/benches/service_benchmarks.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_direct_ast_grep(c: &mut Criterion) {
    let content = include_str!("../test_data/large_file.rs");
    
    c.bench_function("direct ast-grep parse", |b| {
        b.iter(|| {
            let lang = thread_language::Rust;
            let root = lang.ast_grep(black_box(content));
            black_box(root)
        })
    });
}

fn bench_service_layer_parse(c: &mut Criterion) {
    let content = include_str!("../test_data/large_file.rs");
    let parser = AstGrepParser::new();
    
    c.bench_function("service layer parse", |b| {
        b.iter(|| async {
            let result = parser.parse_content(
                black_box(content),
                SupportLang::Rust,
                &AnalysisContext::default()
            ).await;
            black_box(result)
        })
    });
}

criterion_group!(benches, bench_direct_ast_grep, bench_service_layer_parse);
criterion_main!(benches);
```

#### 2. Run Benchmarks
```bash
cargo bench -p thread-services
```

#### 3. Analyze Results
- Document overhead percentage
- If >5%, profile and optimize
- Use #[inline] on hot paths
- Consider removing unnecessary async

**Success Criteria**:
- ✅ Benchmarks run successfully
- ✅ Overhead < 5% for parsing
- ✅ Overhead < 5% for pattern matching
- ✅ Memory usage < 10% increase

**Estimated Time**: 2 days

---

### Day 15: Documentation & Examples 📚

**Goal**: Complete documentation

**Tasks**:

#### 1. API Documentation
```rust
//! # Thread Services - Complete Usage Guide
//!
//! ## Overview
//! Thread services provide a clean abstraction over ast-grep...
//!
//! ## Quick Start
//! ```rust
//! use thread_services::*;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let parser = AstGrepParser::new();
//! let analyzer = AstGrepAnalyzer::new();
//!
//! // Parse code
//! let doc = parser.parse_file(
//!     Path::new("src/main.rs"),
//!     &AnalysisContext::default()
//! ).await?;
//!
//! // Find patterns
//! let matches = analyzer.find_pattern(
//!     &doc,
//!     "fn $NAME($$$) { $$$ }",
//!     &AnalysisContext::default()
//! ).await?;
//! # Ok(())
//! # }
//! ```
```

#### 2. Create Examples
**File**: `crates/services/examples/basic_usage.rs`

```rust
//! Basic usage of Thread services

use thread_services::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create parser and analyzer
    let parser = AstGrepParser::new();
    let analyzer = AstGrepAnalyzer::new();
    
    // Parse a Rust file
    let content = r#"
        fn hello() {
            println!("Hello, world!");
        }
        
        fn goodbye() {
            println!("Goodbye!");
        }
    "#;
    
    let doc = parser.parse_content(
        content,
        SupportLang::Rust,
        &AnalysisContext::default()
    ).await?;
    
    // Find all function declarations
    let matches = analyzer.find_pattern(
        &doc,
        "fn $NAME($$$) { $$$ }",
        &AnalysisContext::default()
    ).await?;
    
    println!("Found {} functions:", matches.len());
    for m in matches {
        if let Some(name) = m.get_env().get_match("NAME") {
            println!("  - {}", name.text());
        }
    }
    
    Ok(())
}
```

#### 3. Migration Guide
**File**: `crates/services/MIGRATION.md`

```markdown
# Migrating from Direct ast-grep to Thread Services

## Before (Direct ast-grep)
```rust
use thread_language::Rust;

let root = Rust.ast_grep(content);
let matches = root.root().find_all("fn $NAME($$$) { $$$ }");
```

## After (Thread Services)
```rust
use thread_services::*;

let parser = AstGrepParser::new();
let analyzer = AstGrepAnalyzer::new();

let doc = parser.parse_content(content, SupportLang::Rust, &context).await?;
let matches = analyzer.find_pattern(&doc, "fn $NAME($$$) { $$$ }", &context).await?;
```

## Benefits
- Testable with mock implementations
- Codebase-level metadata
- Commercial extension points
- Future-proof abstraction
```

**Success Criteria**:
- ✅ All public APIs documented
- ✅ At least 3 working examples
- ✅ Migration guide complete
- ✅ Performance characteristics documented

**Estimated Time**: 1 day

---

## Week 4: Polish & Buffer

### Day 16-18: Final Validation

**Tasks**:
1. Run full test suite multiple times
2. Test all feature combinations
3. Verify CI pipeline works
4. Fix any discovered issues
5. Code review and cleanup

### Day 19-20: Buffer

- Handle unexpected issues
- Additional testing
- Documentation improvements
- Final polish

---

## Success Criteria (Phase 0 Complete)

### Functional
- [ ] All existing ast-engine functionality accessible through services ✅
- [ ] Mock implementations can be swapped for testing ✅
- [ ] Commercial boundaries enforced by feature flags ✅
- [ ] Metadata extraction working (symbols, imports, exports) ✅
- [ ] Cross-file relationship analysis working ✅

### Non-Functional
- [ ] Performance regression < 5% ✅
- [ ] Memory usage increase < 10% ✅
- [ ] Compilation time increase < 15% ✅
- [ ] Workspace builds successfully ✅
- [ ] All tests pass ✅

### Quality
- [ ] 100% test coverage for service implementations ✅
- [ ] Property-based tests validate contracts ✅
- [ ] Integration tests cover complete workflows ✅
- [ ] Performance benchmarks validate targets ✅

### Documentation
- [ ] API documentation complete ✅
- [ ] Implementation examples working ✅
- [ ] Migration guide from direct ast-grep ✅
- [ ] Performance characteristics documented ✅

---

## Daily Standup Template

```markdown
### What I did yesterday:
- [Task completed]
- [Issue encountered]

### What I'm doing today:
- [Current focus]
- [Expected completion]

### Blockers:
- [Any blockers]
- [Help needed]

### Phase 0 Progress:
- Week X, Day Y
- [X%] complete overall
```

---

## Emergency Scope Reduction

If timeline is at risk, reduce scope in this order:

1. **Keep** (Critical path):
   - Basic AstGrepParser/Analyzer
   - Compilation fixes
   - Basic tests
   - Core metadata extraction (functions only)

2. **Defer to Week 5** (Important but not blocking):
   - Advanced metadata (types, exports)
   - Cross-file analysis
   - Multiple language support (focus on Rust first)
   - Performance optimization

3. **Defer to Phase 1** (Nice to have):
   - CompositeService
   - Advanced execution strategies
   - Plugin system integration
   - WASM optimization

---

## Resources & References

### Code References
- `crates/ast-engine/src/lib.rs` - AST operations to wrap
- `crates/language/src/lib.rs` - Language implementations
- Prior assessment documents for context

### Testing Resources
- Use `criterion` for benchmarks
- Use `tokio::test` for async tests
- Use `proptest` for property-based tests

### Performance Tools
```bash
# Profile performance
cargo flamegraph --bench service_benchmarks

# Memory profiling
cargo valgrind --bench service_benchmarks

# Check binary size
cargo bloat --release
```

---

**Roadmap Status**: Draft  
**Next Review**: After Week 1 completion  
**Estimated Completion**: 3-4 weeks from start  

**Remember**: Focus on getting Phase 0 working, not perfect. Optimization can come in later phases.
