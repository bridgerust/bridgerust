/**
 * Aggregation Integration Tests
 * Tests aggregation functionality (count, etc.)
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { EmbexClient } from "../../index.js";
import { randomUUID } from "crypto";

const TEST_DIMENSION = 128;
const TEST_COLLECTION = "embex_aggregation_test";

function randomVector(dim = TEST_DIMENSION) {
  return Array.from({ length: dim }, () => Math.random());
}

describe("Aggregations", () => {
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

  it("should return count aggregation", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = Array.from({ length: 10 }, () => ({
      id: randomUUID(),
      vector: randomVector(),
      metadata: {},
    }));
    await collection.insert(points);

    const builder = collection.buildSearch(randomVector());
    const results = await builder.limit(5).aggregation("count").execute();

    expect(results.aggregations).toBeDefined();
    expect(results.aggregations.count).toBeDefined();
    expect(typeof results.aggregations.count).toBe("number");
    expect(results.aggregations.count).toBeGreaterThanOrEqual(10);
  });

  it("should return count with filters", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "a1", vector: randomVector(), metadata: { type: "A" } },
      { id: "a2", vector: randomVector(), metadata: { type: "A" } },
      { id: "b1", vector: randomVector(), metadata: { type: "B" } },
    ];
    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["type", { op: "eq", args: "A" }],
    };

    const builder = collection.buildSearch(randomVector());
    const results = await builder
      .limit(10)
      .filter(filter)
      .aggregation("count")
      .execute();

    expect(results.aggregations.count).toBeGreaterThanOrEqual(2);
  });

  it("should return count for filter-only queries", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = Array.from({ length: 5 }, () => ({
      id: randomUUID(),
      vector: randomVector(),
      metadata: { status: "active" },
    }));
    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["status", { op: "eq", args: "active" }],
    };

    const builder = collection.buildQuery();
    const results = await builder
      .filter(filter)
      .limit(10)
      .aggregation("count")
      .execute();

    expect(results.aggregations).toBeDefined();
    expect(results.aggregations.count).toBeGreaterThanOrEqual(5);
  });
});
