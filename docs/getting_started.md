# Getting Started with Embex

**Embex** is the fastest way to add vector search to your application. It provides a unified API that works across multiple vector databases, allowing you to start simple and scale later without rewriting code.

## The Recommended Path

| Stage               | Recommendation        | Why?                                |
| :------------------ | :-------------------- | :---------------------------------- |
| **Day 1: Learning** | **LanceDB**           | Embedded. Zero setup. Free.         |
| **Week 2: Staging** | **Qdrant / Pinecone** | Managed cloud. Connection pooling.  |
| **Month 1: Scale**  | **Milvus**            | Distributed. Billion-scale vectors. |
| **Anytime**         | **PgVector**          | You already use PostgreSQL.         |

## Prerequisites

- **Python 3.9+** OR **Node.js 18+** OR **Rust 1.92+**
- **No prior vector database experience required**

## Installation

### Python

```bash
pip install embex lancedb sentence-transformers
```

### Node.js

```bash
npm install @bridgerust/embex lancedb @xenova/transformers
```

## Quick Start: Semantic Search in 5 Minutes

We'll use **LanceDB** (embedded) and a local embedding model. No API keys or servers needed.

### Python Example

```python
import asyncio
from embex import EmbexClient, Vector
from sentence_transformers import SentenceTransformer

async def main():
    # 1. Setup Embedding Model
    # 'all-MiniLM-L6-v2' is small, fast, and local
    model = SentenceTransformer('all-MiniLM-L6-v2')

    # 2. Initialize Client (uses LanceDB just by specifying a path)
    client = await EmbexClient.new_async("lancedb://./data")

    # 3. Create Collection (384 dims for MiniLM)
    await client.create_collection("products", dimension=384)

    # 4. Insert Data
    documents = [
        {"id": "1", "text": "Apple iPhone 15", "category": "electronics"},
        {"id": "2", "text": "Samsung Galaxy S24", "category": "electronics"},
        {"id": "3", "text": "Fresh Organic Bananas", "category": "groceries"},
    ]

    vectors = []
    for doc in documents:
        # Generate real embedding
        embedding = model.encode(doc["text"]).tolist()
        vectors.append(Vector(
            id=doc["id"],
            vector=embedding,
            metadata={"text": doc["text"], "category": doc["category"]}
        ))

    await client.insert("products", vectors)

    # 5. Search
    query = "smartphone"
    query_vector = model.encode(query).tolist()

    results = await client.search(
        collection_name="products",
        vector=query_vector,
        limit=1
    )

    print(f"Query: '{query}'")
    print(f"Match: {results[0].metadata['text']}")

if __name__ == "__main__":
    asyncio.run(main())
```

### Node.js Example

```typescript
import { EmbexClient, Vector } from "@bridgerust/embex";
import { pipeline } from "@xenova/transformers";

async function main() {
  // 1. Setup Embedding Model
  const generateEmbedding = await pipeline(
    "feature-extraction",
    "Xenova/all-MiniLM-L6-v2"
  );
  const embed = async (text: string) => {
    const output = await generateEmbedding(text, {
      pooling: "mean",
      normalize: true,
    });
    return Array.from(output.data);
  };

  // 2. Initialize Client (LanceDB)
  const client = await EmbexClient.newAsync("lancedb://./data");

  // 3. Create Collection
  await client.createCollection("products", 384);

  // 4. Insert Data
  const documents = [
    { id: "1", text: "Apple iPhone 15", category: "electronics" },
    { id: "2", text: "Samsung Galaxy S24", category: "electronics" },
    { id: "3", text: "Fresh Organic Bananas", category: "groceries" },
  ];

  const vectors: Vector[] = [];
  for (const doc of documents) {
    vectors.push({
      id: doc.id,
      vector: await embed(doc.text),
      metadata: { text: doc.text, category: doc.category },
    });
  }

  await client.insert("products", vectors);

  // 5. Search
  const query = "smartphone";
  const queryVector = await embed(query);

  const results = await client.search({
    collection_name: "products",
    vector: queryVector,
    limit: 1,
  });

  console.log(`Query: '${query}'`);
  console.log(`Match: ${results[0].metadata.text}`);
}

main();
```

## Going to Production

When you're ready to scale, you can switch providers without rewriting your application logic.

### Switching from LanceDB to Qdrant

**Before (Development):**

```python
client = await EmbexClient.new_async("lancedb://./data")
```

**After (Production):**

```python
client = EmbexClient(provider="qdrant", url="https://your-qdrant-cluster.com", api_key="...")
```

Everything else (`insert`, `search`, `create_collection`) remains exactly the same.

## Core Concepts

- **Embedding**: A list of numbers (vector) representing data.
- **Collection**: A group of vectors (like a SQL table).
- **Provider**: The underlying database (LanceDB, Qdrant, etc.).

## Next Steps

- [Full API Documentation](api/README.md)
- [Provider Comparison](migration_qdrant.md)
