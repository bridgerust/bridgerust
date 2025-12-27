import asyncio
import random
from typing import List
from embex import EmbexClient, Point

# Mock embedding function (in a real app, use OpenAI/HuggingFace)
def get_embedding(text: str) -> List[float]:
    # Deterministic mock embedding for demo purposes
    random.seed(text)
    return [random.random() for _ in range(768)]

async def main():
    # 1. Initialize Client (using Qdrant as backend for this example)
    # Ensure you have a Qdrant instance running at localhost:6333
    client = EmbexClient(provider="qdrant", url="http://localhost:6333")
    
    collection_name = "semantic_books"
    
    collection = client.collection(collection_name)

    # 2. Cleanup existing collection if any
    try:
        print(f"Deleting existing collection: {collection_name}")
        await collection.delete_collection()
    except Exception:
        pass # Collection might not exist

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

    # 6. Search with aggregations
    query_text = "programming book"
    query_vector = get_embedding(query_text)

    print(f"\nSearching for: '{query_text}'")
    builder = collection.build_search(query_vector)
    results = await builder.limit(2).include_metadata(True).aggregation("count").execute()

    print(f"Total books in collection: {results.aggregations.get('count', 0)}")
    for res in results.results:
        print(f"- {res.metadata['title']} (Score: {res.score:.4f})")
        print(f"  Author: {res.metadata['author']}")

    # 7. Search with filter (using search method with filter parameter)
    print("\nSearching tech books using filter:")
    # Note: build_query and update_metadata are currently Node.js-only features
    # For Python, use the search method with filter parameter
    tech_filter = {
        "op": "key",
        "args": ["tags", {"op": "in", "args": ["tech"]}]
    }
    tech_results = await collection.search(
        vector=query_vector,
        top_k=10,
        filter=tech_filter,
        include_metadata=True
    )

    print(f"Found {len(tech_results.results)} tech books")
    for res in tech_results.results:
        print(f"  - {res.metadata['title']} by {res.metadata['author']}")

if __name__ == "__main__":
    asyncio.run(main())
