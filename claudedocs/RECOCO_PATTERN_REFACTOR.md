# ReCoco Pattern Refactoring - January 27, 2026

## Summary

Refactored all Thread transform functions to use the official ReCoco `SimpleFunctionFactoryBase` pattern instead of the low-level `SimpleFunctionFactory` trait. This aligns with ReCoco's idiomatic operator implementation and enables proper registration with `ExecutorFactoryRegistry`.

## Changes Made

### Transform Functions (4 files)

All transform function files were updated to follow the correct pattern:

**Files Modified**:
- `crates/flow/src/functions/parse.rs` (ThreadParseFactory)
- `crates/flow/src/functions/symbols.rs` (ExtractSymbolsFactory)
- `crates/flow/src/functions/imports.rs` (ExtractImportsFactory)
- `crates/flow/src/functions/calls.rs` (ExtractCallsFactory)

**Pattern Changes**:

#### Before (Incorrect - Direct SimpleFunctionFactory)
```rust
#[async_trait]
impl SimpleFunctionFactory for ThreadParseFactory {
    async fn build(
        self: Arc<Self>,
        _spec: serde_json::Value,
        _args: Vec<recoco::base::schema::OpArgSchema>,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<SimpleFunctionBuildOutput, recoco::prelude::Error> {
        Ok(SimpleFunctionBuildOutput {
            executor: Box::pin(async {
                Ok(Box::new(ThreadParseExecutor) as Box<dyn SimpleFunctionExecutor>)
            }),
            output_type: get_output_schema(),
            behavior_version: Some(1),
        })
    }
}
```

#### After (Correct - SimpleFunctionFactoryBase)
```rust
/// Spec for thread_parse operator
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadParseSpec {}

#[async_trait]
impl SimpleFunctionFactoryBase for ThreadParseFactory {
    type Spec = ThreadParseSpec;
    type ResolvedArgs = ();

    fn name(&self) -> &str {
        "thread_parse"
    }

    async fn analyze<'a>(
        &'a self,
        _spec: &'a Self::Spec,
        _args_resolver: &mut OpArgsResolver<'a>,
        _context: &FlowInstanceContext,
    ) -> Result<SimpleFunctionAnalysisOutput<Self::ResolvedArgs>, recoco::prelude::Error> {
        Ok(SimpleFunctionAnalysisOutput {
            resolved_args: (),
            output_schema: get_output_schema(),
            behavior_version: Some(1),
        })
    }

    async fn build_executor(
        self: Arc<Self>,
        _spec: Self::Spec,
        _resolved_args: Self::ResolvedArgs,
        _context: Arc<FlowInstanceContext>,
    ) -> Result<impl SimpleFunctionExecutor, recoco::prelude::Error> {
        Ok(ThreadParseExecutor)
    }
}
```

**Key Differences**:
1. **Trait**: `SimpleFunctionFactoryBase` instead of `SimpleFunctionFactory`
2. **Associated Types**: Added `type Spec` and `type ResolvedArgs`
3. **Name Method**: Added `fn name(&self) -> &str` returning operator name
4. **Two-Phase Pattern**:
   - `analyze()` validates inputs and returns output schema
   - `build_executor()` creates the executor instance
5. **Automatic Registration**: Base trait provides `.register()` method via blanket impl
6. **Correct Imports**: Use `recoco::ops::sdk::{OpArgsResolver, SimpleFunctionAnalysisOutput}`

### Registry Module

**File Modified**: `crates/flow/src/registry.rs`

**Changes**:
1. Added proper imports:
   ```rust
   use recoco::ops::factory_bases::SimpleFunctionFactoryBase;
   use recoco::ops::sdk::ExecutorFactoryRegistry;
   ```

2. Implemented `register_all()` function:
   ```rust
   pub fn register_all(registry: &mut ExecutorFactoryRegistry) -> Result<(), RecocoError> {
       ThreadParseFactory.register(registry)?;
       ExtractSymbolsFactory.register(registry)?;
       ExtractImportsFactory.register(registry)?;
       ExtractCallsFactory.register(registry)?;
       Ok(())
   }
   ```

3. Added test to verify registration succeeds:
   ```rust
   #[test]
   fn test_register_all() {
       let mut registry = ExecutorFactoryRegistry::new();
       ThreadOperators::register_all(&mut registry).expect("registration should succeed");
   }
   ```

## Import Corrections

Fixed several incorrect import paths discovered during refactoring:

| Incorrect Import | Correct Import |
|------------------|----------------|
| `recoco::builder::analyzer::OpArgsResolver` | `recoco::ops::sdk::OpArgsResolver` |
| `recoco::ops::interface::SimpleFunctionAnalysisOutput` | `recoco::ops::sdk::SimpleFunctionAnalysisOutput` |
| `recoco::ops::registration::ExecutorFactoryRegistry` | `recoco::ops::sdk::ExecutorFactoryRegistry` |

## Field Name Corrections

| Incorrect Field | Correct Field |
|----------------|---------------|
| `output_type` | `output_schema` |

## Benefits of Refactoring

1. **Idiomatic ReCoco**: Follows official pattern used by ReCoco built-in operators
2. **Proper Registration**: Enables explicit operator registration with `ExecutorFactoryRegistry`
3. **Type Safety**: Associated types (`Spec`, `ResolvedArgs`) provide stronger type checking
4. **Two-Phase Analysis**: Separates schema validation (`analyze`) from executor creation (`build_executor`)
5. **Future Extensibility**: Easier to add operator-specific configuration via `Spec` types

## Build & Test Results

✅ **Build**: `cargo build -p thread-flow` - Success
✅ **Tests**: `cargo test -p thread-flow --lib` - 3/3 passed

## Usage Example

```rust
use recoco::ops::sdk::ExecutorFactoryRegistry;
use thread_flow::ThreadOperators;

// Create registry
let mut registry = ExecutorFactoryRegistry::new();

// Register all Thread operators
ThreadOperators::register_all(&mut registry)?;

// Operators are now available for use in ReCoco flows
// - thread_parse
// - extract_symbols
// - extract_imports
// - extract_calls
```

## Next Steps

This refactoring completes Week 2 ReCoco integration tasks with proper operator implementation patterns. The codebase now:

1. Uses official ReCoco patterns throughout
2. Supports explicit operator registration
3. Maintains all functionality from Week 2 deliverables
4. Provides foundation for Week 3 edge deployment

## References

- ReCoco source: `~/.cargo/registry/src/.../recoco-core-0.2.1/src/ops/factory_bases.rs`
- Trait definition: `SimpleFunctionFactoryBase` with blanket impl for `SimpleFunctionFactory`
- Registration pattern: `factory.register(registry)?` using provided `.register()` method
