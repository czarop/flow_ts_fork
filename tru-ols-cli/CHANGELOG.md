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

 - <csr-id-5c6c02a44bcc7abe9a79297d7b33ddbcd15e7fcb/> peak detection, synthetic data, and spectral unmixing
   - Peak detection enabled by default for single-stain control analysis
   - Synthetic FCS data generation with known ground truth
   - Spectral unmixing with --controls auto-detection
   - Examples: generate_synthetic_test_data, compare_with_julia, check_unmixed
   - Peak detection unit tests
 - <csr-id-a086f6c1501996fe7eee5d3b1798f7fab924f853/> integrate automated gating (Task 3.1)
   - Add --auto-gate flag to enable automated preprocessing gates
   - Apply scatter and doublet gates to controls before processing
   - Gate results logged (full filtering requires FCS creation API)
   - Add flow-gates dependency
   - Create comprehensive testing instructions document
   - All compilation errors resolved
 - <csr-id-85475ba5e55685cbc14b6dcea413b0a7110faf23/> integrate peak detection for single-stain controls
   - Add flow-utils dependency for KDE peak detection
   - Add CLI options: --peak-detection, --peak-threshold, --peak-bias
   - Implement calculate_peak_based_median function
   - Replace simple median with peak-based median when enabled
   - Add SingleStainConfig struct for configuration
   - Fallback to simple median if peak detection fails

### Refactor

 - <csr-id-70008ac39d1d08497c2f59e7fde438d0755433d3/> update for faer-based fcs and tru-ols APIs
   - Add faer-ext for ndarray↔faer conversion at boundaries
   - Update commands.rs: MatRef for apply_spectral_unmixing, spill matrix
   - Update compare_with_julia, export_mixing_matrix, create_mixing_matrix_csv
   - Keep ndarray for downstream plotting/export where needed

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 21 calendar days.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update dependencies and align workspace configurations ([`46bee42`](https://github.com/jrmoynihan/flow/commit/46bee42d4f28d185b38446c0d950c2579c422f43))
    - Update for faer-based fcs and tru-ols APIs ([`70008ac`](https://github.com/jrmoynihan/flow/commit/70008ac39d1d08497c2f59e7fde438d0755433d3))
    - Add CLI reference and validation reports ([`292bd20`](https://github.com/jrmoynihan/flow/commit/292bd202b232c6f780a9cc7170cc1d53b443e05e))
    - Peak detection, synthetic data, and spectral unmixing ([`5c6c02a`](https://github.com/jrmoynihan/flow/commit/5c6c02a44bcc7abe9a79297d7b33ddbcd15e7fcb))
    - Integrate automated gating (Task 3.1) ([`a086f6c`](https://github.com/jrmoynihan/flow/commit/a086f6c1501996fe7eee5d3b1798f7fab924f853))
    - Integrate peak detection for single-stain controls ([`85475ba`](https://github.com/jrmoynihan/flow/commit/85475ba5e55685cbc14b6dcea413b0a7110faf23))
    - Clean up unused imports and variables ([`c987a22`](https://github.com/jrmoynihan/flow/commit/c987a225570c2afae480800327d0072ab4b4e4ad))
</details>

