# Migrating from PgVector to Kabod

Kabod abstracts away the SQL complexity of PgVector. This guide helps you migrate from PgVector to Kabod.

## Table of Contents

- [Setup](#setup)
- [Creating Tables](#creating-tables)
- [Insertion](#insertion)
- [Search](#search)
- [Filters](#filters)
- [Batch Operations](#batch-operations)
- [Error Handling](#error-handling)
- [Connection Pooling](#connection-pooling)
- [Key Differences](#key-differences)
- [Troubleshooting](#troubleshooting)

## Setup

### Python

**PgVector (SQL/Python):**

```python
import psycopg2

conn = psycopg2.connect("postgresql://user:password@localhost:5432/dbname")
cur = conn.cursor()
cur.execute("CREATE EXTENSION IF NOT EXISTS vector")
```

**Kabod:**

```python
from kabod import KabodClient

client = await KabodClient.new_async(
    provider="pgvector",
    url="postgresql://user:password@localhost:5432/dbname"
)
```

### Node.js

**PgVector:**

```typescript
import { Pool } from "pg";

const pool = new Pool({
  connectionString: "postgresql://user:password@localhost:5432/dbname",
});
await pool.query("CREATE EXTENSION IF NOT EXISTS vector");
```

**Kabod:**

```typescript
import { KabodClient } from "@bridgerust/kabod";

const client = await KabodClient.newAsync(
  "pgvector",
  "postgresql://user:password@localhost:5432/dbname"
);
```

## Creating Tables

### Python

**PgVector:**

```python
cur.execute("""
    CREATE TABLE items (
        id bigserial PRIMARY KEY,
        embedding vector(768)
    )
""")
```

**Kabod:**

```python
# Kabod manages the table creation for you
await client.collection("items").create(dimension=768, distance="cosine")
```

**Key Difference**: Kabod automatically creates the table with proper vector column and indexes.

### Node.js

**PgVector:**

```typescript
await pool.query(`
    CREATE TABLE items (
        id bigserial PRIMARY KEY,
        embedding vector(768)
    )
`);
```

**Kabod:**

```typescript
await client.collection("items").create(768, "cosine");
```

## Insertion

### Python

**PgVector:**

```python
cur.execute(
    "INSERT INTO items (embedding) VALUES (%s)",
    ([0.1] * 768,)
)
conn.commit()
```

**Kabod:**

```python
from kabod import Point

await client.collection("items").insert([
    Point(id="1", vector=[0.1] * 768, metadata={})
])
```

**Key Difference**: Kabod handles transactions and provides structured Point objects.

### Node.js

**PgVector:**

```typescript
await pool.query("INSERT INTO items (embedding) VALUES ($1)", [
  [0.1, 0.2, 0.3],
]);
```

**Kabod:**

```typescript
await client.collection("items").insert([
  {
    id: "1",
    vector: Array(768).fill(0.1),
    metadata: {},
  },
]);
```

## Search

### Python

**PgVector:**

```python
cur.execute("""
    SELECT *, embedding <-> %s AS distance
    FROM items
    ORDER BY embedding <-> %s
    LIMIT 5
""", ([0.1] * 768, [0.1] * 768))
results = cur.fetchall()
```

**Kabod:**

```python
results = await client.collection("items").search(
    vector=[0.1] * 768,
    top_k=5
)
```

**Key Difference**: Kabod handles SQL complexity and distance calculations automatically.

### Node.js

**PgVector:**

```typescript
const results = await pool.query(
  `
    SELECT *, embedding <-> $1 AS distance
    FROM items
    ORDER BY embedding <-> $1
    LIMIT 5
`,
  [[0.1, 0.2, 0.3]]
);
```

**Kabod:**

```typescript
const results = await client
  .collection("items")
  .search(Array(768).fill(0.1), 5);
```

## Filters

### Python

**PgVector:**

```python
cur.execute("""
    SELECT *, embedding <-> %s AS distance
    FROM items
    WHERE category = %s
    ORDER BY embedding <-> %s
    LIMIT 5
""", ([0.1] * 768, "tech", [0.1] * 768))
```

**Kabod:**

```python
results = await client.collection("items").search(
    vector=[0.1] * 768,
    top_k=5,
    filter={
        "op": "key",
        "args": ["category", {"op": "eq", "args": "tech"}]
    }
)
```

Or using the query builder:

```python
builder = client.collection("items").build_search([0.1] * 768)
results = await builder.filter({
    "op": "key",
    "args": ["category", {"op": "eq", "args": "tech"}]
}).limit(5).execute()
```

### Node.js

**PgVector:**

```typescript
const results = await pool.query(
  `
    SELECT *, embedding <-> $1 AS distance
    FROM items
    WHERE category = $2
    ORDER BY embedding <-> $1
    LIMIT 5
`,
  [[0.1, 0.2, 0.3], "tech"]
);
```

**Kabod:**

```typescript
const results = await client
  .collection("items")
  .search(Array(768).fill(0.1), 5, {
    filter: {
      op: "key",
      args: ["category", { op: "eq", args: "tech" }],
    },
  });
```

## Batch Operations

### Python

**PgVector:**

```python
# Manual batching
for i in range(0, len(vectors), 100):
    batch = vectors[i:i+100]
    cur.executemany(
        "INSERT INTO items (embedding) VALUES (%s)",
        [(v,) for v in batch]
    )
conn.commit()
```

**Kabod:**

```python
# Explicit batch with parallel execution
await client.collection("items").insert_batch(
    points=[...],  # Large list
    batch_size=100,
    parallel=True
)
```

## Error Handling

### Python

**PgVector:**

```python
import psycopg2

try:
    cur.execute(...)
except psycopg2.Error as e:
    print(f"Error: {e}")
```

**Kabod:**

```python
from kabod import KabodError

try:
    await client.collection("items").insert(...)
except KabodError as e:
    print(f"Error: {e}")
```

## Connection Pooling

**PgVector:**

```python
from psycopg2 import pool

connection_pool = psycopg2.pool.SimpleConnectionPool(
    1, 20,  # min, max connections
    "postgresql://..."
)
```

**Kabod:**

```python
from kabod import KabodClient

client = await KabodClient.new_async(
    provider="pgvector",
    url="postgresql://user:password@localhost:5432/dbname",
    pool_size=20,  # Max connections
    idle_timeout_secs=90
)
```

**Key Difference**: Kabod manages connection pooling automatically via `sqlx`.

## Key Differences

1. **SQL Abstraction**: Kabod hides SQL complexity, PgVector requires SQL knowledge
2. **Table Management**: Kabod creates tables automatically, PgVector requires manual DDL
3. **Transactions**: Kabod handles transactions, PgVector requires manual commit/rollback
4. **Connection Pooling**: Kabod manages pooling, PgVector requires manual pool setup
5. **Unified API**: Kabod provides the same API across all providers

## Troubleshooting

### Issue: Extension not created

**Problem**: PgVector extension must exist before use.

**Solution**: Kabod handles this automatically, but ensure PostgreSQL has the extension available:

```sql
CREATE EXTENSION IF NOT EXISTS vector;
```

### Issue: Table already exists

**Problem**: PgVector tables are created manually, Kabod creates them automatically.

**Solution**: Either drop existing tables or use different collection names:

```python
# Drop existing table first
# OR use a new collection name
await client.collection("items_v2").create(dimension=768, distance="cosine")
```

### Issue: SQL syntax errors

**Problem**: PgVector requires SQL knowledge, Kabod abstracts this away.

**Solution**: Use Kabod's unified API instead of SQL:

```python
# Before (PgVector)
cur.execute("SELECT * FROM items WHERE category = %s", ("tech",))

# After (Kabod)
results = await collection.search(
    vector=...,
    filter={"op": "key", "args": ["category", {"op": "eq", "args": "tech"}]}
)
```

## Next Steps

- [API Documentation](api/python.md) - Complete Python API reference
- [Getting Started](getting_started.md) - Quick start guide
- [Best Practices](best_practices.md) - Production patterns
