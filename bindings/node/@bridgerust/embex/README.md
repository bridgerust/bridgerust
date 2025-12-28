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

**Try Embex in 30 seconds - No setup required!** Uses LanceDB embedded mode (no server needed).

```typescript
import { EmbexClient } from "@bridgerust/embex";

async function main() {
  // LanceDB embedded - zero setup, just a local path
  const client = await EmbexClient.newAsync("lancedb", "./data");
  const collection = client.collection("documents");

  // Create collection
  await collection.create(768, "cosine");

  // Insert data
  await collection.insert([
    {
      id: "1",
      vector: Array(768).fill(0.1),
      metadata: { text: "Hello World" },
    },
  ]);

  // Search
  const results = await collection.search(Array(768).fill(0.1), 5);
  console.log(results.results);
}

main();
```

**Run it:** `npx tsx examples/lancedb/node/quickstart.ts`

### All Provider Quick Starts

Try Embex with any provider! Same API, different backend:

| Provider     | Setup           | Quick Start                                    |
| ------------ | --------------- | ---------------------------------------------- |
| **LanceDB**  | None (embedded) | `npx tsx examples/lancedb/node/quickstart.ts`  |
| **Qdrant**   | Docker server   | `npx tsx examples/qdrant/node/quickstart.ts`   |
| **Pinecone** | API key         | `npx tsx examples/pinecone/node/quickstart.ts` |
| **Chroma**   | Optional server | `npx tsx examples/chroma/node/quickstart.ts`   |

> 💡 **Same API everywhere!** Just change the provider name - all code stays the same. See [examples/README.md](../../../../examples/README.md) for setup instructions.

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
