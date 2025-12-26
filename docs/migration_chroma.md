# Migrating from Chroma to Kabod

Kabod allows you to switch from Chroma's embedded or client mode to a unified interface.

## Initialization

**Chroma:**

```python
import chromadb
client = chromadb.HttpClient(host='localhost', port=8000)
# OR
client = chromadb.PersistentClient(path="./my_db")
```

**Kabod:**

```python
from kabod import KabodClient

# For client mode
client = KabodClient(provider="chroma", url="http://localhost:8000")
```

## Creating Collection

**Chroma:**

```python
collection = client.create_collection(name="my_collection")
```

**Kabod:**

```python
await client.collection("my_collection").create(
    dimension=768,
    distance="l2" # Chroma defaults to l2
)
```

## Inserting Documents

**Chroma:**

```python
collection.add(
    documents=["doc1", ...],
    metadatas=[{"source": "notion"}, ...],
    ids=["id1", ...]
    # embeddings optional if using built-in embedding function
)
```

**Kabod:**
Kabod focuses on the vectors themselves, assuming you have an embedding model.

```python
from kabod import Point

await client.collection("my_collection").insert([
    Point(
        id="id1",
        vector=[0.1, ...], # Must provide vector
        metadata={"source": "notion"}
    )
])
```
