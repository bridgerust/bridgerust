import pytest
import asyncio
from kabod import Point, KabodClient

@pytest.mark.asyncio
async def test_insert_stream():
    client = KabodClient("qdrant", "http://localhost:6334")
    # Clean up potentially existing collection
    try:
        col = client.collection("streaming_collection")
        await col.delete_collection()
    except Exception as e:
        # print(f"cleanup error: {e}")
        pass
        
    collection = client.collection("streaming_collection")
    await collection.create(4, "cosine")

    import uuid
    # Store IDs to verify later
    generated_ids = []

    def data_generator():
        for i in range(20):
            uid = str(uuid.uuid4())
            generated_ids.append(uid)
            yield Point(
                id=uid,
                vector=[0.1, 0.1, 0.1, 0.1],
                metadata={"index": i}
            )

    # Insert using stream with batch size 5
    await collection.insert_stream(data_generator(), 5)

    # Verify insertion
    await asyncio.sleep(1) # Give DB time to index
    results = await collection.search([0.1, 0.1, 0.1, 0.1], 30)
    assert len(results) == 20
    
    # Check if correct data
    ids = [res.id for res in results]
    for uid in generated_ids:
        assert uid in ids
    
    await collection.delete_collection()
