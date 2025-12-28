import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { EmbexClient } from "../../index";
import { randomUUID } from "crypto";

const TEST_COLLECTION = "embex_migration_test_node";
const MIGRATION_VERSION = randomUUID();

describe("Migration Adapter", () => {
  let client: EmbexClient;

  beforeAll(() => {
    // Using Qdrant for migration tests as it's reliable
    client = new EmbexClient("qdrant", "http://localhost:6334");
  });

  afterAll(async () => {
    try {
      const collection = client.collection(TEST_COLLECTION);
      await collection.deleteCollection();
    } catch (e) {}
  });

  it("should run migrations successfully", async () => {
    // 1. Define migration to create a collection
    const migrations = [
      {
        version: MIGRATION_VERSION,
        operations: [
          {
            type: "create_collection",
            schema: {
              name: TEST_COLLECTION,
              dimension: 128,
              metric: "cosine",
            },
          },
        ],
        downOperations: [
          {
            type: "delete_collection",
            name: TEST_COLLECTION,
          },
        ],
      },
    ];

    // 2. Run migrations
    await client.runMigrations(migrations);

    // 3. Verify collection exists by trying to insert/query
    const collection = client.collection(TEST_COLLECTION);
    const cv = Array.from({ length: 128 }, () => Math.random());

    await collection.insert([
      { id: randomUUID(), vector: cv, metadata: { test: "migration" } },
    ]);

    const results = await collection.query(cv, { limit: 1 });
    expect(results.results.length).toBeGreaterThan(0);
  });
});
