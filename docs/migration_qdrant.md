# Migrating from Qdrant Client to Kabod

Kabod provides a higher-level abstraction while maintaining Qdrant's performance usage.

## Initialization

**Qdrant Client:**

```python
from qdrant_client import QdrantClient

client = QdrantClient(url="http://localhost:6333")
```

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(provider="qdrant", url="http://localhost:6333")
```

## Creating a Collection

**Qdrant Client:**

```python
from qdrant_client.http import models

client.create_collection(
    collection_name="my_collection",
    vectors_config=models.VectorParams(
        size=768,
        distance=models.Distance.COSINE
    )
)
```

**Kabod:**

```python
await client.collection("my_collection").create(
    dimension=768,
    distance="cosine"
)
```

## Inserting Points

**Qdrant Client:**

```python
from qdrant_client.http import models

client.upsert(
    collection_name="my_collection",
    points=[
        models.PointStruct(
            id=1,
            vector=[0.1, ...],
            payload={"color": "red"}
        )
    ]
)
```

**Kabod:**

```python
from kabod import Point

await client.collection("my_collection").insert([
    Point(
        id="1",
        vector=[0.1, ...],
        metadata={"color": "red"}
    )
])
```

## Searching

**Qdrant Client:**

```python
results = client.search(
    collection_name="my_collection",
    query_vector=[0.1, ...],
    limit=5
)
```

**Kabod:**

```python
results = await client.collection("my_collection").search(
    vector=[0.1, ...],
    top_k=5
)
```
