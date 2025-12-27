import pytest
import uuid
import asyncio
from embex import EmbexClient, Point

TEST_COLLECTION = "embex_migration_test_py"

pytestmark = pytest.mark.integration

class CreateCollectionMigration:
    def __init__(self, version, collection_name):
        self.version = version
        self.collection_name = collection_name

    async def up(self, client: EmbexClient):
        await client.collection(self.collection_name).create(128, "cosine")

    async def down(self, client: EmbexClient):
        await client.collection(self.collection_name).delete_collection()

@pytest.mark.asyncio
async def test_run_migrations():
    client = EmbexClient(provider="qdrant", url="http://localhost:6334")
    
    try:
        await client.collection(TEST_COLLECTION).delete_collection()
    except Exception:
        pass

    migration_version = str(uuid.uuid4())
    migration = CreateCollectionMigration(migration_version, TEST_COLLECTION)

    await client.run_migrations([migration])

    col = client.collection(TEST_COLLECTION)
    point_id = str(uuid.uuid4())
    await col.insert([Point(id=point_id, vector=[0.1]*128, metadata=None)])
    
    results = await col.search([0.1]*128, top_k=1)
    assert len(results.results) > 0
    
    await col.delete_collection()
