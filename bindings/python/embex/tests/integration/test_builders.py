"""
Search Builder Integration Tests
Tests SearchBuilder.filter() and SearchBuilder.aggregation()

Note: These tests require Qdrant since LanceDB doesn't support:
- JSON filter operators (->>)
- Vector-less queries (build_query)
"""

import pytest
import uuid
from embex import EmbexClient, Point


@pytest.mark.asyncio
async def test_search_builder_with_filter():
    """Test build_search() with filter() - requires Qdrant."""
    client = EmbexClient("qdrant", "http://localhost:6334")
    try:
        collection = client.collection("test_search_filter")
        try:
            await collection.delete_collection()
        except:
            pass

        await collection.create(dimension=128, distance="cosine")

        await collection.insert([
            Point(id=str(uuid.uuid4()), vector=[0.1] * 128, metadata={"status": "active", "score": 20}),
            Point(id=str(uuid.uuid4()), vector=[0.2] * 128, metadata={"status": "inactive", "score": 10}),
            Point(id=str(uuid.uuid4()), vector=[0.3] * 128, metadata={"status": "active", "score": 30}),
        ])

        # Use build_search with filter
        filter_dict = {"op": "key", "args": ["status", {"eq": "active"}]}
        results = await collection.build_search([0.1] * 128) \
            .filter(filter_dict) \
            .limit(10) \
            .execute()
        
        assert len(results.results) >= 2
        for r in results.results:
            assert r.metadata["status"] == "active"
            
    except Exception as e:
        if "Connection refused" in str(e) or "Failed to connect" in str(e):
            pytest.skip("Qdrant not available")
        raise e


@pytest.mark.asyncio
async def test_search_builder_with_aggregation():
    """Test build_search() with aggregation('count') - requires Qdrant."""
    client = EmbexClient("qdrant", "http://localhost:6334")
    try:
        collection = client.collection("test_search_agg")
        try:
            await collection.delete_collection()
        except:
            pass

        await collection.create(dimension=128, distance="cosine")

        # Insert 10 points
        points = [Point(id=str(uuid.uuid4()), vector=[0.1 * i] * 128, metadata={}) for i in range(10)]
        await collection.insert(points)

        # Use build_search with aggregation
        results = await collection.build_search([0.1] * 128) \
            .limit(5) \
            .aggregation("count") \
            .execute()
        
        assert results.aggregations is not None
        assert "count" in results.aggregations
        # Note: count returns total matching, not just limited results
        assert results.aggregations["count"] >= 5
            
    except Exception as e:
        if "Connection refused" in str(e) or "Failed to connect" in str(e):
            pytest.skip("Qdrant not available")
        raise e


@pytest.mark.asyncio
async def test_search_builder_filter_and_aggregation():
    """Test build_search() with both filter and aggregation - requires Qdrant."""
    client = EmbexClient("qdrant", "http://localhost:6334")
    try:
        collection = client.collection("test_search_filter_agg")
        try:
            await collection.delete_collection()
        except:
            pass

        await collection.create(dimension=128, distance="cosine")

        await collection.insert([
            Point(id=str(uuid.uuid4()), vector=[0.1] * 128, metadata={"type": "A"}),
            Point(id=str(uuid.uuid4()), vector=[0.2] * 128, metadata={"type": "A"}),
            Point(id=str(uuid.uuid4()), vector=[0.3] * 128, metadata={"type": "B"}),
        ])

        # Use build_search with filter and aggregation
        filter_dict = {"op": "key", "args": ["type", {"eq": "A"}]}
        results = await collection.build_search([0.1] * 128) \
            .filter(filter_dict) \
            .limit(10) \
            .aggregation("count") \
            .execute()
        
        assert results.aggregations is not None
        assert "count" in results.aggregations
        assert results.aggregations["count"] >= 2
        
        for r in results.results:
            assert r.metadata["type"] == "A"
            
    except Exception as e:
        if "Connection refused" in str(e) or "Failed to connect" in str(e):
            pytest.skip("Qdrant not available")
        raise e


@pytest.mark.asyncio  
async def test_query_builder_with_aggregation():
    """Test build_query() with aggregation - requires Qdrant (LanceDB needs vector)."""
    client = EmbexClient("qdrant", "http://localhost:6334")
    try:
        collection = client.collection("test_query_agg")
        try:
            await collection.delete_collection()
        except:
            pass

        await collection.create(dimension=128, distance="cosine")

        await collection.insert([
            Point(id=str(uuid.uuid4()), vector=[0.1] * 128, metadata={"status": "active"}),
            Point(id=str(uuid.uuid4()), vector=[0.2] * 128, metadata={"status": "active"}),
            Point(id=str(uuid.uuid4()), vector=[0.3] * 128, metadata={"status": "active"}),
            Point(id=str(uuid.uuid4()), vector=[0.4] * 128, metadata={"status": "inactive"}),
        ])

        # Filter-only query with count aggregation
        filter_dict = {"op": "key", "args": ["status", {"eq": "active"}]}
        results = await collection.build_query() \
            .filter(filter_dict) \
            .limit(10) \
            .aggregation("count") \
            .execute()
        
        assert results.aggregations is not None
        assert "count" in results.aggregations
        assert results.aggregations["count"] >= 3
            
    except Exception as e:
        if "Connection refused" in str(e) or "Failed to connect" in str(e):
            pytest.skip("Qdrant not available")
        raise e
