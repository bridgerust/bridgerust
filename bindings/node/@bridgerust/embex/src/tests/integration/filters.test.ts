/**
 * Filter Integration Tests
 * Tests comprehensive filter functionality across adapters
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { EmbexClient } from "../../index";
import { randomUUID } from "crypto";

const TEST_DIMENSION = 128;
const TEST_COLLECTION = "embex_filter_test";

function randomVector(dim = TEST_DIMENSION) {
  return Array.from({ length: dim }, () => Math.random());
}

describe("Filter Operations", () => {
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

  it("should filter by equality", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const points = [
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { status: "active" },
      },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { status: "inactive" },
      },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { status: "active" },
      },
    ];
    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["status", { eq: "active" }],
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
      { id: randomUUID(), vector: randomVector(), metadata: { score: 10 } },
      { id: randomUUID(), vector: randomVector(), metadata: { score: 20 } },
      { id: randomUUID(), vector: randomVector(), metadata: { score: 30 } },
    ];
    await collection.insert(points);

    // Test greater than
    const gtFilter = {
      op: "key",
      args: ["score", { gt: 15 }],
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
      args: ["score", { lte: 20 }],
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
      { id: randomUUID(), vector: randomVector(), metadata: { category: "A" } },
      { id: randomUUID(), vector: randomVector(), metadata: { category: "B" } },
      { id: randomUUID(), vector: randomVector(), metadata: { category: "C" } },
      { id: randomUUID(), vector: randomVector(), metadata: { category: "A" } },
    ];
    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["category", { in: ["A", "B"] }],
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
      { id: randomUUID(), vector: randomVector(), metadata: { type: "x" } },
      { id: randomUUID(), vector: randomVector(), metadata: { type: "y" } },
      { id: randomUUID(), vector: randomVector(), metadata: { type: "z" } },
    ];
    await collection.insert(points);

    const filter = {
      op: "key",
      args: ["type", { not_in: ["x", "y"] }],
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
        id: randomUUID(),
        vector: randomVector(),
        metadata: { status: "active", score: 20 },
      },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { status: "active", score: 10 },
      },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { status: "inactive", score: 20 },
      },
    ];
    await collection.insert(points);

    const filter = {
      op: "must",
      args: [
        { op: "key", args: ["status", { eq: "active" }] },
        { op: "key", args: ["score", { gte: 15 }] },
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
      { id: randomUUID(), vector: randomVector(), metadata: { tag: "red" } },
      { id: randomUUID(), vector: randomVector(), metadata: { tag: "blue" } },
      { id: randomUUID(), vector: randomVector(), metadata: { tag: "green" } },
    ];
    await collection.insert(points);

    const filter = {
      op: "should",
      args: [
        { op: "key", args: ["tag", { eq: "red" }] },
        { op: "key", args: ["tag", { eq: "blue" }] },
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
      { id: randomUUID(), vector: randomVector(), metadata: { visible: true } },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { visible: false },
      },
      { id: randomUUID(), vector: randomVector(), metadata: { visible: true } },
    ];
    await collection.insert(points);

    const filter = {
      op: "must_not",
      args: [{ op: "key", args: ["visible", { eq: false }] }],
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
        id: randomUUID(),
        vector: randomVector(),
        metadata: { category: "A", score: 25, active: true },
      },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { category: "B", score: 15, active: true },
      },
      {
        id: randomUUID(),
        vector: randomVector(),
        metadata: { category: "A", score: 30, active: false },
      },
    ];
    await collection.insert(points);

    const filter = {
      op: "must",
      args: [
        { op: "key", args: ["category", { eq: "A" }] },
        {
          op: "should",
          args: [
            { op: "key", args: ["score", { gte: 20 }] },
            { op: "key", args: ["active", { eq: true }] },
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
