# Quick Start: Comparing Rust vs Julia TRU-OLS

## Prerequisites

Install required Julia packages:
```bash
julia -e 'using Pkg; Pkg.add(["CSV", "DataFrames"])'
```

Note: `LinearAlgebra` and `StatsBase` are part of Julia's standard library and don't need to be installed separately.

## Step 1: Run Comparison (Automatic Mixing Matrix Generation)

The comparison example can now automatically generate the mixing matrix from single-stain controls! You can provide either:
- A **CSV file** with an existing mixing matrix
- A **directory** containing single-stain controls (the matrix will be generated automatically)

## Step 2: Run Comparison

**Option A: With Controls Directory (Recommended - Automatic Matrix Generation)**

```bash
cd flow-crates/tru-ols-cli

cargo run --example compare_with_julia -- \
  synthetic_test_data/samples/FullyStained_Sample_1.fcs \
  synthetic_test_data/controls/Unstained_Control.fcs \
  synthetic_test_data/controls \
  comparison_output/
```

This will automatically generate the mixing matrix from the single-stain controls in the directory.

**Option B: With Existing Mixing Matrix CSV**

```bash
cargo run --example compare_with_julia -- \
  synthetic_test_data/samples/FullyStained_Sample_1.fcs \
  synthetic_test_data/controls/Unstained_Control.fcs \
  mixing_matrix.csv \
  comparison_output/
```

**Option C: Export Mixing Matrix First (Using CLI)**

You can also export the mixing matrix using the CLI's `--export-mixing-matrix` flag:

```bash
tru-ols unmix \
  --stained synthetic_test_data/samples/FullyStained_Sample_1.fcs \
  --unstained synthetic_test_data/controls/Unstained_Control.fcs \
  --single-stain-controls synthetic_test_data/controls \
  --export-mixing-matrix mixing_matrix.csv \
  --output /tmp/temp.fcs
```

This will:
1. Load the FCS files and mixing matrix
2. Run Rust TRU-OLS
3. Export all data and results to CSV
4. Generate a Julia script for comparison

## Step 3: Run Julia Comparison

```bash
cd comparison_output
julia compare_with_julia.jl
```

**Note**: If you get an error about missing packages, install them first:
```bash
julia -e 'using Pkg; Pkg.add(["CSV", "DataFrames"])'
```

This runs Julia TRU-OLS on the same data and exports results.

## Step 4: Compare Results

Compare the CSV files:
- `rust_cutoffs.csv` vs `julia_cutoffs.csv`
- `rust_nonspecific.csv` vs `julia_nonspecific.csv`  
- `rust_unmixed.csv` vs `julia_unmixed.csv`

Expected differences: < 1e-6 (numerical precision)

## Alternative: Use Existing Unmixed File

If you've already run unmixing, you can extract the mixing matrix from the internal state, but for now the easiest path is to create the CSV manually or wait for export functionality.
