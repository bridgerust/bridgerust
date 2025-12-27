use crate::declarative::DeclarativeMigration;
use crate::MigrateAction;
use anyhow::{Context, Result};
use bridge_embex::client::EmbexClient;
use bridge_embex::migration::MigrationManager;
use bridge_embex_core::migration::Migration;
use bridge_embex_infrastructure::config::EmbexConfig;
use std::fs;
use std::path::Path;

pub async fn handle(action: MigrateAction) -> Result<()> {
    let config = EmbexConfig::from_env().context("Failed to load configuration. Ensure EMBEX_PROVIDER and EMBEX_URL are set.")?;
    let client = EmbexClient::new(config)?;
    let manager = MigrationManager::new(client.db());

    let mut migrations: Vec<Box<dyn Migration>> = Vec::new();
    let migration_dir = Path::new("migrations");
    
    if !migration_dir.exists() {
        if matches!(action, MigrateAction::Status) {
            println!("No migrations directory found.");
            return Ok(());
        }
        anyhow::bail!("Migrations directory 'migrations' not found. Run 'embex generate migration <name>' to create one.");
    }

    let mut entries: Vec<_> = fs::read_dir(migration_dir)?
        .filter_map(|res| res.ok())
        .map(|dir| dir.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "json"))
        .collect();

    entries.sort();

    for path in entries {
        let content = fs::read_to_string(&path).context(format!("Failed to read {}", path.display()))?;
        let migration: DeclarativeMigration = serde_json::from_str(&content)
            .context(format!("Failed to parse migration {}", path.display()))?;
        migrations.push(Box::new(migration));
    }

    match action {
        MigrateAction::Up => {
            println!("Running pending migrations...");
            manager.run_migrations(migrations).await?;
            println!("Migrations completed successfully.");
        }
        MigrateAction::Down => {
            println!("Rolling back last migration...");
            manager.rollback_last(1, &migrations).await?;
            println!("Rollback completed.");
        }
        MigrateAction::Status => {
            let applied = manager.get_applied_migrations().await?;
            println!("Migration Status:");
            println!("-----------------");
            
            for m in &migrations {
                let version = m.version();
                let status = if applied.contains(&version) {
                    "APPLIED"
                } else {
                    "PENDING"
                };
                println!("[{}] {}", status, version);
            }
        }
    }

    Ok(())
}
