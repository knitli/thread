# Thread-Flow Integration Tests

Comprehensive integration test suite for the thread-flow crate, validating ReCoco dataflow integration and multi-language code parsing capabilities.

## Test Structure

### Test Data (`test_data/`)
- **`sample.rs`** - Realistic Rust code with structs, enums, functions, imports
- **`sample.py`** - Python code with classes, decorators, imports
- **`sample.ts`** - TypeScript code with interfaces, classes, enums
- **`sample.go`** - Go code with structs, interfaces, functions
- **`empty.rs`** - Empty file for edge case testing
- **`syntax_error.rs`** - File with intentional syntax errors
- **`large.rs`** - Larger file for performance testing (~100 lines)

### Test Categories

#### 1. Factory and Schema Tests (✅ Passing)
Tests verifying ReCoco integration works correctly:
- `test_factory_build_succeeds` - Factory creation
- `test_executor_creation` - Executor instantiation
- `test_schema_output_type` - Output schema validation
- `test_behavior_version` - Version tracking
- `test_executor_cache_enabled` - Caching configuration
- `test_executor_timeout` - Timeout configuration

#### 2. Error Handling Tests (✅ Passing)
Tests for proper error handling:
- `test_unsupported_language` - Invalid language detection
- `test_missing_content` - Missing required inputs
- `test_invalid_input_type` - Type validation
- `test_missing_language` - Incomplete inputs

#### 3. Value Serialization Tests (⏸️ Blocked)
Tests validating output structure matches schema:
- `test_output_structure_basic` - Basic structure validation
- `test_empty_tables_structure` - Empty file handling

**Status**: Blocked by pattern matching bug (see Known Issues)

#### 4. Language Support Tests (⏸️ Blocked)
Multi-language parsing validation:
- `test_parse_rust_code` - Rust parsing and extraction
- `test_parse_python_code` - Python parsing
- `test_parse_typescript_code` - TypeScript parsing
- `test_parse_go_code` - Go parsing
- `test_multi_language_support` - Sequential multi-language

**Status**: Blocked by pattern matching bug (see Known Issues)

#### 5. Performance Tests (⏸️ Blocked/Manual)
Performance benchmarking:
- `test_parse_performance` - Large file performance (<1s)
- `test_minimal_parse_performance` - Fast path performance (<100ms)

**Status**: Blocked by pattern matching bug; run manually when fixed

## Current Test Status

### ✅ Passing Tests: 10/19
All factory, schema, and error handling tests pass.

### ⏸️ Blocked Tests: 9/19
Tests blocked by known bug in thread-services conversion module.

## Known Issues

### Pattern Matching Bug

**Issue**: `extract_functions()` in `thread-services/src/conversion.rs` tries all language patterns sequentially and panics when a pattern doesn't parse for the current language.

**Root Cause**:
- `Pattern::new()` calls `.unwrap()` instead of returning `Result<Pattern, PatternError>`
- Location: `crates/ast-engine/src/matchers/pattern.rs:220`
- Example: JavaScript `function` pattern fails to parse on Rust code

**Impact**:
- Any code parsing triggers metadata extraction
- Metadata extraction tries multiple language patterns
- First incompatible pattern causes panic
- Blocks all end-to-end parsing tests

**Fix Required**:
1. Update `Pattern::new()` to return `Result` or use `try_new()`
2. Update `extract_functions()` to handle pattern parse errors gracefully
3. Try patterns only for the detected language, or catch errors per pattern

**Workaround**: Tests are marked with `#[ignore]` until bug is fixed

### Example Error
```
thread panicked at crates/ast-engine/src/matchers/pattern.rs:220:34:
called `Result::unwrap()` on an `Err` value: MultipleNode("function µNAME(µµµPARAMS) { µµµBODY }")
```

## Running Tests

### Run All Non-Ignored Tests
```bash
cargo test -p thread-flow --test integration_tests
```

### Run Specific Test
```bash
cargo test -p thread-flow --test integration_tests test_factory_build_succeeds
```

### Run Ignored Tests (will fail until bug fixed)
```bash
cargo test -p thread-flow --test integration_tests -- --ignored
```

### Run All Tests Including Ignored
```bash
cargo test -p thread-flow --test integration_tests -- --include-ignored
```

## Test Expectations

### When Bug is Fixed

Once the pattern matching bug is resolved:

1. **Remove `#[ignore]` attributes** from blocked tests
2. **Verify all tests pass**:
   ```bash
   cargo test -p thread-flow --test integration_tests
   ```
3. **Validate multi-language support**:
   - Rust: Extract structs, enums, functions, imports
   - Python: Extract classes, functions, imports
   - TypeScript: Extract interfaces, classes, enums
   - Go: Extract structs, interfaces, functions

4. **Performance targets**:
   - Minimal parsing: <100ms
   - Large file (100 lines): <1s
   - Caching enabled and working

## Test Coverage

### Current Coverage
- ✅ ReCoco integration (factory, executor, schema)
- ✅ Error handling (invalid inputs, unsupported languages)
- ⏸️ Value serialization (structure, types)
- ⏸️ Multi-language parsing (Rust, Python, TypeScript, Go)
- ⏸️ Symbol extraction (functions, imports, calls)
- ⏸️ Performance benchmarks

### Future Coverage
- [ ] Incremental parsing with caching
- [ ] Complex language constructs (generics, macros)
- [ ] Cross-language symbol resolution
- [ ] Large codebase performance (1000+ files)
- [ ] Edge cases (Unicode, unusual syntax)

## Contributing

### Adding New Tests

1. **Create test data**: Add files to `tests/test_data/`
2. **Write test**: Add to appropriate section in `integration_tests.rs`
3. **Document**: Update this README with test description
4. **Run**: Verify test passes or properly ignored

### Test Guidelines

- **Realistic test data**: Use actual code patterns, not minimal examples
- **Clear assertions**: Validate specific expected behaviors
- **Proper cleanup**: No temp files or state leakage
- **Performance aware**: Use `#[ignore]` for benchmarks
- **Document blockers**: Clear `#[ignore]` reasons

## See Also

- [Thread Flow Integration Guide](../RECOCO_INTEGRATION.md)
- [Thread Constitution](../../.specify/memory/constitution.md)
- [ReCoco Documentation](https://github.com/knitli/recoco)
