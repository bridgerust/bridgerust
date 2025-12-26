# Getting Started with Kabod

Kabod is a high-performance vector database ORM for Rust, Python, and Node.js. It provides a unified API to interact with multiple vector backend providers.

## Installation

### Python

```bash
pip install kabod-py
# OR
uv pip install kabod-py
```

### Node.js

```bash
npm install @bridgerust/kabod
# OR
bun add @bridgerust/kabod
```

### Rust

Add to `Cargo.toml`:

```toml
[dependencies]
bridge-kabod = { version = "0.1", features = ["qdrant"] } # Enable your provider
tokio = { version = "1", features = ["full"] }
```

## First Steps (Python Example)

1. **Initialize Client**:
   Connect to your preferred backend (e.g., Qdrant).

   ```python
   from kabod import KabodClient

   client = KabodClient(provider="qdrant", url="http://localhost:6333")
   ```

2. **Create Collection**:
   Define the structure of your vector data.

   ```python
   collection = client.collection("my_docs")
   await collection.create(dimension=768, distance="cosine")
   ```

3. **Insert Data**:
   Insert points with vectors and metadata.

   ```python
   from kabod import Point

   await collection.insert([
       Point(id="1", vector=[0.1, ...], metadata={"title": "Hello World"})
   ])
   ```

4. **Search**:
   Find similar vectors.
   ```python
   results = await collection.search(vector=[0.1, ...], top_k=5)
   for r in results.results:
       print(r.score, r.metadata)
   ```

## Supported Providers

- Qdrant
- Pinecone
- Chroma
- PgVector
- LanceDB
