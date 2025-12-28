/**
 * Integration tests for batch operations
 */
import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { Collection, EmbexClient, Point } from "../../index";
import { randomUUID } from "crypto";

describe("Batch Operations", () => {
  const client = new EmbexClient("qdrant", "http://localhost:6334");
  const collectionName = `batch_test_${randomUUID()}`;
  let collection: Collection;

  beforeAll(async () => {
    collection = client.collection(collectionName);
    try {
      await collection.deleteCollection();
    } catch (e) {
      // Ignore if doesn't exist
    }
    await collection.create(4, "cosine");
  });

  afterAll(async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {
      // Ignore cleanup errors
    }
  });

  it("should insert points in batches", async () => {
    const points: Point[] = [];
    const expectedIds: string[] = [];

    for (let i = 0; i < 50; i++) {
      const id = randomUUID();
      expectedIds.push(id);
      points.push({
        id,
        vector: [0.1, 0.2, 0.3, 0.4],
        metadata: { index: i },
      });
    }

    await collection.insertBatch(points, 10);

    // Wait for indexing
    await new Promise((resolve) => setTimeout(resolve, 1000));

    const results = await collection.search([0.1, 0.2, 0.3, 0.4], 100);

    expect(results.results.length).toBeGreaterThanOrEqual(50);

    const foundIds = new Set(results.results.map((r) => r.id));
    for (const id of expectedIds) {
      expect(foundIds.has(id)).toBe(true);
    }
  });
});
