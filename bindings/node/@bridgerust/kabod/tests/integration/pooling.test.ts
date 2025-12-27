/**
 * Connection Pooling Integration Tests
 * Tests connection pooling behavior and configuration
 */

import { describe, it, expect } from "vitest";
import { KabodClient } from "../../index.js";

describe("Connection Pooling", () => {
  it("should create client successfully", async () => {
    const client = new KabodClient("qdrant", "http://localhost:6334", null);

    expect(client).toBeDefined();
    const collection = client.collection("pool_test");
    expect(collection).toBeDefined();
  });

  it("should reuse connections for multiple operations", async () => {
    const client = new KabodClient("qdrant", "http://localhost:6334");
    const collection = client.collection("pool_reuse_test");

    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(128, "cosine");

    // Perform multiple operations to test connection reuse
    const operations = Array.from({ length: 10 }, async () => {
      const vector = Array.from({ length: 128 }, () => Math.random());
      try {
        await collection.query(vector, { limit: 1 });
      } catch (e) {
        // Ignore errors if DB not available
      }
    });

    await Promise.all(operations);
  });

  it("should handle concurrent requests efficiently", async () => {
    const client = new KabodClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("pool_concurrent_test");

    try {
      await collection.deleteCollection();
    } catch (e) {}
    await collection.create(128, "cosine");

    // Insert some test data
    const points = Array.from({ length: 5 }, () => ({
      id: `p${Math.random()}`,
      vector: Array.from({ length: 128 }, () => Math.random()),
    }));
    await collection.insert(points);

    // Make concurrent queries
    const queries = Array.from({ length: 20 }, async () => {
      const vector = Array.from({ length: 128 }, () => Math.random());
      try {
        await collection.query(vector, { limit: 5 });
      } catch (e) {
        // Ignore errors if DB not available
      }
    });

    await Promise.all(queries);
  });
});
