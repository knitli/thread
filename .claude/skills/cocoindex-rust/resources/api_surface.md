# CocoIndex Rust API Surface

Based on analysis of the `cocoindex` crate (v0.1.0+).

## Core API Modules

For detailed usage, see the corresponding resource files.

### 1. Setup & Context (`cocoindex::lib_context`, `cocoindex::settings`)
See [api_setup.md](api_setup.md)

- **`LibContext`**: The main runtime object.
- **`create_lib_context`**: Async initializer.
- **`Settings`**: Configuration struct (DB, namespace, etc).

### 2. Operations Interface (`cocoindex::ops::interface`)

Traits for extending the engine.

- **Sources** (See [api_source.md](api_source.md)):
    - `SourceFactory`
    - `SourceExecutor`
    - `PartialSourceRow`, `SourceValue`, `Ordinal`
- **Functions** (See [api_function.md](api_function.md)):
    - `SimpleFunctionFactory`
    - `SimpleFunctionExecutor`
- **Targets**:
    - `TargetFactory`
    - `ExportTargetMutation`

### 3. Type System (`cocoindex::base`)
See [api_types.md](api_types.md)

- **`value::Value`**: The universal value enum.
- **`value::KeyValue`**: The composite key type.
- **`schema::ValueType`**: Schema definitions.

### 4. Executing Flows (`cocoindex::execution`)

- **`FlowExecutionContext`**: Manage execution state.
- **`FlowContext`**: Access to analyzed flow and execution.

## Crate Structure

```
cocoindex
├── base         # Types (Value, Schema, Key)
├── builder      # Flow analysis & planning
├── execution    # Execution engine
├── lib_context  # Runtime context
├── llm          # LLM integrations
├── ops          # Operator registry & traits
│   └── interface # Core traits
├── prelude      # Common imports
├── service      # Service layer (Axum)
├── settings     # Configuration
└── setup        # State management
```
