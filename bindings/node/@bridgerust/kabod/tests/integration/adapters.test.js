/**
 * Kabod Integration Tests - Node.js
 * Tests all adapter implementations against real database instances.
 *
 * Run Docker Compose first: docker-compose up -d
 *
 * Usage:
 *   cd bindings/node/@bridgerust/kabod
 *   npm run build
 *   npm test tests/integration/
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { KabodClient } from "../../index.js";
import { randomUUID } from "crypto";
import { tmpdir } from "os";
import { join } from "path";

const TEST_DIMENSION = 128;
const TEST_COLLECTION = "kabod_integration_test_node";

function randomVector(dim = TEST_DIMENSION) {
  return Array.from({ length: dim }, () => Math.random());
}

describe("Qdrant Adapter", () => {
  let client;
  let collection;

  beforeAll(() => {
    client = new KabodClient("qdrant", "http://localhost:6334");
    collection = client.collection(TEST_COLLECTION);
  });

  afterAll(async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
  });

  it("should create collection", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");
  });

  it("should insert and search points", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: randomUUID(), vector: randomVector(), metadata: { category: "A" } },
      { id: randomUUID(), vector: randomVector(), metadata: { category: "B" } },
      { id: randomUUID(), vector: randomVector(), metadata: { category: "A" } },
    ];

    await collection.insert(points);

    const results = await collection.query(randomVector(), { limit: 2 });
    expect(results.results.length).toBe(2);
  });

  it("should use search() method with direct parameters", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: randomUUID(), vector: randomVector(), metadata: { category: "A" } },
      { id: randomUUID(), vector: randomVector(), metadata: { category: "B" } },
    ];

    await collection.insert(points);

    const results = await collection.search(
      randomVector(),
      2,
      null,
      true,
      false
    );
    expect(results.results.length).toBe(2);
    expect(results.results[0].id).toBeDefined();
    expect(results.results[0].score).toBeDefined();
  });

  it("should search with filters", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      {
        id: "f1",
        vector: randomVector(),
        metadata: { status: "active", score: 10 },
      },
      {
        id: "f2",
        vector: randomVector(),
        metadata: { status: "inactive", score: 20 },
      },
      {
        id: "f3",
        vector: randomVector(),
        metadata: { status: "active", score: 30 },
      },
    ];

    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["status", { op: "eq", args: "active" }],
    };

    const results = await collection.search(
      randomVector(),
      10,
      filter,
      true,
      false
    );
    expect(results.results.length).toBeGreaterThanOrEqual(2);
    results.results.forEach((r) => {
      expect(r.metadata?.status).toBe("active");
    });
  });

  it("should use search builder pattern", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: randomUUID(), vector: randomVector(), metadata: { type: "test" } },
    ];
    await collection.insert(points);

    const builder = collection.buildSearch(randomVector());
    const results = await builder
      .limit(5)
      .includeMetadata(true)
      .includeVector(false)
      .execute();

    expect(results.results).toBeDefined();
    expect(Array.isArray(results.results)).toBe(true);
  });

  it("should insert batch with parallel option", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = Array.from({ length: 50 }, () => ({
      id: randomUUID(),
      vector: randomVector(),
      metadata: { batch: "test" },
    }));

    await collection.insertBatch(points, 10, 3);

    const results = await collection.query(randomVector(), { limit: 50 });
    expect(results.results.length).toBeGreaterThanOrEqual(50);
  });

  it("should delete points", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const delId = randomUUID();
    await collection.insert([
      { id: delId, vector: randomVector(), metadata: {} },
    ]);

    await collection.delete([delId]);

    const results = await collection.query(randomVector(), { limit: 10 });
    const ids = results.results.map((r) => r.id);
    expect(ids).not.toContain(delId);
  });
});

describe("Chroma Adapter", () => {
  let client;
  let collection;

  beforeAll(() => {
    client = new KabodClient("chroma", "http://localhost:8000");
    collection = client.collection(TEST_COLLECTION);
  });

  afterAll(async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
  });

  it("should perform CRUD operations", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "c1", vector: randomVector(), metadata: { type: "test" } },
      { id: "c2", vector: randomVector(), metadata: { type: "test" } },
    ];

    await collection.insert(points);

    const results = await collection.query(randomVector(), { limit: 2 });
    expect(results.results.length).toBe(2);
  });

  it("should use search() method", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "c3", vector: randomVector(), metadata: { type: "search" } },
    ];
    await collection.insert(points);

    const results = await collection.search(randomVector(), 1);
    expect(results.results.length).toBeGreaterThanOrEqual(1);
  });
});

describe("Weaviate Adapter", () => {
  let client;
  let collection;

  beforeAll(() => {
    client = new KabodClient("weaviate", "http://localhost:8080");
    collection = client.collection("KabodNodeTest");
  });

  afterAll(async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
  });

  it("should perform CRUD operations", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "w1", vector: randomVector(), metadata: { name: "test1" } },
      { id: "w2", vector: randomVector(), metadata: { name: "test2" } },
    ];

    await collection.insert(points);

    const results = await collection.query(randomVector(), { limit: 2 });
    expect(results.results.length).toBeGreaterThanOrEqual(1);
  });
});

describe("Milvus Adapter", () => {
  let client;
  let collection;

  beforeAll(async () => {
    // Milvus requires async initialization
    client = await KabodClient.newAsync("milvus", "http://localhost:19530");
    collection = client.collection(TEST_COLLECTION);
  });

  afterAll(async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
  });

  it("should perform CRUD operations", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "m1", vector: randomVector(), metadata: {} },
      { id: "m2", vector: randomVector(), metadata: {} },
    ];

    await collection.insert(points);

    const results = await collection.query(randomVector(), { limit: 2 });
    expect(results.results.length).toBeGreaterThanOrEqual(1);
  });

  it("should use search() method", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "m3", vector: randomVector(), metadata: { test: "milvus" } },
    ];
    await collection.insert(points);

    const results = await collection.search(randomVector(), 1);
    expect(results.results.length).toBeGreaterThanOrEqual(1);
  });
});

describe("pgvector Adapter", () => {
  let client;
  let collection;

  beforeAll(async () => {
    // pgvector requires async initialization
    client = await KabodClient.newAsync(
      "pgvector",
      "postgresql://kabod:kabod_test@localhost:5432/kabod_test"
    );
    collection = client.collection(TEST_COLLECTION);
  });

  afterAll(async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
  });

  it("should perform CRUD operations", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "pg1", vector: randomVector(), metadata: { info: "test" } },
      { id: "pg2", vector: randomVector(), metadata: { info: "test" } },
    ];

    await collection.insert(points);

    const results = await collection.query(randomVector(), { limit: 2 });
    expect(results.results.length).toBeGreaterThanOrEqual(1);
  });

  it("should use search() method with filters", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "pg3", vector: randomVector(), metadata: { status: "active" } },
    ];
    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["status", { op: "eq", args: "active" }],
    };

    const results = await collection.search(
      randomVector(),
      5,
      filter,
      true,
      false
    );
    expect(results.results.length).toBeGreaterThanOrEqual(1);
  });
});

describe("LanceDB Adapter", () => {
  let client;
  let collection;
  const dbPath = join(tmpdir(), `lancedb_node_test_${Date.now()}`);

  beforeAll(async () => {
    // LanceDB requires async initialization
    client = await KabodClient.newAsync("lancedb", dbPath);
    collection = client.collection(TEST_COLLECTION);
  });

  afterAll(async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
  });

  it("should perform CRUD operations", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "l1", vector: randomVector(), metadata: { name: "lance1" } },
      { id: "l2", vector: randomVector(), metadata: { name: "lance2" } },
    ];

    await collection.insert(points);

    const results = await collection.query(randomVector(), { limit: 2 });
    expect(results.results.length).toBeGreaterThanOrEqual(1);
  });

  it("should use search() method", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "l3", vector: randomVector(), metadata: { name: "lance3" } },
    ];
    await collection.insert(points);

    const results = await collection.search(
      randomVector(),
      1,
      null,
      true,
      false
    );
    expect(results.results.length).toBeGreaterThanOrEqual(1);
  });
});
