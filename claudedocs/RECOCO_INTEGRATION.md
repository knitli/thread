# ReCoco Integration for Thread

This document describes the ReCoco transform functions implemented for Thread's semantic extraction capabilities.

## Overview

The Thread-ReCoco integration provides dataflow-based code analysis through transform functions that extract semantic information from source code. These functions follow the ReCoco SimpleFunctionFactory/SimpleFunctionExecutor pattern.

## Implemented Transform Functions

### 1. ThreadParse (parse.rs)
**Factory**: `ThreadParseFactory`
**Executor**: `ThreadParseExecutor`

**Input**:
- `content` (String): Source code content
- `language` (String): Language identifier or file extension
- `file_path` (String, optional): Path for context

**Output**: Struct containing three tables:
- `symbols`: LTable of symbol definitions
- `imports`: LTable of import statements
- `calls`: LTable of function calls

**Features**:
- Content-addressable caching enabled
- 30-second timeout
- Automatic language detection from extensions
- Hash-based content identification

### 2. ExtractSymbols (symbols.rs)
**Factory**: `ExtractSymbolsFactory`
**Executor**: `ExtractSymbolsExecutor`

**Input**:
- `parsed_document` (Struct): Output from ThreadParse

**Output**: LTable with schema:
- `name` (String): Symbol name
- `kind` (String): Symbol type (Function, Class, Variable, etc.)
- `scope` (String): Lexical scope path

**Features**:
- Extracts first field from parsed document
- Caching enabled
- 30-second timeout

### 3. ExtractImports (imports.rs)
**Factory**: `ExtractImportsFactory`
**Executor**: `ExtractImportsExecutor`

**Input**:
- `parsed_document` (Struct): Output from ThreadParse

**Output**: LTable with schema:
- `symbol_name` (String): Imported symbol name
- `source_path` (String): Import source module/file
- `kind` (String): Import type (Named, Default, Namespace, etc.)

**Features**:
- Extracts second field from parsed document
- Caching enabled
- 30-second timeout

### 4. ExtractCalls (calls.rs)
**Factory**: `ExtractCallsFactory`
**Executor**: `ExtractCallsExecutor`

**Input**:
- `parsed_document` (Struct): Output from ThreadParse

**Output**: LTable with schema:
- `function_name` (String): Called function name
- `arguments_count` (Int64): Number of arguments

**Features**:
- Extracts third field from parsed document
- Caching enabled
- 30-second timeout

## Schema Definitions

All schema types are defined in `conversion.rs`:

```rust
pub fn symbol_type() -> ValueType { /* ... */ }
pub fn import_type() -> ValueType { /* ... */ }
pub fn call_type() -> ValueType { /* ... */ }
```

These schemas use ReCoco's type system (`ValueType`, `StructSchema`, `FieldSchema`) to define the structure of extracted data.

## Module Organization

```
crates/flow/src/
├── functions/
│   ├── mod.rs          # Exports all factories
│   ├── parse.rs        # ThreadParseFactory
│   ├── symbols.rs      # ExtractSymbolsFactory
│   ├── imports.rs      # ExtractImportsFactory
│   └── calls.rs        # ExtractCallsFactory
├── conversion.rs       # Schema definitions and serialization
├── bridge.rs           # CocoIndexAnalyzer integration
└── lib.rs              # Main library entry
```

## Usage Example

```rust
use thread_flow::functions::{
    ThreadParseFactory,
    ExtractSymbolsFactory,
    ExtractImportsFactory,
    ExtractCallsFactory,
};

// Create flow pipeline
let parse_op = ThreadParseFactory;
let symbols_op = ExtractSymbolsFactory;
let imports_op = ExtractImportsFactory;
let calls_op = ExtractCallsFactory;

// Build executors
let parse_executor = parse_op.build(/* ... */).await?;
let symbols_executor = symbols_op.build(/* ... */).await?;

// Execute pipeline
let parsed_doc = parse_executor.evaluate(vec![
    Value::Str("fn main() {}".into()),
    Value::Str("rs".into()),
    Value::Str("main.rs".into()),
]).await?;

let symbols_table = symbols_executor.evaluate(vec![parsed_doc]).await?;
```

## Integration with CocoIndex

These transform functions integrate with CocoIndex's dataflow framework to provide:

1. **Content-Addressed Caching**: Parse results are cached by content hash
2. **Incremental Updates**: Only re-analyze changed files
3. **Dependency Tracking**: Track symbol usage across files
4. **Storage Backend**: Results can be persisted to Postgres, D1, or Qdrant

## Performance Characteristics

- **Parse**: O(n) where n = source code length
- **Extract**: O(1) field access from parsed struct
- **Caching**: Near-instant for cache hits
- **Timeout**: 30 seconds per operation (configurable)

## Error Handling

All functions use ReCoco's error system:
- `Error::client()`: Invalid input or unsupported language
- `Error::internal_msg()`: Internal processing errors

## Future Extensions

Potential additions:
- Type information extraction
- Control flow graph generation
- Complexity metrics calculation
- Documentation extraction
- Cross-reference resolution
