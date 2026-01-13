"""
Query Builder Integration Tests
Tests QueryBuilder (filter-only queries) and query() with options

Note: Filter and build_query tests require Qdrant since LanceDB doesn't support:
- JSON filter operators (->>)
- Vector-less queries (build_query)
"""

import pytest
import tempfile
import os
import uuid
from embex import EmbexClient, Point


@pytest.mark.asyncio
async def test_query_with_options():
    """Test query() method with options dict - uses LanceDB."""
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "lancedb_test")
        client = await EmbexClient.new_async("lancedb", db_path)
        try:
            collection = client.collection("test_query_options")
            try:
                await collection.delete_collection()
            except:
                pass

            await collection.create(dimension=128, distance="cosine")

            await collection.insert([
                Point(id="q1", vector=[0.1] * 128, metadata={"type": "A"}),
                Point(id="q2", vector=[0.2] * 128, metadata={"type": "B"}),
                Point(id="q3", vector=[0.3] * 128, metadata={"type": "A"}),
            ])

            # Test with limit option
            results = await collection.query([0.1] * 128, {"limit": 2})
            assert len(results.results) == 2

            # Test with include_metadata option
            results = await collection.query([0.1] * 128, {"limit": 10, "include_metadata": True})
            assert results.results[0].metadata is not None
            
        except Exception as e:
            if "Connection refused" in str(e) or "Failed to connect" in str(e):
                pytest.skip("DB not available")
            raise e


@pytest.mark.asyncio
async def test_query_with_filter():
    """Test query() method with filter option - requires Qdrant."""
    client = EmbexClient("qdrant", "http://localhost:6334")
    try:
        collection = client.collection("test_query_filter")
        try:
            await collection.delete_collection()
        except:
            pass

        await collection.create(dimension=128, distance="cosine")

        await collection.insert([
            Point(id=str(uuid.uuid4()), vector=[0.1] * 128, metadata={"status": "active"}),
            Point(id=str(uuid.uuid4()), vector=[0.2] * 128, metadata={"status": "inactive"}),
            Point(id=str(uuid.uuid4()), vector=[0.3] * 128, metadata={"status": "active"}),
        ])

        # Test with filter
        filter_dict = {"op": "key", "args": ["status", {"eq": "active"}]}
        results = await collection.query([0.1] * 128, {"limit": 10, "filter": filter_dict})
        
        assert len(results.results) >= 2
        for r in results.results:
            assert r.metadata["status"] == "active"
            
    except Exception as e:
        if "Connection refused" in str(e) or "Failed to connect" in str(e):
            pytest.skip("Qdrant not available")
        raise e


@pytest.mark.asyncio
async def test_build_query_filter_only():
    """Test build_query() for filter-only queries (no vector) - requires Qdrant."""
    client = EmbexClient("qdrant", "http://localhost:6334")
    try:
        collection = client.collection("test_build_query")
        try:
            await collection.delete_collection()
        except:
            pass

        await collection.create(dimension=128, distance="cosine")

        await collection.insert([
            Point(id=str(uuid.uuid4()), vector=[0.1] * 128, metadata={"category": "tech"}),
            Point(id=str(uuid.uuid4()), vector=[0.2] * 128, metadata={"category": "science"}),
            Point(id=str(uuid.uuid4()), vector=[0.3] * 128, metadata={"category": "tech"}),
        ])

        filter_dict = {"op": "key", "args": ["category", {"eq": "tech"}]}
        results = await collection.build_query() \
            .filter(filter_dict) \
            .limit(10) \
            .execute()
        
        assert len(results.results) >= 2
        for r in results.results:
            assert r.metadata["category"] == "tech"
            
    except Exception as e:
        if "Connection refused" in str(e) or "Failed to connect" in str(e):
            pytest.skip("Qdrant not available")
        raise e
