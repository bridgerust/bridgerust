import pytest
import asyncio
from embex import Point, EmbexClient

@pytest.mark.asyncio
async def test_insert_stream():
    client = EmbexClient("qdrant", "http://localhost:6334")
    try:
        col = client.collection("streaming_collection")
        await col.delete_collection()
    except Exception as e:
        pass

    collection = client.collection("streaming_collection")
    await collection.create(4, "cosine")

    import uuid
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

    await collection.insert_stream(data_generator(), 5)
    await asyncio.sleep(1)
    response = await collection.search([0.1, 0.1, 0.1, 0.1], 30)
    results = response.results
    assert len(results) == 20
    
    ids = [res.id for res in results]
    for uid in generated_ids:
        assert uid in ids
    
    await collection.delete_collection()
