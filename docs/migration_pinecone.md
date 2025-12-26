# Migrating from Pinecone to Kabod

Kabod unifies the Pinecone API with other providers.

## Initialization

**Pinecone:**

```python
from pinecone import Pinecone

pc = Pinecone(api_key="YOUR_API_KEY")
idx = pc.Index("my-index")
```

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(
    provider="pinecone",
    url="https://index-host.pinecone.io",
    api_key="YOUR_API_KEY"
)
collection = client.collection("my-index")
```

## Upserting

**Pinecone:**

```python
idx.upsert(
    vectors=[
        {"id": "A", "values": [0.1, ...], "metadata": {"genre": "drama"}}
    ]
)
```

**Kabod:**

```python
from kabod import Point

await collection.insert([
    Point(
        id="A",
        vector=[0.1, ...],
        metadata={"genre": "drama"}
    )
])
```

## Querying

**Pinecone:**

```python
results = idx.query(
    vector=[0.1, ...],
    top_k=5,
    include_metadata=True
)
```

**Kabod:**

```python
results = await collection.search(
    vector=[0.1, ...],
    top_k=5,
    include_metadata=True
)
```
