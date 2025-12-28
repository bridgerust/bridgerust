"""
Embex Quick Start - Pinecone Provider

Requires: Pinecone API key and index name
Set: export PINECONE_API_KEY="your-api-key"
Run: python examples/pinecone/python/quickstart.py
"""
import asyncio
import os
from embex import EmbexClient, Point


async def main():
    print("🚀 Embex Quick Start - Pinecone Provider\n")
    
    # Get API key from environment
    api_key = os.getenv("PINECONE_API_KEY")
    if not api_key:
        print("❌ Error: PINECONE_API_KEY environment variable not set")
        print("   Set it with: export PINECONE_API_KEY='your-api-key'")
        return
    
    # Pinecone - requires API key
    # URL format: https://<index-name>-<project-id>.svc.<environment>.pinecone.io
    # Or use: https://api.pinecone.io for serverless
    index_name = os.getenv("PINECONE_INDEX_NAME", "embex-quickstart")
    url = f"https://{index_name}.svc.pinecone.io"  # Adjust based on your Pinecone setup
    
    print(f"📋 Connecting to Pinecone index: {index_name}\n")
    
    client = EmbexClient(provider="pinecone", url=url, api_key=api_key)
    
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
    
    print("\n🎉 Pinecone quick start complete!")
    print("\n💡 Note: Pinecone is serverless - no local setup needed!")


if __name__ == "__main__":
    asyncio.run(main())

