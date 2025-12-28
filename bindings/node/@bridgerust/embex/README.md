# Embex (Node.js)

**The Universal Vector Database ORM.** One API for Qdrant, Pinecone, Chroma, LanceDB, and more.

Embex is a high-performance, universal client for vector databases, built on a shared Rust core related to [BridgeRust](https://github.com/bridgerust/bridgerust).

## 🚀 Features

- **Unified API**: Switch providers instantly. "Write once, run anywhere."
- **Performance**: Powered by Rust with SIMD acceleration.
- **Type Safety**: Full TypeScript support.

## 📦 Installation

```bash
npm install embex
```

```bash
yarn add embex
```

```bash
bun add embex
```

## ⚡ Quick Start

### 1. Connect to a Provider

```typescript
import { EmbexClient } from "embex";

// Connect to Qdrant
const client = new EmbexClient("qdrant", "http://localhost:6333");

// Or use async initialization (required for some providers like LanceDB/Milvus)
const client = await EmbexClient.newAsync("lancedb", "./data/lancedb");
```

### 2. Create a Collection

```typescript
const collection = client.collection("my_collection");

// Create with specific dimension and metric
await collection.create(768, "cosine");
```

### 3. Insert Vectors

```typescript
await collection.insert([
  {
    id: "1",
    vector: [0.1, 0.2, ...], // 768 dimensions
    metadata: { title: "Hello World", category: "greeting" }
  }
]);
```

### 4. Search

```typescript
const results = await collection.search(
  [0.1, 0.2, ...], // Query vector
  5                // Limit
);

console.log(results.results);
```

### 5. Filtered Search (Builder Pattern)

```typescript
const results = await collection.buildSearch([0.1, 0.2, ...])
  .limit(10)
  .filter({
    course: "CS101"
  })
  .execute();
```

## 🔌 Supported Providers

| Provider | Key        | Async Init? |
| -------- | ---------- | ----------- |
| Qdrant   | `qdrant`   | No          |
| Chroma   | `chroma`   | No          |
| Pinecone | `pinecone` | No          |
| Weaviate | `weaviate` | No          |
| LanceDB  | `lancedb`  | **Yes**     |
| Milvus   | `milvus`   | **Yes**     |
| PgVector | `pgvector` | **Yes**     |

## 🔗 Resources

- **Main Repository**: [github.com/bridgerust/bridgerust](https://github.com/bridgerust/bridgerust)
- **Issues**: [github.com/bridgerust/bridgerust/issues](https://github.com/bridgerust/bridgerust/issues)
- **Documentation**: [Full Docs](https://github.com/bridgerust/bridgerust#documentation)
