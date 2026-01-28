# ReCoco Target Factory Pattern Guide

**Purpose**: Document the correct pattern for implementing D1 target factory following ReCoco conventions

**Date**: January 27, 2026
**Reference**: ReCoco core 0.2.1 - postgres target implementation

---

## TargetFactoryBase Trait

Similar to `SimpleFunctionFactoryBase` for functions, targets use `TargetFactoryBase` trait with blanket implementation for `TargetFactory`.

### Associated Types

```rust
pub trait TargetFactoryBase: Send + Sync + 'static {
    type Spec: DeserializeOwned + Send + Sync;
    type DeclarationSpec: DeserializeOwned + Send + Sync;

    type SetupKey: Debug + Clone + Serialize + DeserializeOwned + Eq + Hash + Send + Sync;
    type SetupState: Debug + Clone + Serialize + DeserializeOwned + Send + Sync;
    type SetupChange: ResourceSetupChange;

    type ExportContext: Send + Sync + 'static;

    // ... methods
}
```

**For D1**:
- `Spec`: D1 connection configuration (account_id, database_id, api_token, table)
- `DeclarationSpec`: Usually `()` (empty)
- `SetupKey`: Table identifier (database + table name)
- `SetupState`: Schema state (columns, indexes, constraints)
- `SetupChange`: SQL migrations to apply
- `ExportContext`: Runtime context with HTTP client, connection info

---

## Required Methods

### 1. name() - Factory Identifier

```rust
fn name(&self) -> &str {
    "d1"
}
```

### 2. build() - Initialize Target

**Purpose**: Parse specs, create export contexts, return setup keys/states

**Signature**:
```rust
async fn build(
    self: Arc<Self>,
    data_collections: Vec<TypedExportDataCollectionSpec<Self>>,
    declarations: Vec<Self::DeclarationSpec>,
    context: Arc<FlowInstanceContext>,
) -> Result<(
    Vec<TypedExportDataCollectionBuildOutput<Self>>,
    Vec<(Self::SetupKey, Self::SetupState)>,
)>;
```

**Responsibilities**:
1. Validate specs (e.g., table name required if schema specified)
2. Create `SetupKey` (table identifier)
3. Create `SetupState` (desired schema)
4. Create `ExportContext` (async future returning connection info)
5. Return build output with setup key + state + export context

**Example from Postgres**:
```rust
let table_id = TableId {
    database: spec.database.clone(),
    schema: spec.schema.clone(),
    table_name: spec.table_name.unwrap_or_else(|| {
        utils::db::sanitize_identifier(&format!(
            "{}__{}",
            context.flow_instance_name, collection_name
        ))
    }),
};

let setup_state = SetupState::new(
    &table_id,
    &key_fields_schema,
    &value_fields_schema,
    &index_options,
    &column_options,
)?;

let export_context = Box::pin(async move {
    let db_pool = get_db_pool(db_ref.as_ref(), &auth_registry).await?;
    Ok(Arc::new(ExportContext::new(db_pool, table_id, schemas)?))
});

Ok(TypedExportDataCollectionBuildOutput {
    setup_key: table_id,
    desired_setup_state: setup_state,
    export_context,
})
```

---

### 3. diff_setup_states() - Schema Migration Planning

**Purpose**: Compare desired vs existing schema, generate migration changes

**Signature**:
```rust
async fn diff_setup_states(
    &self,
    key: Self::SetupKey,
    desired_state: Option<Self::SetupState>,
    existing_states: setup::CombinedState<Self::SetupState>,
    flow_instance_ctx: Arc<FlowInstanceContext>,
) -> Result<Self::SetupChange>;
```

**Responsibilities**:
1. Compare desired schema with existing schema
2. Generate SQL migrations (CREATE TABLE, ALTER TABLE, CREATE INDEX)
3. Return `SetupChange` with migration instructions

**For D1**: Generate SQLite DDL for schema changes

---

### 4. check_state_compatibility() - Schema Compatibility

**Purpose**: Validate if existing schema is compatible with desired schema

**Signature**:
```rust
fn check_state_compatibility(
    &self,
    desired_state: &Self::SetupState,
    existing_state: &Self::SetupState,
) -> Result<SetupStateCompatibility>;
```

**Returns**: `Compatible`, `Incompatible`, or `NeedMigration`

---

### 5. describe_resource() - Human-Readable Description

```rust
fn describe_resource(&self, key: &Self::SetupKey) -> Result<String> {
    Ok(format!("D1 table: {}.{}", key.database_id, key.table_name))
}
```

---

### 6. **apply_mutation() - Critical Method for Data Operations**

**Purpose**: Execute upserts and deletes

**Signature**:
```rust
async fn apply_mutation(
    &self,
    mutations: Vec<ExportTargetMutationWithContext<'async_trait, Self::ExportContext>>,
) -> Result<()>;
```

**Mutation Structure**:
```rust
pub struct ExportTargetMutation {
    pub upserts: Vec<(KeyValue, FieldValues)>,
    pub deletes: Vec<KeyValue>,
}

pub struct ExportTargetMutationWithContext<'a, C> {
    pub mutation: &'a ExportTargetMutation,
    pub export_context: &'a C,
}
```

**Postgres Example**:
```rust
async fn apply_mutation(
    &self,
    mutations: Vec<ExportTargetMutationWithContext<'async_trait, Self::ExportContext>>,
) -> Result<()> {
    let mut_groups = mutations
        .into_iter()
        .into_group_map_by(|m| m.export_context.db_pool.clone());

    for (db_pool, mut_groups) in mut_groups {
        let mut txn = db_pool.begin().await?;

        // Execute all upserts in transaction
        for mut_group in mut_groups.iter() {
            mut_group
                .export_context
                .upsert(&mut_group.mutation.upserts, &mut txn)
                .await?;
        }

        // Execute all deletes in transaction
        for mut_group in mut_groups.iter() {
            mut_group
                .export_context
                .delete(&mut_group.mutation.deletes, &mut txn)
                .await?;
        }

        txn.commit().await?;
    }
    Ok(())
}
```

**For D1**:
1. Group mutations by database
2. Convert to D1 prepared statements
3. Use batch API for upserts (ON CONFLICT pattern)
4. Use batch API for deletes
5. Execute as transaction

---

### 7. apply_setup_changes() - Execute Schema Migrations

**Purpose**: Apply schema changes to database

**Signature**:
```rust
async fn apply_setup_changes(
    &self,
    changes: Vec<TypedResourceSetupChangeItem<'async_trait, Self>>,
    context: Arc<FlowInstanceContext>,
) -> Result<()>;
```

**Postgres Example**:
```rust
async fn apply_setup_changes(
    &self,
    changes: Vec<TypedResourceSetupChangeItem<'async_trait, Self>>,
    context: Arc<FlowInstanceContext>,
) -> Result<()> {
    for change in changes.iter() {
        let db_pool = get_db_pool(change.key.database.as_ref(), &context.auth_registry).await?;
        change.setup_change.apply_change(&db_pool, &change.key).await?;
    }
    Ok(())
}
```

**For D1**: Execute DDL via D1 API (CREATE TABLE, CREATE INDEX, etc.)

---

## Supporting Types

### SetupKey (Table Identifier)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct D1TableId {
    pub database_id: String,
    pub table_name: String,
}
```

### SetupState (Schema Definition)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D1SetupState {
    pub columns: Vec<ColumnSchema>,
    pub primary_key: Vec<String>,
    pub indexes: Vec<IndexSchema>,
}
```

### SetupChange (Migration Instructions)

```rust
pub struct D1SetupChange {
    pub create_table_sql: Option<String>,
    pub create_indexes_sql: Vec<String>,
    pub alter_table_sql: Vec<String>,
}

#[async_trait]
impl ResourceSetupChange for D1SetupChange {
    fn describe_changes(&self) -> Vec<String> {
        let mut changes = vec![];
        if let Some(sql) = &self.create_table_sql {
            changes.push(format!("CREATE TABLE: {}", sql));
        }
        for sql in &self.create_indexes_sql {
            changes.push(format!("CREATE INDEX: {}", sql));
        }
        changes
    }
}
```

### ExportContext (Runtime State)

```rust
pub struct D1ExportContext {
    pub database_id: String,
    pub table_name: String,
    pub http_client: reqwest::Client,
    pub api_token: String,
    pub account_id: String,
    pub key_fields_schema: Vec<FieldSchema>,
    pub value_fields_schema: Vec<FieldSchema>,
}

impl D1ExportContext {
    pub async fn upsert(
        &self,
        upserts: &[(KeyValue, FieldValues)],
    ) -> Result<()> {
        // Build batch UPSERT statements
        let statements = upserts
            .iter()
            .map(|(key, values)| self.build_upsert_stmt(key, values))
            .collect::<Result<Vec<_>>>()?;

        // Execute batch via D1 API
        self.execute_batch(statements).await
    }

    pub async fn delete(
        &self,
        deletes: &[KeyValue],
    ) -> Result<()> {
        // Build batch DELETE statements
        let statements = deletes
            .iter()
            .map(|key| self.build_delete_stmt(key))
            .collect::<Result<Vec<_>>>()?;

        // Execute batch via D1 API
        self.execute_batch(statements).await
    }
}
```

---

## Implementation Checklist for D1

### Core Structure
- [ ] Define `D1TargetFactory` struct
- [ ] Define `D1Spec` (account_id, database_id, api_token, table)
- [ ] Define `D1TableId` (SetupKey)
- [ ] Define `D1SetupState` (schema)
- [ ] Define `D1SetupChange` (migrations)
- [ ] Define `D1ExportContext` (runtime state with HTTP client)

### TargetFactoryBase Implementation
- [ ] Implement `name()` → "d1"
- [ ] Implement `build()` → parse specs, create contexts
- [ ] Implement `diff_setup_states()` → generate migrations
- [ ] Implement `check_state_compatibility()` → validate schemas
- [ ] Implement `describe_resource()` → human-readable names
- [ ] Implement `apply_mutation()` → **CRITICAL - upsert/delete via D1 API**
- [ ] Implement `apply_setup_changes()` → execute DDL

### ExportContext Methods
- [ ] Implement `upsert()` → batch INSERT ... ON CONFLICT
- [ ] Implement `delete()` → batch DELETE
- [ ] Implement `execute_batch()` → call D1 HTTP API
- [ ] Implement `build_upsert_stmt()` → generate UPSERT SQL
- [ ] Implement `build_delete_stmt()` → generate DELETE SQL

### HTTP Client Integration
- [ ] Use `reqwest` for D1 REST API
- [ ] Implement authentication (Bearer token)
- [ ] Implement batch request formatting
- [ ] Implement response parsing
- [ ] Implement error handling (retries, timeouts)

### Registration
- [ ] Add to `ExecutorFactoryRegistry` (similar to SimpleFunctionFactory)
- [ ] Export from `targets/mod.rs`
- [ ] Update `ThreadOperators` registry if needed

---

## Key Differences from SimpleFunctionFactory

| Aspect | Function | Target |
|--------|----------|--------|
| **Purpose** | Transform data | Store data |
| **Key Method** | `build_executor()` → executor | `apply_mutation()` → upsert/delete |
| **Associated Types** | `Spec`, `ResolvedArgs` | `Spec`, `SetupKey`, `SetupState`, `SetupChange`, `ExportContext` |
| **Complexity** | Simple (transform only) | Complex (schema management + data operations) |
| **Setup** | None | Schema creation, migrations, indexes |

---

## Next Steps

1. Implement D1-specific types (TableId, SetupState, SetupChange, ExportContext)
2. Implement `TargetFactoryBase` for `D1TargetFactory`
3. Implement `ExportContext` methods for HTTP API interaction
4. Test with local Wrangler D1 database
5. Integrate with `ThreadFlowBuilder`

---

## References

- ReCoco source: `~/.cargo/registry/.../recoco-core-0.2.1/src/ops/`
- Trait definition: `ops/factory_bases.rs`
- Postgres example: `ops/targets/postgres.rs`
- Registration: `ops/sdk.rs` (ExecutorFactoryRegistry)
