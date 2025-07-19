# Rule Engine Benchmarks

This directory contains performance benchmarks for the `thread-rule-engine` crate.

## Working Benchmarks

### 1. `simple_benchmarks.rs`

- **Status**: ✅ Working
- **Purpose**: Core thread-rule-engine functionality benchmarks
- **Benchmarks**:
  - `bench_rule_parsing`: Simple and complex rule parsing from YAML
  - `bench_rule_compilation`: Multiple rule compilation performance
  - `bench_rule_transformation`: Rule transformation parsing
  - `bench_yaml_deserialization`: Large YAML rule deserialization

### 2. `ast_grep_comparison.rs`

- **Status**: ✅ Working
- **Purpose**: Direct performance comparison between thread-rule-engine and ast-grep-config
- **Benchmarks**:
  - `bench_rule_parsing_comparison`: Side-by-side parsing performance
  - `bench_rule_matching_comparison`: Pattern matching speed comparison
  - `bench_combined_scan_comparison`: Multi-rule scanning performance
  - `bench_memory_usage_comparison`: Memory allocation patterns

## Test Data

The benchmarks use realistic code samples:

- `test_data/sample_typescript.ts` - TypeScript code with classes, functions, async/await
- `test_data/sample_javascript.js` - JavaScript with ES6+ features
- `test_data/sample_python.py` - Python with async functions and decorators
- `test_data/sample_rust.rs` - Rust with structs, traits, and macros

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench simple_benchmarks
cargo bench --bench ast_grep_comparison

# Generate HTML reports
cargo bench
open target/criterion/report/index.html
```

## Key Features Benchmarked

1. **Rule Parsing**: YAML to internal rule representation
2. **Pattern Compilation**: Converting patterns to matchers
3. **Memory Usage**: Allocation patterns during rule creation
4. **Transformation Processing**: Rule transformation parsing
5. **Deserialization**: YAML parsing performance

## Performance Expectations

- **Simple rules**: Should parse in < 100μs
- **Complex rules**: Should parse in < 1ms
- **Memory overhead**: Should be comparable to ast-grep-config
- **Compilation**: Should scale linearly with rule complexity

## Architecture

The benchmarks are designed to:

- Measure only the thread-rule-engine operations (not AST matching)
- Use realistic rule patterns from actual use cases
- Focus on serialization/deserialization performance
- Compare directly with ast-grep-config where possible

## Limitations

- Some benchmarks focus only on parsing/compilation, not pattern matching
- AST matching benchmarks require complex setup and may not be representative
- Memory benchmarks measure allocation patterns, not peak usage
