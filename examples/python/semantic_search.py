import asyncio
import random
from typing import List
from kabod import KabodClient, Point

# Mock embedding function (in a real app, use OpenAI/HuggingFace)
def get_embedding(text: str) -> List[float]:
    # Deterministic mock embedding for demo purposes
    random.seed(text)
    return [random.random() for _ in range(768)]

async def main():
    # 1. Initialize Client (using Qdrant as backend for this example)
    # Ensure you have a Qdrant instance running at localhost:6333
    client = KabodClient(provider="qdrant", url="http://localhost:6333")
    
    collection_name = "semantic_books"
    
    # 2. Cleanup existing collection if any
    try:
        print(f"Deleting existing collection: {collection_name}")
        await client.delete_collection()  # Note: Client.delete_collection might need name arg or Collection object
        # Wait, the API I implemented on Collection is delete_collection() which deletes *that* collection.
        # But here I don't have a collection object yet.
        # Let's verify the API in lib.rs.
        # Collection::delete_collection -> calls inner.delete_collection() which uses self.name.
        # So I need to get a collection object first.
    except Exception:
        pass # Collection might not exist

    collection = client.collection(collection_name)

    # 3. Create Collection
    print(f"Creating collection: {collection_name}")
    await collection.create(dimension=768, distance="cosine")

    # 4. Prepare Data
    books = [
        {"id": "1", "title": "The Rust Programming Language", "author": "Steve Klabnik", "tags": ["rust", "tech"]},
        {"id": "2", "title": "The Great Gatsby", "author": "F. Scott Fitzgerald", "tags": ["fiction", "classic"]},
        {"id": "3", "title": "Effective Python", "author": "Brett Slatkin", "tags": ["python", "tech"]},
        {"id": "4", "title": "1984", "author": "George Orwell", "tags": ["fiction", "dystopia"]},
    ]

    points = []
    for book in books:
        vector = get_embedding(book["title"])
        points.append(Point(
            id=book["id"],
            vector=vector,
            metadata=book # Python dict as metadata
        ))

    # 5. Insert Data
    print(f"Inserting {len(points)} books...")
    await collection.insert(points)

    # 6. Search
    query_text = "programming book"
    query_vector = get_embedding(query_text)

    print(f"\nSearching for: '{query_text}'")
    results = await collection.search(
        vector=query_vector,
        top_k=2,
        include_metadata=True
    )

    for res in results.results:
        print(f"- {res.metadata['title']} (Score: {res.score:.4f})")
        print(f"  Author: {res.metadata['author']}")

if __name__ == "__main__":
    asyncio.run(main())
