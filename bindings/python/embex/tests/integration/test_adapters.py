"""
Embex Integration Tests
-----------------------
Tests all adapter implementations against real database instances.
Run Docker Compose first: docker-compose up -d

Usage:
    cd bindings/python/embex
    source .venv/bin/activate
    maturin develop --features all
    pytest tests/integration/test_adapters.py -v --integration
"""
import asyncio
import pytest
import uuid

# Skip all tests if databases aren't running
pytestmark = pytest.mark.integration

# Test configuration
TEST_DIMENSION = 128
TEST_COLLECTION = "embex_integration_test"


def random_vector(dim: int = TEST_DIMENSION) -> list[float]:
    import random
    return [random.random() for _ in range(dim)]


class TestQdrantAdapter:
    """Integration tests for Qdrant."""
    
    @pytest.fixture
    def client(self):
        from embex import EmbexClient
        # Qdrant Rust client uses gRPC on port 6334, not HTTP on 6333
        return EmbexClient(provider="qdrant", url="http://localhost:6334")
    
    @pytest.fixture
    def collection(self, client):
        return client.collection(TEST_COLLECTION)
    
    @pytest.mark.asyncio
    async def test_create_collection(self, client, collection):
        try:
            await collection.delete_collection()
        except Exception:
            pass
        await collection.create(dimension=TEST_DIMENSION, distance="cosine")
        await collection.delete_collection()
    
    @pytest.mark.asyncio
    async def test_insert_and_search(self, client, collection):
        from embex import Point
        
        try:
            await collection.delete_collection()
        except Exception:
            pass
        await collection.create(dimension=TEST_DIMENSION, distance="cosine")
        
        points = [
            Point(id=str(uuid.uuid4()), vector=random_vector(), metadata={"category": "A"}),
            Point(id=str(uuid.uuid4()), vector=random_vector(), metadata={"category": "B"}),
            Point(id=str(uuid.uuid4()), vector=random_vector(), metadata={"category": "A"}),
        ]
        await collection.insert(points)
        
        results = await collection.search(vector=random_vector(), top_k=2)
        assert len(results.results) == 2
        
        await collection.delete_collection()
    
    @pytest.mark.asyncio
    async def test_delete_points(self, client, collection):
        from embex import Point
        
        try:
            await collection.delete_collection()
        except Exception:
            pass
        await collection.create(dimension=TEST_DIMENSION, distance="cosine")
        
        del_id = str(uuid.uuid4())
        points = [Point(id=del_id, vector=random_vector(), metadata={})]
        await collection.insert(points)
        
        await collection.delete(ids=[del_id])
        
        results = await collection.search(vector=random_vector(), top_k=10)
        ids = [r.id for r in results.results]
        assert del_id not in ids
        
        await collection.delete_collection()


class TestChromaAdapter:
    """Integration tests for Chroma."""
    
    @pytest.fixture
    def client(self):
        from embex import EmbexClient
        return EmbexClient(provider="chroma", url="http://localhost:8000")
    
    @pytest.fixture
    def collection(self, client):
        return client.collection(TEST_COLLECTION)
    
    @pytest.mark.asyncio
    async def test_crud_operations(self, client, collection):
        from embex import Point
        
        try:
            await collection.delete_collection()
        except Exception:
            pass
        await collection.create(dimension=TEST_DIMENSION, distance="cosine")
        
        points = [
            Point(id="c1", vector=random_vector(), metadata={"type": "test"}),
            Point(id="c2", vector=random_vector(), metadata={"type": "test"}),
        ]
        await collection.insert(points)
        
        results = await collection.search(vector=random_vector(), top_k=2)
        assert len(results.results) == 2
        
        await collection.delete_collection()
    
    @pytest.mark.asyncio
    async def test_create_auto_dimension_inference(self, client, collection):
        """Test that Chroma can infer dimension from first insert."""
        from embex import Point
        
        try:
            await collection.delete_collection()
        except Exception:
            pass
        
        # Create collection without specifying dimension (Chroma will infer)
        await collection.create_auto(dimension=None, distance="cosine")
        
        # Insert points - Chroma will infer dimension from first insert
        points = [
            Point(id="c1", vector=random_vector(), metadata={"type": "test"}),
            Point(id="c2", vector=random_vector(), metadata={"type": "test"}),
        ]
        await collection.insert(points)
        
        # Verify search works
        results = await collection.search(vector=random_vector(), top_k=2)
        assert len(results.results) == 2
        
        await collection.delete_collection()


class TestWeaviateAdapter:
    """Integration tests for Weaviate."""
    
    @pytest.fixture
    def client(self):
        from embex import EmbexClient
        return EmbexClient(provider="weaviate", url="http://localhost:8080")
    
    @pytest.fixture
    def collection(self, client):
        return client.collection("EmbexIntegrationTest")
    
    @pytest.mark.asyncio
    async def test_crud_operations(self, client, collection):
        from embex import Point
        
        try:
            await collection.delete_collection()
        except Exception:
            pass
        await collection.create(dimension=TEST_DIMENSION, distance="cosine")
        
        points = [
            Point(id="w1", vector=random_vector(), metadata={"name": "test1"}),
            Point(id="w2", vector=random_vector(), metadata={"name": "test2"}),
        ]
        await collection.insert(points)
        
        results = await collection.search(vector=random_vector(), top_k=2)
        assert len(results.results) >= 1
        
        await collection.delete_collection()


class TestMilvusAdapter:
    """Integration tests for Milvus."""
    
    @pytest.mark.asyncio
    async def test_crud_operations(self):
        from embex import EmbexClient, Point
        
        # Milvus requires async initialization
        client = await EmbexClient.new_async(provider="milvus", url="http://localhost:19530")
        collection = client.collection(TEST_COLLECTION)
        
        try:
            await collection.delete_collection()
        except Exception:
            pass
        
        await collection.create(dimension=TEST_DIMENSION, distance="cosine")
        
        points = [
            Point(id="m1", vector=random_vector(), metadata={}),
            Point(id="m2", vector=random_vector(), metadata={}),
        ]
        await collection.insert(points)
        
        results = await collection.search(vector=random_vector(), top_k=2)
        assert len(results.results) >= 1
        
        await collection.delete_collection()


class TestPgvectorAdapter:
    """Integration tests for pgvector."""
    
    @pytest.mark.asyncio
    async def test_crud_operations(self):
        from embex import EmbexClient, Point
        
        # pgvector requires async initialization
        client = await EmbexClient.new_async(
            provider="pgvector",
            url="postgresql://embex:embex_test@localhost:5432/embex_test"
        )
        collection = client.collection(TEST_COLLECTION)
        
        try:
            await collection.delete_collection()
        except Exception:
            pass
        
        await collection.create(dimension=TEST_DIMENSION, distance="cosine")
        
        points = [
            Point(id="pg1", vector=random_vector(), metadata={"info": "test"}),
            Point(id="pg2", vector=random_vector(), metadata={"info": "test"}),
        ]
        await collection.insert(points)
        
        results = await collection.search(vector=random_vector(), top_k=2)
        assert len(results.results) >= 1
        
        await collection.delete_collection()


class TestLanceDBAdapter:
    """Integration tests for LanceDB (embedded, no Docker needed)."""
    
    @pytest.mark.asyncio
    async def test_crud_operations(self, tmp_path):
        from embex import EmbexClient, Point
        
        # LanceDB is embedded, uses local path
        db_path = str(tmp_path / "lancedb_test")
        client = await EmbexClient.new_async(provider="lancedb", url=db_path)
        collection = client.collection(TEST_COLLECTION)
        
        try:
            await collection.delete_collection()
        except Exception:
            pass
        
        await collection.create(dimension=TEST_DIMENSION, distance="cosine")
        
        points = [
            Point(id="l1", vector=random_vector(), metadata={"name": "lance1"}),
            Point(id="l2", vector=random_vector(), metadata={"name": "lance2"}),
        ]
        await collection.insert(points)
        
        results = await collection.search(vector=random_vector(), top_k=2)
        assert len(results.results) >= 1
        
        await collection.delete_collection()
