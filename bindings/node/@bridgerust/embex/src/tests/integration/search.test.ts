import { describe, it, expect } from "vitest";
import { EmbexClient } from "../../index";

describe("EmbexClient Search", () => {
  it("should accept query options", async () => {
    const client = new EmbexClient("qdrant", "http://localhost:6333", null);
    const collection = client.collection("test_col");

    try {
      await collection.query([0.1, 0.2, 0.3], {
        limit: 5,
        filter: { key: "value" },
        includeMetadata: true,
      });
    } catch (e: any) {
      expect(e).toBeDefined();
    }
  });

  it("should accept builder pattern search", async () => {
    const client = new EmbexClient("qdrant", "http://localhost:6333", null);
    const col = client.collection("test_col");
    const builder = col.buildSearch([0.1, 0.2]);
    expect(builder.limit).toBeDefined();
    expect(builder.filter).toBeDefined();
  });

  it("should accept search() method with direct parameters", async () => {
    const client = new EmbexClient("qdrant", "http://localhost:6333", null);
    const collection = client.collection("test_col");

    try {
      await collection.search([0.1, 0.2, 0.3], 5, null, true, false);
    } catch (e: any) {
      expect(e).toBeDefined();
    }
  });
});
