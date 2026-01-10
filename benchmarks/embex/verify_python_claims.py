
import time
import math
import random

def generate_vector(dim):
    return [random.random() for _ in range(dim)]

def normalize_pure_python(v):
    norm = math.sqrt(sum(x * x for x in v))
    if norm == 0:
        return v
    return [x / norm for x in v]

def cosine_similarity_pure_python(a, b):
    dot = sum(x * y for x, y in zip(a, b))
    norm_a = math.sqrt(sum(x * x for x in a))
    norm_b = math.sqrt(sum(x * x for x in b))
    if norm_a == 0 or norm_b == 0:
        return 0.0
    return dot / (norm_a * norm_b)

def metadata_filter_pure_python(items, key, value):
    return [item for item in items if item.get(key) == value]

def run_benchmarks():
    print("Running verified Python benchmarks...")

    # 1. Vector Normalization (10k dims)
    dim = 10000
    v = generate_vector(dim)
    
    # Warmup
    for _ in range(10):
        normalize_pure_python(v)

    start = time.time()
    iterations = 100
    for _ in range(iterations):
        normalize_pure_python(v)
    avg_time = (time.time() - start) / iterations
    print(f"Vector normalization (10k dims): {avg_time * 1000:.2f}ms")

    # 2. Cosine Similarity (batch 1000, dim=768 usually, but let's check table context)
    # Table says: "Cosine similarity (batch 1000)". Usually means 1000 comparisons? or batch size 1000?
    # Assuming batch size 1000 comparisons of 768 dim vectors (typical embedding size).
    dim_cosine = 768
    batch_size = 1000
    q = generate_vector(dim_cosine)
    vectors = [generate_vector(dim_cosine) for _ in range(batch_size)]

    start = time.time()
    iterations_cosine = 10
    for _ in range(iterations_cosine):
        for vec in vectors:
            cosine_similarity_pure_python(q, vec)
    avg_time_cosine = (time.time() - start) / iterations_cosine
    print(f"Cosine similarity (batch 1000, 768 dim): {avg_time_cosine * 1000:.2f}ms")

    # 3. Metadata Filtering
    # Table says: "Metadata filtering | 180ms"
    # Need to guess dataset size. 180ms for filtering implies a large list.
    # Python list iteration is fast. simple check `x == y` is fast.
    # To take 180ms, it might need ~1M items or complex filter?
    # Let's try 100k items first.
    
    count = 1000000 # 1 Million items
    items = [{"id": i, "category": "books" if i % 2 == 0 else "electronics"} for i in range(count)]
    
    start = time.time()
    iterations_filter = 10
    for _ in range(iterations_filter):
        metadata_filter_pure_python(items, "category", "books")
    avg_time_filter = (time.time() - start) / iterations_filter
    print(f"Metadata filtering (1M items): {avg_time_filter * 1000:.2f}ms")

if __name__ == "__main__":
    run_benchmarks()
