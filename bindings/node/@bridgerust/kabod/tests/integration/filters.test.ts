/**
 * Filter Integration Tests
 * Tests comprehensive filter functionality across adapters
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { KabodClient } from "../../index.js";
import { randomUUID } from "crypto";

const TEST_DIMENSION = 128;
const TEST_COLLECTION = "kabod_filter_test";

function randomVector(dim = TEST_DIMENSION) {
  return Array.from({ length: dim }, () => Math.random());
}

describe("Filter Operations", () => {
  let client: KabodClient;
  let collection: any;

  beforeAll(() => {
    client = new KabodClient("qdrant", "http://localhost:6334");
    collection = client.collection(TEST_COLLECTION);
  });

  afterAll(async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
  });

  it("should filter by equality", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "f1", vector: randomVector(), metadata: { status: "active" } },
      { id: "f2", vector: randomVector(), metadata: { status: "inactive" } },
      { id: "f3", vector: randomVector(), metadata: { status: "active" } },
    ];
    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["status", { op: "eq", args: "active" }],
    };

    const results = await collection.query(randomVector(), {
      limit: 10,
      filter,
    });
    expect(results.results.length).toBeGreaterThanOrEqual(2);
    results.results.forEach((r: any) => {
      expect(r.metadata?.status).toBe("active");
    });
  });

  it("should filter by comparison operators", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "c1", vector: randomVector(), metadata: { score: 10 } },
      { id: "c2", vector: randomVector(), metadata: { score: 20 } },
      { id: "c3", vector: randomVector(), metadata: { score: 30 } },
    ];
    await collection.insert(points);

    // Test greater than
    const gtFilter = {
      op: "key",
      args: ["score", { op: "gt", args: 15 }],
    };
    const gtResults = await collection.query(randomVector(), {
      limit: 10,
      filter: gtFilter,
    });
    expect(gtResults.results.length).toBeGreaterThanOrEqual(2);
    gtResults.results.forEach((r: any) => {
      expect(r.metadata?.score).toBeGreaterThan(15);
    });

    // Test less than or equal
    const lteFilter = {
      op: "key",
      args: ["score", { op: "lte", args: 20 }],
    };
    const lteResults = await collection.query(randomVector(), {
      limit: 10,
      filter: lteFilter,
    });
    expect(lteResults.results.length).toBeGreaterThanOrEqual(2);
    lteResults.results.forEach((r: any) => {
      expect(r.metadata?.score).toBeLessThanOrEqual(20);
    });
  });

  it("should filter by in operator", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "i1", vector: randomVector(), metadata: { category: "A" } },
      { id: "i2", vector: randomVector(), metadata: { category: "B" } },
      { id: "i3", vector: randomVector(), metadata: { category: "C" } },
      { id: "i4", vector: randomVector(), metadata: { category: "A" } },
    ];
    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["category", { op: "in", args: ["A", "B"] }],
    };

    const results = await collection.query(randomVector(), {
      limit: 10,
      filter,
    });
    expect(results.results.length).toBeGreaterThanOrEqual(3);
    results.results.forEach((r: any) => {
      expect(["A", "B"]).toContain(r.metadata?.category);
    });
  });

  it("should filter by not_in operator", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "n1", vector: randomVector(), metadata: { type: "x" } },
      { id: "n2", vector: randomVector(), metadata: { type: "y" } },
      { id: "n3", vector: randomVector(), metadata: { type: "z" } },
    ];
    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["type", { op: "not_in", args: ["x", "y"] }],
    };

    const results = await collection.query(randomVector(), {
      limit: 10,
      filter,
    });
    expect(results.results.length).toBeGreaterThanOrEqual(1);
    results.results.forEach((r: any) => {
      expect(r.metadata?.type).toBe("z");
    });
  });

  it("should support must (AND) filters", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      {
        id: "m1",
        vector: randomVector(),
        metadata: { status: "active", score: 20 },
      },
      {
        id: "m2",
        vector: randomVector(),
        metadata: { status: "active", score: 10 },
      },
      {
        id: "m3",
        vector: randomVector(),
        metadata: { status: "inactive", score: 20 },
      },
    ];
    await collection.insert(points);

    const filter = {
      op: "must",
      args: [
        { op: "key", args: ["status", { op: "eq", args: "active" }] },
        { op: "key", args: ["score", { op: "gte", args: 15 }] },
      ],
    };

    const results = await collection.query(randomVector(), {
      limit: 10,
      filter,
    });
    expect(results.results.length).toBeGreaterThanOrEqual(1);
    results.results.forEach((r: any) => {
      expect(r.metadata?.status).toBe("active");
      expect(r.metadata?.score).toBeGreaterThanOrEqual(15);
    });
  });

  it("should support should (OR) filters", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "s1", vector: randomVector(), metadata: { tag: "red" } },
      { id: "s2", vector: randomVector(), metadata: { tag: "blue" } },
      { id: "s3", vector: randomVector(), metadata: { tag: "green" } },
    ];
    await collection.insert(points);

    const filter = {
      op: "should",
      args: [
        { op: "key", args: ["tag", { op: "eq", args: "red" }] },
        { op: "key", args: ["tag", { op: "eq", args: "blue" }] },
      ],
    };

    const results = await collection.query(randomVector(), {
      limit: 10,
      filter,
    });
    expect(results.results.length).toBeGreaterThanOrEqual(2);
    results.results.forEach((r: any) => {
      expect(["red", "blue"]).toContain(r.metadata?.tag);
    });
  });

  it("should support must_not filters", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      { id: "mn1", vector: randomVector(), metadata: { visible: true } },
      { id: "mn2", vector: randomVector(), metadata: { visible: false } },
      { id: "mn3", vector: randomVector(), metadata: { visible: true } },
    ];
    await collection.insert(points);

    const filter = {
      op: "must_not",
      args: [{ op: "key", args: ["visible", { op: "eq", args: false }] }],
    };

    const results = await collection.query(randomVector(), {
      limit: 10,
      filter,
    });
    expect(results.results.length).toBeGreaterThanOrEqual(2);
    results.results.forEach((r: any) => {
      expect(r.metadata?.visible).not.toBe(false);
    });
  });

  it("should support complex nested filters", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      {
        id: "cn1",
        vector: randomVector(),
        metadata: { category: "A", score: 25, active: true },
      },
      {
        id: "cn2",
        vector: randomVector(),
        metadata: { category: "B", score: 15, active: true },
      },
      {
        id: "cn3",
        vector: randomVector(),
        metadata: { category: "A", score: 30, active: false },
      },
    ];
    await collection.insert(points);

    const filter = {
      op: "must",
      args: [
        { op: "key", args: ["category", { op: "eq", args: "A" }] },
        {
          op: "should",
          args: [
            { op: "key", args: ["score", { op: "gte", args: 20 }] },
            { op: "key", args: ["active", { op: "eq", args: true }] },
          ],
        },
      ],
    };

    const results = await collection.query(randomVector(), {
      limit: 10,
      filter,
    });
    expect(results.results.length).toBeGreaterThanOrEqual(1);
  });
});
