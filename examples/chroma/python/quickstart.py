"""
Embex Quick Start - Chroma Provider

Requires: Chroma server running (docker run -p 8000:8000 chromadb/chroma)
Or: Use Chroma in-memory mode (no server needed)
Run: python examples/chroma/python/quickstart.py
"""
import asyncio
import os
from embex import EmbexClient, Point


async def main():
    print("🚀 Embex Quick Start - Chroma Provider\n")
    print("📋 Option 1: Chroma server (docker run -p 8000:8000 chromadb/chroma)")
    print("📋 Option 2: In-memory mode (no server needed)\n")
    
    # Chroma - can use server or in-memory
    # For server: url="http://localhost:8000"
    # For in-memory: url=":memory:" (if supported) or use local path
    url = os.getenv("CHROMA_URL", "http://localhost:8000")
    
    if url == "http://localhost:8000":
        print("⚠️  Using Chroma server mode")
        print("   Start server: docker run -p 8000:8000 chromadb/chroma\n")
    else:
        print(f"📋 Using Chroma at: {url}\n")
    
    client = EmbexClient(provider="chroma", url=url)
    
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
    
    print("\n🎉 Chroma quick start complete!")


if __name__ == "__main__":
    asyncio.run(main())

