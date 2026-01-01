# Examples

## BridgeRust Framework Examples

### [bridgerust-example](bridgerust-example/)

Comprehensive example demonstrating all BridgeRust features:

- Function exports with various types (primitives, Option, Vec, Result)
- Struct exports with methods
- Error handling with `#[bridgerust::error]`
- Complete Python and Node.js usage examples

**Quick Start:**

```bash
cd examples/bridgerust-example
bridge build --all
python python/example.py
node nodejs/example.js
```

See [bridgerust-example/README.md](bridgerust-example/README.md) for details.

---

## Embex Examples

Quick start examples for different vector database providers, organized by provider and language.

## 📁 Directory Structure

```
examples/
├── lancedb/
│   ├── python/quickstart.py
│   └── node/
│       ├── quickstart.ts
│       ├── package.json
│       └── tsconfig.json
├── qdrant/
│   ├── python/quickstart.py
│   └── node/
│       ├── quickstart.ts
│       ├── package.json
│       └── tsconfig.json
├── pinecone/
│   ├── python/quickstart.py
│   └── node/
│       ├── quickstart.ts
│       ├── package.json
│       └── tsconfig.json
├── chroma/
│   ├── python/quickstart.py
│   └── node/
│       ├── quickstart.ts
│       ├── package.json
│       └── tsconfig.json
├── python/
│   └── semantic_search.py
└── node/
    └── rag_system.ts
```

## 🚀 Quick Starts (Zero to Running in 5 Minutes)

> **Note for Node.js examples:** Each provider's `node/` directory has its own `package.json`. Run `npm install` in the provider's `node/` directory before running the example to resolve TypeScript module errors.

### LanceDB (Recommended - Zero Setup)

**No server required!** Perfect for getting started.

- **Python:** `python examples/lancedb/python/quickstart.py`
- **Node.js:**
  ```bash
  cd examples/lancedb/node
  npm install
  npx tsx quickstart.ts
  ```

### Qdrant (Local Server)

**Requires:** Qdrant server running

```bash
# Start Qdrant
docker run -p 6333:6333 qdrant/qdrant

# Run example
python examples/qdrant/python/quickstart.py
# or (Node.js)
cd examples/qdrant/node
npm install
npx tsx quickstart.ts
```

### Pinecone (Serverless)

**Requires:** Pinecone API key

```bash
# Set API key
export PINECONE_API_KEY="your-api-key"
export PINECONE_INDEX_NAME="your-index-name"

# Run example
python examples/pinecone/python/quickstart.py
# or (Node.js)
cd examples/pinecone/node
npm install
npx tsx quickstart.ts
```

### Chroma (Local Server or In-Memory)

**Requires:** Chroma server (optional - can use in-memory)

```bash
# Start Chroma (optional)
docker run -p 8000:8000 chromadb/chroma

# Run example
python examples/chroma/python/quickstart.py
# or (Node.js)
cd examples/chroma/node
npm install
npx tsx quickstart.ts
```

## 📚 Real-World Examples

### Semantic Search

Full-featured semantic search example with filtering and aggregations.

- **Python:** `python examples/python/semantic_search.py`

### RAG System

Complete RAG (Retrieval-Augmented Generation) system example.

- **Node.js:** `npx tsx examples/node/rag_system.ts`

## 🔄 Switching Providers

The beauty of Embex is that **the API is identical across all providers**. To switch providers, just change the initialization:

```python
# Python
# LanceDB (embedded)
client = await EmbexClient.new_async("lancedb", "./data")

# Qdrant (server)
client = EmbexClient("qdrant", "http://localhost:6333")

# Pinecone (serverless)
client = EmbexClient("pinecone", "https://api.pinecone.io", api_key="...")

# Chroma (server)
client = EmbexClient("chroma", "http://localhost:8000")
```

```typescript
// Node.js
// LanceDB (embedded)
const client = await EmbexClient.newAsync("lancedb", "./data");

// Qdrant (server)
const client = new EmbexClient("qdrant", "http://localhost:6333");

// Pinecone (serverless)
const client = new EmbexClient(
  "pinecone",
  "https://api.pinecone.io",
  "api-key"
);

// Chroma (server)
const client = new EmbexClient("chroma", "http://localhost:8000");
```

**Everything else stays the same!** Same API, same code, different backend.

## 📋 Provider Requirements

| Provider     | Setup Required  | Notes                                               |
| ------------ | --------------- | --------------------------------------------------- |
| **LanceDB**  | None            | Embedded, zero setup                                |
| **Qdrant**   | Docker server   | `docker run -p 6333:6333 qdrant/qdrant`             |
| **Pinecone** | API key         | Serverless, cloud-based                             |
| **Chroma**   | Optional server | Can use in-memory mode                              |
| **PgVector** | PostgreSQL      | Requires PostgreSQL with pgvector extension         |
| **Milvus**   | Docker server   | `docker run -p 19530:19530 milvusdb/milvus`         |
| **Weaviate** | Docker server   | `docker run -p 8080:8080 semitechnologies/weaviate` |

## 💡 Tips

1. **Start with LanceDB** - No setup needed, works immediately
2. **Use Qdrant for local development** - Easy Docker setup
3. **Use Pinecone for production** - Serverless, scalable
4. **Test with multiple providers** - Same code, different backends!

## 🐛 Troubleshooting

### "Connection refused" errors

- Make sure the server is running (Qdrant, Chroma, etc.)
- Check the URL/port matches your server configuration

### "API key required" errors

- Set environment variables: `export PINECONE_API_KEY="..."`
- Or pass API key directly in code

### "Collection not found" errors

- Collections are created automatically on first use
- Check provider-specific requirements (some need pre-created indexes)
