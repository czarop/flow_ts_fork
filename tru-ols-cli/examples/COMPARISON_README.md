# Comparing Rust TRU-OLS with Julia Implementation

This guide explains how to compare the Rust TRU-OLS implementation with the original Julia implementation.

## Prerequisites

1. **Julia** installed with required packages:
   ```bash
   julia -e 'using Pkg; Pkg.add(["CSV", "DataFrames"])'
   ```
   
   Note: `LinearAlgebra` and `StatsBase` are part of Julia's standard library and don't need to be installed separately.

2. **Rust** and Cargo installed

3. **TRU-OLS.jl** file available (should be in `TRU-OLS/TRU-OLS.jl` relative to workspace root)

## Step 1: Prepare Mixing Matrix

You have three options:

**Option A: Use Controls Directory (Recommended - Automatic Generation)**

The comparison example can automatically generate the mixing matrix from single-stain controls. Just provide the controls directory path instead of a CSV file.

**Option B: Export from CLI**

Use the `--export-mixing-matrix` flag to export the matrix:
```bash
tru-ols unmix \
  --stained synthetic_test_data/samples/FullyStained_Sample_1.fcs \
  --unstained synthetic_test_data/controls/Unstained_Control.fcs \
  --single-stain-controls synthetic_test_data/controls \
  --export-mixing-matrix mixing_matrix.csv \
  --output /tmp/temp.fcs
```

**Option C: Create manually** (see format below)

The mixing matrix format:
```csv
RowName,Fluor_B510,Fluor_B660,...,Autofluorescence
UV379-A,0.0,0.0,...,0.2
UV446-A,0.0,0.0,...,0.4
...
```

Where:
- First column: Detector names
- Remaining columns: Endmember names (Autofluorescence must be last)
- Values: Normalized spectral signatures (0.0 to 1.0)

## Step 2: Export Data from Rust

Run the comparison example to export data in CSV format:

```bash
cd flow-crates/tru-ols-cli

# Using synthetic test data with mixing matrix CSV
cargo run --example compare_with_julia -- \
    synthetic_test_data/samples/FullyStained_Sample_1.fcs \
    synthetic_test_data/controls/Unstained_Control.fcs \
    mixing_matrix.csv \
    comparison_output/
```

This will create:
- `comparison_output/mixing_matrix.csv` - Mixing matrix (detectors × endmembers)
- `comparison_output/unstained_data.csv` - Unstained control data
- `comparison_output/stained_data.csv` - Stained sample data
- `comparison_output/rust_cutoffs.csv` - Rust calculated cutoffs
- `comparison_output/rust_nonspecific.csv` - Rust nonspecific observation
- `comparison_output/rust_unmixed.csv` - Rust unmixed abundances
- `comparison_output/endmember_names.csv` - Endmember names
- `comparison_output/compare_with_julia.jl` - Julia comparison script

## Step 2: Run Julia Comparison

```bash
cd comparison_output
julia compare_with_julia.jl
```

This will:
1. Load the exported CSV files
2. Run Julia TRU-OLS on the same data
3. Export Julia results to:
   - `julia_cutoffs.csv`
   - `julia_nonspecific.csv`
   - `julia_unmixed.csv`

## Step 3: Compare Results

You can compare the CSV files manually or use a script. Key comparisons:

1. **Cutoffs**: Should be very similar (within numerical precision)
2. **Nonspecific Observation**: Should match closely
3. **Unmixed Abundances**: Should be similar, with small differences due to:
   - Numerical precision differences between Rust and Julia
   - Different BLAS/LAPACK implementations
   - Floating-point rounding differences

## Expected Differences

Small differences (< 1e-6) are expected due to:
- Different numerical libraries (OpenBLAS vs MKL)
- Different floating-point rounding behavior
- Different least squares implementations (normal equations vs QR)

Large differences (> 1e-3) may indicate:
- Algorithm implementation differences
- Matrix ordering issues
- Index mismatches

## Troubleshooting

### Julia can't find TRU-OLS.jl
Make sure `TRU-OLS.jl` is in the correct location. The script assumes it's in the workspace root `TRU-OLS/` directory.

### Dimension mismatches
Check that:
- Mixing matrix dimensions match (detectors × endmembers)
- Data matrices have correct dimensions (events × detectors)
- Endmember names match between Rust and Julia

### Numerical differences
If differences are larger than expected:
1. Check that both use the same percentile (0.995)
2. Verify matrix ordering (row-major vs column-major)
3. Check for any preprocessing differences
