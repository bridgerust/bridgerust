import pytest
import tempfile
import os
from embex import EmbexClient, Point


@pytest.mark.asyncio
async def test_insert_batch_parallel():
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "lancedb_test")
        client = await EmbexClient.new_async("lancedb", db_path)
        try:
            collection = client.collection("test_features")
            # cleanup
            try:
                await collection.delete_collection()
            except:
                pass
                
            await collection.create(dimension=2, distance="cosine")

            points = []
            for i in range(100):
                points.append(Point(
                    id=f"p{i}",
                    vector=[0.1, 0.2],
                    metadata=None
                ))

            # Test with parallel=4
            await collection.insert_batch(points, batch_size=10, parallel=4)
            
            # Verify count (using search count agg if available, or just search)
            results = await collection.search([0.1, 0.2], top_k=100)
            assert len(results.results) == 100
        except Exception as e:
            if "Connection refused" in str(e) or "Failed to connect" in str(e):
                pytest.skip("DB not available")
            raise e


@pytest.mark.asyncio
async def test_insert_batch_sequential():
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "lancedb_test")
        client = await EmbexClient.new_async("lancedb", db_path)
        try:    
            collection = client.collection("test_features_seq")
            try:
                await collection.delete_collection()
            except:
                pass

            await collection.create(dimension=2, distance="cosine")

            points = [Point(id=f"s{i}", vector=[0.3, 0.4], metadata=None) for i in range(50)]
            
            # Test with default (parallel=None -> 1)
            await collection.insert_batch(points, batch_size=10)
            
            results = await collection.search([0.3, 0.4], top_k=100)
            assert len(results.results) == 50
        except Exception as e:
            if "Connection refused" in str(e) or "Failed to connect" in str(e):
                pytest.skip("DB not available")
            raise e
