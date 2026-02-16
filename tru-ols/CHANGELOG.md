# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.0 (2026-02-16)

<csr-id-46bee42d4f28d185b38446c0d950c2579c422f43/>
<csr-id-c987a225570c2afae480800327d0072ab4b4e4ad/>
<csr-id-60d00956fa56c883b3c04e4c58bad677b27c6b24/>
<csr-id-089feff624625a5ddf0b1da570e4f60b6fedf09b/>

### Chore

 - <csr-id-46bee42d4f28d185b38446c0d950c2579c422f43/> update dependencies and align workspace configurations
   - Updated various dependencies in Cargo.toml files across multiple crates to their latest versions for improved functionality and compatibility.
   - Changed several dependencies to use workspace references for consistency and to reduce duplication.
   - Notable updates include polars to version 0.53.0, faer to version 0.24, and ndarray-linalg to version 0.18.1.
   - Adjusted dev-dependencies to utilize workspace settings for better management.
 - <csr-id-c987a225570c2afae480800327d0072ab4b4e4ad/> clean up unused imports and variables
   - Remove unused imports in clustering and gating modules
   - Fix unreachable code warning in DBSCAN
   - Remove unused mut keywords
   - Clean up warnings for better code quality

### Bug Fixes

 - <csr-id-758d02ec3225382d81b566db15a30e5cd4863e16/> update rand and polars APIs for compatibility
   - Use rand::RngExt for random_range (rand 0.10)
   - Use DataFrame::new_infer_height for polars 0.53

### Chore

 - <csr-id-089feff624625a5ddf0b1da570e4f60b6fedf09b/> update changelogs prior to release

### Documentation

 - <csr-id-292bd202b232c6f780a9cc7170cc1d53b443e05e/> add CLI reference and validation reports
   - CLI_ARGUMENTS_REFERENCE: complete argument reference for tru-ols unmix

### New Features

 - <csr-id-9c3354e3667460949ce836783ce02604c972efde/> unmixing, preprocessing, and FCS integration
   - TRU-OLS unmixing with cutoff and iterative removal

### Refactor

 - <csr-id-60d00956fa56c883b3c04e4c58bad677b27c6b24/> replace ndarray with faer for linear algebra
   - Use faer Mat/Col/MatRef/ColRef in TruOls, preprocessing, unmixing
   - solve_linear_system uses faer pure-Rust solver
   - Add optional blas feature for ndarray-linalg least-squares
   - extract_detector_data, apply_tru_ols_unmixing use faer types
   - Update plotting, fcs_integration, tests, benchmarks
   - Update lib doctest to faer API

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update rand and polars APIs for compatibility ([`758d02e`](https://github.com/jrmoynihan/flow/commit/758d02ec3225382d81b566db15a30e5cd4863e16))
    - Merge pull request #14 from jrmoynihan/gpu-acceleration ([`01edbec`](https://github.com/jrmoynihan/flow/commit/01edbecfc222685a8e052eb26b001d3fae4dfe13))
</details>

