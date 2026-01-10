---
title: Build a Chatbot in 10 Minutes
description: Use Embex to build a RAG-powered chatbot.
---

In this tutorial, we will build a simple Retrieval Augmented Generation (RAG) system using **Embex** and **LanceDB** (embedded, no setup required).

## Prerequisite

Install the libraries:

```bash
pip install embex lancedb sentence-transformers openai
```

## Step 1: Initialize

Create a new file `bot.py`:

```python
import asyncio
from embex import EmbexClient, Point
from sentence_transformers import SentenceTransformer

# 1. Setup
embedder = SentenceTransformer('all-MiniLM-L6-v2')
client = await EmbexClient.new_async("lancedb", "./chatbot_data")
collection = client.collection("knowledge_base")
```

## Step 2: Ingest Data

Let's add some knowledge.

```python
async def ingest():
    texts = [
        "Embex is a unified vector database ORM.",
        "It supports Qdrant, Chroma, and more.",
        "Rust core makes it blazingly fast."
    ]

    # Create DB
    await collection.create(384) # dimensionality of all-MiniLM-L6-v2

    # Embed and Insert
    points = []
    for i, text in enumerate(texts):
        vector = embedder.encode(text).tolist()
        points.append(Point(id=str(i), vector=vector, metadata={"text": text}))

    await collection.insert(points)
    print("Ingestion complete!")
```

## Step 3: Search

Now let's search.

```python
async def search(query):
    vector = embedder.encode(query).tolist()
    results = await collection.search(vector, top_k=1)

    if results:
        best = results[0]
        print(f"Query: {query}")
        print(f"Answer: {best.metadata['text']}")
    else:
        print("I don't know.")

async def main():
    await ingest()
    await search("What makes Embex fast?")

if __name__ == "__main__":
    asyncio.run(main())
```

Run it:

```bash
python bot.py
# Output:
# Query: What makes Embex fast?
# Answer: Rust core makes it blazingly fast.
```
