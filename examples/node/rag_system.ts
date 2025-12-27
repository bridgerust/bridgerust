import { KabodClient } from "@bridgerust/kabod";

// Mock LLM function
async function generateAnswer(
  context: string,
  question: string
): Promise<string> {
  console.log("LLM generating answer...");
  return `Based on "${context}", the answer to "${question}" is: ...`;
}

// Mock Embedding function
function getEmbedding(text: string): number[] {
  return Array(768)
    .fill(0)
    .map(() => Math.random());
}

async function main() {
  // 1. Initialize Client
  const client = new KabodClient("qdrant", "http://localhost:6333");

  const collectionName = "knowledge_base";

  // 2. Setup knowledge base
  try {
    await client.collection(collectionName).create(768, "cosine");
    console.log(`Created collection: ${collectionName}`);
  } catch (e) {
    // Assume exists
  }

  const collection = client.collection(collectionName);

  // 3. Ingest Data
  const documents = [
    {
      id: "1",
      content:
        "Rust is a systems programming language that runs blazingly fast.",
      source: "rust-lang.org",
    },
    {
      id: "2",
      content: "Kabod is a vector ORM for Rust.",
      source: "bridgerust.dev",
    },
  ];

  const points = documents.map((doc) => ({
    id: doc.id,
    vector: getEmbedding(doc.content),
    metadata: {
      content: doc.content,
      source: doc.source,
    },
  }));

  await collection.insert(points);
  console.log(`Ingested ${points.length} documents.`);

  // 4. RAG Flow
  const question = "What is Kabod?";
  console.log(`\nUser asks: "${question}"`);

  const queryVector = getEmbedding(question);

  // Retrieve relevant context using search builder with aggregations
  const searchBuilder = collection.buildSearch(queryVector);
  const data = await searchBuilder
    .limit(3)
    .includeMetadata(true)
    .aggregation("count")
    .execute();

  console.log(
    `\nTotal documents in knowledge base: ${data.aggregations.count}`
  );

  if (data.results.length > 0) {
    const topResult = data.results[0];
    const context = topResult.metadata?.content as string;

    console.log(
      `Retrieved Context: "${context}" (Score: ${topResult.score.toFixed(4)})`
    );

    // Generate Answer
    const answer = await generateAnswer(context, question);
    console.log(`Answer: ${answer}`);

    // 5. Update metadata to track access
    await collection.updateMetadata([
      {
        id: topResult.id,
        updates: {
          last_accessed: new Date().toISOString(),
          access_count: ((topResult.metadata?.access_count as number) || 0) + 1,
        },
      },
    ]);
    console.log(`\nUpdated access metadata for document ${topResult.id}`);
  } else {
    console.log("No relevant context found.");
  }

  // 6. Query without vector search (filter-only)
  console.log("\nQuerying documents by source using buildQuery:");
  const queryResults = await collection
    .buildQuery()
    .filter({
      op: "key",
      args: ["source", { op: "eq", args: "bridgerust.dev" }],
    })
    .limit(10)
    .includeMetadata(true)
    .aggregation("count")
    .execute();

  console.log(
    `Found ${queryResults.aggregations.count} documents from bridgerust.dev`
  );
  for (const result of queryResults.results) {
    console.log(`  - ${result.id}: ${result.metadata?.content}`);
  }
}

main().catch(console.error);
