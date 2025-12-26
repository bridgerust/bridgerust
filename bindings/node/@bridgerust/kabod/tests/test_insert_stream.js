/**
 * Test for insertStream functionality
 */
const { KabodClient } = require("../kabod.js");
const crypto = require("crypto");

// Async generator that yields points
async function* generatePoints(count) {
  for (let i = 0; i < count; i++) {
    yield {
      id: crypto.randomUUID(),
      vector: [0.1, 0.2, 0.3, 0.4],
      metadata: { index: i },
    };
  }
}

async function testInsertStream() {
  console.log("Testing insertStream...");

  const client = new KabodClient("qdrant", "http://localhost:6334");

  // Clean up if exists
  try {
    const col = client.collection("stream_test_collection");
    await col.deleteCollection();
  } catch (e) {
    // Ignore
  }

  const collection = client.collection("stream_test_collection");
  await collection.create(4, "cosine");

  // Insert using async generator with batch size 5
  await collection.insertStream(generatePoints(25), 5);

  // Wait for indexing
  await new Promise((resolve) => setTimeout(resolve, 1000));

  // Search and verify
  const results = await collection.search([0.1, 0.2, 0.3, 0.4], 50);

  console.log(`Inserted 25 points via stream, found ${results.length}`);

  if (results.length !== 25) {
    throw new Error(`Expected 25 results, got ${results.length}`);
  }

  console.log("insertStream test PASSED!");

  // Cleanup
  await collection.deleteCollection();
}

testInsertStream().catch((err) => {
  console.error("Test failed:", err);
  process.exit(1);
});
