# flow-utils

Shared algorithms and utilities for flow cytometry crates.

This crate provides high-performance implementations of common algorithms used across multiple flow cytometry crates.

## Features

- **Kernel Density Estimation (KDE)**: FFT-accelerated KDE with automatic bandwidth selection
- **Clustering**: K-means, DBSCAN (pending), and Gaussian Mixture Model clustering
- **PCA**: Principal Component Analysis for dimensionality reduction

## Usage

### Kernel Density Estimation

```rust
use flow_utils::KernelDensity;

let data = vec![1.0, 2.0, 3.0, 2.0, 1.0, 5.0, 6.0, 7.0, 6.0, 5.0];
let kde = KernelDensity::estimate(&data, 1.0, 512)?;

// Find peaks in the density
let peaks = kde.find_peaks(0.3);
```

### K-means Clustering

```rust
use flow_utils::{KMeans, KMeansConfig};
use ndarray::Array2;

let data = Array2::from_shape_vec((100, 2), vec![...])?;
let config = KMeansConfig {
    n_clusters: 3,
    max_iterations: 300,
    tolerance: 1e-4,
    seed: None,
};

let result = KMeans::fit(&data, &config)?;
println!("Found {} clusters", result.centroids.nrows());
```

### PCA

```rust
use flow_utils::Pca;
use ndarray::Array2;

let data = Array2::from_shape_vec((100, 10), vec![...])?;
let mut pca = Pca::new(2); // Keep 2 components
let pca = pca.fit(&data)?;

let transformed = pca.transform(&data)?;
```

## Performance

- **KDE**: Uses FFT acceleration for O(n log n) performance
- **Clustering**: Uses linfa-clustering for optimized implementations
- **PCA**: Uses linfa-linalg SVD for efficient decomposition

## Known Limitations

- **DBSCAN**: Temporarily disabled due to linfa-clustering API limitations (ParamGuard trait bound issue)
- Use K-means or GMM as alternatives for density-based clustering needs

## Dependencies

- `ndarray` 0.16 (for linfa compatibility)
- `linfa-clustering` 0.8
- `linfa-linalg` 0.2 (for SVD)
- `realfft` 3.5.0 (for FFT-accelerated KDE)
