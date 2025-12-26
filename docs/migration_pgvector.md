# Migrating from PgVector to Kabod

Kabod abstract away the SQL complexity of PgVector.

## Setup

**PgVector (SQL/Python):**

```python
import psycopg2
conn = psycopg2.connect("postgresql://user:password@localhost:5432/dbname")
cur = conn.cursor()
cur.execute("CREATE EXTENSION IF NOT EXISTS vector")
cur.execute("CREATE TABLE items (id bigserial PRIMARY KEY, embedding vector(3))")
```

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(
    provider="pgvector",
    url="postgresql://user:password@localhost:5432/dbname"
)

# Kabod manages the table creation for you
await client.collection("items").create(dimension=3, distance="cosine")
```

## Insertion

**PgVector:**

```python
cur.execute("INSERT INTO items (embedding) VALUES (%s)", ([1,2,3],))
```

**Kabod:**

```python
from kabod import Point

await client.collection("items").insert([
    Point(id="1", vector=[1,2,3], metadata={})
])
```

## Search

**PgVector:**

```python
cur.execute("SELECT * FROM items ORDER BY embedding <-> %s LIMIT 5", ([1,2,3],))
```

**Kabod:**

```python
results = await client.collection("items").search(vector=[1,2,3], top_k=5)
```
