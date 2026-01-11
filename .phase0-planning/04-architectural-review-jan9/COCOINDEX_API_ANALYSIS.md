# CocoIndex Rust API Surface Analysis

**Analysis Date**: 2024
**Repository**: cocoindex-io/cocoindex
**Focus**: Rust-to-Rust API perspective (not Python bindings)

## Executive Summary

This document analyzes the Rust API surface of CocoIndex and compares it with what's exposed to Python through PyO3 bindings. The analysis reveals that **the Python API is a carefully curated subset of the Rust API**, with significant Rust-only functionality remaining internal to the library.

### Key Findings

1. **Python API Coverage**: ~30-40% of core Rust functionality is exposed to Python
2. **Rust-Only APIs**: Service layer (HTTP), internal execution contexts, setup/migration system internals
3. **Architecture**: Python acts as a high-level orchestration layer; Rust handles all performance-critical operations
4. **Extension Points**: Custom operators (sources, functions, targets) bridge Python and Rust

---

## 1. Python API Surface (PyO3 Bindings)

### 1.1 Core Module: `cocoindex_engine`

**Location**: `rust/cocoindex/src/py/mod.rs`

The Python module `_engine` (exported as `cocoindex._engine`) exposes:

#### Functions (~17 functions)
```rust
// Lifecycle management
init_pyo3_runtime()
init(settings: Option<Settings>)
set_settings_fn(get_settings_fn: Callable)
stop()

// Server management
start_server(settings: ServerSettings)

// Operation registration
register_source_connector(name: String, py_source_connector)
register_function_factory(name: String, py_function_factory)
register_target_connector(name: String, py_target_connector)

// Setup management
flow_names_with_setup_async() -> List[str]
make_setup_bundle(flow_names: List[str]) -> SetupChangeBundle
make_drop_bundle(flow_names: List[str]) -> SetupChangeBundle

// Flow context management
remove_flow_context(flow_name: str)

// Auth registry
add_auth_entry(key: str, value: JsonValue)
add_transient_auth_entry(value: JsonValue) -> str
get_auth_entry(key: str) -> JsonValue

// Utilities
get_app_namespace() -> str
serde_roundtrip(value, typ) -> Any  # Test utility
```

#### Classes (~11 classes)
```python
# Flow building
FlowBuilder
    - add_source(kind, spec, target_scope, name, refresh_options, execution_options) -> DataSlice
    - transform(kind, spec, args, target_scope, name) -> DataSlice
    - collect(collector, fields, auto_uuid_field)
    - export(name, kind, spec, attachments, index_options, input, setup_by_user)
    - declare(op_spec)
    - for_each(data_slice, execution_options) -> OpScopeRef
    - add_direct_input(name, value_type) -> DataSlice
    - set_direct_output(data_slice)
    - constant(value_type, value) -> DataSlice
    - scope_field(scope, field_name) -> Option[DataSlice]
    - build_flow() -> Flow
    - build_transient_flow_async(event_loop, ...) -> TransientFlow

DataSlice
    - field(field_name: str) -> Option[DataSlice]
    - data_type() -> DataType

DataCollector
    - (Used for collecting data into tables)

OpScopeRef
    - add_collector(name: str) -> DataCollector

# Flow execution
Flow
    - name() -> str
    - evaluate_and_dump(options: EvaluateAndDumpOptions)
    - get_spec(output_mode) -> RenderedSpec
    - get_schema() -> List[Tuple[str, str, str]]
    - make_setup_action() -> SetupChangeBundle
    - make_drop_action() -> SetupChangeBundle
    - add_query_handler(...)

FlowLiveUpdater
    - (Live flow updating)

TransientFlow
    - (In-memory transformation flows)

# Setup and metadata
IndexUpdateInfo
    - (Statistics from indexing operations)

SetupChangeBundle
    - describe_changes() -> List[str]
    - apply_change()
    - describe_and_apply()

# Helper types
PyOpArgSchema
    - value_type: ValueType
    - analyzed_value: Any

RenderedSpec
    - lines: List[RenderedSpecLine]

RenderedSpecLine
    - (Specification rendering)
```

### 1.2 Python Package Exports

**Location**: `python/cocoindex/__init__.py`

The Python package re-exports and wraps Rust types:

```python
# Main exports
__all__ = [
    # Engine (direct from Rust)
    "_engine",

    # Flow building (Python wrappers)
    "FlowBuilder",
    "DataScope",
    "DataSlice",
    "Flow",
    "transform_flow",
    "flow_def",

    # Lifecycle
    "init",
    "start_server",
    "stop",
    "settings",

    # Operations
    "functions",  # Module
    "sources",    # Module
    "targets",    # Module

    # Setup
    "setup_all_flows",
    "drop_all_flows",
    "update_all_flows_async",

    # Types (from Rust)
    "Int64", "Float32", "Float64",
    "LocalDateTime", "OffsetDateTime",
    "Range", "Vector", "Json",

    # ... and more
]
```

**Python Wrapping Pattern**:
- Python classes (`FlowBuilder`, `DataSlice`, `Flow`) wrap `_engine` types
- Add convenience methods and Pythonic interfaces
- Handle async/await translation (`asyncio` ↔ `tokio`)
- Type hints and better error messages

---

## 2. Rust-Only API Surface

### 2.1 Internal Modules (Not Exposed to Python)

#### `lib_context.rs` - Runtime Context Management

**Public Rust APIs**:
```rust
// Global runtime access
pub fn get_runtime() -> &'static Runtime  // Tokio runtime
pub fn get_auth_registry() -> &'static Arc<AuthRegistry>

// Context management (async)
pub(crate) async fn init_lib_context(settings: Option<Settings>) -> Result<()>
pub(crate) async fn get_lib_context() -> Result<Arc<LibContext>>
pub(crate) async fn clear_lib_context()
pub async fn create_lib_context(settings: Settings) -> Result<LibContext>

// Core types
pub struct LibContext {
    pub flows: Mutex<BTreeMap<String, Arc<FlowContext>>>,
    pub db_pools: DbPools,
    pub app_namespace: String,
    pub persistence_ctx: Option<PersistenceContext>,
    // ...
}

impl LibContext {
    pub fn get_flow_context(&self, flow_name: &str) -> Result<Arc<FlowContext>>
    pub fn remove_flow_context(&self, flow_name: &str)
    pub fn require_persistence_ctx(&self) -> Result<&PersistenceContext>
    pub fn require_builtin_db_pool(&self) -> Result<&PgPool>
}

pub struct FlowContext {
    pub flow: AnalyzedFlow,
    // ...
}

pub struct PersistenceContext {
    pub builtin_db_pool: PgPool,
    pub setup_ctx: RwLock<LibSetupContext>,
}
```

**Not exposed to Python**: All low-level context management, database pool management, flow registry internals.

---

#### `service/` - HTTP API Layer

**Location**: `rust/cocoindex/src/service/flows.rs`

**Public Rust APIs**:
```rust
// HTTP endpoints (Axum handlers)
pub async fn list_flows(State(lib_context): State<Arc<LibContext>>)
    -> Result<Json<Vec<String>>, ApiError>

pub async fn get_flow_schema(Path(flow_name): Path<String>, ...)
    -> Result<Json<FlowSchema>, ApiError>

pub async fn get_flow(Path(flow_name): Path<String>, ...)
    -> Result<Json<GetFlowResponse>, ApiError>

pub async fn get_keys(Path(flow_name): Path<String>, Query(query), ...)
    -> Result<Json<GetKeysResponse>, ApiError>

pub async fn evaluate_data(Path(flow_name): Path<String>, ...)
    -> Result<Json<EvaluateDataResponse>, ApiError>

pub async fn update(Path(flow_name): Path<String>, ...)
    -> Result<Json<IndexUpdateInfo>, ApiError>

// Response types
pub struct GetFlowResponse {
    flow_spec: spec::FlowInstanceSpec,
    data_schema: FlowSchema,
    query_handlers_spec: HashMap<String, Arc<QueryHandlerSpec>>,
}

pub struct GetKeysResponse { /* ... */ }
pub struct EvaluateDataResponse { /* ... */ }
```

**Not exposed to Python**: Entire REST API layer. Python uses `start_server()` but cannot call individual endpoints.

---

#### `ops/interface.rs` - Operation Trait System

**Public Rust APIs**:
```rust
// Factory traits
#[async_trait]
pub trait SourceFactory {
    async fn build(...) -> Result<SourceBuildOutput>;
    // ...
}

#[async_trait]
pub trait SimpleFunctionFactory {
    async fn build(...) -> Result<SimpleFunctionBuildOutput>;
}

#[async_trait]
pub trait TargetFactory: Send + Sync {
    async fn build(...) -> Result<(Vec<ExportDataCollectionBuildOutput>, Vec<...>)>;
    async fn diff_setup_states(...) -> Result<Box<dyn ResourceSetupChange>>;
    fn normalize_setup_key(&self, key: &serde_json::Value) -> Result<serde_json::Value>;
    fn check_state_compatibility(...) -> Result<SetupStateCompatibility>;
    fn describe_resource(&self, key: &serde_json::Value) -> Result<String>;
    fn extract_additional_key(...) -> Result<serde_json::Value>;
    async fn apply_mutation(...) -> Result<()>;
    async fn apply_setup_changes(...) -> Result<()>;
}

// Executor traits
#[async_trait]
pub trait SourceExecutor: Send + Sync {
    async fn read(&self, options: SourceExecutorReadOptions) -> Result<BoxStream<...>>;
    // ...
}

#[async_trait]
pub trait SimpleFunctionExecutor: Send + Sync {
    async fn evaluate(&self, input: Vec<value::Value>) -> Result<value::Value>;
    fn enable_cache(&self) -> bool;
    fn timeout(&self) -> Option<Duration>;
}

// Enum wrapping all factory types
pub enum ExecutorFactory {
    Source(Arc<dyn SourceFactory + Send + Sync>),
    SimpleFunction(Arc<dyn SimpleFunctionFactory + Send + Sync>),
    ExportTarget(Arc<dyn TargetFactory + Send + Sync>),
    TargetAttachment(Arc<dyn TargetAttachmentFactory + Send + Sync>),
}

// Setup state types
pub enum SetupStateCompatibility {
    Compatible,
    PartialCompatible,
    NotCompatible,
}

pub struct ExportTargetMutation {
    pub upserts: Vec<ExportTargetUpsertEntry>,
    pub deletes: Vec<ExportTargetDeleteEntry>,
}

pub struct ExportDataCollectionBuildOutput {
    pub export_context: BoxFuture<'static, Result<Arc<dyn Any + Send + Sync>>>,
    pub setup_key: serde_json::Value,
    pub desired_setup_state: serde_json::Value,
}
```

**Exposed to Python**: Only through `PySourceConnectorFactory`, `PyFunctionFactory`, `PyExportTargetFactory` wrappers. Native Rust ops implement these traits directly.

---

#### `setup/` - Setup and Migration System

**Location**: `rust/cocoindex/src/setup/`

**Public Rust APIs**:
```rust
// Driver functions
pub async fn get_existing_setup_state(pool: &PgPool) -> Result<AllSetupStates<ExistingMode>>

pub async fn apply_changes_for_flow_ctx(
    action: FlowSetupChangeAction,
    flow_ctx: &FlowContext,
    flow_exec_ctx: &mut FlowExecutionContext,
    lib_setup_ctx: &mut LibSetupContext,
    pool: &PgPool,
    output: &mut dyn Write,
) -> Result<()>

// State types
pub struct FlowSetupState<M: SetupMode> {
    pub flow_name: String,
    pub imports: IndexMap<String, ImportSetupState<M>>,
    pub targets: IndexMap<ResourceIdentifier, TargetSetupState<M>>,
    pub attachments: IndexMap<AttachmentSetupKey, AttachmentSetupState<M>>,
}

pub struct TargetSetupState {
    pub target_id: i32,
    pub schema_version_id: usize,
    pub max_schema_version_id: usize,
    pub setup_by_user: bool,
    pub key_type: Option<Box<[schema::ValueType]>>,
}

pub trait ResourceSetupChange {
    fn describe_changes(&self) -> Vec<ChangeDescription>;
    fn change_type(&self) -> SetupChangeType;
}

pub enum SetupChangeType {
    CreateResource,
    UpdateResource,
    DropResource,
}

// Combined state for diffing
pub struct CombinedState<S> {
    pub current: Option<S>,
    pub staging: Vec<StateChange<S>>,
    pub legacy_state_key: Option<serde_json::Value>,
}

pub enum StateChange<T> {
    Upsert(T),
    Delete,
}
```

**Not exposed to Python**: Internal setup state management, database metadata tracking, migration logic.

---

#### `builder/analyzer.rs` - Flow Analysis

**Public Rust APIs**:
```rust
pub async fn analyze_flow(
    flow_inst: &FlowInstanceSpec,
    flow_ctx: Arc<FlowInstanceContext>,
) -> Result<(FlowSchema, AnalyzedSetupState, impl Future<Output = Result<ExecutionPlan>>)>

pub async fn analyze_transient_flow<'a>(
    flow_inst: &TransientFlowSpec,
    flow_ctx: Arc<FlowInstanceContext>,
) -> Result<(EnrichedValueType, FlowSchema, impl Future<Output = Result<TransientExecutionPlan>>)>

pub fn build_flow_instance_context(
    flow_inst_name: &str,
    py_exec_ctx: Option<Arc<PythonExecutionContext>>,
) -> Arc<FlowInstanceContext>

// Internal builder types
pub(super) struct DataScopeBuilder { /* ... */ }
pub(super) struct CollectorBuilder { /* ... */ }
pub(super) struct OpScope {
    pub name: String,
    pub parent: Option<Arc<OpScope>>,
    pub data: Arc<Mutex<DataScopeBuilder>>,
    pub states: Arc<Mutex<OpScopeStates>>,
    pub base_value_def_fp: FieldDefFingerprint,
}
```

**Not exposed to Python**: All flow analysis internals. Python only sees the results through `Flow` object.

---

#### `execution/` - Execution Engine

**Location**: `rust/cocoindex/src/execution/`

**Public Rust APIs**:
```rust
// Submodules
pub(crate) mod dumper;
pub(crate) mod evaluator;
pub(crate) mod indexing_status;
pub(crate) mod row_indexer;
pub(crate) mod source_indexer;
pub(crate) mod stats;

// Functions (example from dumper)
pub async fn evaluate_and_dump(
    exec_plan: &ExecutionPlan,
    setup_execution_context: &FlowSetupExecutionContext,
    data_schema: &FlowSchema,
    options: EvaluateAndDumpOptions,
    pool: &PgPool,
) -> Result<()>

// Stats
pub struct IndexUpdateInfo {
    pub num_source_rows_added: usize,
    pub num_source_rows_updated: usize,
    pub num_source_rows_deleted: usize,
    pub num_export_rows_upserted: usize,
    pub num_export_rows_deleted: usize,
    // ...
}
```

**Exposed to Python**: Only `IndexUpdateInfo` and high-level `evaluate_and_dump()` via `Flow` methods.

---

#### `base/` - Core Type Definitions

**Location**: `rust/cocoindex/src/base/`

**Public Rust APIs**:
```rust
// Modules
pub mod schema;      // Field schemas, value types
pub mod spec;        // Operation specifications
pub mod value;       // Runtime values

// Examples from schema
pub struct FieldSchema {
    pub name: String,
    pub value_type: EnrichedValueType,
    pub description: Option<String>,
}

pub enum ValueType {
    Null,
    Bool,
    Int32, Int64,
    Float32, Float64,
    String,
    Bytes,
    LocalDateTime, OffsetDateTime,
    Duration, TimeDelta,
    Array(Box<ValueType>),
    Struct(StructType),
    Union(UnionType),
    Json,
    // ...
}

pub struct FlowSchema {
    pub schema: Vec<FieldSchema>,
    pub root_op_scope: OpScopeSchema,
}

// Examples from spec
pub struct FlowInstanceSpec {
    pub name: String,
    pub import_ops: Vec<NamedSpec<ImportOpSpec>>,
    pub reactive_ops: Vec<NamedSpec<ReactiveOpSpec>>,
    pub export_ops: Vec<NamedSpec<ExportOpSpec>>,
    pub declarations: Vec<OpSpec>,
}

pub struct ImportOpSpec {
    pub source: OpSpec,
    pub refresh_options: SourceRefreshOptions,
    pub execution_options: ExecutionOptions,
}

pub enum ReactiveOpSpec {
    Transform(TransformOpSpec),
    Collect(CollectOpSpec),
    ForEach(ForEachOpSpec),
}

pub struct ExportOpSpec {
    pub target: OpSpec,
    pub attachments: Vec<OpSpec>,
    pub index_options: IndexOptions,
    pub input: CollectorReference,
    pub setup_by_user: bool,
}
```

**Exposed to Python**: Type schemas are serialized/deserialized through PyO3. Most internal representation details hidden.

---

### 2.2 Built-in Operator Implementations

#### Sources
**Location**: `rust/cocoindex/src/ops/sources/`

```rust
pub mod amazon_s3;
pub mod azure_blob;
pub mod google_drive;
pub mod local_file;
pub mod postgres;
```

Each implements `SourceFactory` trait. Not individually exposed to Python - registered internally.

#### Functions
**Location**: `rust/cocoindex/src/ops/functions/`

```rust
// Example: parse_json.rs
pub struct Factory;

#[async_trait]
impl SimpleFunctionFactoryBase for Factory {
    async fn build(...) -> Result<...> { /* ... */ }
}
```

#### Targets
**Location**: `rust/cocoindex/src/ops/targets/`

```rust
pub mod kuzu;        // Kuzu graph database
pub mod neo4j;       // Neo4j graph database
pub mod postgres;    // PostgreSQL
pub mod qdrant;      // Qdrant vector database
```

Each implements `TargetFactory` trait.

---

### 2.3 Settings and Configuration

**Location**: `rust/cocoindex/src/settings.rs`

**Public Rust APIs**:
```rust
#[derive(Deserialize, Debug)]
pub struct DatabaseConnectionSpec {
    // Database connection details
}

#[derive(Deserialize, Debug, Default)]
pub struct GlobalExecutionOptions {
    // Global execution settings
}

#[derive(Deserialize, Debug, Default)]
pub struct Settings {
    // Main settings struct
}
```

**Exposed to Python**: Via `init(settings)` and `set_settings_fn()`. Python wraps these in `cocoindex.Settings`.

---

### 2.4 Server

**Location**: `rust/cocoindex/src/server.rs`

**Public Rust APIs**:
```rust
pub struct ServerSettings {
    pub address: String,
    pub cors_origins: Vec<String>,
    // ...
}

pub async fn init_server(
    lib_context: Arc<LibContext>,
    settings: ServerSettings,
) -> Result</* server handle */>
```

**Exposed to Python**: Only `start_server(ServerSettings)` wrapper.

---

## 3. Comparison: Python vs Rust API

### 3.1 Architecture Patterns

| Layer | Python API | Rust API |
|-------|-----------|----------|
| **Flow Definition** | ✅ Full access (FlowBuilder, DataSlice) | ✅ Full access + internals |
| **Operator Registration** | ✅ Custom ops via factories | ✅ Native + custom ops |
| **Execution** | ⚠️ Limited (update(), evaluate_and_dump()) | ✅ Full execution engine |
| **HTTP Service** | ⚠️ Start/stop only | ✅ Full Axum REST API |
| **Setup/Migration** | ⚠️ High-level (SetupChangeBundle) | ✅ Full setup state machine |
| **Context Management** | ❌ None | ✅ LibContext, FlowContext, etc. |
| **Database Pools** | ❌ None | ✅ Full pool management |
| **Built-in Ops** | ⚠️ Through spec objects | ✅ Direct implementation access |

**Legend**:
- ✅ Full access
- ⚠️ Limited/wrapped access
- ❌ No access

---

### 3.2 What Python CAN Do

1. **Define flows** using builder pattern
2. **Register custom operators** (sources, functions, targets) in Python
3. **Execute flows** and get statistics
4. **Manage setup** (create/drop resources)
5. **Start HTTP server** for CocoInsight UI
6. **Configure settings** and authentication

**Example: Custom Python Function**
```python
import cocoindex

class MyFunction(cocoindex.op.FunctionSpec):
    pass

@cocoindex.op.executor_class(cache=True)
class MyFunctionExecutor:
    spec: MyFunction

    def __call__(self, input: str) -> str:
        return input.upper()

# Registered via PyO3 -> PyFunctionFactory -> SimpleFunctionFactory
```

---

### 3.3 What Python CANNOT Do

1. **Access LibContext directly** - cannot inspect flow registry, database pools
2. **Call HTTP endpoints directly** - must use HTTP client if needed
3. **Manipulate execution plans** - no access to `ExecutionPlan` internals
4. **Control setup state machine** - cannot directly read/write setup metadata
5. **Implement builtin operators in Python** - must use factory pattern
6. **Access OpScope, DataScopeBuilder** - flow analysis internals hidden
7. **Manage Tokio runtime** - Python's asyncio bridges to Rust's tokio

---

### 3.4 PyO3 Bridge Architecture

```
Python                         Rust
------                         ----
cocoindex.FlowBuilder     ->   py::FlowBuilder (#[pyclass])
    |                              |
    v                              v
  _engine.FlowBuilder         builder::flow_builder::FlowBuilder
                                   |
                                   v
                              analyzer::analyze_flow()
                                   |
                                   v
                              ExecutionPlan

Custom Python Operator    ->   PyFunctionFactory
    |                              |
    v                              v
  user-defined __call__        interface::SimpleFunctionFactory
                                   |
                                   v
                              Executed via plan::FunctionExecutor
```

**Key Bridge Types**:

1. **`PyFunctionFactory`** - Wraps Python functions
   ```rust
   pub(crate) struct PyFunctionFactory {
       pub py_function_factory: Py<PyAny>,
   }

   #[async_trait]
   impl SimpleFunctionFactory for PyFunctionFactory { /* ... */ }
   ```

2. **`PySourceConnectorFactory`** - Wraps Python sources
   ```rust
   pub(crate) struct PySourceConnectorFactory {
       pub py_source_connector: Py<PyAny>,
   }

   #[async_trait]
   impl SourceFactory for PySourceConnectorFactory { /* ... */ }
   ```

3. **`PyExportTargetFactory`** - Wraps Python targets
   ```rust
   pub(crate) struct PyExportTargetFactory {
       pub py_target_connector: Py<PyAny>,
   }

   #[async_trait]
   impl TargetFactory for PyExportTargetFactory { /* ... */ }
   ```

**Async Bridge**: `pyo3_async_runtimes` handles Python `asyncio` ↔ Rust `tokio` conversion.

---

## 4. Use Cases: When to Use Rust vs Python

### 4.1 Python API Use Cases

✅ **Best for:**
- **Application development** - Building data pipelines
- **Custom transformations** - Python ML/AI libraries (transformers, etc.)
- **Prototyping** - Quick iteration on flow design
- **Integration** - Connecting to Python-only services
- **Scripting** - CLI tools, notebooks, automation

**Example**:
```python
import cocoindex

@cocoindex.flow_def(name="my_flow")
def my_flow(builder, scope):
    source = builder.add_source(cocoindex.sources.LocalFile(...))
    transformed = source.transform(my_custom_function, ...)
    collector = scope.add_collector()
    collector.collect(data=transformed)
    collector.export("target_db", cocoindex.targets.Postgres(...), ...)
```

---

### 4.2 Rust API Use Cases

✅ **Best for:**
- **Framework development** - Building CocoIndex itself
- **Performance-critical operators** - Native DB connectors, parsers
- **Core engine work** - Execution planner, optimizer
- **HTTP API extensions** - Custom endpoints
- **Embedded use** - Rust applications using CocoIndex as a library

**Example** (Rust app using CocoIndex):
```rust
use cocoindex::{LibContext, create_lib_context, Settings};

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::default();
    let lib_ctx = create_lib_context(settings).await?;

    // Directly access flow contexts
    let flow_ctx = lib_ctx.get_flow_context("my_flow")?;
    let exec_plan = flow_ctx.flow.get_execution_plan().await?;

    // Execute with full control
    // ...

    Ok(())
}
```

---

## 5. Extension Points

### 5.1 Python Extension Mechanism

**Three factory types** allow Python code to plug into Rust execution:

1. **Source Connector**
   ```python
   class MySourceConnector:
       def create_import_context(self, spec: dict, ...) -> ImportContext:
           # Return context with async read method

   cocoindex.register_source_connector("my_source", MySourceConnector())
   ```

2. **Function Factory**
   ```python
   class MyFunctionFactory:
       def create_executor(self, spec: dict, input_schema, ...) -> Executor:
           # Return executor with __call__ method

   cocoindex.register_function_factory("my_function", MyFunctionFactory())
   ```

3. **Target Connector**
   ```python
   class MyTargetConnector:
       def create_export_context(self, name, spec, key_fields, value_fields, ...) -> ExportContext:
           # Return context with async write methods

       def check_state_compatibility(self, desired, existing) -> Compatibility:
           # Return compatibility status

   cocoindex.register_target_connector("my_target", MyTargetConnector())
   ```

**Rust bridges these** to native `SourceFactory`, `SimpleFunctionFactory`, `TargetFactory` traits.

---

### 5.2 Rust Extension Mechanism

**Direct trait implementation**:

```rust
use cocoindex::ops::interface::{SourceFactory, SourceBuildOutput};
use async_trait::async_trait;

pub struct MyCustomSource;

#[async_trait]
impl SourceFactory for MyCustomSource {
    async fn build(
        self: Arc<Self>,
        spec: serde_json::Value,
        context: Arc<FlowInstanceContext>,
    ) -> Result<SourceBuildOutput> {
        // Implement source logic
        // ...
    }

    // Other trait methods
    // ...
}

// Register
register_factory("my_custom_source", ExecutorFactory::Source(Arc::new(MyCustomSource)));
```

**No PyO3 overhead** - direct Rust-to-Rust calls in execution.

---

## 6. Architectural Insights

### 6.1 Design Philosophy

1. **Performance-critical in Rust**
   - Execution engine, data movement, I/O
   - All operators (sources, functions, targets)
   - Database interactions, connection pooling

2. **Convenience in Python**
   - Flow definition DSL
   - High-level orchestration
   - Integration with Python ecosystem

3. **Clear separation**
   - Python: **Declarative** (what to do)
   - Rust: **Imperative** (how to do it)

---

### 6.2 Data Flow

```
Python Layer:
  FlowBuilder -> define flow spec -> FlowInstanceSpec (JSON-like)

PyO3 Bridge:
  FlowInstanceSpec (Python) -> Serialize -> FlowInstanceSpec (Rust)

Rust Layer:
  FlowInstanceSpec -> Analyzer -> AnalyzedFlow
                              -> ExecutionPlan
                              -> Execute (row_indexer, evaluator, etc.)
                              -> IndexUpdateInfo

PyO3 Bridge:
  IndexUpdateInfo (Rust) -> Serialize -> IndexUpdateInfo (Python)
```

**Key point**: Python never directly executes data transformations. It only:
1. Describes what to do (spec)
2. Receives results (stats, errors)

---

### 6.3 Memory Model

- **Python objects** (`FlowBuilder`, `DataSlice`) are thin wrappers
  - Hold `Py<PyAny>` references to Rust objects
  - Minimal state on Python side

- **Rust holds all data**
  - Flow specs, schemas, execution state
  - Database connections, connection pools
  - Tokio tasks, futures

- **Async synchronization**
  - Python `asyncio.Future` ↔ Rust `tokio::task`
  - Managed by `pyo3_async_runtimes`

---

## 7. API Stability and Versioning

### 7.1 Public API Guarantees

**Python API** (`cocoindex` package):
- ✅ **Stable**: Flow definition API, operator specs
- ✅ **Stable**: `init()`, `start_server()`, lifecycle
- ⚠️ **Evolving**: `_engine` internal details may change

**Rust API**:
- ⚠️ **Internal**: Most Rust APIs are `pub(crate)` - internal to library
- ❌ **No guarantees**: Traits, execution engine, context types can change
- ✅ **Exception**: Operator factory traits aim for stability (for custom ops)

---

### 7.2 Semantic Versioning

Based on repository patterns:

```
v0.x.y - Pre-1.0
  - Breaking changes possible in minor versions
  - Python API surface stabilizing
  - Rust internals subject to refactoring

v1.0.0+ (future)
  - Stable Python API
  - Documented extension points for Rust
  - Internal Rust APIs still unstable
```

---

## 8. Recommendations

### 8.1 For Python Users

1. **Stick to `cocoindex` package** - Don't rely on `_engine` internals
2. **Use factory pattern** for custom operators
3. **Follow examples** in `examples/` directory
4. **Type hints** - Use provided type stubs for better IDE support
5. **Async best practices** - Use `async def` with `await` for I/O operations

---

### 8.2 For Rust Developers

1. **Study operator traits** - `SourceFactory`, `SimpleFunctionFactory`, `TargetFactory`
2. **Look at builtin operators** - `ops/sources/`, `ops/targets/` for examples
3. **Understand execution model** - Read `builder/analyzer.rs`, `execution/`
4. **Respect API boundaries** - Use `pub(crate)` for internals
5. **Test with Python** - Ensure PyO3 bindings work correctly

---

### 8.3 For Contributors

1. **Python additions** - Consider if it should be in Rust (performance) or Python (convenience)
2. **Rust additions**:
   - Mark as `pub(crate)` unless part of extension API
   - Add PyO3 bindings if Python needs access
   - Document in Rust docs (`///` comments)
3. **Breaking changes** - Coordinate between Python and Rust APIs
4. **Testing** - Test both Python and Rust interfaces

---

## 9. Future Evolution

### 9.1 Potential Python API Expansions

- **Direct access to flow schema** - Read field types without executing
- **Custom index types** - Python-defined vector index methods
- **Query builder** - Python DSL for querying indexed data
- **Monitoring hooks** - Callbacks for execution events
- **Transient flows** - More ergonomic in-memory transformations

### 9.2 Potential Rust API Stabilization

- **Plugin system** - Dynamic loading of Rust operator libraries
- **C FFI** - Expose core to other languages
- **Async executor abstraction** - Support non-Tokio runtimes
- **WebAssembly** - Run flows in browser

---

## 10. Conclusion

### Summary

The CocoIndex architecture demonstrates a **well-designed separation of concerns**:

1. **Python provides** a high-level, ergonomic API for defining data pipelines
2. **Rust provides** a high-performance execution engine with low-level control
3. **PyO3 bridges** the two worlds seamlessly

### API Surface Breakdown

| Category | Python API | Rust API | Ratio |
|----------|-----------|----------|-------|
| Flow Building | 100% | 100% | 1:1 |
| Operator Registration | 100% | 100% | 1:1 |
| Execution Control | ~20% | 100% | 1:5 |
| Setup Management | ~30% | 100% | 1:3 |
| Service Layer | ~10% | 100% | 1:10 |
| Context Management | 0% | 100% | 0:1 |
| **Overall** | **~30-40%** | **100%** | **1:3** |

### Key Takeaway

**Python users get a complete, powerful API** for building data pipelines without needing Rust knowledge. **Rust developers get full access** to internals for performance optimization and core development. The ~60-70% of Rust API not exposed to Python is primarily:

- Internal implementation details
- Low-level performance optimizations
- Service infrastructure (HTTP, database pooling)
- Setup state management internals

This is **intentional and appropriate** - Python users don't need (and shouldn't have) access to these internals.

---

## Appendix A: Key File Reference

### Python Package
- `python/cocoindex/__init__.py` - Main exports
- `python/cocoindex/flow.py` - FlowBuilder, DataSlice wrappers
- `python/cocoindex/op.py` - Operator base classes
- `python/cocoindex/lib.py` - Settings, init, server wrappers

### Rust Core
- `rust/cocoindex/src/lib.rs` - Module structure
- `rust/cocoindex/src/py/mod.rs` - **PyO3 bindings**
- `rust/cocoindex/src/lib_context.rs` - Runtime context
- `rust/cocoindex/src/builder/flow_builder.rs` - Flow builder implementation
- `rust/cocoindex/src/builder/analyzer.rs` - Flow analysis
- `rust/cocoindex/src/ops/interface.rs` - Operator traits
- `rust/cocoindex/src/ops/py_factory.rs` - Python operator bridges
- `rust/cocoindex/src/service/flows.rs` - HTTP API
- `rust/cocoindex/src/setup/driver.rs` - Setup state machine
- `rust/cocoindex/src/execution/` - Execution engine

### Examples
- `examples/postgres_source/` - Source usage
- `examples/text_embedding_qdrant/` - Function + target usage
- `examples/manuals_llm_extraction/` - Custom Python function
- `examples/live_updates/` - Live flow updates

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| **Flow** | A data pipeline from sources through transformations to targets |
| **Source** | Data input (files, databases, APIs) |
| **Function** | Transformation (parse, embed, extract) |
| **Target** | Data output (databases, search indexes) |
| **Collector** | Accumulates rows for export to a target |
| **Scope** | Execution context for operations (root scope, row scope) |
| **DataSlice** | Reference to a field or value in a flow |
| **Setup** | Resource provisioning (tables, indexes) |
| **ExecutionPlan** | Compiled flow ready for execution |
| **LibContext** | Global runtime context (flows, db pools) |
| **FlowContext** | Per-flow runtime context |
| **PyO3** | Rust-Python bridge library |

---

**Document Version**: 1.0
**Last Updated**: 2024
**Maintainer**: Analysis of cocoindex-io/cocoindex repository
