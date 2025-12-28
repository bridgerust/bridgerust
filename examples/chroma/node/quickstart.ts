/**
 * Embex Quick Start - Chroma Provider
 *
 * Requires: Chroma server running (docker run -p 8000:8000 chromadb/chroma)
 * Or: Use Chroma in-memory mode (no server needed)
 * Run: npx tsx examples/chroma/node/quickstart.ts
 */

import { EmbexClient } from "@bridgerust/embex";

async function main() {
  console.log("🚀 Embex Quick Start - Chroma Provider\n");
  console.log(
    "📋 Option 1: Chroma server (docker run -p 8000:8000 chromadb/chroma)"
  );
  console.log("📋 Option 2: In-memory mode (no server needed)\n");

  // Chroma - can use server or in-memory
  // For server: url="http://localhost:8000"
  const url = process.env.CHROMA_URL || "http://localhost:8000";

  if (url === "http://localhost:8000") {
    console.log("⚠️  Using Chroma server mode");
    console.log("   Start server: docker run -p 8000:8000 chromadb/chroma\n");
  } else {
    console.log(`📋 Using Chroma at: ${url}\n`);
  }

  const client = new EmbexClient("chroma", url);

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

  console.log("\n🎉 Chroma quick start complete!");
}

main().catch(console.error);
