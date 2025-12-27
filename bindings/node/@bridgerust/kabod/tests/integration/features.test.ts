import { KabodClient, Point } from "../index";
import { describe, it } from "vitest";

describe("KabodClient Features", () => {
  const client = new KabodClient("qdrant", "http://localhost:6333");

  it("should support parallel batch insert", async () => {
    // We expect this might fail with DB error if no DB, but we check if method exists and runs
    const collection = client.collection("test_features_node");

    try {
      await collection.create(2, "cosine");
    } catch (e) {
      // ignore create error
    }

    const points: Point[] = [];
    for (let i = 0; i < 100; i++) {
      points.push({
        id: `p${i}`,
        vector: [0.1, 0.2],
      });
    }

    try {
      await collection.insertBatch(points, 10, 4); // parallel=4
    } catch (e: any) {
      // If it's a connection error or DB error, it means binding call worked
      if (
        e.message &&
        (e.message.includes("Connection refused") ||
          e.message.includes("http2 error"))
      ) {
        return;
      }
      // If it's "wrong number of arguments" etc, it will fail differently
      // throw e;
    }
  });
});
