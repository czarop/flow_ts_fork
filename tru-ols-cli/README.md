# TRU-OLS CLI

Command-line tool for TRU-OLS (Truncated ReUnmixing OLS) flow cytometry unmixing.

## Installation

```bash
cargo install --path tru-ols-cli
```

## Usage

### Basic Unmixing

```bash
tru-ols unmix \
  --stained sample.fcs \
  --unstained unstained.fcs \
  --mixing-matrix matrix.csv \
  --detectors "FL1-A,FL2-A,FL3-A,FL4-A" \
  --endmembers "AF488,PE,APC,AF647,Autofluorescence" \
  --autofluorescence Autofluorescence \
  --output unmixed.fcs
```

### With Plotting

```bash
tru-ols unmix \
  --stained sample.fcs \
  --unstained unstained.fcs \
  --mixing-matrix matrix.csv \
  --detectors "FL1-A,FL2-A,FL3-A,FL4-A" \
  --endmembers "AF488,PE,APC,AF647,Autofluorescence" \
  --plot \
  --plot-output-dir plots/
```

### Compare with OLS

```bash
tru-ols unmix \
  --stained sample.fcs \
  --unstained unstained.fcs \
  --mixing-matrix matrix.csv \
  --detectors "FL1-A,FL2-A,FL3-A,FL4-A" \
  --endmembers "AF488,PE,APC,AF647,Autofluorescence" \
  --compare-ols \
  --plot-both
```

## Mixing Matrix Format

The mixing matrix should be a CSV file with:
- Rows: Detectors (channels)
- Columns: Endmembers (dyes)
- Values: Mixing coefficients (typically between 0 and 1)

Example:
```csv
0.9,0.1,0.05,0.0
0.1,0.9,0.1,0.0
0.05,0.1,0.85,0.0
0.0,0.0,0.0,1.0
```

## Options

- `--stained`: Path to stained sample FCS file (required)
- `--unstained`: Path to unstained control FCS file (required)
- `--mixing-matrix`: Path to mixing matrix CSV file (required)
- `--detectors`: Comma-separated detector names (required)
- `--endmembers`: Comma-separated endmember names (required)
- `--autofluorescence`: Autofluorescence endmember name (default: "Autofluorescence")
- `--cutoff-percentile`: Percentile for cutoff (default: 0.995)
- `--strategy`: "zero" or "ucm" (default: "zero")
- `--output`: Output FCS file path (optional)
- `--plot`: Generate comparison plots
- `--plot-format`: Plot format: png, svg, or pdf (default: png)
- `--plot-output-dir`: Directory for plot outputs
- `--compare-ols`: Also run standard OLS and compare
- `--plot-both`: Generate plots for both OLS and TRU-OLS
