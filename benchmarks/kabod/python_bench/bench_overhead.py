import time
import asyncio
import random
import sys
from typing import List

# Try imports
try:
    from kabod import Point, KabodClient
    print("Kabod imported successfully")
except ImportError:
    print("Failed to import kabod. Make sure bindings are built and in PYTHONPATH")
    sys.exit(1)

try:
    from qdrant_client import models
    print("QdrantClient imported successfully")
except ImportError:
    print("Failed to import qdrant_client. Install it with: pip install qdrant-client")
    # We will skip qdrant comparison if not installed
    pass

def generate_vector(dim: int) -> List[float]:
    return [random.random() for _ in range(dim)]

def bench_point_creation(count: int, dim: int):
    print(f"\n--- Benchmarking Point Creation (N={count}, Dim={dim}) ---")
    
    # Kabod
    start = time.time()
    for i in range(count):
        _ = Point(
            id=str(i),
            vector=generate_vector(dim),
            metadata={"a": 1, "b": "test"}
        )
    kabod_time = time.time() - start
    print(f"Kabod: {kabod_time:.4f}s ({count/kabod_time:.0f} ops/s)")

    # Qdrant (models.PointStruct)
    if 'models' in globals():
        start = time.time()
        for i in range(count):
            _ = models.PointStruct(
                id=i,
                vector=generate_vector(dim),
                payload={"a": 1, "b": "test"}
            )
        qdrant_time = time.time() - start
        print(f"Qdrant: {qdrant_time:.4f}s ({count/qdrant_time:.0f} ops/s)")
        print(f"Speedup: {qdrant_time/kabod_time:.2f}x")

async def main():
    bench_point_creation(10000, 768)

if __name__ == "__main__":
    asyncio.run(main())
