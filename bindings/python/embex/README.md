# Embex (Python)

**The Universal Vector Database ORM.** One API for Qdrant, Pinecone, Chroma, LanceDB, and more.

Embex is a high-performance, universal client for vector databases, built on a shared Rust core related to [BridgeRust](https://github.com/bridgerust/bridgerust).

## 🚀 Features

- **Unified API**: Switch providers instantly. "Write once, run anywhere."
- **Performance**: Powered by Rust with SIMD acceleration.
- **Type Safety**: Fully typed Python bindings.

## 📦 Installation

```bash
uv pip install embex
```

```bash
pip install embex
```

## ⚡ Quick Start

### 1. Connect to a Provider

```python
from embex import EmbexClient

# Connect to Qdrant
client = EmbexClient("qdrant", "http://localhost:6333")

# Or use async initialization (required for some providers like LanceDB/Milvus)
# Note: Python client handles async init internally via await if needed,
# or use the async factory if exposed.
# For standard usage, EmbexClient constructor handles most sync/async bridging.
```

### 2. Create a Collection

```python
collection = client.collection("my_collection")

# Create with specific dimension and metric
collection.create(768, "cosine")
```

### 3. Insert Vectors

```python
collection.insert([
  {
    "id": "1",
    "vector": [0.1, 0.2, ...], # 768 dimensions
    "metadata": {"title": "Hello World", "category": "greeting"}
  }
])
```

### 4. Search

```python
results = collection.search(
  vector=[0.1, 0.2, ...], # Query vector
  limit=5
)

for result in results.results:
    print(result.id, result.score, result.metadata)
```

### 5. Filtered Search (Builder Pattern)

```python
# Coming soon: Python Builder Pattern
# Currently supported via search() arguments:

results = collection.search(
    vector=[0.1, 0.2, ...],
    limit=10,
    filter={"course": "CS101"}
)
```

## 🔌 Supported Providers

| Provider | Key        | Status    |
| -------- | ---------- | --------- |
| Qdrant   | `qdrant`   | Supported |
| Chroma   | `chroma`   | Supported |
| Pinecone | `pinecone` | Supported |
| Weaviate | `weaviate` | Supported |
| LanceDB  | `lancedb`  | Supported |
| Milvus   | `milvus`   | Supported |
| PgVector | `pgvector` | Supported |

## 🔗 Resources

- **Main Repository**: [github.com/bridgerust/bridgerust](https://github.com/bridgerust/bridgerust)
- **Issues**: [github.com/bridgerust/bridgerust/issues](https://github.com/bridgerust/bridgerust/issues)
- **Documentation**: [Full Docs](https://github.com/bridgerust/bridgerust#documentation)
