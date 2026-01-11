<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# CocoIndex Rust API: Setup & Context

Using `cocoindex::lib_context` and `cocoindex::settings`.

## Library Initialization

To use CocoIndex in a Rust application, you must initialize the `LibContext`. This context manages database pools, flow instances, and global resources.

### `Settings` Configuration

`cocoindex::settings::Settings` is the configuration struct.

```rust
use cocoindex::settings::{Settings, DatabaseConnectionSpec, GlobalExecutionOptions};

let settings = Settings {
    // Database connection details (Required for persistence)
    database: Some(DatabaseConnectionSpec {
        url: "postgresql://localhost:5432/mydb".to_string(),
        user: Some("user".to_string()),
        password: Some("pass".to_string()),
        min_connections: 5,
        max_connections: 20,
    }),
    app_namespace: "my_app".to_string(), // Isolates flows
    global_execution_options: GlobalExecutionOptions::default(),
    ignore_target_drop_failures: false,
};
```

### Creating `LibContext`

```rust
use cocoindex::lib_context::{create_lib_context, LibContext};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize settings...
    let lib_ctx = create_lib_context(settings).await?;

    // Access the runtime if needed (though you are widely in tokio scope)
    // let runtime = lib_ctx.get_runtime();

    Ok(())
}
```

## Accessing Flows

Once initialized, you can access compiled flows.

```rust
use std::sync::Arc;

async fn run_flow(lib_ctx: &LibContext) -> anyhow::Result<()> {
    // Get a handle to a specific flow context
    let flow_ctx = lib_ctx.get_flow_context("user_ranking_flow")?;

    let flow_name = flow_ctx.flow_name();
    println!("Accessed flow: {}", flow_name);

    // Access the execution context (checks for valid setup state)
    let exec_ctx = flow_ctx.use_execution_ctx().await?;

    Ok(())
}
```

## Database Access

You can access the internal connection pool if needed.

```rust
let pool = lib_ctx.require_builtin_db_pool()?;
// Use sqlx pool...
```
