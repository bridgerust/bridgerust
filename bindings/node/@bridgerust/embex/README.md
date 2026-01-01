# Embex (Node.js)

**The Universal Vector Database ORM.** One API for Qdrant, Pinecone, Chroma, LanceDB, and more.

Embex is a high-performance, universal client for vector databases, built on a shared Rust core related to [BridgeRust](https://github.com/bridgerust/bridgerust).

## 🚀 Features

- **Unified API**: Switch providers instantly. "Write once, run anywhere."
- **Performance**: Powered by Rust with SIMD acceleration.
- **Type Safety**: Full TypeScript support.

## 📦 Installation

```bash
npm install @bridgerust/embex
```

```bash
yarn add @bridgerust/embex
```

```bash
bun add @bridgerust/embex
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

> 💡 **Same API everywhere!** Just change the provider name - all code stays the same. See [examples/README.md](https://github.com/bridgerust/bridgerust/blob/main/examples/README.md) for setup instructions.

**New to vector databases?** Check out the [Getting Started Guide](https://github.com/bridgerust/bridgerust/blob/main/docs/getting_started.md) for a beginner-friendly introduction with core concepts explained.

### 5. Filtered Search (Builder Pattern)

```typescript
const results = await collection.buildSearch([0.1, 0.2, ...])
  .limit(10)
  .filter({
    course: "CS101"
  })
  .execute();
```

## ☁️ Connecting to Cloud Providers

To connect to managed services like Pinecone, Qdrant Cloud, or Zilliz (Milvus), simply provide your API key and endpoint URL.

```typescript
import { EmbexClient } from "@bridgerust/embex";

// Connect to Pinecone
const client = new EmbexClient(
  "pinecone",
  "https://index-name.svc.pinecone.io",
  process.env.PINECONE_API_KEY
);

// Connect to Qdrant Cloud
const qdrantClient = new EmbexClient(
  "qdrant",
  "https://xyz-example.eu-central.aws.cloud.qdrant.io:6333",
  process.env.QDRANT_API_KEY
);
```

### Official Documentation & API Keys

Need help finding your API key? Check the official provider documentation:

- **Pinecone**: [Authentication & API Keys](https://docs.pinecone.io/guides/get-started/quickstart#2-get-an-api-key)
- **Qdrant**: [Cloud Authentication](https://qdrant.tech/documentation/cloud/authentication/)
- **Milvus (Zilliz)**: [Manage Credentials](https://docs.zilliz.com/docs/manage-api-keys)
- **Weaviate**: [Authentication](https://weaviate.io/developers/weaviate/configuration/authentication)
- **Chroma**: [Auth & Client Settings](https://docs.trychroma.com/guides#authentication)

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

## ⭐ Star Us

If you find Embex useful, please star the repository! It helps others discover the project.

[⭐ Star on GitHub](https://github.com/bridgerust/bridgerust)

## 🔗 Resources

- **Getting Started**: [Complete Guide](https://github.com/bridgerust/bridgerust/blob/main/docs/getting_started.md) - Beginner-friendly tutorial with core concepts
- **Main Repository**: [github.com/bridgerust/bridgerust](https://github.com/bridgerust/bridgerust)
- **Issues**: [github.com/bridgerust/bridgerust/issues](https://github.com/bridgerust/bridgerust/issues)
- **Documentation**: [Full Docs](https://github.com/bridgerust/bridgerust/tree/main/bindings/node/%40bridgerust/embex)
