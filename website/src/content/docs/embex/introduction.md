---
title: Introduction
description: Introduction to Embex, the universal vector database client.
---

import { Tabs, TabItem } from '@astrojs/starlight/components';

**Embex** is the universal vector database ORM for Rust, Python, and Node.js.

## Why Async Initialization?

We use asynchronous initialization (`new_async` / `newAsync`) to ensure compatibility with all providers. Some embedded databases (like LanceDB) require async setup to handle file I/O non-blockingly.

<Tabs>
  <TabItem label="Python">

    ```python
    # Initialize properly for ANY provider (local or cloud)
    client = await EmbexClient.new_async("lancedb://./data")
    ```

  </TabItem>
  <TabItem label="Node.js">

    ```typescript
    // Initialize properly for ANY provider (local or cloud)
    const client = await EmbexClient.newAsync("lancedb://./data");
    ```

  </TabItem>
</Tabs>

## Features

- **Unified API**: Switch providers (Qdrant, Pinecone, LanceDB, Chroma, etc.) with a single line of config.
- **High Performance**: Powered by a shared Rust core with SIMD acceleration.
- **Production Ready**: Connection pooling, migrations, and observability built-in.
