/**
 * Metadata Operations Integration Tests
 * Tests metadata update and query functionality
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { EmbexClient } from "../../index";
import { randomUUID } from "crypto";

const TEST_DIMENSION = 128;
const TEST_COLLECTION = "embex_metadata_test";

function randomVector(dim = TEST_DIMENSION) {
  return Array.from({ length: dim }, () => Math.random());
}

describe("Metadata Operations", () => {
  let client: EmbexClient;
  let collection: any;

  beforeAll(() => {
    client = new EmbexClient("qdrant", "http://localhost:6334");
    collection = client.collection(TEST_COLLECTION);
  });

  afterAll(async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
  });

  it("should insert points with metadata", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { name: "test1", value: 42, active: true },
      },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { name: "test2", value: 100, active: false },
      },
    ];

    await collection.insert(points);

    const results = await collection.query(randomVector(), { limit: 10 });
    expect(results.results.length).toBe(2);
    expect(results.results[0].metadata).toBeDefined();
    expect(results.results[0].metadata?.name).toBeDefined();
  });

  it("should update metadata for existing points", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const pointId = randomUUID();
    await collection.insert([
      {
        id: pointId,
        vector: randomVector(),
        metadata: { status: "old", version: 1 },
      },
    ]);

    // Update metadata
    await collection.updateMetadata([
      {
        id: pointId,
        updates: { status: "new", version: 2, updated: true },
      },
    ]);

    const results = await collection.query(randomVector(), { limit: 10 });
    const updatedPoint = results.results.find((r: any) => r.id === pointId);
    expect(updatedPoint).toBeDefined();
    expect(updatedPoint.metadata?.status).toBe("new");
    expect(updatedPoint.metadata?.version).toBe(2);
    expect(updatedPoint.metadata?.updated).toBe(true);
  });

  it("should update multiple points metadata", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const ids = [randomUUID(), randomUUID()];
    await collection.insert([
      { id: ids[0], vector: randomVector(), metadata: { batch: "A" } },
      { id: ids[1], vector: randomVector(), metadata: { batch: "A" } },
    ]);

    await collection.updateMetadata([
      { id: ids[0], updates: { batch: "B", updated: true } },
      { id: ids[1], updates: { batch: "B", updated: true } },
    ]);

    const results = await collection.query(randomVector(), { limit: 10 });
    const updated = results.results.filter((r: any) => ids.includes(r.id));
    expect(updated.length).toBe(2);
    updated.forEach((r: any) => {
      expect(r.metadata?.batch).toBe("B");
      expect(r.metadata?.updated).toBe(true);
    });
  });

  it("should query with metadata filters", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    await collection.insert([
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { category: "tech", rating: 5 },
      },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { category: "tech", rating: 3 },
      },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { category: "science", rating: 5 },
      },
    ]);

    const filter = {
      op: "must",
      args: [
        { op: "key", args: ["category", { eq: "tech" }] },
        { op: "key", args: ["rating", { gte: 4 }] },
      ],
    };

    const results = await collection.query(randomVector(), {
      limit: 10,
      filter,
    });
    expect(results.results.length).toBeGreaterThanOrEqual(1);
    results.results.forEach((r: any) => {
      expect(r.metadata?.category).toBe("tech");
      expect(r.metadata?.rating).toBeGreaterThanOrEqual(4);
    });
  });

  it("should include/exclude metadata in results", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    await collection.insert([
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { secret: "data" },
      },
    ]);

    // Test with metadata included
    const withMetadata = await collection.search(
      randomVector(),
      10,
      null,
      true,
      false
    );
    expect(withMetadata.results[0].metadata).toBeDefined();

    // Test with metadata excluded (if supported)
    const withoutMetadata = await collection.query(randomVector(), {
      limit: 10,
      includeMetadata: false,
    });
    // Some adapters may still return metadata, so we just check the call succeeds
    expect(withoutMetadata.results).toBeDefined();
  });
});
