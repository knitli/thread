use crate::prelude::*;

use super::StateChange;
use sqlx::PgPool;

const SETUP_METADATA_TABLE_NAME: &str = "cocoindex_setup_metadata";
pub const FLOW_VERSION_RESOURCE_TYPE: &str = "__FlowVersion";

#[derive(sqlx::FromRow, Debug)]
pub struct SetupMetadataRecord {
    pub flow_name: String,
    // e.g. "Flow", "SourceTracking", "Target:{TargetType}"
    pub resource_type: String,
    pub key: serde_json::Value,
    pub state: Option<serde_json::Value>,
    pub staging_changes: sqlx::types::Json<Vec<StateChange<serde_json::Value>>>,
}

pub fn parse_flow_version(state: &Option<serde_json::Value>) -> Option<u64> {
    match state {
        Some(serde_json::Value::Number(n)) => n.as_u64(),
        _ => None,
    }
}

/// Returns None if metadata table doesn't exist.
pub async fn read_setup_metadata(pool: &PgPool) -> Result<Option<Vec<SetupMetadataRecord>>> {
    let mut db_conn = pool.acquire().await?;
    let query_str = format!(
        "SELECT flow_name, resource_type, key, state, staging_changes FROM {SETUP_METADATA_TABLE_NAME}",
    );
    let metadata = sqlx::query_as(&query_str).fetch_all(&mut *db_conn).await;
    let result = match metadata {
        Ok(metadata) => Some(metadata),
        Err(err) => {
            let exists: Option<bool> = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = $1)",
            )
            .bind(SETUP_METADATA_TABLE_NAME)
            .fetch_one(&mut *db_conn)
            .await?;
            if !exists.unwrap_or(false) {
                None
            } else {
                return Err(err.into());
            }
        }
    };
    Ok(result)
}
