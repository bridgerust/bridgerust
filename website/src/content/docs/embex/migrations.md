---
title: Migrations
description: See how easy it is to switch to Embex.
---

## Real World Example: Sarah's RAG App

Sarah built a RAG chatbot with Pinecone. 6 months later, traffic spiked and costs hit $500/mo. She needed to switch to a cheaper option like Qdrant or LanceDB (self-hosted).

**The Problem**: Switching meant rewriting her data ingestion layer and search logic.
**The Solution**: She switched to Embex.

### Before (Native Client)

Heavy vendor lock-in specific to one SDK.

```python
# Pinecone specific code
import pinecone
pinecone.init(api_key="...")
idx = pinecone.Index("my-index")
idx.upsert(vectors=[...])
```

### After (Embex)

Vendor-agnostic. Switching from Pinecone to Qdrant is just changing one string.

```python
# Embex code
from embex import EmbexClient
# To switch provider, just change "pinecone" to "qdrant"
client = await EmbexClient.new_async("pinecone", "...")
await client.collection("my-index").insert([...])
```

**Result**: Sarah moved to Qdrant in 2 minutes of code changes.
