/**
 * Embex Quick Start - LanceDB Provider
 *
 * Works with ZERO setup! LanceDB is embedded - no server required.
 * Run: npx tsx examples/lancedb/node/quickstart.ts
 */

import { EmbexClient } from "@bridgerust/embex";

async function main() {
  console.log("🚀 Embex Quick Start - Zero Setup Required!\n");

  // LanceDB is embedded - just needs a local directory path
  // No server, no Docker, no setup needed!
  const dbPath = "./data/embex_quickstart";
  const client = await EmbexClient.newAsync("lancedb", dbPath);

  const collectionName = "documents";
  const collection = client.collection(collectionName);

  // Clean up if exists (for re-running)
  try {
    await collection.deleteCollection();
    console.log(`✅ Cleaned up existing collection: ${collectionName}`);
  } catch (e) {
    // Collection doesn't exist yet
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
      vector: Array(768).fill(0.1), // 768-dimensional vector
      metadata: { title: "Hello World", category: "greeting" },
    },
    {
      id: "2",
      vector: Array(768).fill(0.2),
      metadata: { title: "Embex is Fast", category: "tech" },
    },
    {
      id: "3",
      vector: Array(768).fill(0.15),
      metadata: { title: "Rust Powered", category: "tech" },
    },
  ];
  await collection.insert(points);
  console.log(`   ✅ Inserted ${points.length} documents!`);

  // 3. Search
  console.log("\n🔍 Searching for similar documents...");
  const queryVector = Array(768).fill(0.12); // Query vector
  const results = await collection.search(queryVector, 2);

  console.log(`   ✅ Found ${results.results.length} results:\n`);
  results.results.forEach((result, i) => {
    console.log(`   ${i + 1}. ${result.metadata?.title || "N/A"}`);
    console.log(`      Score: ${result.score.toFixed(4)}`);
    console.log(`      Category: ${result.metadata?.category || "N/A"}\n`);
  });

  // 4. Cleanup (optional)
  console.log("🧹 Cleaning up...");
  await collection.deleteCollection();
  console.log("   ✅ Done! Collection deleted.\n");

  console.log("🎉 Quick start complete! Embex is working perfectly.");
  console.log("\n💡 Next steps:");
  console.log("   - Try with other providers: Qdrant, Pinecone, Chroma, etc.");
  console.log(
    "   - Check out examples/node/rag_system.ts for a real-world example"
  );
  console.log(
    "   - Read the docs: https://github.com/bridgerust/bridgerust/tree/main/docs"
  );
}

main().catch(console.error);
