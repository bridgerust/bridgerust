
import time
import asyncio
import random
import os
import shutil
from typing import List, Dict, Any
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

# Provider Imports
try:
    from embex import EmbexClient, Point
    from qdrant_client import QdrantClient
    from qdrant_client.http import models as qmodels
    import chromadb
    import weaviate
    from pymilvus import connections, Collection, FieldSchema, CollectionSchema, DataType, utility
    import psycopg2
    from psycopg2.extras import execute_values
except ImportError as e:
    print(f"Missing dependency: {e}")
    exit(1)

# Config
DIM = 384  # Standard small embedding
COUNT = 10000
SEARCH_REPEATS = 100
HOST = "localhost"

# Check if running in CI or local
QDRANT_URL = f"http://{HOST}:6333"
CHROMA_HOST = HOST
WEAVIATE_URL = f"http://{HOST}:8080"
MILVUS_HOST = HOST
PG_DSN = f"postgresql://embex:embex_test@{HOST}:5432/embex_test"

def generate_vectors(count, dim):
    return [[random.random() for _ in range(dim)] for _ in range(count)]

class BenchmarkResult:
    def __init__(self, provider, client_type, insert_time, search_latency):
        self.provider = provider
        self.client_type = client_type
        self.insert_ops_per_sec = COUNT / insert_time
        self.search_latency_ms = search_latency * 1000

results = []

async def bench_embex(provider, url, vectors, dim):
    print(f"--- Embex ({provider}) ---")
    try:
        client = await EmbexClient.new_async(provider, url)
    except Exception as e:
        print(f"Failed to connect to {provider}: {e}")
        return None

    # Cleanup/Create
    col_name = f"Bench_{provider}_embex"
    try:
        # Use a new client instance for cleanup to ensure no state issues?
        # Or just use collection object
        await client.collection(col_name).delete_collection()
    except:
        pass
    
    # Create
    col = client.collection(col_name)
    try:
        await col.create(dim, "cosine")
    except Exception as e:
            print(f"Create failed: {e}")

    # Insert
    import uuid
    # Use deterministic UUIDs for reproducibility if needed, or random
    # For speed, pre-generate
    ids = [str(uuid.uuid4()) for _ in range(len(vectors))]
    points = [Point(id=ids[i], vector=v, metadata={"id": i}) for i, v in enumerate(vectors)]
    start = time.time()
    await col.insert(points)
    insert_time = time.time() - start
    
    # Search
    query = vectors[0]
    start = time.time()
    for _ in range(SEARCH_REPEATS):
        await col.search(vector=query, top_k=10)
    search_time = (time.time() - start) / SEARCH_REPEATS

    print(f"Insert: {insert_time:.2f}s, Search: {search_time*1000:.2f}ms")
    return BenchmarkResult(provider, "Embex", insert_time, search_time)

# --- Native Benchmarks ---

def bench_qdrant_native(vectors, dim):
    print("--- Native Qdrant ---")
    client = QdrantClient(url=QDRANT_URL)
    col_name = "bench_qdrant_native"
    client.recreate_collection(
        collection_name=col_name,
        vectors_config=qmodels.VectorParams(size=dim, distance=qmodels.Distance.COSINE),
    )
    
    import uuid
    ids = [str(uuid.uuid4()) for _ in range(len(vectors))]
    points = [
        qmodels.PointStruct(id=ids[i], vector=v, payload={"id": i})
        for i, v in enumerate(vectors)
    ]
    
    start = time.time()
    client.upload_points(collection_name=col_name, points=points)
    insert_time = time.time() - start
    
    query = vectors[0]
    start = time.time()
    for _ in range(SEARCH_REPEATS):
        client.query_points(collection_name=col_name, query=query, limit=10)
    search_time = (time.time() - start) / SEARCH_REPEATS
    
    return BenchmarkResult("Qdrant", "Native", insert_time, search_time)

def bench_chroma_native(vectors, dim):
    print("--- Native Chroma ---")
    client = chromadb.HttpClient(host=CHROMA_HOST, port=8000)
    col_name = "bench_chroma_native"
    try:
        client.delete_collection(col_name)
    except:
        pass
    collection = client.create_collection(name=col_name)
    
    import uuid
    ids = [str(uuid.uuid4()) for _ in range(len(vectors))]
    metadatas = [{"id": i} for i in range(len(vectors))]
    
    start = time.time()
    # Chroma insert limits? Batching helps but let's try direct
    batch_size = 5000
    for i in range(0, len(vectors), batch_size):
        end = i + batch_size
        collection.add(
            ids=ids[i:end],
            embeddings=vectors[i:end],
            metadatas=metadatas[i:end]
        )
    insert_time = time.time() - start
    
    query = vectors[0]
    start = time.time()
    for _ in range(SEARCH_REPEATS):
        collection.query(query_embeddings=[query], n_results=10)
    search_time = (time.time() - start) / SEARCH_REPEATS
    
    return BenchmarkResult("Chroma", "Native", insert_time, search_time)

def bench_weaviate_native(vectors, dim):
    print("--- Native Weaviate ---")
    # v4 API
    try:
        client = weaviate.connect_to_local()
    except Exception as e:
        print(f"Weaviate connect failed: {e}")
        return None

    class_name = "Bench_weaviate_native_v4"
    
    try:
        client.collections.delete(class_name)
    except:
        pass
        
    # Create collection
    # v4 auto-infers or we can be explicit. explicit is better for bench.
    import weaviate.classes.config as wvc
    client.collections.create(
        name=class_name,
        vectorizer_config=wvc.Configure.Vectorizer.none(),
        vector_index_config=wvc.Configure.VectorIndex.hnsw(
            distance_metric=wvc.VectorDistances.COSINE
        ),
        properties=[
            wvc.Property(name="obj_id", data_type=wvc.DataType.INT)
        ]
    )
    
    collection = client.collections.get(class_name)
    
    start = time.time()
    # Batch insert
    import weaviate.classes.data as wvd
    objs = [
        wvd.DataObject(properties={"obj_id": i}, vector=v)
        for i, v in enumerate(vectors)
    ]
    collection.data.insert_many(objs)
    insert_time = time.time() - start
    
    query = vectors[0]
    start = time.time()
    for _ in range(SEARCH_REPEATS):
        collection.query.near_vector(
            near_vector=query,
            limit=10,
            return_properties=["obj_id"]
        )
    search_time = (time.time() - start) / SEARCH_REPEATS
    
    client.close()
    return BenchmarkResult("Weaviate", "Native", insert_time, search_time)

def bench_milvus_native(vectors, dim):
    print("--- Native Milvus ---")
    connections.connect("default", host=MILVUS_HOST, port="19530")
    col_name = "bench_milvus_native"
    
    if utility.has_collection(col_name):
        utility.drop_collection(col_name)
        
    fields = [
        FieldSchema(name="id", dtype=DataType.INT64, is_primary=True, auto_id=False),
        FieldSchema(name="vector", dtype=DataType.FLOAT_VECTOR, dim=dim)
    ]
    schema = CollectionSchema(fields)
    collection = Collection(col_name, schema)
    
    import uuid
    ids = [i for i in range(len(vectors))] # Milvus int64
    # Milvus auto_id=False, expecting int64 if schema says INT64
    # If we want UUIDs, we need VARCHAR schema.
    # For fair comparison with Embex (which uses String/UUID for Qdrant), 
    # lets stick to what the native client supports best or match schema.
    # Embex creates schema with string IDs? 
    # If Embex maps to Milvus, it likely uses VarChar for ID or Int64.
    # Let's keep Int64 for Milvus Native as it's standard performance path.
    # But wait, earlier I said Embex uses strings.
    # If Embex maps to Milvus, does it use auto_id?
    # Let's assume Int64 for Milvus Native is fine validation.
    # If Embex fails on Milvus with "0", it implies Schema mismatch or String ID expectation.
    # I will leave Milvus as is for now as I disabled Milvus Embex benchmark anyway.
    ids = [i for i in range(len(vectors))]
    entities = [ids, vectors]
    
    start = time.time()
    collection.insert(entities)
    collection.create_index("vector", {"index_type": "IVF_FLAT", "metric_type": "COSINE", "params": {"nlist": 128}})
    collection.load()
    insert_time = time.time() - start
    
    query = [vectors[0]]
    search_params = {"metric_type": "COSINE", "params": {"nprobe": 10}}
    start = time.time()
    for _ in range(SEARCH_REPEATS):
        collection.search(query, "vector", search_params, limit=10)
    search_time = (time.time() - start) / SEARCH_REPEATS
    
    return BenchmarkResult("Milvus", "Native", insert_time, search_time)

def bench_pgvector_native(vectors, dim):
    print("--- Native PgVector ---")
    conn = psycopg2.connect(PG_DSN)
    cur = conn.cursor()
    
    cur.execute("CREATE EXTENSION IF NOT EXISTS vector")
    cur.execute("DROP TABLE IF EXISTS bench_native")
    cur.execute(f"CREATE TABLE bench_native (id bigserial PRIMARY KEY, embedding vector({dim}))")
    
    start = time.time()
    data = [(v,) for v in vectors]
    execute_values(cur, "INSERT INTO bench_native (embedding) VALUES %s", data)
    cur.execute("CREATE INDEX ON bench_native USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100)")
    conn.commit()
    insert_time = time.time() - start
    
    query = str(vectors[0])
    start = time.time()
    for _ in range(SEARCH_REPEATS):
        cur.execute(f"SELECT id FROM bench_native ORDER BY embedding <=> '{query}' LIMIT 10")
        cur.fetchall()
    search_time = (time.time() - start) / SEARCH_REPEATS
    
    cur.close()
    conn.close()
    return BenchmarkResult("PgVector", "Native", insert_time, search_time)


async def main():
    print("Generatng vectors...")
    vectors = generate_vectors(COUNT, DIM)
    
    # 1. Qdrant
    results.append(await bench_embex("qdrant", QDRANT_URL, vectors, DIM))
    results.append(bench_qdrant_native(vectors, DIM))
    
    # 2. Chroma
    results.append(await bench_embex("chroma", CHROMA_HOST, vectors, DIM))
    results.append(bench_chroma_native(vectors, DIM))
    
    # 3. Weaviate
    results.append(await bench_embex("weaviate", WEAVIATE_URL, vectors, DIM))
    results.append(bench_weaviate_native(vectors, DIM))
    
    # 4. Milvus
    # Skipping Milvus Embex for now as complex setup in python bindings might be tricky without full env
    # But let's verify if Embex python supports it fully. The README says yes.
    # results.append(await bench_embex("milvus", MILVUS_HOST, vectors, DIM)) 
    # results.append(bench_milvus_native(vectors, DIM))
    
    # 5. PgVector
    results.append(await bench_embex("pgvector", PG_DSN, vectors, DIM))
    results.append(bench_pgvector_native(vectors, DIM))

    # Gen Graph
    df = pd.DataFrame([
        {
            "Provider": r.provider, 
            "Client": r.client_type, 
            "Insert (ops/s)": r.insert_ops_per_sec, 
            "Search Latency (ms)": r.search_latency_ms
        } 
        for r in results if r is not None
    ])
    
    print("\nResults:")
    print(df)
    
    sns.set_theme(style="whitegrid")
    
    # Plot Insert
    plt.figure(figsize=(10, 6))
    g = sns.barplot(data=df, x="Provider", y="Insert (ops/s)", hue="Client", palette="viridis")
    plt.title("Insert Throughput (Higher is better)")
    plt.savefig("benchmark_insert.png")
    
    # Plot Search
    plt.figure(figsize=(10, 6))
    g = sns.barplot(data=df, x="Provider", y="Search Latency (ms)", hue="Client", palette="viridis")
    plt.title("Search Latency (Lower is better)")
    plt.savefig("benchmark_search.png")
    
    print("\nGraphs saved to benchmark_insert.png and benchmark_search.png")

if __name__ == "__main__":
    asyncio.run(main())
