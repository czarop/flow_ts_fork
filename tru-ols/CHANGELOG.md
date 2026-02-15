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
 - <csr-id-c987a225570c2afae480800327d0072ab4b4e4ad/> clean up unused imports and variables
   - Remove unused imports in clustering and gating modules
   - Fix unreachable code warning in DBSCAN
   - Remove unused mut keywords
   - Clean up warnings for better code quality

### Documentation

 - <csr-id-292bd202b232c6f780a9cc7170cc1d53b443e05e/> add CLI reference and validation reports
   - CLI_ARGUMENTS_REFERENCE: complete argument reference for tru-ols unmix
   - COMPARISON_WITH_JULIA: Rust vs Julia comparison framework
   - PEAK_DETECTION_VALIDATION: peak detection validation report
   - VALIDATION_REPORT: algorithm validation and fixes
   - TRU-OLS vs AutoSpectral: academic comparison
   - UNMIXING_RESULTS_PLATE001: Plate_001 analysis results

### New Features

 - <csr-id-9c3354e3667460949ce836783ce02604c972efde/> unmixing, preprocessing, and FCS integration
   - TRU-OLS unmixing with cutoff and iterative removal
   - Preprocessing and autofluorescence handling
   - FCS integration for spectral unmixing output
   - Plotting support for unmixed results

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

 - 5 commits contributed to the release over the course of 21 calendar days.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update dependencies and align workspace configurations ([`46bee42`](https://github.com/jrmoynihan/flow/commit/46bee42d4f28d185b38446c0d950c2579c422f43))
    - Replace ndarray with faer for linear algebra ([`60d0095`](https://github.com/jrmoynihan/flow/commit/60d00956fa56c883b3c04e4c58bad677b27c6b24))
    - Add CLI reference and validation reports ([`292bd20`](https://github.com/jrmoynihan/flow/commit/292bd202b232c6f780a9cc7170cc1d53b443e05e))
    - Unmixing, preprocessing, and FCS integration ([`9c3354e`](https://github.com/jrmoynihan/flow/commit/9c3354e3667460949ce836783ce02604c972efde))
    - Clean up unused imports and variables ([`c987a22`](https://github.com/jrmoynihan/flow/commit/c987a225570c2afae480800327d0072ab4b4e4ad))
</details>

