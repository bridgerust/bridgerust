use anyhow::{Context, Result};
use chrono::Utc;
use dialoguer::{Select};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

pub async fn handle(resource: String, name: String) -> Result<()> {
    match resource.as_str() {
        "migration" => generate_migration(&name).await?,
        _ => anyhow::bail!("Unknown resource type: {}. Supported: migration", resource),
    }
    Ok(())
}

async fn generate_migration(name: &str) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let filename = format!("{}_{}.json", timestamp, name);
    let path = PathBuf::from("migrations").join(&filename);

    if !path.parent().unwrap().exists() {
        fs::create_dir_all(path.parent().unwrap())?;
    }

    let items = vec!["Create Collection", "Delete Collection", "Empty"];
    let selection = Select::new()
        .with_prompt("Choose migration template")
        .items(&items)
        .default(0)
        .interact()?;

    let (up_ops, down_ops) = match selection {
        0 => (
            vec![json!({
                "type": "create_collection",
                "schema": {
                    "name": "example_collection",
                    "dimension": 768,
                    "metric": "cosine"
                }
            })],
            vec![json!({
                "type": "delete_collection",
                "name": "example_collection"
            })],
        ),
        1 => (
            vec![json!({
                "type": "delete_collection",
                "name": "example_collection"
            })],
            vec![json!({
                "type": "create_collection",
                "schema": {
                    "name": "example_collection",
                    "dimension": 768,
                    "metric": "cosine"
                }
            })],
        ),
        _ => (vec![], vec![]),
    };

    let migration = json!({
        "version": format!("{}_{}", timestamp, name),
        "description": format!("Migration for {}", name),
        "operations": up_ops,
        "down_operations": down_ops
    });

    fs::write(&path, serde_json::to_string_pretty(&migration)?)
        .context("Failed to write migration file")?;

    println!("Created migration: {}", path.display());
    Ok(())
}
