import embex
from embex import Point, EmbexClient, Collection, SearchResult
import asyncio

def test_imports():
    print("Testing imports...")
    assert Point
    assert EmbexClient
    assert Collection
    assert SearchResult
    print("Imports success.")

def test_point_creation():
    print("Testing Point creation...")
    p = Point(id="1", vector=[0.5, 0.25], metadata={"key": "value"})
    assert p.id == "1"
    assert p.vector == [0.5, 0.25]
    assert p.metadata["key"] == "value"
    print("Point creation success.")

def test_client_init_attempt():
    print("Testing Client init (expecting failure if no DB)...")
    try:
        client = EmbexClient(provider="qdrant", url="http://localhost:6333")
        print("Client initialized (unexpected if DB down, but valid if lazy)")
    except Exception as e:
        print(f"Client init failed as expected (or not?): {e}")

if __name__ == "__main__":
    test_imports()
    test_point_creation()
    test_client_init_attempt()
