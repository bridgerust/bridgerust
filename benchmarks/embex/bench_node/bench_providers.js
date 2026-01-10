import pkg from "@bridgerust/embex";
const { EmbexClient } = pkg;
import { QdrantClient } from "@qdrant/js-client-rest";
import { ChromaClient } from "chromadb";
import weaviate from "weaviate-client";
import pg from "pg";
import { v4 as uuidv4 } from "uuid";

const { Client } = pg;

const DIM = 384;
const COUNT = 10000;
const SEARCH_REPEATS = 100;
const HOST = "localhost";

const QDRANT_URL = `http://${HOST}:6333`;
const CHROMA_URL = `http://${HOST}:8000`; // Chroma JS client usually takes url
const WEAVIATE_HOST = `${HOST}:8080`;
const PG_DSN = `postgres://embex:embex_test@${HOST}:5432/embex_test`;

function generateVectors(count, dim) {
  const vectors = [];
  for (let i = 0; i < count; i++) {
    const v = [];
    for (let j = 0; j < dim; j++) {
      v.push(Math.random());
    }
    vectors.push(v);
  }
  return vectors;
}

const results = [];

async function benchEmbex(provider, url, vectors) {
  console.log(`--- Embex (${provider}) ---`);
  try {
    const client = new EmbexClient(provider, url);
    const colName = `Bench_${provider}_embex_node`;

    try {
      const col = client.collection(colName);
      // Clean up? delete_collection not exposed well or try/catch
      // Let's assume create fails if exists, or auto-clean
    } catch {}

    const col = client.collection(colName);
    // Explicit create
    try {
      await col.create(DIM, "cosine");
    } catch (e) {
      // console.log("Create info:", e.message);
    }

    // Insert

    const points = vectors.map((v, i) => ({
      id: uuidv4(),
      vector: v,
      metadata: { id: i },
    }));

    const start = performance.now();
    await col.insert(points);
    const insertTime = (performance.now() - start) / 1000;

    // Search
    const query = vectors[0];
    const startSearch = performance.now();
    for (let i = 0; i < SEARCH_REPEATS; i++) {
      await col.search(query, 10);
    }
    const searchTime = (performance.now() - startSearch) / SEARCH_REPEATS;

    console.log(
      `Insert: ${insertTime.toFixed(2)}s, Search: ${searchTime.toFixed(2)}ms`
    );
    results.push({
      provider,
      client: "Embex",
      insertOps: COUNT / insertTime,
      searchMs: searchTime,
    });
  } catch (e) {
    console.error(`Embex ${provider} failed:`, e);
  }
}

// --- Native ---

async function benchQdrantNative(vectors) {
  console.log("--- Native Qdrant ---");
  const client = new QdrantClient({ url: QDRANT_URL });
  const colName = "Bench_qdrant_native_node";

  try {
    await client.deleteCollection(colName);
  } catch {}
  await client.createCollection(colName, {
    vectors: { size: DIM, distance: "Cosine" },
  });

  const points = vectors.map((v, i) => ({
    id: uuidv4(),
    vector: v,
    payload: { id: i },
  }));

  const start = performance.now();
  // Qdrant JS batch upload? upsert?
  await client.upsert(colName, { points });
  const insertTime = (performance.now() - start) / 1000;

  const query = vectors[0];
  const startSearch = performance.now();
  for (let i = 0; i < SEARCH_REPEATS; i++) {
    await client.search(colName, { vector: query, limit: 10 });
  }
  const searchTime = (performance.now() - startSearch) / SEARCH_REPEATS;

  results.push({
    provider: "Qdrant",
    client: "Native",
    insertOps: COUNT / insertTime,
    searchMs: searchTime,
  });
}

async function benchChromaNative(vectors) {
  console.log("--- Native Chroma ---");
  const client = new ChromaClient({ path: CHROMA_URL });
  const colName = "Bench_chroma_native_node";

  try {
    await client.deleteCollection({ name: colName });
  } catch {}
  const collection = await client.createCollection({ name: colName });

  const ids = vectors.map(() => uuidv4());
  const metadatas = vectors.map((_, i) => ({ id: i }));

  const start = performance.now();
  // Chroma JS add
  // Batching? defaults strictly
  // Let's dump all at once if possible (10k might be too huge for one http req?)
  // We'll trust client/server to handle 10k or simple batch
  // Actually Chroma client validates batch size usually.
  // We'll iterate manually 5000
  const batchSize = 2000;
  for (let i = 0; i < vectors.length; i += batchSize) {
    const end = Math.min(i + batchSize, vectors.length);
    await collection.add({
      ids: ids.slice(i, end),
      embeddings: vectors.slice(i, end),
      metadatas: metadatas.slice(i, end),
    });
  }

  const insertTime = (performance.now() - start) / 1000;

  const query = vectors[0];
  const startSearch = performance.now();
  for (let i = 0; i < SEARCH_REPEATS; i++) {
    await collection.query({ queryEmbeddings: [query], nResults: 10 });
  }
  const searchTime = (performance.now() - startSearch) / SEARCH_REPEATS;
  results.push({
    provider: "Chroma",
    client: "Native",
    insertOps: COUNT / insertTime,
    searchMs: searchTime,
  });
}

async function benchWeaviateNative(vectors) {
  console.log("--- Native Weaviate ---");
  // weaviate-client v3
  // Use v3 API for JS
  const client = weaviate.client({
    scheme: "http",
    host: WEAVIATE_HOST, // localhost:8080
  });

  const className = "Bench_weaviate_native_node";
  try {
    await client.schema.classdeleter().withClassName(className).do();
  } catch {}

  await client.schema
    .classCreator()
    .withClass({
      class: className,
      vectorizer: "none",
      vectorIndexConfig: { distance: "cosine" },
    })
    .do();

  const start = performance.now();
  let batcher = client.batch.objectsBatcher();
  for (let i = 0; i < vectors.length; i++) {
    batcher = batcher.withObject({
      class: className,
      properties: { obj_id: i },
      vector: vectors[i],
    });
    if ((i + 1) % 100 === 0) {
      await batcher.do();
      batcher = client.batch.objectsBatcher();
    }
  }
  // Flush remaining
  // await batcher.do(); // if empty? 10000 % 100 == 0

  const insertTime = (performance.now() - start) / 1000;

  const query = { vector: vectors[0] };
  const startSearch = performance.now();
  for (let i = 0; i < SEARCH_REPEATS; i++) {
    await client.graphql
      .get()
      .withClassName(className)
      .withFields("obj_id")
      .withNearVector(query)
      .withLimit(10)
      .do();
  }
  const searchTime = (performance.now() - startSearch) / SEARCH_REPEATS;

  results.push({
    provider: "Weaviate",
    client: "Native",
    insertOps: COUNT / insertTime,
    searchMs: searchTime,
  });
}

async function main() {
  console.log("Generating vectors...");
  const vectors = generateVectors(COUNT, DIM);

  await benchEmbex("qdrant", QDRANT_URL, vectors);
  await benchQdrantNative(vectors);

  await benchEmbex("chroma", CHROMA_URL, vectors);
  await benchChromaNative(vectors);

  await benchEmbex("weaviate", `http://${WEAVIATE_HOST}`, vectors);
  await benchWeaviateNative(vectors);

  console.table(results);
}

main().catch(console.error);
