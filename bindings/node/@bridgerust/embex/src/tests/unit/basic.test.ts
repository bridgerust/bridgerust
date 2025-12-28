/**
 * Unit tests for basic EmbexClient functionality
 */
import { describe, it, expect } from "vitest";
import { EmbexClient } from "../../../index";

describe("EmbexClient - Basic", () => {
  it("should be instantiable with valid config", () => {
    // Client creation doesn't connect immediately, so this should work
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    expect(client).toBeDefined();
    expect(client.collection).toBeDefined();
  });

  it("should create a collection instance", () => {
    const client = new EmbexClient("qdrant", "http://localhost:6334", null);
    const collection = client.collection("test_col");
    expect(collection).toBeDefined();
    expect(collection.insert).toBeDefined();
    expect(collection.search).toBeDefined();
    expect(collection.query).toBeDefined();
  });
});
