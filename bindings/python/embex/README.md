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

**Try Embex in 30 seconds - No setup required!** Uses LanceDB embedded mode (no server needed).

```python
import asyncio
from embex import EmbexClient, Point

async def main():
    # LanceDB embedded - zero setup, just a local path
    client = await EmbexClient.new_async("lancedb", "./data")
    collection = client.collection("documents")

    # Create collection
    await collection.create(dimension=768, distance="cosine")

    # Insert data
    await collection.insert([
        Point(id="1", vector=[0.1] * 768, metadata={"text": "Hello World"})
    ])

    # Search
    results = await collection.search(vector=[0.1] * 768, top_k=5)
    print(results.results)

asyncio.run(main())
```

**Run it:** `python examples/lancedb/python/quickstart.py`

### All Provider Quick Starts

Try Embex with any provider! Same API, different backend:

| Provider     | Setup           | Quick Start                                     |
| ------------ | --------------- | ----------------------------------------------- |
| **LanceDB**  | None (embedded) | `python examples/lancedb/python/quickstart.py`  |
| **Qdrant**   | Docker server   | `python examples/qdrant/python/quickstart.py`   |
| **Pinecone** | API key         | `python examples/pinecone/python/quickstart.py` |
| **Chroma**   | Optional server | `python examples/chroma/python/quickstart.py`   |

> 💡 **Same API everywhere!** Just change the provider name - all code stays the same. See [examples/README.md](../../../examples/README.md) for setup instructions.

### 5. Filtered Search (Builder Pattern)

````python
# Coming soon: Python Builder Pattern
# Currently supported via search() arguments:

```python
results = collection.search(
    vector=[0.1, 0.2, ...],
    limit=10,
    filter={"course": "CS101"}
)
````

## ☁️ Connecting to Cloud Providers

To connect to managed services like Pinecone, Qdrant Cloud, or Zilliz (Milvus), simply provide your API key and endpoint URL.

```python
import os
from embex import EmbexClient

# Connect to Pinecone (or other cloud providers)
client = EmbexClient(
    provider="pinecone",
    url="https://index-name.svc.pinecone.io",
    api_key=os.getenv("PINECONE_API_KEY")
)

# Connect to Qdrant Cloud
client_qdrant = EmbexClient(
    provider="qdrant",
    url="https://xyz-example.eu-central.aws.cloud.qdrant.io:6333",
    api_key=os.getenv("QDRANT_API_KEY")
)
```

### Official Documentation & API Keys

Need help finding your API key? Check the official provider documentation:

- **Pinecone**: [Authentication & API Keys](https://docs.pinecone.io/guides/get-started/quickstart#2-get-an-api-key)
- **Qdrant**: [Cloud Authentication](https://qdrant.tech/documentation/cloud/authentication/)
- **Milvus (Zilliz)**: [Manage Credentials](https://docs.zilliz.com/docs/manage-api-keys)
- **Weaviate**: [Authentication](https://weaviate.io/developers/weaviate/configuration/authentication)
- **Chroma**: [Auth & Client Settings](https://docs.trychroma.com/guides#authentication)

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
