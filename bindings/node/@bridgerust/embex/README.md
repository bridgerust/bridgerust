# Embex (Node.js)

**The fastest way to add vector search to your app.**

Embex is a universal vector database client that lets you start with zero setup and scale to production without rewriting code.

## 🚀 Features

- **Start Simple**: Use LanceDB (embedded) for zero-setup local development.
- **Unified API**: Switch to Qdrant, Pinecone, or Milvus just by changing the config.
- **Performance**: Powered by a shared Rust core with SIMD acceleration.
- **Type Safety**: Full TypeScript support.

## 📦 Installation

```bash
npm install @bridgerust/embex lancedb @xenova/transformers
```

## ⚡ Quick Start

Build semantic search in 5 minutes using **LanceDB** (embedded) and local embeddings. No API keys or Docker needed!

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

  // 2. Initialize Client (uses LanceDB embedded)
  const client = await EmbexClient.newAsync("lancedb://./data");

  // 3. Create Collection (384 dimensions for MiniLM)
  await client.createCollection("products", 384);

  // 4. Insert Data
  const documents = [
    { id: "1", text: "Apple iPhone 15", category: "electronics" },
    { id: "2", text: "Samsung Galaxy S24", category: "electronics" },
  ];

  const vectors: Vector[] = [];
  for (const doc of documents) {
    vectors.push({
      id: doc.id,
      vector: await embed(doc.text),
      metadata: { text: doc.text },
    });
  }

  await client.insert("products", vectors);

  // 5. Search
  const query = "smartphone";
  const results = await client.search({
    collection_name: "products",
    vector: await embed(query),
    limit: 1,
  });

  console.log(`Query: '${query}'`);
  console.log(`Match: ${results[0].metadata.text}`);
}

main();
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

```typescript
const client = await EmbexClient.newAsync("lancedb://./data");
```

**To Qdrant Cloud (Prod):**

```typescript
const client = new EmbexClient(
  "qdrant",
  "https://your-cluster.qdrant.io",
  process.env.QDRANT_API_KEY
);
```

## 🔗 Resources

- **Full Documentation**: [bridgerust.dev/embex](https://bridgerust.dev/embex/introduction)
- **GitHub**: [bridgerust/bridgerust](https://github.com/bridgerust/bridgerust)
