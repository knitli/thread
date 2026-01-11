# CocoIndex Rust API: Functions

Implement stateless transformations using `cocoindex::ops::interface`.

## Key Traits

### `SimpleFunctionFactory`

```rust
#[async_trait]
pub trait SimpleFunctionFactory {
    async fn build(
        self: Arc<Self>,
        spec: serde_json::Value,
        input_schema: Vec<OpArgSchema>,
        context: Arc<FlowInstanceContext>,
    ) -> Result<SimpleFunctionBuildOutput>;
}

pub struct SimpleFunctionBuildOutput {
    pub output_type: EnrichedValueType,
    pub behavior_version: Option<u32>, // For cache invalidation
    pub executor: BoxFuture<'static, Result<Box<dyn SimpleFunctionExecutor>>>,
}
```

### `SimpleFunctionExecutor`

```rust
#[async_trait]
pub trait SimpleFunctionExecutor: Send + Sync {
    /// Evaluate the function on input arguments.
    async fn evaluate(&self, args: Vec<Value>) -> Result<Value>;

    fn enable_cache(&self) -> bool { false }
    fn timeout(&self) -> Option<std::time::Duration> { None }
}
```

## Example Implementation

A function that concatenates two strings.

```rust
use cocoindex::ops::interface::*;
use cocoindex::base::value::{Value, BasicValue};
use cocoindex::base::schema::{EnrichedValueType, ValueType, BasicValueType};
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::{Result, anyhow};

pub struct ConcatFactory;

#[async_trait]
impl SimpleFunctionFactory for ConcatFactory {
    async fn build(
        self: Arc<Self>,
        _spec: serde_json::Value,
        input_schema: Vec<OpArgSchema>,
        _ctx: Arc<FlowInstanceContext>,
    ) -> Result<SimpleFunctionBuildOutput> {
         // Validate input schema if needed

        Ok(SimpleFunctionBuildOutput {
            output_type: EnrichedValueType::simple(ValueType::Basic(BasicValueType::Str)),
            behavior_version: None,
            executor: Box::pin(async move {
                Ok(Box::new(ConcatExecutor) as Box<dyn SimpleFunctionExecutor>)
            }),
        })
    }
}

struct ConcatExecutor;

#[async_trait]
impl SimpleFunctionExecutor for ConcatExecutor {
    async fn evaluate(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(anyhow!("Expected 2 arguments"));
        }

        let s1 = args[0].as_str().map_err(|_| anyhow!("Arg 1 not string"))?;
        let s2 = args[1].as_str().map_err(|_| anyhow!("Arg 2 not string"))?;

        // Coerce Arc<str> to String for format!
        Ok(Value::from(format!("{}{}", s1, s2)))
    }
}
```
