import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { EmbexClient, DataMigrator, Point } from "../../index"; // Assuming index exports DataMigrator
import * as fs from "fs";
import * as path from "path";
import * as os from "os";

// Helper to create temp dir
const createTempDir = () => {
  return fs.mkdtempSync(path.join(os.tmpdir(), "embex-test-"));
};

describe("Data Migrator and Scroll", () => {
  let sourceDbPath: string;
  let destDbPath: string;
  let sourceClient: EmbexClient;
  let destClient: EmbexClient;

  beforeAll(async () => {
    sourceDbPath = createTempDir();
    destDbPath = createTempDir();

    // Initialize source and destination (LanceDB)
    sourceClient = await EmbexClient.newAsync("lancedb", sourceDbPath);
    destClient = await EmbexClient.newAsync("lancedb", destDbPath);
  });

  afterAll(async () => {
    // Cleanup
    try {
      fs.rmSync(sourceDbPath, { recursive: true, force: true });
      fs.rmSync(destDbPath, { recursive: true, force: true });
    } catch (e) {
      console.error("Cleanup failed", e);
    }
  });

  it("should migrate data between databases", async () => {
    const srcColName = "products_src";
    const destColName = "products_dest";

    const srcCol = sourceClient.collection(srcColName);

    // Insert data into source
    // Note: LanceDB creates auto-schema on first insert if not exists
    const points: Point[] = [
      {
        id: "p1",
        vector: [0.1, 0.1, 0.1, 0.1],
        metadata: { type: "electronics" },
      },
      {
        id: "p2",
        vector: [0.2, 0.2, 0.2, 0.2],
        metadata: { type: "clothing" },
      },
      {
        id: "p3",
        vector: [0.3, 0.3, 0.3, 0.3],
        metadata: { type: "electronics" },
      },
    ];

    await srcCol.createAuto(4, "cosine");
    await srcCol.insert(points);

    // Verify source
    const srcRes = await srcCol.search([0.1, 0.1, 0.1, 0.1], 10);
    expect(srcRes.results.length).toBeGreaterThanOrEqual(3);

    // Test scroll explicitly
    const scrollRes = await srcCol.scroll(undefined, 2); // limit 2
    expect(scrollRes.points.length).toBe(2);
    expect(scrollRes.nextOffset).toBeDefined(); // Should have next offset

    // Migrate
    const migrator = new DataMigrator(sourceClient, destClient);

    // Simple migration
    const result = await migrator.migrateSimple(srcColName, destColName, 10);

    console.log("Migration result:", result);
    expect(result.pointsMigrated).toBe(3);

    // Verify destination
    const destCol = destClient.collection(destColName);
    const destRes = await destCol.search([0.1, 0.1, 0.1, 0.1], 10);
    expect(destRes.results.length).toBe(3);

    // Verify metadata
    const p1 = destRes.results.find((r) => r.id === "p1");
    expect(p1).toBeDefined();
    expect(p1?.metadata?.type).toBe("electronics");
  });
});
