<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# CocoIndex Rust API: Sources

Implement high-performance data sources using `cocoindex::ops::interface`.

## Key Traits

### `SourceFactory`

Responsible for parsing the flow specification and creating an executor.

```rust
#[async_trait]
pub trait SourceFactory {
    async fn build(
        self: Arc<Self>,
        source_name: &str,
        spec: serde_json::Value,
        context: Arc<FlowInstanceContext>,
    ) -> Result<(
        EnrichedValueType, // The output schema of this source
        BoxFuture<'static, Result<Box<dyn SourceExecutor>>>,
    )>;
}
```

### `SourceExecutor`

The runtime component that reads data.

```rust
#[async_trait]
pub trait SourceExecutor: Send + Sync {
    /// List available keys.
    async fn list(
        &self,
        options: &SourceExecutorReadOptions,
    ) -> Result<BoxStream<'async_trait, Result<Vec<PartialSourceRow>>>>;

    /// Get value for a specific key.
    async fn get_value(
        &self,
        key: &KeyValue,
        key_aux_info: &serde_json::Value,
        options: &SourceExecutorReadOptions,
    ) -> Result<PartialSourceRowData>;

    /// Optional: Provide a stream of changes for incremental processing.
    async fn change_stream(
        &self,
    ) -> Result<Option<BoxStream<'async_trait, Result<SourceChangeMessage>>>> {
        Ok(None)
    }

    fn provides_ordinal(&self) -> bool;
}
```

## Data Types

### `PartialSourceRow`

Represents a single row from the source.

```rust
pub struct PartialSourceRow {
    pub key: KeyValue,
    pub key_aux_info: serde_json::Value, // e.g. version info
    pub data: PartialSourceRowData,
}

pub struct PartialSourceRowData {
    pub ordinal: Option<Ordinal>, // Timing info
    pub content_version_fp: Option<Vec<u8>>, // Fingerprint
    pub value: Option<SourceValue>, // The actual data
}
```

### `SourceValue`

```rust
pub enum SourceValue {
    Existence(FieldValues),
    NonExistence,
}
```

## Example Implementation

```rust
use cocoindex::ops::interface::*;
use cocoindex::base::value::{Value, KeyPart, KeyValue, FieldValues};
use cocoindex::base::schema::{EnrichedValueType, ValueType, BasicValueType};
use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use std::sync::Arc;
use anyhow::Result;

pub struct MySourceFactory;

#[async_trait]
impl SourceFactory for MySourceFactory {
    async fn build(
        self: Arc<Self>,
        name: &str,
        source_spec: serde_json::Value,
        _ctx: Arc<FlowInstanceContext>,
    ) -> Result<(EnrichedValueType, BoxFuture<'static, Result<Box<dyn SourceExecutor>>>)> {
        // Parse options from source_spec...

        let output_type = EnrichedValueType::simple(ValueType::Basic(BasicValueType::Str));

        // Return creating the executor
        Ok((output_type, Box::pin(async move {
            Ok(Box::new(MySourceExecutor) as Box<dyn SourceExecutor>)
        })))
    }
}

struct MySourceExecutor;

#[async_trait]
impl SourceExecutor for MySourceExecutor {
    async fn list(
        &self,
        _options: &SourceExecutorReadOptions,
    ) -> Result<BoxStream<'_, Result<Vec<PartialSourceRow>>>> {
        let stream = async_stream::try_stream! {
            // Yield a batch of rows
            let rows = vec![
                PartialSourceRow {
                    key: KeyValue::from_single_part(KeyPart::Int64(1)),
                    key_aux_info: serde_json::Value::Null,
                    data: PartialSourceRowData {
                        ordinal: Some(Ordinal::try_from(std::time::SystemTime::now())?),
                        content_version_fp: None,
                        // Data: { "value": "hello" } assuming schema is just a string?
                        // Actually SourceValue expects FieldValues (a Struct of values).
                        // If schema is simple String, it's wrapped in a struct internally?
                        // Usually source returns a struct of fields.
                        value: Some(SourceValue::Existence(FieldValues {
                            fields: vec![Value::from("hello")]
                        })),
                    }
                }
            ];
            yield rows;
        };
        Ok(Box::pin(stream))
    }

    async fn get_value(
        &self,
        key: &KeyValue,
        _info: &serde_json::Value,
        _opts: &SourceExecutorReadOptions,
    ) -> Result<PartialSourceRowData> {
        // Implement single fetch...
        Ok(PartialSourceRowData::default()) // Placeholder
    }

    fn provides_ordinal(&self) -> bool { true }
}
```
