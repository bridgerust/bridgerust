# Migrating from LanceDB to Embex

Embex provides a unified interface for LanceDB, abstracting away the embedded database details. This guide helps you migrate from LanceDB to Embex.

## Table of Contents

- [Initialization](#initialization)
- [Creating Tables](#creating-tables)
- [Inserting Data](#inserting-data)
- [Search](#search)
- [Filters](#filters)
- [Batch Operations](#batch-operations)
- [Error Handling](#error-handling)
- [Key Differences](#key-differences)
- [Troubleshooting](#troubleshooting)

## Initialization

### Python

**LanceDB:**

```python
import lancedb

db = lancedb.connect("./data")
```

**Embex:**

```python
from embex import EmbexClient

client = await EmbexClient.new_async(
    provider="lancedb",
    url="./data"
)
```

### Node.js

**LanceDB:**

```typescript
import { connect } from "vectordb";

const db = await connect("./data");
```

**Embex:**

```typescript
import { EmbexClient } from "@bridgerust/embex";

const client = await EmbexClient.newAsync("lancedb", "./data");
```

## Creating Tables

### Python

**LanceDB:**

```python
data = [
    {"vector": [1.1, 1.2], "item": "foo", "price": 10.0},
    {"vector": [2.1, 2.2], "item": "bar", "price": 20.0}
]
tbl = db.create_table("my_table", data)
```

**Embex:**

```python
await client.collection("my_table").create(
    dimension=2,
    distance="euclidean"
)
```

**Key Difference**: LanceDB creates tables with initial data, Embex creates empty collections.

### Node.js

**LanceDB:**

```typescript
const data = [
  { vector: [1.1, 1.2], item: "foo", price: 10.0 },
  { vector: [2.1, 2.2], item: "bar", price: 20.0 },
];
const tbl = await db.createTable("my_table", data);
```

**Embex:**

```typescript
await client.collection("my_table").create(2, "euclidean");
```

## Inserting Data

### Python

**LanceDB:**

```python
tbl.add([
    {"vector": [3.1, 3.2], "item": "baz", "price": 30.0}
])
```

**Embex:**

```python
from embex import Point

await client.collection("my_table").insert([
    Point(
        id="1",
        vector=[3.1, 3.2],
        metadata={"item": "baz", "price": 30.0}
    )
])
```

**Key Difference**: LanceDB uses dicts with vector as a key, Embex uses Point objects with metadata.

### Node.js

**LanceDB:**

```typescript
await tbl.add([{ vector: [3.1, 3.2], item: "baz", price: 30.0 }]);
```

**Embex:**

```typescript
await client.collection("my_table").insert([
  {
    id: "1",
    vector: [3.1, 3.2],
    metadata: { item: "baz", price: 30.0 },
  },
]);
```

## Search

### Python

**LanceDB:**

```python
results = tbl.search([1.1, 1.2]).limit(5).to_pandas()
# OR
results = tbl.search([1.1, 1.2]).limit(5).to_arrow()
```

**Embex:**

```python
results = await client.collection("my_table").search(
    vector=[1.1, 1.2],
    top_k=5
)
```

**Key Difference**: LanceDB returns pandas/arrow, Embex returns structured results.

### Node.js

**LanceDB:**

```typescript
const results = await tbl.search([1.1, 1.2]).limit(5).toArray();
```

**Embex:**

```typescript
const results = await client.collection("my_table").search([1.1, 1.2], 5);
```

## Filters

### Python

**LanceDB:**

```python
results = tbl.search([1.1, 1.2]).where("price > 15").limit(5).to_pandas()
```

**Embex:**

```python
results = await client.collection("my_table").search(
    vector=[1.1, 1.2],
    top_k=5,
    filter={
        "op": "key",
        "args": ["price", {"op": "gt", "args": 15}]
    }
)
```

Or using the query builder:

```python
builder = client.collection("my_table").build_search([1.1, 1.2])
results = await builder.filter({
    "op": "key",
    "args": ["price", {"op": "gt", "args": 15}]
}).limit(5).execute()
```

### Node.js

**LanceDB:**

```typescript
const results = await tbl
  .search([1.1, 1.2])
  .where("price > 15")
  .limit(5)
  .toArray();
```

**Embex:**

```typescript
const results = await client.collection("my_table").search([1.1, 1.2], 5, {
  filter: {
    op: "key",
    args: ["price", { op: "gt", args: 15 }],
  },
});
```

## Batch Operations

### Python

**LanceDB:**

```python
# LanceDB handles batching internally
tbl.add([...])  # Large list
```

**Embex:**

```python
# Explicit batch with parallel execution
await client.collection("my_table").insert_batch(
    points=[...],  # Large list
    batch_size=100,
    parallel=True
)
```

## Error Handling

### Python

**LanceDB:**

```python
try:
    tbl.add(...)
except Exception as e:
    print(f"Error: {e}")
```

**Embex:**

```python
from embex import EmbexError

try:
    await client.collection("my_table").insert(...)
except EmbexError as e:
    print(f"Error: {e}")
```

## Key Differences

1. **Table Creation**: LanceDB creates with initial data, Embex creates empty collections
2. **Data Format**: LanceDB uses dicts with `vector` key, Embex uses Point objects
3. **Return Format**: LanceDB returns pandas/arrow, Embex returns structured results
4. **Filter Syntax**: LanceDB uses SQL-like strings, Embex uses unified filter format
5. **Async**: LanceDB Python is sync, Embex is async

## Troubleshooting

### Issue: Table creation with data

**Problem**: LanceDB creates tables with initial data, Embex creates empty collections.

**Solution**: Create collection first, then insert data:

```python
# Before (LanceDB)
tbl = db.create_table("my_table", data)

# After (Embex)
await client.collection("my_table").create(dimension=2, distance="euclidean")
await client.collection("my_table").insert([...])  # Insert data separately
```

### Issue: Vector key vs metadata

**Problem**: LanceDB uses `vector` as a dict key, Embex uses `vector` as a Point field.

**Solution**: Restructure data:

```python
# Before (LanceDB)
{"vector": [1.1, 1.2], "item": "foo"}

# After (Embex)
Point(id="1", vector=[1.1, 1.2], metadata={"item": "foo"})
```

### Issue: Sync vs Async

**Problem**: LanceDB Python is sync, Embex is async.

**Solution**: Use `await` for all Embex operations:

```python
# Before (LanceDB)
tbl = db.create_table("my_table", data)

# After (Embex)
await client.collection("my_table").create(dimension=2, distance="euclidean")
```

## Next Steps

- [API Documentation](api/python.md) - Complete Python API reference
- [Getting Started](getting_started.md) - Quick start guide
- [Best Practices](best_practices.md) - Production patterns
