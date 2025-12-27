# Chroma Adapter Improvements

This document outlines improvements that can be made to the Chroma adapter to address the limitations mentioned in the migration guide.

## Issues to Address

1. **Dimension Inference**: Make dimension optional and infer from first insert
2. **Text-based Search**: Add support for text queries with embedding generation
3. **Persistent Mode**: Add support for Chroma's persistent/embedded client

## Implementation Plan

### 1. Dimension Inference

**Current State**:

- `CollectionSchema` requires `dimension: usize`
- Chroma adapter ignores dimension in `create_collection` (Chroma infers from first insert)

**Proposed Solution**:

- Make `dimension` optional in `CollectionSchema` (use `Option<usize>`)
- Update `create_collection` to handle optional dimension
- For Chroma: If dimension is `None`, create collection without dimension (Chroma will infer)
- For other adapters: Still require dimension (or provide sensible defaults)

**Code Changes**:

```rust
// In bridge-embex-core/src/types.rs
pub struct CollectionSchema {
    pub name: String,
    pub dimension: Option<usize>,  // Changed from usize
    pub metric: DistanceMetric,
}

// In chroma adapter
async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
    // Chroma doesn't need dimension at creation, it infers from first insert
    self.client
        .create_collection(schema.name.clone(), None, None)
        .await?;
    Ok(())
}
```

**Breaking Change**: Yes - requires updating all adapters and call sites

### 2. Text-based Search

**Current State**:

- Only supports vector-based search
- No built-in embedding generation

**Proposed Solution**:

- Add optional embedding function configuration to `ChromaAdapter`
- Add a new method `search_by_text` that:
  1. Generates embeddings from text
  2. Performs vector search
- Support common embedding models (sentence-transformers, OpenAI, etc.)

**Code Changes**:

```rust
pub struct ChromaAdapter {
    client: ChromaHttpClient,
    embedding_fn: Option<Box<dyn Fn(&str) -> Vec<f32> + Send + Sync>>,
}

impl ChromaAdapter {
    pub fn with_embedding_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Vec<f32> + Send + Sync + 'static,
    {
        self.embedding_fn = Some(Box::new(f));
        self
    }

    pub async fn search_by_text(
        &self,
        collection: &str,
        query_text: &str,
        top_k: usize,
    ) -> Result<SearchResponse> {
        let embedding = if let Some(ref fn_) = self.embedding_fn {
            fn_(query_text)
        } else {
            return Err(EmbexError::Configuration(
                "No embedding function configured".to_string()
            ));
        };

        self.search(&VectorQuery {
            collection: collection.to_string(),
            vector: Some(embedding),
            top_k,
            // ... other fields
        }).await
    }
}
```

**Dependencies**: Would need to add embedding model dependencies (optional feature)

### 3. Persistent Mode

**Current State**:

- Only supports `ChromaHttpClient` (HTTP mode)
- No support for `PersistentClient` (embedded mode)

**Proposed Solution**:

- Add a new constructor `new_persistent` that uses `PersistentClient`
- Keep HTTP client as default for consistency
- Add configuration option in `EmbexConfig`

**Code Changes**:

```rust
use chroma::PersistentClient;  // If available in chroma crate

impl ChromaAdapter {
    pub fn new_persistent(path: impl AsRef<Path>) -> Result<Self> {
        let client = PersistentClient::new(path)
            .map_err(|e| EmbexError::Database(format!("Failed to create persistent client: {}", e)))?;

        Ok(Self {
            client: ChromaHttpClient::from_persistent(client)?,  // Or similar API
        })
    }
}
```

**Note**: Need to verify if `chroma` crate supports `PersistentClient` and how to convert it to HTTP client interface

## Implementation Priority

1. **High Priority**: Dimension inference (most requested, easier to implement)
2. **Medium Priority**: Persistent mode (useful for local development)
3. **Low Priority**: Text-based search (requires additional dependencies, can be handled by users)

## Breaking Changes

- Making `dimension` optional in `CollectionSchema` is a breaking change
- Would require updating all adapters
- Would require updating all call sites
- Could be done in a major version bump

## Alternative: Non-breaking Approach

Instead of making `dimension` optional in `CollectionSchema`, we could:

1. Add a new method `create_collection_auto` that doesn't require dimension
2. Keep existing `create_collection` with required dimension
3. For Chroma, allow `create_collection` to work even if dimension is 0 (ignored)

This maintains backward compatibility while adding new functionality.

## Recommendation

Start with **dimension inference** using the non-breaking approach:

- Add `create_collection_auto` method
- For Chroma, make dimension optional in the adapter layer (not in core types)
- Document that Chroma infers dimension from first insert

Then add **persistent mode** support if the `chroma` crate supports it.

**Text-based search** can be left as a user responsibility (they provide embeddings) or added as an optional feature with embedding dependencies.
