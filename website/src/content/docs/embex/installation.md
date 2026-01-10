---
title: Installation
description: Get started with Embex in minutes.
---

Embex is available for Python and Node.js. It requires no external services to get started (using embedded LanceDB), but works with Qdrant, Chroma, Weaviate, and more.

## Python

Install the `embex` package from PyPI.

```bash
pip install embex lancedb sentence-transformers
```

### Requirements

- Python 3.9+
- pip

## Node.js

Install the `@bridgerust/embex` package from npm.

```bash
npm install @bridgerust/embex lancedb
```

### Requirements

- Node.js 18+ (tested on 20 LTS)

## Docker Support

If you plan to use providers like Qdrant, Weaviate, or PgVector, you'll need to run them. We provide a `docker-compose` setup for convenience.

```yaml
# docker-compose.yml
version: "3.8"
services:
  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
```
