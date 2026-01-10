
import time
import asyncio
import random
import shutil
import os
import numpy as np
import pyarrow as pa

# Try imports
try:
    import lancedb
    from embex import EmbexClient, Point
    print("Libraries imported successfully")
except ImportError as e:
    print(f"Failed to import libraries: {e}")
    print("Please install: pip install embex lancedb numpy pyarrow")
    exit(1)

def generate_vectors(count, dim):
    return [[random.random() for _ in range(dim)] for _ in range(count)]

async def bench_embex(tmp_path, vectors, dim):
    print(f"\n--- Embex (LanceDB) ---")
    
    # 1. Init
    start = time.time()
    client = await EmbexClient.new_async("lancedb", tmp_path)
    print(f"Init: {(time.time() - start)*1000:.2f}ms")

    # 2. Create Collection
    start = time.time()
    col = client.collection("bench")
    await col.create(dim, "cosine")
    print(f"Create Collection: {(time.time() - start)*1000:.2f}ms")

    # 3. Insert
    points = [
        Point(id=str(i), vector=v, metadata={"id": i}) 
        for i, v in enumerate(vectors)
    ]
    
    start = time.time()
    col = client.collection("bench")
    await col.insert(points)
    insert_time = time.time() - start
    print(f"Insert {len(vectors)}: {insert_time:.4f}s ({len(vectors)/insert_time:.0f} ops/s)")

    # 4. Search
    query = vectors[0]
    start = time.time()
    for _ in range(100):
        await col.search(vector=query, top_k=10)
    search_time = (time.time() - start) / 100
    print(f"Search (avg of 100): {search_time*1000:.2f}ms")

    return insert_time, search_time

def bench_native_lancedb(tmp_path, vectors, dim):
    print(f"\n--- Native LanceDB ---")

    # 1. Init
    start = time.time()
    db = lancedb.connect(tmp_path)
    print(f"Init: {(time.time() - start)*1000:.2f}ms")

    # 2. Create Table
    # LanceDB native expects data to infer schema or explicit schema
    # constructing data for pyarrow table or list of dicts
    data = [
        {"vector": v, "id": i}
        for i, v in enumerate(vectors)
    ]
    
    start = time.time()
    # Native LanceDB create_table typically does insertion too if data provided
    # To be fair, we should measure creation + insertion
    tbl = db.create_table("bench", data=data, mode="overwrite")
    insert_time = time.time() - start
    print(f"Create+Insert {len(vectors)}: {insert_time:.4f}s ({len(vectors)/insert_time:.0f} ops/s)")

    # 4. Search
    # Need to convert query to numpy/list
    # LanceDB native search
    query = vectors[0]
    start = time.time()
    for _ in range(100):
        tbl.search(query).limit(10).to_list()
    search_time = (time.time() - start) / 100
    print(f"Search (avg of 100): {search_time*1000:.2f}ms")

    return insert_time, search_time

async def main():
    dim = 768
    count = 10000
    vectors = generate_vectors(count, dim)
    
    path_embex = "./data_bench_embex"
    path_native = "./data_bench_native"
    
    # Clean up
    if os.path.exists(path_embex): shutil.rmtree(path_embex)
    if os.path.exists(path_native): shutil.rmtree(path_native)
    
    try:
        e_ins, e_search = await bench_embex(path_embex, vectors, dim)
        n_ins, n_search = bench_native_lancedb(path_native, vectors, dim)
        
        print(f"\n--- Comparison ---")
        print(f"Insert Overhead: {(e_ins - n_ins) / n_ins * 100:.1f}%")
        print(f"Search Overhead: {(e_search - n_search) / n_search * 100:.1f}%")
        
    finally:
         if os.path.exists(path_embex): shutil.rmtree(path_embex)
         if os.path.exists(path_native): shutil.rmtree(path_native)

if __name__ == "__main__":
    asyncio.run(main())
