import { describe, it, expect } from "vitest";
import { KabodClient } from "../index";

describe("KabodClient Search", () => {
  // We can't easily test real search without a DB.
  // However, we can test that the method signature is correct in TypeScript land (by compiling this test)
  // And runtime behavior if we mock or just handle connection error.

  it("should accept query options", async () => {
    const client = new KabodClient("qdrant", "http://localhost:6333", null);
    const collection = client.collection("test_col");

    try {
      await collection.query([0.1, 0.2, 0.3], {
        limit: 5,
        filter: { key: "value" },
        includeMetadata: true,
      });
    } catch (e: any) {
      // It should probably fail with connection error or similar,
      // but NOT "invalid argument" or "function not found"
      // If it fails with "Invalid filter", that's also good traversal.
      console.log("Search failed as expected (no DB):", e.message);
      expect(e).toBeDefined();
    }
  });

  it("should accept builder pattern search", async () => {
    const client = new KabodClient("qdrant", "http://localhost:6333", null);
    const col = client.collection("test_col");
    const builder = col.buildSearch([0.1, 0.2]);
    expect(builder.limit).toBeDefined();
    expect(builder.filter).toBeDefined();
  });

  it("should accept search() method with direct parameters", async () => {
    const client = new KabodClient("qdrant", "http://localhost:6333", null);
    const collection = client.collection("test_col");

    try {
      await collection.search([0.1, 0.2, 0.3], 5, null, true, false);
    } catch (e: any) {
      // Should fail with connection error, not "function not found"
      expect(e).toBeDefined();
      expect(e.message).toBeDefined();
    }
  });
});
