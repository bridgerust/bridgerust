"""
Embex Quick Start - Qdrant Provider

Requires: Qdrant server running (docker run -p 6333:6333 qdrant/qdrant)
Run: python examples/qdrant/python/quickstart.py
"""
import asyncio
from embex import EmbexClient, Point


async def main():
    print("🚀 Embex Quick Start - Qdrant Provider\n")
    print("📋 Prerequisites: Qdrant server running at http://localhost:6333")
    print("   Start with: docker run -p 6333:6333 qdrant/qdrant\n")
    
    # Qdrant - requires server running
    client = EmbexClient(provider="qdrant", url="http://localhost:6333")
    
    collection_name = "documents"
    collection = client.collection(collection_name)
    
    # Clean up if exists
    try:
        await collection.delete_collection()
        print(f"✅ Cleaned up existing collection: {collection_name}")
    except Exception:
        pass
    
    # 1. Create Collection
    print(f"\n📦 Creating collection: {collection_name}")
    await collection.create(dimension=768, distance="cosine")
    print("   ✅ Collection created!")
    
    # 2. Insert Data
    print("\n📝 Inserting documents...")
    points = [
        Point(id="1", vector=[0.1] * 768, metadata={"title": "Hello World"}),
        Point(id="2", vector=[0.2] * 768, metadata={"title": "Embex is Fast"}),
        Point(id="3", vector=[0.15] * 768, metadata={"title": "Rust Powered"}),
    ]
    await collection.insert(points)
    print(f"   ✅ Inserted {len(points)} documents!")
    
    # 3. Search
    print("\n🔍 Searching...")
    results = await collection.search(vector=[0.12] * 768, top_k=2)
    print(f"   ✅ Found {len(results.results)} results:\n")
    for i, result in enumerate(results.results, 1):
        print(f"   {i}. {result.metadata.get('title', 'N/A')} (Score: {result.score:.4f})")
    
    print("\n🎉 Qdrant quick start complete!")
    print("\n💡 Next: Try examples/python/semantic_search.py for a real-world example")


if __name__ == "__main__":
    asyncio.run(main())

