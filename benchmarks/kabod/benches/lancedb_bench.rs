use bridge_kabod::VectorDatabase;
use bridge_kabod::adapters::LanceDBAdapter;
use bridge_kabod::{CollectionSchema, DistanceMetric, Point};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::Rng;
use std::hint::black_box;
use std::time::Duration;

fn generate_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    (0..dim).map(|_| rng.random::<f32>()).collect()
}

fn generate_points(count: usize, dim: usize) -> Vec<Point> {
    (0..count)
        .map(|i| Point {
            id: format!("point_{}", i),
            vector: generate_vector(dim),
            metadata: None,
        })
        .collect()
}

fn bench_insert(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();

    let adapter = rt.block_on(async { LanceDBAdapter::new(db_path).await.unwrap() });

    let schema = CollectionSchema {
        name: "bench_collection".to_string(),
        dimension: 768,
        metric: DistanceMetric::Cosine,
    };

    rt.block_on(async {
        adapter.create_collection(&schema).await.unwrap();
    });

    let mut group = c.benchmark_group("lancedb_insert");
    group.measurement_time(Duration::from_secs(10));

    for batch_size in [10, 100, 1000].iter() {
        let points = generate_points(*batch_size, 768);

        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            &points,
            |b, points| {
                b.to_async(&rt).iter(|| async {
                    adapter
                        .insert("bench_collection", black_box(points.clone()))
                        .await
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();

    let adapter = rt.block_on(async { LanceDBAdapter::new(db_path).await.unwrap() });

    let schema = CollectionSchema {
        name: "search_collection".to_string(),
        dimension: 768,
        metric: DistanceMetric::Cosine,
    };

    rt.block_on(async {
        adapter.create_collection(&schema).await.unwrap();
        let points = generate_points(10000, 768);
        adapter.insert("search_collection", points).await.unwrap();
    });

    let query_vector = generate_vector(768);

    let mut group = c.benchmark_group("lancedb_search");
    group.measurement_time(Duration::from_secs(10));

    for k in [1, 10, 100].iter() {
        let query = bridge_kabod::types::VectorQuery {
            collection: "search_collection".to_string(),
            vector: query_vector.clone(),
            top_k: *k,
            filter: None,
            offset: None,
            include_metadata: true,
            include_vector: true,
            aggregations: Vec::new(),
        };

        group.bench_with_input(BenchmarkId::new("top_k", k), &query, |b, query| {
            b.to_async(&rt).iter(|| async {
                adapter.search(black_box(query)).await.unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_insert, bench_search);
criterion_main!(benches);
