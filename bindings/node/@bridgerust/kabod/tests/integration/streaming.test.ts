/**
 * Integration tests for streaming operations
 */
import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { KabodClient, Point } from "../../index";
import { randomUUID } from "crypto";

// Async generator that yields points
async function* generatePoints(count: number): AsyncGenerator<Point> {
  for (let i = 0; i < count; i++) {
    yield {
      id: randomUUID(),
      vector: [0.1, 0.2, 0.3, 0.4],
      metadata: { index: i },
    };
  }
}

describe("Streaming Operations", () => {
  const client = new KabodClient("qdrant", "http://localhost:6334");
  const collectionName = `stream_test_${randomUUID()}`;
  let collection;

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

  it("should insert points from async generator stream", async () => {
    await collection.insertStream(generatePoints(25), 5);

    // Wait for indexing
    await new Promise((resolve) => setTimeout(resolve, 1000));

    const results = await collection.search([0.1, 0.2, 0.3, 0.4], 50);

    expect(results.results.length).toBeGreaterThanOrEqual(25);
  });
});
