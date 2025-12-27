use bridge_kabod::query::QueryBuilder;
use bridge_kabod::types::{CollectionSchema, DistanceMetric, Point, VectorQuery};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;
use std::collections::HashMap;
use std::hint::black_box;

fn generate_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    (0..dim).map(|_| rng.random::<f32>()).collect()
}

fn bench_point_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("point_creation");

    for dim in [128, 384, 768, 1536].iter() {
        let vector = generate_vector(*dim);

        group.bench_with_input(BenchmarkId::new("dimension", dim), &vector, |b, vector| {
            b.iter(|| Point {
                id: black_box("test_id".to_string()),
                vector: black_box(vector.clone()),
                metadata: None,
            });
        });
    }

    group.finish();
}

fn bench_point_with_metadata(c: &mut Criterion) {
    let mut group = c.benchmark_group("point_with_metadata");
    let vector = generate_vector(768);

    for metadata_fields in [1, 5, 10, 20].iter() {
        let mut metadata = HashMap::new();
        for i in 0..*metadata_fields {
            metadata.insert(
                format!("field_{}", i),
                serde_json::json!(format!("value_{}", i)),
            );
        }

        group.bench_with_input(
            BenchmarkId::new("fields", metadata_fields),
            &metadata,
            |b, metadata| {
                b.iter(|| Point {
                    id: black_box("test_id".to_string()),
                    vector: black_box(vector.clone()),
                    metadata: Some(black_box(metadata.clone())),
                });
            },
        );
    }

    group.finish();
}

fn bench_query_builder(c: &mut Criterion) {
    let vector = generate_vector(768);

    c.bench_function("query_builder_simple", |b| {
        b.iter(|| {
            QueryBuilder::new(black_box("test_collection"), black_box(vector.clone()))
                .limit(black_box(10))
                .build()
        });
    });
}

fn bench_schema_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("schema_creation");

    for dim in [128, 384, 768, 1536].iter() {
        group.bench_with_input(BenchmarkId::new("dimension", dim), dim, |b, dim| {
            b.iter(|| CollectionSchema {
                name: black_box("test_collection".to_string()),
                dimension: black_box(*dim),
                metric: black_box(DistanceMetric::Cosine),
            });
        });
    }

    group.finish();
}

fn bench_vector_query_creation(c: &mut Criterion) {
    let vector = generate_vector(768);

    c.bench_function("vector_query_creation", |b| {
        b.iter(|| VectorQuery {
            collection: black_box("test_collection".to_string()),
            vector: black_box(Some(vector.clone())),
            top_k: black_box(10),
            filter: None,
            offset: None,
            include_metadata: true,
            include_vector: true,
            aggregations: Vec::new(),
        });
    });
}

criterion_group!(
    benches,
    bench_point_creation,
    bench_point_with_metadata,
    bench_query_builder,
    bench_schema_creation,
    bench_vector_query_creation
);
criterion_main!(benches);
