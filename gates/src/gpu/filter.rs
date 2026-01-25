//! GPU-accelerated batch filtering operations
//!
//! NOTE: Based on extensive benchmarking, GPU acceleration is NOT worthwhile for gate filtering.
//! GPU implementations are 2-10x slower than CPU due to overhead (data transfer, kernel launch, WGPU abstraction).
//! All GPU functions delegate directly to the optimized CPU implementation.
//! See GPU_PERFORMANCE_FINDINGS.md for detailed analysis.

use crate::error::Result;

/// Batch point-in-polygon query (GPU)
///
/// **Note**: GPU overhead is not worthwhile - delegates to optimized CPU implementation.
/// See GPU_PERFORMANCE_FINDINGS.md for performance analysis.
pub fn filter_by_polygon_batch_gpu(
    points: &[(f32, f32)],
    polygon: &[(f32, f32)],
) -> Result<Vec<bool>> {
    // GPU overhead not worthwhile - use CPU directly
    // Benchmarking shows GPU is 2-10x slower at all batch sizes
    crate::gpu::filter_by_polygon_batch_cpu(points, polygon)
}

/// Batch point-in-rectangle query (GPU)
///
/// **Note**: GPU overhead is not worthwhile - delegates to optimized CPU implementation.
/// See GPU_PERFORMANCE_FINDINGS.md for performance analysis.
pub fn filter_by_rectangle_batch_gpu(
    points: &[(f32, f32)],
    bounds: (f32, f32, f32, f32),
) -> Result<Vec<bool>> {
    // GPU overhead not worthwhile - use CPU directly
    crate::gpu::filter_by_rectangle_batch_cpu(points, bounds)
}

/// Batch point-in-ellipse query (GPU)
///
/// **Note**: GPU overhead is not worthwhile - delegates to optimized CPU implementation.
/// See GPU_PERFORMANCE_FINDINGS.md for performance analysis.
pub fn filter_by_ellipse_batch_gpu(
    points: &[(f32, f32)],
    center: (f32, f32),
    radius_x: f32,
    radius_y: f32,
    angle: f32,
) -> Result<Vec<bool>> {
    // GPU overhead not worthwhile - use CPU directly
    crate::gpu::filter_by_ellipse_batch_cpu(points, center, radius_x, radius_y, angle)
}
