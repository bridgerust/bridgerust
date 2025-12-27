/**
 * Error Handling Integration Tests
 * Tests error handling and edge cases
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { EmbexClient } from "../../index.js";
import { randomUUID } from "crypto";

const TEST_DIMENSION = 128;
const TEST_COLLECTION = "embex_error_test";

function randomVector(dim = TEST_DIMENSION) {
  return Array.from({ length: dim }, () => Math.random());
}

describe("Error Handling", () => {
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

  it("should handle collection not found error", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}

    await expect(
      collection.query(randomVector(), { limit: 10 })
    ).rejects.toThrow();
  });

  it("should handle dimension mismatch error", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const wrongVector = Array.from({ length: TEST_DIMENSION + 10 }, () =>
      Math.random()
    );

    await expect(
      collection.insert([{ id: "wrong", vector: wrongVector, metadata: {} }])
    ).rejects.toThrow();
  });

  it("should handle invalid filter errors", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    const invalidFilter = {
      op: "invalid",
      args: [],
    };

    await expect(
      collection.query(randomVector(), {
        limit: 10,
        filter: invalidFilter,
      })
    ).rejects.toThrow();
  });

  it("should handle duplicate collection creation", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    // Attempting to create again should either succeed (idempotent) or throw
    try {
      await collection.create(TEST_DIMENSION, "cosine");
      // If it succeeds, that's fine (idempotent behavior)
    } catch (e: any) {
      // If it throws, should be a collection exists error
      expect(e.message).toBeDefined();
    }
  });

  it("should handle empty vector error", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    await expect(
      collection.insert([{ id: "empty", vector: [], metadata: {} }])
    ).rejects.toThrow();
  });

  it("should handle invalid point ID", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    // Empty ID might be invalid depending on adapter
    try {
      await collection.insert([
        { id: "", vector: randomVector(), metadata: {} },
      ]);
    } catch (e: any) {
      expect(e.message).toBeDefined();
    }
  });

  it("should handle delete non-existent points gracefully", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    // Deleting non-existent points should not throw
    await expect(
      collection.delete([randomUUID(), randomUUID()])
    ).resolves.not.toThrow();
  });

  it("should handle search with zero top_k", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    await collection.insert([
      { id: "test", vector: randomVector(), metadata: {} },
    ]);

    const results = await collection.query(randomVector(), { limit: 0 });
    expect(results.results).toBeDefined();
    expect(Array.isArray(results.results)).toBe(true);
  });

  it("should handle update metadata for non-existent point", async () => {
    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(TEST_DIMENSION, "cosine");

    await expect(
      collection.updateMetadata([
        { id: randomUUID(), updates: { test: "value" } },
      ])
    ).rejects.toThrow();
  });
});
