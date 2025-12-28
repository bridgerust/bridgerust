/**
 * Embex Quick Start - Qdrant Provider
 *
 * Requires: Qdrant server running (docker run -p 6333:6333 qdrant/qdrant)
 * Run: npx tsx examples/qdrant/node/quickstart.ts
 */

import { EmbexClient } from "@bridgerust/embex";

async function main() {
  console.log("🚀 Embex Quick Start - Qdrant Provider\n");
  console.log(
    "📋 Prerequisites: Qdrant server running at http://localhost:6333"
  );
  console.log("   Start with: docker run -p 6333:6333 qdrant/qdrant\n");

  // Qdrant - requires server running
  const client = new EmbexClient("qdrant", "http://localhost:6333");

  const collectionName = "documents";
  const collection = client.collection(collectionName);

  // Clean up if exists
  try {
    await collection.deleteCollection();
    console.log(`✅ Cleaned up existing collection: ${collectionName}`);
  } catch (e) {
    // Collection doesn't exist
  }

  // 1. Create Collection
  console.log(`\n📦 Creating collection: ${collectionName}`);
  await collection.create(768, "cosine");
  console.log("   ✅ Collection created!");

  // 2. Insert Data
  console.log("\n📝 Inserting documents...");
  const points = [
    {
      id: "1",
      vector: Array(768).fill(0.1),
      metadata: { title: "Hello World" },
    },
    {
      id: "2",
      vector: Array(768).fill(0.2),
      metadata: { title: "Embex is Fast" },
    },
    {
      id: "3",
      vector: Array(768).fill(0.15),
      metadata: { title: "Rust Powered" },
    },
  ];
  await collection.insert(points);
  console.log(`   ✅ Inserted ${points.length} documents!`);

  // 3. Search
  console.log("\n🔍 Searching...");
  const results = await collection.search(Array(768).fill(0.12), 2);
  console.log(`   ✅ Found ${results.results.length} results:\n`);
  results.results.forEach((result, i) => {
    console.log(
      `   ${i + 1}. ${
        result.metadata?.title || "N/A"
      } (Score: ${result.score.toFixed(4)})`
    );
  });

  console.log("\n🎉 Qdrant quick start complete!");
  console.log(
    "\n💡 Next: Try examples/node/rag_system.ts for a real-world example"
  );
}

main().catch(console.error);
