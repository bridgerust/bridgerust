# Migration System

Embex provides a robust migration system for managing database schema changes and data transformations over time.

## Table of Contents

- [Overview](#overview)
- [Creating Migrations](#creating-migrations)
- [Running Migrations](#running-migrations)
- [Rollback](#rollback)
- [Validation](#validation)
- [Best Practices](#best-practices)

## Overview

The migration system tracks applied migrations in a special `_embex_migrations` collection and ensures:

- Migrations are applied in order
- Already-applied migrations are skipped
- Failed migrations trigger automatic rollback
- Migration state can be validated

## Creating Migrations

Migrations implement the `Migration` trait with `up()` and `down()` methods:

```rust
use bridge_embex::Migration;
use bridge_embex::VectorDatabase;
use bridge_embex_core::error::Result;
use std::sync::Arc;

struct CreateUsersCollection;

#[async_trait::async_trait]
impl Migration for CreateUsersCollection {
    fn version(&self) -> String {
        "20240101000000_create_users".to_string()
    }

    async fn up(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
        use bridge_embex_core::types::{CollectionSchema, DistanceMetric};

        let schema = CollectionSchema {
            name: "users".to_string(),
            dimension: 384,
            metric: DistanceMetric::Cosine,
        };

        db.create_collection(&schema).await
    }

    async fn down(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
        db.delete_collection("users").await
    }
}
```

### Migration Versioning

Use a consistent versioning scheme:

- **Timestamp-based**: `20240101000000_description` (recommended)
- **Sequential**: `001_description`, `002_description`
- **Semantic**: `v1.0.0_description`

## Running Migrations

### Basic Usage

```rust
use bridge_embex::{EmbexClient, MigrationManager};
use bridge_embex_infrastructure::config::EmbexConfig;

let config = EmbexConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    ..Default::default()
};

let client = EmbexClient::new(config)?;
let manager = MigrationManager::new(client.db());

let migrations: Vec<Box<dyn Migration>> = vec![
    Box::new(CreateUsersCollection),
    Box::new(AddIndexes),
];

// Validate migrations before running
MigrationManager::validate_migrations(&migrations)?;

// Run pending migrations
manager.run_migrations(migrations).await?;
```

### Migration Execution

Migrations are executed:

1. **Sequentially** - One at a time, in order
2. **Atomically** - Each migration is recorded only after successful completion
3. **With rollback** - If a migration fails, previously applied migrations in the same run are rolled back

## Rollback

### Rolling Back Specific Migrations

```rust
let migrations: Vec<Box<dyn Migration>> = vec![
    Box::new(CreateUsersCollection),
    Box::new(AddIndexes),
];

// Rollback specific migrations (in reverse order)
let to_rollback = vec![
    "20240102000000_add_indexes".to_string(),
];
manager.rollback_migrations(&to_rollback, &migrations).await?;
```

### Rolling Back Last N Migrations

```rust
// Rollback the last 2 migrations
manager.rollback_last(2, &migrations).await?;
```

### Automatic Rollback

If a migration fails during `run_migrations()`, all migrations applied in that run are automatically rolled back:

```rust
// If AddIndexes fails, CreateUsersCollection will be rolled back
let migrations: Vec<Box<dyn Migration>> = vec![
    Box::new(CreateUsersCollection),  // Applied
    Box::new(AddIndexes),              // Fails -> triggers rollback
];

let result = manager.run_migrations(migrations).await;
// CreateUsersCollection is rolled back automatically
```

## Validation

### Validate Migration List

Check for duplicate versions before running:

```rust
let migrations: Vec<Box<dyn Migration>> = vec![
    Box::new(CreateUsersCollection),
    Box::new(AddIndexes),
];

// Returns error if duplicate versions found
MigrationManager::validate_migrations(&migrations)?;
```

### Validate Migration State

Check consistency between applied migrations and available migrations:

```rust
// Warns about migrations that are applied but not in the list
manager.validate_migration_state(&migrations).await?;
```

### Get Applied Migrations

```rust
let applied = manager.get_applied_migrations().await?;
println!("Applied migrations: {:?}", applied);
```

### Get Latest Migration

```rust
let latest = manager.get_latest_migration().await?;
match latest {
    Some(version) => println!("Latest: {}", version),
    None => println!("No migrations applied"),
}
```

## Best Practices

### 1. Use Timestamp-Based Versioning

```rust
fn version(&self) -> String {
    // Format: YYYYMMDDHHMMSS_description
    "20240101120000_create_users".to_string()
}
```

### 2. Make Migrations Idempotent

```rust
async fn up(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
    // Check if collection exists before creating
    // Or handle CollectionExists error gracefully
    match db.create_collection(&schema).await {
        Ok(_) => Ok(()),
        Err(e) if e.is_collection_error() => Ok(()), // Already exists
        Err(e) => Err(e),
    }
}
```

### 3. Implement Proper Rollback

Always implement `down()` to reverse `up()`:

```rust
async fn down(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
    // Reverse all changes made in up()
    db.delete_collection("users").await
}
```

### 4. Test Migrations

```rust
#[tokio::test]
async fn test_migration_up_down() {
    let mock_db = Arc::new(MockDatabase::new());
    let migration = CreateUsersCollection;

    // Test up
    migration.up(mock_db.clone()).await.unwrap();
    assert!(collection_exists(&mock_db, "users"));

    // Test down
    migration.down(mock_db.clone()).await.unwrap();
    assert!(!collection_exists(&mock_db, "users"));
}
```

### 5. Validate Before Running

Always validate migrations before running in production:

```rust
// Validate no duplicates
MigrationManager::validate_migrations(&migrations)?;

// Validate state consistency
manager.validate_migration_state(&migrations).await?;

// Then run
manager.run_migrations(migrations).await?;
```

### 6. Use Transactions When Possible

Some adapters support transactions. Use them for complex migrations:

```rust
async fn up(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
    // Create multiple collections atomically if supported
    db.create_collection(&schema1).await?;
    db.create_collection(&schema2).await?;
    Ok(())
}
```

### 7. Document Migration Purpose

Add comments explaining what the migration does:

```rust
/// Creates the users collection with 384-dimensional embeddings
/// for storing user profile vectors.
struct CreateUsersCollection;
```

## Error Handling

### Migration Failures

If a migration fails:

1. The migration is **not** recorded as applied
2. Previously applied migrations in the same run are **rolled back**
3. An error is returned with details

```rust
match manager.run_migrations(migrations).await {
    Ok(_) => println!("All migrations applied successfully"),
    Err(e) => {
        eprintln!("Migration failed: {}", e);
        // Check which migrations were rolled back
        let applied = manager.get_applied_migrations().await?;
        println!("Remaining applied: {:?}", applied);
    }
}
```

### Rollback Failures

If a rollback fails:

- The error is logged
- Rollback continues for remaining migrations
- An error is returned at the end

## Examples

### Complete Example

```rust
use bridge_embex::{EmbexClient, Migration, MigrationManager};
use bridge_embex_infrastructure::config::EmbexConfig;
use std::sync::Arc;

#[async_trait::async_trait]
impl Migration for CreateUsersCollection {
    fn version(&self) -> String {
        "20240101000000_create_users".to_string()
    }

    async fn up(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
        // Implementation
        Ok(())
    }

    async fn down(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
        // Implementation
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = EmbexConfig {
        provider: "qdrant".to_string(),
        url: "http://localhost:6333".to_string(),
        ..Default::default()
    };

    let client = EmbexClient::new(config)?;
    let manager = MigrationManager::new(client.db());

    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(CreateUsersCollection),
    ];

    // Validate and run
    MigrationManager::validate_migrations(&migrations)?;
    manager.run_migrations(migrations).await?;

    println!("Migrations completed successfully");
    Ok(())
}
```

### Node.js Example

In Node.js, migrations are defined as declarative objects:

```javascript
const { EmbexClient } = require("@bridgerust/embex");

const client = new EmbexClient("qdrant", "http://localhost:6333");

const migrations = [
  {
    version: "20240101000000_create_users",
    operations: [
      {
        type: "create_collection",
        schema: {
          name: "users",
          dimension: 384,
          metric: "cosine",
        },
      },
    ],
    downOperations: [
      {
        type: "delete_collection",
        name: "users",
      },
    ],
  },
];

// Run migrations
await client.runMigrations(migrations);
```

### Python Example

In Python, you can use any object that has `version` attribute and `up`/`down` methods:

```python
import asyncio
from embex import EmbexClient

class CreateUsersMigration:
    def __init__(self):
        self.version = "20240101000000_create_users"

    async def up(self, client: EmbexClient):
        await client.collection("users").create(dimension=384, distance="cosine")

    async def down(self, client: EmbexClient):
        await client.collection("users").delete_collection()

async def main():
    client = EmbexClient("qdrant", "http://localhost:6333")

    migrations = [CreateUsersMigration()]

    # Run migrations
    await client.run_migrations(migrations)

if __name__ == "__main__":
    asyncio.run(main())
```

## Summary

The migration system provides:

- ✅ Sequential, atomic migration execution
- ✅ Automatic rollback on failure
- ✅ Migration validation
- ✅ State consistency checking
- ✅ Flexible rollback options

For more information, see:

- [API Reference](api/rust.md#migrations) - Migration API details
- [Best Practices](best_practices.md) - General best practices
