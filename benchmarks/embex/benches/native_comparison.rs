use bridge_embex::types::Point;
use bridge_embex_infrastructure::observability::EmbexMetrics;
use criterion::{criterion_group, criterion_main, Criterion};
use bridge_embex_core::types::{CollectionSchema, DistanceMetric};
use std::time::Duration;
use std::hint::black_box;
use bridge_embex::client::EmbexClient;
use bridge_embex_infrastructure::config::EmbexConfig;


fn generate_vector(dim: usize) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    dim.hash(&mut hasher);
    let mut rng = hasher.finish();
    
    (0..dim)
        .map(|_| {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            (rng as f32) / (u64::MAX as f32) * 2.0 - 1.0
        })
        .collect()
}


/// Benchmark filter conversion overhead
fn bench_filter_conversion(c: &mut Criterion) {
    use bridge_embex::types::Filter;
    use serde_json::json;

    let mut group = c.benchmark_group("filter_conversion");

    // Simple filter
    let simple_filter = json!({
        "op": "key",
        "args": ["status", {"op": "eq", "args": "active"}]
    });

    group.bench_function("simple_filter", |b| {
        b.iter(|| {
            let filter: Filter = serde_json::from_value(black_box(simple_filter.clone())).unwrap();
            black_box(filter);
        });
    });

    // Complex nested filter
    let complex_filter = json!({
        "op": "and",
        "args": [
            {"op": "key", "args": ["status", {"op": "eq", "args": "active"}]},
            {"op": "key", "args": ["score", {"op": "gte", "args": 10}]},
            {
                "op": "or",
                "args": [
                    {"op": "key", "args": ["category", {"op": "eq", "args": "A"}]},
                    {"op": "key", "args": ["category", {"op": "eq", "args": "B"}]}
                ]
            }
        ]
    });

    group.bench_function("complex_filter", |b| {
        b.iter(|| {
            let filter: Filter = serde_json::from_value(black_box(complex_filter.clone())).unwrap();
            black_box(filter);
        });
    });

    group.finish();
}

/// Benchmark serialization/deserialization overhead
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    let metadata: std::collections::HashMap<String, serde_json::Value> = serde_json::from_value(
        serde_json::json!({
            "field1": "value1",
            "field2": 42,
            "field3": true
        })
    ).unwrap();
    let point = Point {
        id: "test_id".to_string(),
        vector: generate_vector(768),
        metadata: Some(metadata),
    };

    group.bench_function("point_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_value(black_box(&point)).unwrap();
            black_box(json);
        });
    });

    group.bench_function("json_to_point", |b| {
        let json = serde_json::to_value(&point).unwrap();
        b.iter(|| {
            let point: Point = serde_json::from_value(black_box(json.clone())).unwrap();
            black_box(point);
        });
    });

    group.finish();
}

/// Benchmark query builder overhead
fn bench_query_builder_overhead(c: &mut Criterion) {
    use bridge_embex::query::QueryBuilder;
    
    let vector = generate_vector(768);
    let mut group = c.benchmark_group("query_builder");

    group.bench_function("simple_builder", |b| {
        b.iter(|| {
            QueryBuilder::new("test_collection", black_box(vector.clone()))
                .limit(10)
                .build()
        });
    });

    group.bench_function("builder_with_filter", |b| {
        use bridge_embex::types::Filter;
        let filter = Filter::eq("status", "active");
        b.iter(|| {
            QueryBuilder::new("test_collection", black_box(vector.clone()))
                .limit(10)
                .filter(black_box(filter.clone()))
                .build()
        });
    });

    group.finish();
}

/// Benchmark metrics recording overhead
fn bench_metrics_overhead(c: &mut Criterion) {
    let metrics = EmbexMetrics::new();
    let mut group = c.benchmark_group("metrics");

    group.bench_function("record_insert", |b| {
        b.iter(|| {
            metrics.record_insert(black_box(10));
        });
    });

    group.bench_function("record_search", |b| {
        b.iter(|| {
            metrics.record_search(black_box(20));
        });
    });

    group.bench_function("snapshot", |b| {
        b.iter(|| {
            let snapshot = metrics.snapshot();
            black_box(snapshot);
        });
    });

    group.finish();
}

/// Benchmark client initialization overhead
fn bench_client_init(c: &mut Criterion) {
    
    let group = c.benchmark_group("client_init");

    #[cfg(feature = "qdrant")]
    {
        let config = EmbexConfig {
            provider: "qdrant".to_string(),
            url: "http://localhost:6333".to_string(),
            api_key: None,
            timeout_ms: None,
            pool_size: 10,
            idle_timeout_secs: 90,
            options: Default::default(),
        };

        group.bench_function("qdrant_init", |b| {
            b.iter(|| {
                let client = EmbexClient::new(black_box(config.clone())).ok();
                black_box(client);
            });
        });
    }

    #[cfg(feature = "weaviate")]
    {
        let config = EmbexConfig {
            provider: "weaviate".to_string(),
            url: "http://localhost:8080".to_string(),
            api_key: None,
            timeout_ms: None,
            pool_size: 10,
            idle_timeout_secs: 90,
            options: Default::default(),
        };

        group.bench_function("weaviate_init", |b| {
            b.iter(|| {
                let client = EmbexClient::new(black_box(config.clone())).ok();
                black_box(client);
            });
        });
    }

    group.finish();
}

/// Benchmark collection operations overhead
fn bench_collection_operations(c: &mut Criterion) {
    
    let _rt = tokio::runtime::Runtime::new().unwrap();
    
    #[cfg(feature = "lancedb")]
    {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        
        let config = EmbexConfig {
            provider: "lancedb".to_string(),
            url: db_path.to_string(),
            api_key: None,
            timeout_ms: None,
            pool_size: 10,
            idle_timeout_secs: 90,
            options: Default::default(),
        };

        let client = _rt.block_on(async {
            EmbexClient::new_async(config).await.unwrap()
        });

        let schema = CollectionSchema {
            name: "bench_collection".to_string(),
            dimension: 768,
            metric: DistanceMetric::Cosine,
        };

        _rt.block_on(async {
            let collection = client.collection("bench_collection");
            collection.create(schema).await.unwrap();
        });

        let mut group = c.benchmark_group("collection_operations");
        group.measurement_time(Duration::from_secs(5));

        // Benchmark collection access
        group.bench_function("collection_access", |b| {
            b.iter(|| {
                let collection = client.collection(black_box("bench_collection"));
                black_box(collection);
            });
        });

        // Benchmark insert with metrics
        let points: Vec<Point> = (0..100)
            .map(|i| {
                Point::new(
                    format!("point_{}", i),
                    generate_vector(768),
                )
            })
            .collect();
        group.bench_function("insert_with_metrics", |b| {
            b.to_async(&_rt).iter(|| async {
                let collection = client.collection("bench_collection");
                collection.insert(black_box(points.clone())).await.ok();
            });
        });

        group.finish();
    }
}

criterion_group!(
    benches,
    bench_filter_conversion,
    bench_serialization,
    bench_query_builder_overhead,
    bench_metrics_overhead,
    bench_client_init,
    bench_collection_operations
);
criterion_main!(benches);

