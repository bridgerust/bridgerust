import { describe, it, expect } from "vitest";
import { KabodClient } from "../index";

describe("KabodClient", () => {
  it("should be instantiable with valid config", () => {
    // Assuming "qdrant" is a valid provider and it doesn't connect immediately or we can instantiate it.
    // If it connects immediately, this might fail without a mock.
    // But for now, we test the binding availability.
    const client = new KabodClient("qdrant", "http://localhost:6333", null);
    expect(client).toBeDefined();
    expect(client.collection).toBeDefined();
  });

  it("should create a collection instance", () => {
    const client = new KabodClient("qdrant", "http://localhost:6333", null);
    const collection = client.collection("test_col");
    expect(collection).toBeDefined();
    expect(collection.insert).toBeDefined();
    expect(collection.search).toBeDefined();
    expect(collection.query).toBeDefined();
  });
});
