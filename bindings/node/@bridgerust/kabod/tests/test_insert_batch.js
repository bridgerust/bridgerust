const { KabodClient } = require("../index.js");
const crypto = require("crypto");

async function testInsertBatch() {
  console.log("Testing insertBatch...");

  const client = new KabodClient("qdrant", "http://localhost:6334");

  try {
    const col = client.collection("batch_test_collection");
    await col.deleteCollection();
  } catch (e) {}

  const collection = client.collection("batch_test_collection");
  await collection.create(4, "cosine");

  const points = [];
  const expectedIds = [];
  for (let i = 0; i < 50; i++) {
    const id = crypto.randomUUID();
    expectedIds.push(id);
    points.push({
      id: id,
      vector: [0.1, 0.2, 0.3, 0.4],
      metadata: { index: i },
    });
  }

  await collection.insertBatch(points, 10);

  await new Promise((resolve) => setTimeout(resolve, 1000));

  const results = await collection.search([0.1, 0.2, 0.3, 0.4], 100);

  console.log(`Inserted ${points.length} points, found ${results.length}`);

  if (results.length !== 50) {
    throw new Error(`Expected 50 results, got ${results.length}`);
  }

  const foundIds = new Set(results.map((r) => r.id));
  for (const id of expectedIds) {
    if (!foundIds.has(id)) {
      throw new Error(`Missing ID: ${id}`);
    }
  }

  console.log("All IDs verified!");

  await collection.deleteCollection();

  console.log("insertBatch test PASSED!");
}

testInsertBatch().catch((err) => {
  console.error("Test failed:", err);
  process.exit(1);
});
