# Embex Node.js API Reference

Complete API documentation for the Node.js/TypeScript implementation of Embex.

## Installation

```bash
npm install @bridgerust/embex
# OR
bun add @bridgerust/embex
```

## Quick Start

```typescript
import { EmbexClient } from "@bridgerust/embex";

// Initialize client
const client = new EmbexClient("qdrant", "http://localhost:6333");

// Get collection
const collection = client.collection("my_docs");

// Create collection
await collection.create(768, "cosine");

// Insert points
const points = [
  { id: "1", vector: Array(768).fill(0.1), metadata: { title: "Doc 1" } },
];
await collection.insert(points);

// Search
const results = await collection.search(Array(768).fill(0.1), 5);
for (const r of results.results) {
  console.log(`ID: ${r.id}, Score: ${r.score}`);
}
```

## EmbexClient

### Constructor

```typescript
new EmbexClient(
  provider: string,
  url: string,
  apiKey?: string | null
)
```

**Parameters:**

- `provider`: Database provider (`"qdrant"`, `"pinecone"`, `"chroma"`, etc.)
- `url`: Connection URL
- `apiKey`: Optional API key for authenticated providers

**Example:**

```typescript
// Qdrant
const client = new EmbexClient("qdrant", "http://localhost:6333");

// Pinecone
const client = new EmbexClient("pinecone", "", "your-api-key");

// Chroma
const client = new EmbexClient("chroma", "http://localhost:8000");
```

### Static Method: `newAsync`

For providers requiring async initialization (Milvus, PgVector, LanceDB):

```typescript
const client = await EmbexClient.newAsync("lancedb", "/path/to/database");
```

### Methods

#### `collection(name: string): Collection`

Get a handle to a specific collection.

```typescript
const collection = client.collection("my_collection");
```

## Collection

### Methods

#### `create(dimension: number, distance: string): Promise<void>`

Create a new collection.

**Parameters:**

- `dimension`: Vector dimension (e.g., 768)
- `distance`: Distance metric (`"cosine"`, `"euclidean"`, `"dot"`)

**Example:**

```typescript
await collection.create(768, "cosine");
```

#### `insert(points: Point[]): Promise<void>`

Insert points into the collection.

**Parameters:**

- `points`: Array of point objects

**Example:**

```typescript
const points = [
  {
    id: "doc1",
    vector: [0.1, 0.2, 0.3],
    metadata: { title: "Document 1", category: "tech" },
  },
  {
    id: "doc2",
    vector: [0.4, 0.5, 0.6],
    metadata: { title: "Document 2", category: "science" },
  },
];

await collection.insert(points);
```

#### `insertBatch(points: Point[], batchSize?: number, parallel?: number): Promise<void>`

Insert points in parallel batches.

**Parameters:**

- `points`: Array of points to insert
- `batchSize`: Points per batch (default: 1000)
- `parallel`: Max concurrent requests (default: 1)

**Example:**

```typescript
// Insert 10,000 points in batches of 1000 with 3 parallel requests
await collection.insertBatch(points, 1000, 3);
```

#### `search(vector: number[], topK?: number, filter?: any, includeMetadata?: boolean, includeVector?: boolean): Promise<SearchResponse>`

Search for similar vectors with direct parameters.

**Parameters:**

- `vector`: Query vector
- `topK`: Number of results (default: 10)
- `filter`: Optional metadata filter
- `includeMetadata`: Include metadata in results (default: true)
- `includeVector`: Include vectors in results (default: false)

**Example:**

```typescript
// Simple search
const results = await collection.search(Array(768).fill(0.1), 5);

// Search with filter
const filter = {
  op: "key",
  args: ["category", { op: "eq", args: "tech" }],
};

const results = await collection.search(
  Array(768).fill(0.1),
  5,
  filter,
  true,
  false
);

// Access results
for (const result of results.results) {
  console.log(`ID: ${result.id}, Score: ${result.score}`);
  if (result.metadata) {
    console.log(`Metadata:`, result.metadata);
  }
}
```

#### `query(vector: number[], options?: SearchOptions): Promise<SearchResponse>`

Search using options object.

**Parameters:**

- `vector`: Query vector
- `options`: Optional search options

**Example:**

```typescript
const results = await collection.query(Array(768).fill(0.1), {
  limit: 10,
  filter: {
    op: "key",
    args: ["status", { op: "eq", args: "active" }],
  },
  includeMetadata: true,
  includeVector: false,
  offset: 0,
});
```

#### `buildSearch(vector: number[]): SearchBuilder`

Create a search builder for method chaining.

**Example:**

```typescript
const results = await collection
  .buildSearch(Array(768).fill(0.1))
  .limit(10)
  .includeMetadata(true)
  .includeVector(false)
  .filter({
    op: "key",
    args: ["status", { op: "eq", args: "active" }],
  })
  .execute();
```

#### `buildQuery(): QueryBuilder`

Create a query builder for filter-only queries (no vector search). Useful for filtering and aggregating data without similarity search.

**Example:**

```typescript
// Count documents with a filter
const results = await collection
  .buildQuery()
  .limit(0) // No results needed, just aggregation
  .filter({
    op: "key",
    args: ["status", { op: "eq", args: "active" }],
  })
  .aggregation("count")
  .execute();

console.log(`Total active documents: ${results.aggregations.count}`);
```

#### `updateMetadata(updates: MetadataUpdate[]): Promise<void>`

Update metadata for existing points in the collection.

**Parameters:**

- `updates`: Array of metadata update objects

**Example:**

```typescript
// Update metadata for specific points
await collection.updateMetadata([
  {
    id: "doc1",
    updates: {
      status: "archived",
      updated_at: new Date().toISOString(),
    },
  },
  {
    id: "doc2",
    updates: {
      views: 1000,
      featured: true,
    },
  },
]);
```

#### `delete(ids: string[]): Promise<void>`

Delete points by their IDs.

**Example:**

```typescript
await collection.delete(["doc1", "doc2", "doc3"]);
```

#### `deleteCollection(): Promise<void>`

Delete the entire collection.

**Example:**

```typescript
await collection.deleteCollection();
```

## Types

### Point

```typescript
interface Point {
  id: string;
  vector: number[];
  metadata?: Record<string, any>;
}
```

### SearchOptions

```typescript
interface SearchOptions {
  limit?: number;
  filter?: any;
  includeMetadata?: boolean;
  includeVector?: boolean;
  offset?: number;
}
```

### SearchResponse

```typescript
interface SearchResponse {
  results: SearchResult[];
  aggregations: Record<string, any>;
}
```

### SearchResult

```typescript
interface SearchResult {
  id: string;
  score: number;
  vector?: number[];
  metadata?: Record<string, any>;
}
```

### MetadataUpdate

```typescript
interface MetadataUpdate {
  id: string;
  updates: Record<string, any>;
}
```

## QueryBuilder

Builder pattern for filter-only queries (no vector search). Useful for filtering, aggregating, and paginating data.

### Methods

#### `limit(limit: number): Promise<QueryBuilder>`

Set the maximum number of results to return.

#### `offset(offset: number): Promise<QueryBuilder>`

Set the pagination offset.

#### `includeMetadata(include: boolean): Promise<QueryBuilder>`

Include metadata in results.

#### `includeVector(include: boolean): Promise<QueryBuilder>`

Include vectors in results.

#### `filter(filter: any): Promise<QueryBuilder>`

Apply a metadata filter.

#### `aggregation(aggType: string): Promise<QueryBuilder>`

Add an aggregation to the query. Currently supports `"count"`.

#### `execute(): Promise<SearchResponse>`

Execute the query.

**Example:**

```typescript
// Count documents by category
const results = await collection
  .buildQuery()
  .filter({
    op: "key",
    args: ["category", { op: "eq", args: "tech" }],
  })
  .aggregation("count")
  .execute();

console.log(`Tech documents: ${results.aggregations.count}`);

// Get all documents with pagination
const page1 = await collection
  .buildQuery()
  .limit(10)
  .offset(0)
  .includeMetadata(true)
  .execute();

const page2 = await collection
  .buildQuery()
  .limit(10)
  .offset(10)
  .includeMetadata(true)
  .execute();
```

## SearchBuilder

Builder pattern for constructing search queries.

### Methods

#### `limit(limit: number): Promise<SearchBuilder>`

Set the maximum number of results.

#### `offset(offset: number): Promise<SearchBuilder>`

Set the pagination offset.

#### `includeMetadata(include: boolean): Promise<SearchBuilder>`

Include metadata in results.

#### `includeVector(include: boolean): Promise<SearchBuilder>`

Include vectors in results.

#### `filter(filter: any): Promise<SearchBuilder>`

Apply a metadata filter.

#### `aggregation(aggType: string): Promise<SearchBuilder>`

Add an aggregation to the query. Currently supports `"count"`.

**Example:**

```typescript
const results = await collection
  .buildSearch(queryVector)
  .limit(10)
  .filter({
    op: "key",
    args: ["category", { op: "eq", args: "tech" }],
  })
  .aggregation("count")
  .execute();

console.log(`Total matching documents: ${results.aggregations.count}`);
```

#### `execute(): Promise<SearchResponse>`

Execute the search query.

**Example:**

```typescript
const builder = collection.buildSearch(queryVector);
const results = await builder
  .limit(10)
  .offset(0)
  .includeMetadata(true)
  .includeVector(false)
  .filter({
    op: "and",
    args: [
      { op: "key", args: ["status", { op: "eq", args: "active" }] },
      { op: "key", args: ["score", { op: "gte", args: 10 }] },
    ],
  })
  .execute();
```

## Filters

Filters use a JSON-like structure:

```typescript
// Equality
const filter = {
  op: "key",
  args: ["status", { op: "eq", args: "active" }],
};

// Comparison
const filter = {
  op: "key",
  args: ["score", { op: "gte", args: 10 }],
};

// In array
const filter = {
  op: "key",
  args: ["category", { op: "in", args: ["tech", "science"] }],
};

// Complex (AND)
const filter = {
  op: "and",
  args: [
    { op: "key", args: ["status", { op: "eq", args: "active" }] },
    { op: "key", args: ["score", { op: "gte", args: 10 }] },
  ],
};

// OR
const filter = {
  op: "or",
  args: [
    { op: "key", args: ["category", { op: "eq", args: "tech" }] },
    { op: "key", args: ["category", { op: "eq", args: "science" }] },
  ],
};
```

## Error Handling

Embex throws JavaScript errors for failures:

```typescript
import { EmbexClient } from "@bridgerust/embex";

try {
  const client = new EmbexClient("qdrant", "http://localhost:6333");
  const collection = client.collection("test");
  await collection.create(768, "cosine");
} catch (error) {
  console.error("Error:", error.message);
}
```

## Complete Example

```typescript
import { EmbexClient } from "@bridgerust/embex";

async function main() {
  // Initialize client
  const client = new EmbexClient("qdrant", "http://localhost:6333");
  const collection = client.collection("documents");

  // Create collection
  try {
    await collection.deleteCollection();
  } catch (e) {
    // Collection might not exist
  }

  await collection.create(768, "cosine");

  // Insert documents
  const documents = [
    { id: "1", text: "Rust is fast", category: "programming" },
    { id: "2", text: "Python is easy", category: "programming" },
    { id: "3", text: "Physics is cool", category: "science" },
  ];

  const points = documents.map((doc) => ({
    id: doc.id,
    vector: Array(768).fill(0.1), // Replace with actual embeddings
    metadata: { text: doc.text, category: doc.category },
  }));

  await collection.insert(points);

  // Search with aggregations
  const queryVector = Array(768).fill(0.1);
  const results = await collection
    .buildSearch(queryVector)
    .limit(2)
    .filter({
      op: "key",
      args: ["category", { op: "eq", args: "programming" }],
    })
    .aggregation("count")
    .execute();

  console.log(`Total matching documents: ${results.aggregations.count}`);
  for (const result of results.results) {
    console.log(`Found: ${result.id} (score: ${result.score.toFixed(4)})`);
    console.log(`Text: ${result.metadata?.text}`);
  }

  // Update metadata
  await collection.updateMetadata([
    {
      id: "1",
      updates: { views: 100, last_accessed: new Date().toISOString() },
    },
  ]);
}

main().catch(console.error);
```

## TypeScript Support

Full TypeScript definitions are included:

```typescript
import { EmbexClient, Point, SearchResponse } from "@bridgerust/embex";

const client: EmbexClient = new EmbexClient("qdrant", "http://localhost:6333");
const points: Point[] = [
  { id: "1", vector: [0.1, 0.2], metadata: { title: "Doc" } },
];
const results: SearchResponse = await collection.search([0.1, 0.2], 5);
```
