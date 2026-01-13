# Embex (Python)

**The fastest way to add vector search to your app.**

Embex is a universal vector database client that lets you start with zero setup and scale to production without rewriting code.

## 🚀 Features

- **Start Simple**: Use LanceDB (embedded) for zero-setup local development.
- **Unified API**: Switch to Qdrant, Pinecone, or Milvus just by changing the config.
- **Performance**: Powered by a shared Rust core with SIMD acceleration.
- **Type Safety**: Fully typed Python bindings.

## 📦 Installation

```bash
pip install embex lancedb sentence-transformers
```

## ⚡ Quick Start

Build semantic search in 5 minutes using **LanceDB** (embedded) and local embeddings. No API keys or Docker needed!

```python
import asyncio
from embex import EmbexClient, Vector
from sentence_transformers import SentenceTransformer

async def main():
    # 1. Setup Embedding Model
    model = SentenceTransformer('all-MiniLM-L6-v2')

    # 2. Initialize Client (uses LanceDB embedded)
    client = await EmbexClient.new_async(provider="lancedb", url="./data")

    # 3. Create Collection (384 dimensions for MiniLM)
    await client.create_collection("products", dimension=384)

    # 4. Insert Data
    documents = [
        {"id": "1", "text": "Apple iPhone 15", "category": "electronics"},
        {"id": "2", "text": "Samsung Galaxy S24", "category": "electronics"},
    ]

    vectors = []
    for doc in documents:
        vectors.append(Vector(
            id=doc["id"],
            vector=model.encode(doc["text"]).tolist(),
            metadata={"text": doc["text"]}
        ))

    await client.insert("products", vectors)

    # 5. Search
    query = "smartphone"
    results = await client.search(
        collection_name="products",
        vector=model.encode(query).tolist(),
        limit=1
    )

    print(f"Query: '{query}'")
    print(f"Match: {results[0].metadata['text']}")

if __name__ == "__main__":
    asyncio.run(main())
```

## 🗺️ Development → Production Roadmap

| Stage               | Recommendation        | Why?                                |
| :------------------ | :-------------------- | :---------------------------------- |
| **Day 1: Learning** | **LanceDB**           | Embedded. Zero setup. Free.         |
| **Week 2: Staging** | **Qdrant / Pinecone** | Managed cloud. Connection pooling.  |
| **Month 1: Scale**  | **Milvus**            | Distributed. Billion-scale vectors. |
| **Anytime**         | **PgVector**          | You already use PostgreSQL.         |

## ☁️ Switch Provider (Zero Code Changes)

Ready for production? Just change the initialization line.

**From LanceDB (Dev):**

```python
client = await EmbexClient.new_async(provider="lancedb", url="./data")
```

**To Qdrant Cloud (Prod):**

```python
client = EmbexClient(
    provider="qdrant",
    url="https://your-cluster.qdrant.io",
    api_key="..."
)
```

## 🔄 Data Migration

Move data between providers effortlessly using the built-in `DataMigrator`.

```python
from embex import DataMigrator, EmbexClient

# 1. Setup clients
source = await EmbexClient.new_async("lancedb", "./local_data")
dest = EmbexClient("qdrant", "http://prod-db:6333")

# 2. Migrate
migrator = DataMigrator(source, dest)
result = await migrator.migrate_simple(
    source_collection="products",
    dest_collection="products_v2"
)

print(f"Migrated {result.points_migrated} points!")
```

## 🔗 Resources

- **Full Documentation**: [bridgerust.dev/embex](https://bridgerust.dev/embex/introduction)
- **GitHub**: [bridgerust/bridgerust](https://github.com/bridgerust/bridgerust)
