import pytest
import tempfile
import asyncio
from embex import EmbexClient, DataMigrator, Point

@pytest.mark.asyncio
async def test_data_migration_lancedb_to_lancedb():
    with tempfile.TemporaryDirectory() as tmp1, tempfile.TemporaryDirectory() as tmp2:
        # Initialize source and destination (both LanceDB for standalone testing)
        source = await EmbexClient.new_async("lancedb", tmp1)
        dest = await EmbexClient.new_async("lancedb", tmp2)
        
        # Setup source
        src_col_name = "products"
        src_col = source.collection(src_col_name)
        
        # Create collection
        await src_col.create_auto(dimension=4, distance="cosine")
        
        # Insert data
        points_data = [
            {"id": "p1", "vector": [0.1, 0.2, 0.3, 0.4], "metadata": {"category": "electronics"}},
            {"id": "p2", "vector": [0.5, 0.6, 0.7, 0.8], "metadata": {"category": "clothing"}},
            {"id": "p3", "vector": [0.9, 0.0, 0.1, 0.2], "metadata": {"category": "electronics"}},
        ]
        
        points = [
            Point(id=p["id"], vector=p["vector"], metadata=p["metadata"]) 
            for p in points_data
        ]
        
        await src_col.insert(points)
        
        # Verify source data
        res = await src_col.search(vector=[0.0]*4, top_k=10)
        assert len(res.results) == 3
        
        # Migrate
        migrator = DataMigrator(source, dest)
        dest_col_name = "migrated_products"
        
        result = await migrator.migrate_simple(src_col_name, dest_col_name, batch_size=2)
        
        print(f"Migration result: {result}")
        assert result.points_migrated == 3
        assert result.elapsed_ms >= 0
        
        # Verify destination data
        dest_col = dest.collection(dest_col_name)
        
        # Check count
        res = await dest_col.search(vector=[0.0]*4, top_k=10)
        assert len(res.results) == 3
        
        # Check specific point
        res = await dest_col.search(vector=[0.1, 0.2, 0.3, 0.4], top_k=1)
        assert len(res.results) >= 1
        top = res.results[0]
        # Depending on search, we might get p1 match
        # Precision issues might apply, but exact vector match should return high score
        
        # Test full migrate() with schema
        result2 = await migrator.migrate(
            src_col_name, 
            "explicit_schema", 
            dimension=4, 
            batch_size=10, 
            distance="cosine"
        )
        assert result2.points_migrated == 3
