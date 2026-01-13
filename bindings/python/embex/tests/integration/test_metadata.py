"""
Metadata Operations Integration Tests
Tests metadata update and query functionality
"""

import pytest
import tempfile
import os
from embex import EmbexClient, Point, MetadataUpdate


@pytest.mark.asyncio
async def test_update_metadata():
    """Test updating metadata for existing points."""
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "lancedb_test")
        client = await EmbexClient.new_async("lancedb", db_path)
        try:
            collection = client.collection("test_metadata_update")
            try:
                await collection.delete_collection()
            except:
                pass

            await collection.create(dimension=128, distance="cosine")

            # Insert a point with initial metadata
            point_id = "meta_test_1"
            await collection.insert([
                Point(id=point_id, vector=[0.1] * 128, metadata={"status": "old", "version": 1})
            ])

            # Update the metadata
            await collection.update_metadata([
                MetadataUpdate(id=point_id, updates={"status": "new", "version": 2, "updated": True})
            ])

            # Verify the update
            results = await collection.search([0.1] * 128, top_k=10)
            updated_point = next((r for r in results.results if r.id == point_id), None)
            
            assert updated_point is not None
            assert updated_point.metadata["status"] == "new"
            assert updated_point.metadata["version"] == 2
            assert updated_point.metadata["updated"] == True
            
        except Exception as e:
            if "Connection refused" in str(e) or "Failed to connect" in str(e):
                pytest.skip("DB not available")
            raise e


@pytest.mark.asyncio
async def test_update_metadata_multiple_points():
    """Test updating metadata for multiple points at once."""
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "lancedb_test")
        client = await EmbexClient.new_async("lancedb", db_path)
        try:
            collection = client.collection("test_metadata_multi")
            try:
                await collection.delete_collection()
            except:
                pass

            await collection.create(dimension=128, distance="cosine")

            # Insert multiple points
            ids = ["multi_1", "multi_2"]
            await collection.insert([
                Point(id=ids[0], vector=[0.1] * 128, metadata={"batch": "A"}),
                Point(id=ids[1], vector=[0.2] * 128, metadata={"batch": "A"})
            ])

            # Update both points
            await collection.update_metadata([
                MetadataUpdate(id=ids[0], updates={"batch": "B", "updated": True}),
                MetadataUpdate(id=ids[1], updates={"batch": "B", "updated": True})
            ])

            # Verify updates
            results = await collection.search([0.1] * 128, top_k=10)
            updated = [r for r in results.results if r.id in ids]
            
            assert len(updated) == 2
            for r in updated:
                assert r.metadata["batch"] == "B"
                assert r.metadata["updated"] == True
                
        except Exception as e:
            if "Connection refused" in str(e) or "Failed to connect" in str(e):
                pytest.skip("DB not available")
            raise e
