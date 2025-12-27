use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use bridge_core::simd;

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

fn bench_dot_product(c: &mut Criterion) {
    let dims = vec![128, 256, 512, 768, 1536];
    
    for dim in dims {
        let a = generate_vector(dim, 1);
        let b = generate_vector(dim, 2);
        
        c.bench_function(&format!("dot_product_{}", dim), |bench| {
            bench.iter(|| simd::dot_product(black_box(&a), black_box(&b)))
        });
    }
}

fn bench_l2_distance(c: &mut Criterion) {
    let dims = vec![128, 256, 512, 768, 1536];
    
    for dim in dims {
        let a = generate_vector(dim, 1);
        let b = generate_vector(dim, 2);
        
        c.bench_function(&format!("l2_distance_{}", dim), |bench| {
            bench.iter(|| simd::l2_distance(black_box(&a), black_box(&b)))
        });
    }
}

fn bench_cosine_similarity(c: &mut Criterion) {
    let dims = vec![128, 256, 512, 768, 1536];
    
    for dim in dims {
        let a = generate_vector(dim, 1);
        let b = generate_vector(dim, 2);
        
        c.bench_function(&format!("cosine_similarity_{}", dim), |bench| {
            bench.iter(|| simd::cosine_similarity(black_box(&a), black_box(&b)))
        });
    }
}

fn bench_normalize(c: &mut Criterion) {
    let dims = vec![128, 256, 512, 768, 1536];
    
    for dim in dims {
        let v = generate_vector(dim, 1);
        
        c.bench_function(&format!("normalize_{}", dim), |bench| {
            bench.iter(|| {
                let mut v = v.clone();
                simd::normalize_in_place(black_box(&mut v));
            })
        });
    }
}

criterion_group!(
    benches,
    bench_dot_product,
    bench_l2_distance,
    bench_cosine_similarity,
    bench_normalize
);
criterion_main!(benches);

