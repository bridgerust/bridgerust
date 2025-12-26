# Migrating from LanceDB to Kabod

## Initialization

**LanceDB:**

```python
import lancedb
db = lancedb.connect("./data")
```

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(provider="lancedb", url="./data")
```

## Creating Table

**LanceDB:**

```python
data = [{"vector": [1.1, 1.2], "item": "foo"}]
tbl = db.create_table("my_table", data)
```

**Kabod:**

```python
await client.collection("my_table").create(dimension=2, distance="euclidean")
```

## Search

**LanceDB:**

```python
tbl.search([1.1, 1.2]).limit(5).to_df()
```

**Kabod:**

```python
results = await client.collection("my_table").search(
    vector=[1.1, 1.2],
    top_k=5
)
```
