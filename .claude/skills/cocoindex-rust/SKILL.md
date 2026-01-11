---
name: cocoindex-rust
description: Comprehensive toolkit for developing with the CocoIndex Rust API. Use when building high-performance operators, embedding the engine in Rust applications, or extending the core framework. Covers LibContext management, custom native operators, and direct execution control.
---

# CocoIndex Rust API

## Overview

CocoIndex's Rust API provides low-level, high-performance access to the core incremental dataflow engine. Unlike the Python API which focuses on flow definition and orchestration, the Rust API allows for:

1.  **Embedding the Engine** - Run CocoIndex within Rust services.
2.  **Native Operators** - Implement zero-overhead sources, functions, and targets.
3.  **Deep Integration** - Direct access to `LibContext`, database pools, and execution plans.

## Resources

Detailed documentation for each part of the system:

- **[API Surface](resources/api_surface.md)**: Overview of modules and crate structure.
- **[Setup & Context](resources/api_setup.md)**: How to initialize `LibContext`, settings, and access flows.
- **[Sources](resources/api_source.md)**: Implementing `SourceFactory` and `SourceExecutor` for custom data ingestion.
- **[Functions](resources/api_function.md)**: Implementing `SimpleFunctionFactory` for high-performance transformations.
- **[Types](resources/api_types.md)**: Understanding `Value`, `KeyPart`, and schema systems.

## When to Use This Skill

Use when users request:

- "Integrate CocoIndex into a Rust application"
- "Implement a high-performance custom source in Rust"
- "How do I write a native Rust function for CocoIndex?"
- "Access internal flow state or database pools from Rust"
- "What is `LibContext`?"
- "Explain the `Value` enum in CocoIndex Rust"

## Common Usage Patterns

### 1. Initialization

See [resources/api_setup.md](resources/api_setup.md) for full details.

```rust
use cocoindex::settings::Settings;
use cocoindex::lib_context::create_lib_context;

let settings = Settings::default();
let lib_ctx = create_lib_context(settings).await?;
```

### 2. Implementing a Native Source

See [resources/api_source.md](resources/api_source.md).

Implement `SourceFactory` and `SourceExecutor` to bridge external data systems (Kafka, Postgres, APIs) into CocoIndex efficiently.

### 3. Implementing a Native Function

See [resources/api_function.md](resources/api_function.md).

Implement `SimpleFunctionFactory` for computationally intensive tasks where Python overhead is undesirable.
