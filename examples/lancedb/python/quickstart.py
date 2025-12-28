"""
Embex Quick Start - LanceDB Provider

Works with ZERO setup! LanceDB is embedded - no server required.
Run: python examples/lancedb/python/quickstart.py
"""
import asyncio
from embex import EmbexClient, Point


async def main():
    print("🚀 Embex Quick Start - Zero Setup Required!\n")
    
    # LanceDB is embedded - just needs a local directory path
    # No server, no Docker, no setup needed!
    db_path = "./data/embex_quickstart"
    client = await EmbexClient.new_async(provider="lancedb", url=db_path)
    
    collection_name = "documents"
    collection = client.collection(collection_name)
    
    # Clean up if exists (for re-running)
    try:
        await collection.delete_collection()
        print(f"✅ Cleaned up existing collection: {collection_name}")
    except Exception:
        pass  # Collection doesn't exist yet
    
    # 1. Create Collection
    print(f"\n📦 Creating collection: {collection_name}")
    await collection.create(dimension=768, distance="cosine")
    print("   ✅ Collection created!")
    
    # 2. Insert Data
    print("\n📝 Inserting documents...")
    points = [
        Point(
            id="1",
            vector=[0.1] * 768,  # 768-dimensional vector
            metadata={"title": "Hello World", "category": "greeting"}
        ),
        Point(
            id="2",
            vector=[0.2] * 768,
            metadata={"title": "Embex is Fast", "category": "tech"}
        ),
        Point(
            id="3",
            vector=[0.15] * 768,
            metadata={"title": "Rust Powered", "category": "tech"}
        ),
    ]
    await collection.insert(points)
    print(f"   ✅ Inserted {len(points)} documents!")
    
    # 3. Search
    print("\n🔍 Searching for similar documents...")
    query_vector = [0.12] * 768  # Query vector
    results = await collection.search(vector=query_vector, top_k=2)
    
    print(f"   ✅ Found {len(results.results)} results:\n")
    for i, result in enumerate(results.results, 1):
        print(f"   {i}. {result.metadata.get('title', 'N/A')}")
        print(f"      Score: {result.score:.4f}")
        print(f"      Category: {result.metadata.get('category', 'N/A')}\n")
    
    # 4. Cleanup (optional)
    print("🧹 Cleaning up...")
    await collection.delete_collection()
    print("   ✅ Done! Collection deleted.\n")
    
    print("🎉 Quick start complete! Embex is working perfectly.")
    print("\n💡 Next steps:")
    print("   - Try with other providers: Qdrant, Pinecone, Chroma, etc.")
    print("   - Check out examples/python/semantic_search.py for a real-world example")
    print("   - Read the docs: https://github.com/bridgerust/bridgerust/tree/main/docs")


if __name__ == "__main__":
    asyncio.run(main())

