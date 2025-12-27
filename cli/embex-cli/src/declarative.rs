use async_trait::async_trait;
use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{Result, EmbexError};
use bridge_embex_core::migration::Migration;
use bridge_embex_core::types::CollectionSchema;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

#[derive(Deserialize, Clone)]
pub struct DeclarativeMigration {
    pub version: String,
    #[serde(default)]
    pub _description: Option<String>,
    pub operations: Vec<Value>,
    pub down_operations: Vec<Value>,
}

#[async_trait]
impl Migration for DeclarativeMigration {
    fn version(&self) -> String {
        self.version.clone()
    }

    async fn up(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
        for op in &self.operations {
            execute_op(op, db.clone()).await?;
        }
        Ok(())
    }

    async fn down(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
        for op in &self.down_operations {
            execute_op(op, db.clone()).await?;
        }
        Ok(())
    }
}

async fn execute_op(op: &Value, db: Arc<dyn VectorDatabase>) -> Result<()> {
    let type_ = op.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
        EmbexError::Validation("Operation missing type".to_string())
    })?;

    match type_ {
        "create_collection" => {
            let schema_val = op.get("schema").ok_or_else(|| {
                EmbexError::Validation("create_collection missing schema".to_string())
            })?;
            let schema: CollectionSchema = serde_json::from_value(schema_val.clone()).map_err(|e| {
                EmbexError::Validation(format!("Invalid schema: {}", e))
            })?;
            db.create_collection(&schema).await
        }
        "delete_collection" => {
            let name = op.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                EmbexError::Validation("delete_collection missing name".to_string())
            })?;
            db.delete_collection(name).await
        }
        _ => Err(EmbexError::Validation(format!(
            "Unknown operation type: {}",
            type_
        ))),
    }
}
