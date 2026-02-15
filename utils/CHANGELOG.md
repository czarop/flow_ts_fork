# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Chore

 - <csr-id-46bee42d4f28d185b38446c0d950c2579c422f43/> update dependencies and align workspace configurations
   - Updated various dependencies in Cargo.toml files across multiple crates to their latest versions for improved functionality and compatibility.
   - Changed several dependencies to use workspace references for consistency and to reduce duplication.
   - Notable updates include polars to version 0.53.0, faer to version 0.24, and ndarray-linalg to version 0.18.1.
   - Adjusted dev-dependencies to utilize workspace settings for better management.
 - <csr-id-126b1a9a8542ca5314763a1857d276bd1ed1d46b/> use workspace ndarray dependency
   Align with workspace ndarray 0.16 for linfa compatibility
 - <csr-id-c987a225570c2afae480800327d0072ab4b4e4ad/> clean up unused imports and variables
   - Remove unused imports in clustering and gating modules
   - Fix unreachable code warning in DBSCAN
   - Remove unused mut keywords
   - Clean up warnings for better code quality

### Documentation

 - <csr-id-6f6d0f59369453e3f0018b37f1377b204b023223/> add comprehensive documentation for flow-utils and research notes
   - Add README for flow-utils crate with usage examples
   - Add CRATE_RESEARCH.md documenting crate evaluation and decisions
   - Add RESEARCH_NOTES.md for automated gating algorithms and decisions
   - Document performance vs accuracy tradeoffs
   - Note known limitations and future work

### New Features

 - <csr-id-232769369212d1bd89895937a8de375d38eaff19/> extend KDE module for peak detection
   - Add KDE utilities for spectral analysis
   - Update kde2d implementation
 - <csr-id-10de203537b2d74c98658d01e037d07494966c61/> add fit_from_rows helpers for version compatibility
   - Add KMeans::fit_from_rows to accept Vec<Vec<f64>>
   - Add Gmm::fit_from_rows to accept Vec<Vec<f64>>
   - Enables compatibility between ndarray 0.17 (flow-gates) and 0.16 (flow-utils)
 - <csr-id-c89944be9c68a1f688dfb5ee333c7562b28f90b1/> add 2D KDE for improved density contours
   - Implement KernelDensity2D for 2D scatter plot density estimation
   - Use 2D FFT convolution for efficient computation
   - Add contour extraction at density thresholds
   - Update scatter gating to use 2D KDE for better density contours
   - Generate polygon gates from density contours
 - <csr-id-824d6d407a673c2d2ad511e15b16d7f8596b5700/> create flow-utils crate with KDE, clustering, and PCA modules
   - Add Kernel Density Estimation (KDE) with FFT acceleration
   - Add clustering algorithms: K-means, DBSCAN, GMM using linfa-clustering
   - Add Principal Component Analysis (PCA) using ndarray-linalg
   - Add common utilities for statistics (std dev, IQR, gaussian kernel)
   - Note: linfa API integration needs refinement (compilation errors remain)

### Bug Fixes

 - <csr-id-38013b28d81af8510a1065745d203bd5e2057518/> resolve ndarray version mismatch for clustering
   - Add fit_from_rows helper methods to KMeans and GMM
   - Convert Array2 from ndarray 0.17 to Vec<Vec<f64>> for compatibility
   - Resolve type mismatch between flow-gates (ndarray 0.17) and flow-utils (ndarray 0.16)
 - <csr-id-9ab00ceb749f95e663080d4fb67992eee6cc7f0f/> fix DBSCAN compilation error
   - Remove references to model variable that was commented out
   - Return placeholder result until API is fixed
 - <csr-id-82c4b1b0c8859638d3b721d0dfb873c461dbf15f/> temporarily disable DBSCAN due to linfa API limitations
   - DBSCAN ValidParams doesn't satisfy ParamGuard trait bound
   - Add clear error message directing users to K-means/GMM alternatives
   - Document as known limitation for future resolution
 - <csr-id-afcf794b099ac04beba555d1b79d495de6cbd71b/> fix linfa DatasetBase creation for clustering
   - Use DatasetBase::new(data, ()) instead of DatasetBase::from
   - Explicitly set targets to () for unsupervised learning
   - Fixes Array2 Records trait compatibility issue
 - <csr-id-437c9588b42030748987c766b90e0c4ce5bfb847/> fix PCA SVD result handling
   - Remove duplicate ok_or_else call on vt (already unwrapped)
   - SVD returns tuple with Option<U>, S, Option<Vt>
 - <csr-id-2c0585b4a6ba64f1e1e073d0716faf3d30395ac8/> fix SVD result handling in PCA module
   - Fix tuple destructuring for linfa-linalg SVD result
   - Use linfa-linalg instead of ndarray-linalg for ndarray 0.16 compatibility

### Other

 - <csr-id-d03eaad90c006d26f5a5536e263d2d11f01c73ba/> add clustering modules with linfa integration
   - Add K-means, DBSCAN, and GMM clustering wrappers
   - Note: linfa DatasetBase API integration needs refinement
   - Array2 Records trait implementation issue to be resolved

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 15 commits contributed to the release over the course of 21 calendar days.
 - 15 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update dependencies and align workspace configurations ([`46bee42`](https://github.com/jrmoynihan/flow/commit/46bee42d4f28d185b38446c0d950c2579c422f43))
    - Use workspace ndarray dependency ([`126b1a9`](https://github.com/jrmoynihan/flow/commit/126b1a9a8542ca5314763a1857d276bd1ed1d46b))
    - Extend KDE module for peak detection ([`2327693`](https://github.com/jrmoynihan/flow/commit/232769369212d1bd89895937a8de375d38eaff19))
    - Add fit_from_rows helpers for version compatibility ([`10de203`](https://github.com/jrmoynihan/flow/commit/10de203537b2d74c98658d01e037d07494966c61))
    - Resolve ndarray version mismatch for clustering ([`38013b2`](https://github.com/jrmoynihan/flow/commit/38013b28d81af8510a1065745d203bd5e2057518))
    - Add 2D KDE for improved density contours ([`c89944b`](https://github.com/jrmoynihan/flow/commit/c89944be9c68a1f688dfb5ee333c7562b28f90b1))
    - Clean up unused imports and variables ([`c987a22`](https://github.com/jrmoynihan/flow/commit/c987a225570c2afae480800327d0072ab4b4e4ad))
    - Add comprehensive documentation for flow-utils and research notes ([`6f6d0f5`](https://github.com/jrmoynihan/flow/commit/6f6d0f59369453e3f0018b37f1377b204b023223))
    - Fix DBSCAN compilation error ([`9ab00ce`](https://github.com/jrmoynihan/flow/commit/9ab00ceb749f95e663080d4fb67992eee6cc7f0f))
    - Temporarily disable DBSCAN due to linfa API limitations ([`82c4b1b`](https://github.com/jrmoynihan/flow/commit/82c4b1b0c8859638d3b721d0dfb873c461dbf15f))
    - Fix linfa DatasetBase creation for clustering ([`afcf794`](https://github.com/jrmoynihan/flow/commit/afcf794b099ac04beba555d1b79d495de6cbd71b))
    - Add clustering modules with linfa integration ([`d03eaad`](https://github.com/jrmoynihan/flow/commit/d03eaad90c006d26f5a5536e263d2d11f01c73ba))
    - Fix PCA SVD result handling ([`437c958`](https://github.com/jrmoynihan/flow/commit/437c9588b42030748987c766b90e0c4ce5bfb847))
    - Fix SVD result handling in PCA module ([`2c0585b`](https://github.com/jrmoynihan/flow/commit/2c0585b4a6ba64f1e1e073d0716faf3d30395ac8))
    - Create flow-utils crate with KDE, clustering, and PCA modules ([`824d6d4`](https://github.com/jrmoynihan/flow/commit/824d6d407a673c2d2ad511e15b16d7f8596b5700))
</details>

