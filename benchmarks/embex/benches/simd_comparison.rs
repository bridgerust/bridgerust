//! SIMD vs Scalar Performance Comparison
//!
//! This benchmark compares SIMD-accelerated vector operations against scalar implementations
//! to measure the performance improvement from SIMD optimizations.

use bridge_core::simd;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn generate_vector(dim: usize, seed: u32) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let mut rng = hasher.finish();
    
    (0..dim)
        .map(|_| {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            (rng as f32) / (u64::MAX as f32) * 2.0 - 1.0
        })
        .collect()
}

// Scalar implementations for comparison
fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn l2_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = x - y;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
}

fn cosine_similarity_scalar(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product_scalar(a, b);
    let norm_a = dot_product_scalar(a, a).sqrt();
    let norm_b = dot_product_scalar(b, b).sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}

fn bench_dot_product_comparison(c: &mut Criterion) {
    let dims = vec![128, 256, 512, 768, 1536];
    
    for dim in dims {
        let a = generate_vector(dim, 1);
        let b = generate_vector(dim, 2);
        
        let mut group = c.benchmark_group(&format!("dot_product_{}", dim));
        
        group.bench_function("simd", |bench| {
            bench.iter(|| simd::dot_product(black_box(&a), black_box(&b)))
        });
        
        group.bench_function("scalar", |bench| {
            bench.iter(|| dot_product_scalar(black_box(&a), black_box(&b)))
        });
        
        group.finish();
    }
}

fn bench_l2_distance_comparison(c: &mut Criterion) {
    let dims = vec![128, 256, 512, 768, 1536];
    
    for dim in dims {
        let a = generate_vector(dim, 1);
        let b = generate_vector(dim, 2);
        
        let mut group = c.benchmark_group(&format!("l2_distance_{}", dim));
        
        group.bench_function("simd", |bench| {
            bench.iter(|| simd::l2_distance(black_box(&a), black_box(&b)))
        });
        
        group.bench_function("scalar", |bench| {
            bench.iter(|| l2_distance_scalar(black_box(&a), black_box(&b)))
        });
        
        group.finish();
    }
}

fn bench_cosine_similarity_comparison(c: &mut Criterion) {
    let dims = vec![128, 256, 512, 768, 1536];
    
    for dim in dims {
        let a = generate_vector(dim, 1);
        let b = generate_vector(dim, 2);
        
        let mut group = c.benchmark_group(&format!("cosine_similarity_{}", dim));
        
        group.bench_function("simd", |bench| {
            bench.iter(|| simd::cosine_similarity(black_box(&a), black_box(&b)))
        });
        
        group.bench_function("scalar", |bench| {
            bench.iter(|| cosine_similarity_scalar(black_box(&a), black_box(&b)))
        });
        
        group.finish();
    }
}

fn bench_normalize_comparison(c: &mut Criterion) {
    let dims = vec![128, 256, 512, 768, 1536];
    
    for dim in dims {
        let mut group = c.benchmark_group(&format!("normalize_{}", dim));
        
        group.bench_function("simd", |bench| {
            bench.iter(|| {
                let mut v = generate_vector(dim, 1);
                simd::normalize_in_place(black_box(&mut v));
            })
        });
        
        group.bench_function("scalar", |bench| {
            bench.iter(|| {
                let mut v = generate_vector(dim, 1);
                let norm = dot_product_scalar(&v, &v).sqrt();
                let inv_norm = 1.0 / norm;
                for x in v.iter_mut() {
                    *x *= inv_norm;
                }
            })
        });
        
        group.finish();
    }
}

fn bench_batch_operations(c: &mut Criterion) {
    let dim = 768;
    let batch_sizes = vec![10, 100, 1000];
    
    for batch_size in batch_sizes {
        let query_vector = generate_vector(dim, 0);
        let vectors: Vec<Vec<f32>> = (0..batch_size)
            .map(|i| generate_vector(dim, i as u32))
            .collect();
        
        let mut group = c.benchmark_group(&format!("batch_dot_product_{}", batch_size));
        
        group.bench_function("simd", |bench| {
            bench.iter(|| {
                let mut results = Vec::with_capacity(batch_size);
                for vec in &vectors {
                    results.push(simd::dot_product(black_box(&query_vector), black_box(vec)));
                }
                black_box(results);
            })
        });
        
        group.bench_function("scalar", |bench| {
            bench.iter(|| {
                let mut results = Vec::with_capacity(batch_size);
                for vec in &vectors {
                    results.push(dot_product_scalar(black_box(&query_vector), black_box(vec)));
                }
                black_box(results);
            })
        });
        
        group.finish();
    }
}

criterion_group!(
    benches,
    bench_dot_product_comparison,
    bench_l2_distance_comparison,
    bench_cosine_similarity_comparison,
    bench_normalize_comparison,
    bench_batch_operations
);
criterion_main!(benches);

