<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# CocoIndex Rust API: Types

The type system is defined in `cocoindex::base`.

## Values

`cocoindex::base::value::Value` is the core runtime type.

```rust
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Value<VS = ScopeValue> {
    #[default]
    Null,
    Basic(BasicValue),
    Struct(FieldValues<VS>),
    UTable(Vec<VS>),
    KTable(BTreeMap<KeyValue, VS>),
    LTable(Vec<VS>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum BasicValue {
    Bytes(Bytes),
    Str(Arc<str>),
    Bool(bool),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Range(RangeValue),
    Uuid(uuid::Uuid),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
    LocalDateTime(chrono::NaiveDateTime),
    OffsetDateTime(chrono::DateTime<chrono::FixedOffset>),
    TimeDelta(chrono::Duration),
    Json(Arc<serde_json::Value>),
    Vector(Arc<[BasicValue]>),
    // ...
}
```

### Creating Values

```rust
use cocoindex::base::value::Value;

let s = Value::from("hello world"); // Creates Basic(Str(...))
let i = Value::from(42);            // Creates Basic(Int64(42))
let f = Value::from(3.14);          // Creates Basic(Float64(3.14))
```

## Keys

Keys in CocoIndex are composite.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyValue(pub Box<[KeyPart]>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyPart {
    Bytes(Bytes),
    Str(Arc<str>),
    Bool(bool),
    Int64(i64),
    Range(RangeValue),
    Uuid(uuid::Uuid),
    Date(chrono::NaiveDate),
    Struct(Vec<KeyPart>),
}
```

## Schemas

Schemas define the type of values.

```rust
use cocoindex::base::schema::{ValueType, BasicValueType, StructSchema, TableSchema};

// Example: Definition of a struct type
let struct_schema = StructSchema {
    fields: Arc::new(vec![
        // FieldSchema...
    ]),
    description: None,
};
```
