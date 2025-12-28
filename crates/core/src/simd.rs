//! SIMD-accelerated vector operations
//!
//! This module provides high-performance vector operations using SIMD instructions
//! with automatic fallback to scalar implementations when SIMD is not available.
//!
//! # Usage
//!
//! Enable SIMD optimizations by compiling with appropriate target features:
//! ```bash
//! RUSTFLAGS="-C target-feature=+avx2" cargo build --release
//! # or for SSE4.1:
//! RUSTFLAGS="-C target-feature=+sse4.1" cargo build --release
//! ```
//!
//! The functions automatically use SIMD when available, otherwise fall back to scalar operations.

/// Computes the dot product of two vectors.
///
/// # Panics
/// Panics if the vectors have different lengths.
#[inline]
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    
    #[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "avx2"))]
    return unsafe { dot_product_avx2(a, b) };
    
    #[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "sse4.1", not(target_feature = "avx2")))]
    return unsafe { dot_product_sse4(a, b) };
    
    #[cfg(all(target_arch = "aarch64", feature = "simd", target_feature = "neon"))]
    return unsafe { dot_product_neon(a, b) };
    
    #[allow(unreachable_code)]
    // Scalar fallback (always available when SIMD is not enabled)
    dot_product_scalar(a, b)
}

/// Computes the L2 (Euclidean) distance between two vectors.
///
/// # Panics
/// Panics if the vectors have different lengths.
#[inline]
pub fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    
    #[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "avx2"))]
    return unsafe { l2_distance_avx2(a, b) };
    
    #[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "sse4.1", not(target_feature = "avx2")))]
    return unsafe { l2_distance_sse4(a, b) };
    
    #[cfg(all(target_arch = "aarch64", feature = "simd", target_feature = "neon"))]
    return unsafe { l2_distance_neon(a, b) };
    
    #[allow(unreachable_code)]
    // Scalar fallback (always available when SIMD is not enabled)
    l2_distance_scalar(a, b)
}

/// Computes the cosine similarity between two vectors.
///
/// Returns a value in the range [-1, 1], where 1 means identical direction.
///
/// # Panics
/// Panics if the vectors have different lengths or if either vector has zero magnitude.
#[inline]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    
    let dot = dot_product(a, b);
    let norm_a = dot_product(a, a).sqrt();
    let norm_b = dot_product(b, b).sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}

/// Normalizes a vector in-place to unit length.
///
/// # Panics
/// Panics if the vector has zero magnitude.
#[inline]
pub fn normalize_in_place(v: &mut [f32]) {
    let norm = dot_product(v, v).sqrt();
    assert!(norm > 0.0, "Cannot normalize zero vector");
    
    let inv_norm = 1.0 / norm;
    
    #[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "avx2"))]
    {
        unsafe { normalize_in_place_avx2(v, inv_norm) };
        return;
    }
    
    #[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "sse4.1", not(target_feature = "avx2")))]
    {
        unsafe { normalize_in_place_sse4(v, inv_norm) };
        return;
    }
    
    #[cfg(all(target_arch = "aarch64", feature = "simd", target_feature = "neon"))]
    {
        unsafe { normalize_in_place_neon(v, inv_norm) };
        return;
    }
    
    #[allow(unreachable_code)]
    // Scalar fallback
    normalize_in_place_scalar(v, inv_norm);
}

/// Computes the L2 norm (magnitude) of a vector.
#[inline]
pub fn l2_norm(v: &[f32]) -> f32 {
    dot_product(v, v).sqrt()
}

// ============================================================================
// Scalar implementations (always available as fallback)
// ============================================================================

#[inline(never)]
fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[inline(never)]
fn l2_distance_squared_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = x - y;
            diff * diff
        })
        .sum::<f32>()
}

#[inline(never)]
fn l2_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    l2_distance_squared_scalar(a, b).sqrt()
}

#[inline(never)]
fn normalize_in_place_scalar(v: &mut [f32], inv_norm: f32) {
    for x in v.iter_mut() {
        *x *= inv_norm;
    }
}

// ============================================================================
// x86_64 AVX2 implementations
// ============================================================================

#[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "avx2"))]
#[target_feature(enable = "avx2")]
unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    
    let len = a.len();
    let mut sum = _mm256_setzero_ps();
    
    let chunks = len / 8;
    for i in 0..chunks {
        let idx = i * 8;
        let va = _mm256_loadu_ps(a.as_ptr().add(idx));
        let vb = _mm256_loadu_ps(b.as_ptr().add(idx));
        sum = _mm256_fmadd_ps(va, vb, sum);
    }
    
    // Horizontal sum: reduce 8 elements to 1
    let sum = _mm256_add_ps(sum, _mm256_permute2f128_ps(sum, sum, 1));
    let sum = _mm256_hadd_ps(sum, sum);
    let sum = _mm256_hadd_ps(sum, sum);
    let result = _mm_cvtss_f32(_mm256_castps256_ps128(sum));
    
    // Handle remainder
    let remainder = len % 8;
    if remainder > 0 {
        let start = chunks * 8;
        result + dot_product_scalar(&a[start..], &b[start..])
    } else {
        result
    }
}

#[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "avx2"))]
#[target_feature(enable = "avx2")]
unsafe fn l2_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    
    let len = a.len();
    let mut sum = _mm256_setzero_ps();
    
    let chunks = len / 8;
    for i in 0..chunks {
        let idx = i * 8;
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(idx)) };
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(idx)) };
        let diff = _mm256_sub_ps(va, vb);
        sum = _mm256_fmadd_ps(diff, diff, sum);
    }
    
    // Horizontal sum
    let sum = _mm256_add_ps(sum, _mm256_permute2f128_ps(sum, sum, 1));
    let sum = _mm256_hadd_ps(sum, sum);
    let sum = _mm256_hadd_ps(sum, sum);
    let result = _mm_cvtss_f32(_mm256_castps256_ps128(sum));
    
    // Handle remainder
    let remainder = len % 8;
    if remainder > 0 {
        let start = chunks * 8;
        (result + l2_distance_squared_scalar(&a[start..], &b[start..])).sqrt()
    } else {
        result.sqrt()
    }
}

#[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "avx2"))]
#[target_feature(enable = "avx2")]
unsafe fn normalize_in_place_avx2(v: &mut [f32], inv_norm: f32) {
    use std::arch::x86_64::*;
    
    let len = v.len();
    let scale = _mm256_set1_ps(inv_norm);
    
    let chunks = len / 8;
    for i in 0..chunks {
        let idx = i * 8;
        let val = unsafe { _mm256_loadu_ps(v.as_ptr().add(idx)) };
        let scaled = _mm256_mul_ps(val, scale);
        unsafe { _mm256_storeu_ps(v.as_mut_ptr().add(idx), scaled) };
    }
    
    // Handle remainder
    let remainder = len % 8;
    if remainder > 0 {
        let start = chunks * 8;
        normalize_in_place_scalar(&mut v[start..], inv_norm);
    }
}

// ============================================================================
// x86_64 SSE4.1 implementations
// ============================================================================

#[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "sse4.1"))]
#[target_feature(enable = "sse4.1")]
unsafe fn dot_product_sse4(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    
    let len = a.len();
    let mut sum = _mm_setzero_ps();
    
    let chunks = len / 4;
    for i in 0..chunks {
        let idx = i * 4;
        let va = unsafe { _mm_loadu_ps(a.as_ptr().add(idx)) };
        let vb = unsafe { _mm_loadu_ps(b.as_ptr().add(idx)) };
        sum = _mm_add_ps(sum, _mm_mul_ps(va, vb));
    }
    
    // Horizontal sum
    let sum = _mm_hadd_ps(sum, sum);
    let sum = _mm_hadd_ps(sum, sum);
    let result = _mm_cvtss_f32(sum);
    
    // Handle remainder
    let remainder = len % 4;
    if remainder > 0 {
        let start = chunks * 4;
        result + dot_product_scalar(&a[start..], &b[start..])
    } else {
        result
    }
}

#[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "sse4.1"))]
#[target_feature(enable = "sse4.1")]
unsafe fn l2_distance_sse4(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    
    let len = a.len();
    let mut sum = _mm_setzero_ps();
    
    let chunks = len / 4;
    for i in 0..chunks {
        let idx = i * 4;
        let va = unsafe { _mm_loadu_ps(a.as_ptr().add(idx)) };
        let vb = unsafe { _mm_loadu_ps(b.as_ptr().add(idx)) };
        let diff = _mm_sub_ps(va, vb);
        sum = _mm_add_ps(sum, _mm_mul_ps(diff, diff));
    }
    
    // Horizontal sum
    let sum = _mm_hadd_ps(sum, sum);
    let sum = _mm_hadd_ps(sum, sum);
    let result = _mm_cvtss_f32(sum);
    
    // Handle remainder
    let remainder = len % 4;
    if remainder > 0 {
        let start = chunks * 4;
        (result + l2_distance_squared_scalar(&a[start..], &b[start..])).sqrt()
    } else {
        result.sqrt()
    }
}

#[cfg(all(target_arch = "x86_64", feature = "simd", target_feature = "sse4.1"))]
#[target_feature(enable = "sse4.1")]
unsafe fn normalize_in_place_sse4(v: &mut [f32], inv_norm: f32) {
    use std::arch::x86_64::*;
    
    let len = v.len();
    let scale = _mm_set1_ps(inv_norm);
    
    let chunks = len / 4;
    for i in 0..chunks {
        let idx = i * 4;
        let val = unsafe { _mm_loadu_ps(v.as_ptr().add(idx)) };
        let scaled = _mm_mul_ps(val, scale);
        unsafe { _mm_storeu_ps(v.as_mut_ptr().add(idx), scaled) };
    }
    
    // Handle remainder
    let remainder = len % 4;
    if remainder > 0 {
        let start = chunks * 4;
        normalize_in_place_scalar(&mut v[start..], inv_norm);
    }
}

// ============================================================================
// ARM NEON implementations
// ============================================================================

#[cfg(all(target_arch = "aarch64", feature = "simd", target_feature = "neon"))]
#[target_feature(enable = "neon")]
unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::aarch64::*;
    
    let len = a.len();
    let mut sum = vdupq_n_f32(0.0);
    
    let chunks = len / 4;
    for i in 0..chunks {
        let idx = i * 4;
        let va = unsafe { vld1q_f32(a.as_ptr().add(idx)) };
        let vb = unsafe { vld1q_f32(b.as_ptr().add(idx)) };
        sum = vfmaq_f32(sum, va, vb);
    }
    
    // Horizontal sum
    let sum = vaddvq_f32(sum);
    
    // Handle remainder
    let remainder = len % 4;
    if remainder > 0 {
        let start = chunks * 4;
        sum + dot_product_scalar(&a[start..], &b[start..])
    } else {
        sum
    }
}

#[cfg(all(target_arch = "aarch64", feature = "simd", target_feature = "neon"))]
#[target_feature(enable = "neon")]
unsafe fn l2_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::aarch64::*;
    
    let len = a.len();
    let mut sum = vdupq_n_f32(0.0);
    
    let chunks = len / 4;
    for i in 0..chunks {
        let idx = i * 4;
        let va = unsafe { vld1q_f32(a.as_ptr().add(idx)) };
        let vb = unsafe { vld1q_f32(b.as_ptr().add(idx)) };
        let diff = vsubq_f32(va, vb);
        sum = vfmaq_f32(sum, diff, diff);
    }
    
    // Horizontal sum
    let sum = vaddvq_f32(sum);
    
    // Handle remainder
    let remainder = len % 4;
    if remainder > 0 {
        let start = chunks * 4;
        (sum + l2_distance_squared_scalar(&a[start..], &b[start..])).sqrt()
    } else {
        sum.sqrt()
    }
}

#[cfg(all(target_arch = "aarch64", feature = "simd", target_feature = "neon"))]
#[target_feature(enable = "neon")]
unsafe fn normalize_in_place_neon(v: &mut [f32], inv_norm: f32) {
    use std::arch::aarch64::*;
    
    let len = v.len();
    let scale = vdupq_n_f32(inv_norm);
    
    let chunks = len / 4;
    for i in 0..chunks {
        let idx = i * 4;
        let val = unsafe { vld1q_f32(v.as_ptr().add(idx)) };
        let scaled = vmulq_f32(val, scale);
        unsafe { vst1q_f32(v.as_mut_ptr().add(idx), scaled) };
    }
    
    // Handle remainder
    let remainder = len % 4;
    if remainder > 0 {
        let start = chunks * 4;
        normalize_in_place_scalar(&mut v[start..], inv_norm);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let result = dot_product(&a, &b);
        assert!((result - 32.0).abs() < 1e-5);
    }

    #[test]
    fn test_dot_product_large() {
        let a = generate_vector(768, 1);
        let b = generate_vector(768, 2);
        let result = dot_product(&a, &b);
        assert!(result.is_finite());
    }

    #[test]
    fn test_l2_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        let result = l2_distance(&a, &b);
        assert!((result - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_l2_distance_large() {
        let a = generate_vector(768, 1);
        let b = generate_vector(768, 2);
        let result = l2_distance(&a, &b);
        assert!(result.is_finite() && result >= 0.0);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        let result = cosine_similarity(&a, &b);
        assert!((result - 1.0).abs() < 1e-5);
        
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let result = cosine_similarity(&a, &b);
        assert!(result.abs() < 1e-5);
    }

    #[test]
    fn test_cosine_similarity_large() {
        let a = generate_vector(768, 1);
        let b = generate_vector(768, 2);
        let result = cosine_similarity(&a, &b);
        assert!(result >= -1.0 && result <= 1.0);
    }

    #[test]
    fn test_normalize_in_place() {
        let mut v = vec![3.0, 4.0];
        normalize_in_place(&mut v);
        let norm = (v[0] * v[0] + v[1] * v[1]).sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_in_place_large() {
        let mut v = generate_vector(768, 1);
        normalize_in_place(&mut v);
        let norm = l2_norm(&v);
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_l2_norm() {
        let v = vec![3.0, 4.0];
        let norm = l2_norm(&v);
        assert!((norm - 5.0).abs() < 1e-5);
    }
}
