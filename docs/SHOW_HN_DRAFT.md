# Show HN: Embex – Universal Vector Database ORM (Rust + SIMD)

**One API. Any Vector Database. 4x Faster.**

I've been building Embex, a universal ORM for vector databases that lets you switch between Qdrant, Pinecone, Chroma, LanceDB, Milvus, Weaviate, and PgVector without changing a single line of code.

## The Problem

Every vector database has a different API. If you want to switch providers (e.g., from Qdrant to Pinecone), you're rewriting your entire codebase. Plus, most clients are written in Python/JavaScript, which means you're leaving performance on the table.

## The Solution

Embex provides:

- **Unified API**: Same code works with 7 different vector databases
- **Rust Core**: Built on a shared Rust core with SIMD acceleration (4x faster vector ops)
- **Zero Overhead**: < 5% overhead vs native clients
- **Type Safety**: Full TypeScript and Python type hints
- **Production Ready**: Built-in migrations, connection pooling, observability

## Performance

- **SIMD Acceleration**: 3.6x - 4.0x faster dot product/cosine similarity on ARM64
- **Minimal Overhead**: < 5% vs native clients
- **Optimized Packages**: 8-15MB (down from 65MB+)

## Quick Example

```typescript
import { EmbexClient } from '@bridgerust/embex';

// Switch providers by changing one string
const client = new EmbexClient('qdrant', 'http://localhost:6333');
// const client = new EmbexClient('pinecone', 'https://api.pinecone.io');

const collection = client.collection('documents');
await collection.create(768, 'cosine');

await collection.insert([{
  id: '1',
  vector: [0.1, 0.2, ...],
  metadata: { text: 'Hello world' }
}]);

const results = await collection.search([0.1, 0.2, ...], 5);
```

## What Makes It Different

1. **Rust Core**: Not a wrapper—actual Rust bindings via PyO3/NAPI-RS
2. **SIMD Optimized**: Uses AVX2/NEON intrinsics for vector operations
3. **True Abstraction**: Provider differences are handled internally
4. **Cross-Platform**: Works on macOS, Linux, Windows (all via CI/CD)

## Current Status

- ✅ 7 vector database providers supported
- ✅ Python and Node.js bindings
- ✅ Production-ready (optimized builds, CI/CD)
- ✅ Full documentation and examples
- ✅ Benchmarks and performance guides

## Try It

```bash
# Python
pip install embex

# Node.js
npm install @bridgerust/embex
```

**GitHub**: https://github.com/bridgerust/bridgerust
**Docs**: https://github.com/bridgerust/bridgerust/tree/main/docs

## What I'd Love Feedback On

1. **API Design**: Does the unified API feel natural?
2. **Performance**: Are the benchmarks realistic for your use case?
3. **Missing Features**: What would make you switch from native clients?
4. **Provider Support**: Any other vector databases you'd like to see?

I'm particularly interested in:

- Real-world performance feedback
- API ergonomics (especially the builder pattern)
- What features would make this a no-brainer switch

Thanks for checking it out! 🚀
