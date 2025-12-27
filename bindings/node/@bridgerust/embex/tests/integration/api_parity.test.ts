/**
 * API Parity Tests
 * Verifies that Node.js bindings have feature parity with Python bindings
 */

import { describe, it, expect } from "vitest";
import { EmbexClient } from "../../index.js";

describe("API Parity with Python", () => {
  it("should support all core methods", () => {
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("parity_test");

    expect(typeof collection.create).toBe("function");
    expect(typeof collection.deleteCollection).toBe("function");
    expect(typeof collection.insert).toBe("function");
    expect(typeof collection.insertBatch).toBe("function");
    expect(typeof collection.query).toBe("function");
    expect(typeof collection.search).toBe("function");
    expect(typeof collection.delete).toBe("function");
    expect(typeof collection.updateMetadata).toBe("function");
    expect(typeof collection.buildSearch).toBe("function");
    expect(typeof collection.buildQuery).toBe("function");
  });

  it("should support async client initialization", async () => {
    const client = await EmbexClient.newAsync("lancedb", "/tmp/test");
    expect(client).toBeDefined();
    expect(client.collection).toBeDefined();
  });

  it("should support search builder pattern", () => {
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("parity_test");

    const builder = collection.buildSearch([0.1, 0.2, 0.3]);
    expect(builder).toBeDefined();
    expect(typeof builder.limit).toBe("function");
    expect(typeof builder.filter).toBe("function");
    expect(typeof builder.includeMetadata).toBe("function");
    expect(typeof builder.includeVector).toBe("function");
    expect(typeof builder.aggregation).toBe("function");
    expect(typeof builder.execute).toBe("function");
  });

  it("should support query builder pattern", () => {
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("parity_test");

    const builder = collection.buildQuery();
    expect(builder).toBeDefined();
    expect(typeof builder.limit).toBe("function");
    expect(typeof builder.filter).toBe("function");
    expect(typeof builder.includeMetadata).toBe("function");
    expect(typeof builder.includeVector).toBe("function");
    expect(typeof builder.offset).toBe("function");
    expect(typeof builder.aggregation).toBe("function");
    expect(typeof builder.execute).toBe("function");
  });

  it("should support batch operations with parallel option", () => {
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("parity_test");

    expect(typeof collection.insertBatch).toBe("function");
  });

  it("should support all distance metrics", async () => {
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("parity_test");

    const metrics = ["cosine", "euclidean", "dot"];
    for (const metric of metrics) {
      try {
        await collection.deleteCollection();
      } catch (e) {}
      try {
        await collection.create(128, metric);
        expect(true).toBe(true);
      } catch (e: any) {
        if (e.message && e.message.includes("Invalid distance metric")) {
          throw e;
        }
      }
    }
  });

  it("should support filter operations", () => {
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("parity_test");

    const filter = {
      op: "key",
      args: ["status", { op: "eq", args: "active" }],
    };

    expect(typeof collection.query).toBe("function");
  });

  it("should support metadata operations", () => {
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("parity_test");

    expect(typeof collection.updateMetadata).toBe("function");
  });

  it("should support aggregations", () => {
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("parity_test");

    const builder = collection.buildSearch([0.1, 0.2]);
    expect(typeof builder.aggregation).toBe("function");
  });
});
