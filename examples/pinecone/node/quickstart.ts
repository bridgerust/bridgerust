/**
 * Embex Quick Start - Pinecone Provider
 *
 * Requires: Pinecone API key and index name
 * Set: export PINECONE_API_KEY="your-api-key"
 * Run: npx tsx examples/pinecone/node/quickstart.ts
 */

import { EmbexClient } from "@bridgerust/embex";

async function main() {
  console.log("🚀 Embex Quick Start - Pinecone Provider\n");

  // Get API key from environment
  const apiKey = process.env.PINECONE_API_KEY;
  if (!apiKey) {
    console.error("❌ Error: PINECONE_API_KEY environment variable not set");
    console.error("   Set it with: export PINECONE_API_KEY='your-api-key'");
    process.exit(1);
  }

  // Pinecone - requires API key
  // URL format: https://<index-name>-<project-id>.svc.<environment>.pinecone.io
  // Or use: https://api.pinecone.io for serverless
  const indexName = process.env.PINECONE_INDEX_NAME || "embex-quickstart";
  const url = `https://${indexName}.svc.pinecone.io`; // Adjust based on your Pinecone setup

  console.log(`📋 Connecting to Pinecone index: ${indexName}\n`);

  const client = new EmbexClient("pinecone", url, apiKey);

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

  console.log("\n🎉 Pinecone quick start complete!");
  console.log("\n💡 Note: Pinecone is serverless - no local setup needed!");
}

main().catch(console.error);
