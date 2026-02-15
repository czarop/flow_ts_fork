# TRU-OLS CLI Arguments Reference

Complete reference for all CLI arguments, options, and usage patterns.

## Quick Reference

### Simplest Usage (Recommended)

```bash
tru-ols unmix --stained <sample.fcs> --controls <controls_dir/> --output <unmixed.fcs>
```

Auto-detects everything: unstained control, single-stains, detectors, endmembers, and builds mixing matrix.

### What Gets Auto-Detected?

When using `--controls`:
- **Unstained control**: File with "unstained" in filename (case-insensitive)
- **Single-stain controls**: All other `.fcs` files in directory
- **Detectors**: Fluorescent parameters (excludes FSC, SSC, Time)
- **Endmembers**: Extracted from control filenames

## Required Arguments

### Stained Sample (Always Required)

**`--stained <PATH>`**
- Path to stained sample FCS file OR directory of FCS files
- If directory: processes all `.fcs` files (batch mode)

### Mixing Matrix Source (Choose ONE)

You must provide exactly one of these:

#### Option 1: `--controls <PATH>` (Recommended)

Directory containing ALL controls (single-stains + unstained).

**Auto-detects:**
- Unstained control (filename contains "unstained")
- Single-stain controls (all other FCS files)
- Detector channels (fluorescent parameters only)
- Endmember names (from filenames)

**Example:**
```bash
tru-ols unmix \
  --stained sample.fcs \
  --controls ./controls/ \
  --output unmixed.fcs
```

#### Option 2: `--single-stain-controls <PATH>` + `--unstained <PATH>`

Separate paths for single-stains and unstained control.

**Auto-detects:**
- Detector channels
- Endmember names

**Manually provide:**
- Unstained control path

**Example:**
```bash
tru-ols unmix \
  --stained sample.fcs \
  --unstained unstained.fcs \
  --single-stain-controls ./single_stains/ \
  --output unmixed.fcs
```

#### Option 3: `--use-spill` + `--unstained <PATH>`

Extract mixing matrix from FCS SPILL/SPILLOVER keyword.

**Auto-detects:**
- Mixing matrix (from SPILL keyword)
- Detector names (from SPILL keyword)

**Manually provide:**
- `--unstained <PATH>`
- `--endmembers <NAMES>` (comma-separated)

**Example:**
```bash
tru-ols unmix \
  --stained sample.fcs \
  --unstained unstained.fcs \
  --use-spill \
  --endmembers "FITC,PE,PerCP,APC,Autofluorescence" \
  --output unmixed.fcs
```

#### Option 4: `--mixing-matrix <PATH>` + `--unstained <PATH>`

Pre-computed mixing matrix from CSV file.

**Manually provide:**
- `--mixing-matrix <PATH>` (CSV file)
- `--unstained <PATH>`
- `--detectors <NAMES>` (comma-separated)
- `--endmembers <NAMES>` (comma-separated)

**Example:**
```bash
tru-ols unmix \
  --stained sample.fcs \
  --unstained unstained.fcs \
  --mixing-matrix matrix.csv \
  --detectors "FL1-A,FL2-A,FL3-A,FL4-A" \
  --endmembers "FITC,PE,PerCP,APC,Autofluorescence" \
  --output unmixed.fcs
```

## Optional Arguments

### Basic Options

**`--output <PATH>`**
- Output FCS file path (single file mode)
- Output directory path (batch mode)
- Default: current directory
- Batch mode adds `_unmixed` suffix to filenames

**`--autofluorescence <NAME>`**
- Name of autofluorescence endmember
- Default: `"Autofluorescence"`
- Must match one of the endmember names

**`--cutoff-percentile <VALUE>`**
- Percentile for truncation cutoff
- Default: `0.995` (99.5th percentile)
- Range: 0.0 to 1.0

**`--strategy <STRATEGY>`**
- Unmixing strategy: `"zero"` or `"ucm"`
- Default: `"ucm"`
- `"zero"`: Zero strategy (recommended)
- `"ucm"`: Unstained Control Mapping

### Plotting Options

**`--plot`**
- Generate plots for unmixed results
- Creates density plots and distributions

**`--plot-format <FORMAT>`**
- Plot output format: `png`, `svg`, or `pdf`
- Default: `"png"`

**`--plot-output-dir <PATH>`**
- Directory for plot outputs
- Default: current directory

**`--compare-ols`**
- Also run standard OLS unmixing for comparison
- Useful for evaluating TRU-OLS improvements

**`--plot-both`**
- Generate plots for both OLS and TRU-OLS
- Requires `--compare-ols` flag
- Creates side-by-side comparison plots

### Single-Stain Control Options

**`--peak-detection`**
- Enable peak-based median selection
- Uses KDE to find highest intensity peak
- More robust for multi-modal distributions
- Default: enabled

**`--peak-threshold <VALUE>`**
- Peak detection threshold (fraction of max density)
- Default: `0.3`
- Lower values detect more peaks
- Higher values detect only strong peaks

**`--peak-bias <VALUE>`**
- Bias fraction for positive peaks
- Default: `0.5` (upper 50% of peak)
- Range: 0.0 to 1.0
- Higher values bias toward brighter events

**`--peak-bias-negative <VALUE>`**
- Bias fraction for negative peaks
- Default: `0.5` (lower 50% of negative peak)
- Used for autofluorescence estimation

### Autofluorescence Options

**`--use-negative-events`**
- Extract autofluorescence from negative events in single-stains
- More specific autofluorescence per fluorophore

**`--min-negative-events <COUNT>`**
- Minimum negative events required
- Default: `100`
- Falls back to universal AF if insufficient events

**`--autofluorescence-mode <MODE>`**
- Autofluorescence calculation mode:
  - `"universal"` (default): Use unstained control only
  - `"negative-events"`: Use negative events from single-stains
  - `"hybrid"`: Weighted combination of both
- Only relevant with `--use-negative-events`

**`--af-weight <VALUE>`**
- Weight for hybrid autofluorescence mode
- Default: `0.7` (70% unstained, 30% negative events)
- Range: 0.0 to 1.0
- Only used with `--autofluorescence-mode hybrid`

### Advanced Options

**`--auto-gate`**
- Apply automated scatter and doublet gating
- Uses PeacoQC-style margin and doublet removal
- Applied to all controls before matrix creation
- Default: enabled

**`--export-mixing-matrix <PATH>`**
- Export computed mixing matrix to CSV
- Useful for inspection or reuse
- Outputs detector × endmember matrix

## Batch Processing

### Process Multiple Files

Provide a directory as `--stained`:

```bash
tru-ols unmix \
  --stained ./samples/ \
  --controls ./controls/ \
  --output ./unmixed/
```

**Behavior:**
- Processes all `.fcs` files in `./samples/`
- Outputs to `./unmixed/` with `_unmixed` suffix
- Example: `sample1.fcs` → `sample1_unmixed.fcs`
- Mixing matrix computed once, reused for all files
- Progress shown for each file

### Batch with Plots

```bash
tru-ols unmix \
  --stained ./samples/ \
  --controls ./controls/ \
  --output ./unmixed/ \
  --compare-ols \
  --plot-both \
  --plot-output-dir ./unmixed/plots/
```

## Output Files

### Unmixed FCS Files

Output FCS files contain:
- All original parameters (FSC, SSC, etc.)
- New columns for each endmember using **actual names**
  - Example: `"CD4"`, `"CD8"`, `"CD19"`, etc.
  - NOT generic `"Endmember1"`, `"Endmember2"`
- f32 data type for endmember abundances

### Plot Files

When using `--plot-both --compare-ols`:
- `comparison_ols_<EM1>_vs_<EM2>.png`: OLS scatter plots
- `comparison_tru_ols_<EM1>_vs_<EM2>.png`: TRU-OLS scatter plots
- Generates plots for first 4 endmember pairs

When using `--plot` only:
- `tru_ols_<EM>_distribution.png`: Distribution plots
- `tru_ols_<EM1>_vs_<EM2>.png`: Pairwise scatter plots

## Mixing Matrix CSV Format

If providing `--mixing-matrix`, format should be:
- Rows: Detector channels
- Columns: Endmembers (including autofluorescence)
- Values: Spectral signatures (typically 0 to 1)
- No headers

**Example:**
```csv
0.95,0.05,0.02,0.01,0.10
0.08,0.89,0.08,0.02,0.12
0.02,0.12,0.91,0.05,0.08
0.01,0.03,0.06,0.88,0.09
0.05,0.08,0.04,0.03,1.00
```

For a 5-detector, 5-endmember system (4 fluorophores + autofluorescence).

## Usage Patterns

### Standard Spectral Unmixing

```bash
tru-ols unmix \
  --stained sample.fcs \
  --controls ./controls/ \
  --output unmixed.fcs
```

### With Quality Assessment

```bash
tru-ols unmix \
  --stained sample.fcs \
  --controls ./controls/ \
  --output unmixed.fcs \
  --compare-ols \
  --plot-both \
  --plot-output-dir ./plots/
```

### Batch with Advanced Options

```bash
tru-ols unmix \
  --stained ./samples/ \
  --controls ./controls/ \
  --output ./unmixed/ \
  --peak-detection \
  --peak-threshold 0.25 \
  --auto-gate \
  --compare-ols \
  --plot-both \
  --export-mixing-matrix ./mixing_matrix.csv
```

### Using Embedded SPILL

```bash
tru-ols unmix \
  --stained sample.fcs \
  --unstained unstained.fcs \
  --use-spill \
  --endmembers "AF488,PE,PerCP-Cy5.5,APC,AF700,Autofluorescence" \
  --output unmixed.fcs
```

## Troubleshooting

### "No unstained control found"

When using `--controls`, ensure one file contains "unstained" in filename:
- ✅ `Unstained.fcs`
- ✅ `reference_unstained_control.fcs`
- ❌ `blank.fcs` (doesn't contain "unstained")

### "Detector names must be provided"

Using `--mixing-matrix` requires `--detectors`:
```bash
--detectors "FL1-A,FL2-A,FL3-A,FL4-A"
```

### "No fluorescent detector parameters found"

FCS file parameters must include fluorescent channels.
Excluded automatically: FSC, SSC, Time parameters.

### High Spillover Warnings

Normal for spectral cytometry with many overlapping fluorophores.
Consider using `--peak-detection` for better signal selection.

### Matrix Dimension Errors

Ensure:
- Number of detectors matches matrix rows
- Number of endmembers matches matrix columns
- Autofluorescence included in endmembers list

## Performance Notes

### Batch Processing

Mixing matrix computed once and reused → very fast for multiple files.

**Example:** 6 samples, 64 detectors, 13 endmembers:
- Matrix computation: ~30 seconds
- Per-file unmixing: ~1-2 seconds
- Total: ~40 seconds for all 6 files

### GPU Acceleration

Future versions may support GPU acceleration for large datasets.
Current implementation is CPU-only using optimized BLAS/LAPACK.

## See Also

- `tru-ols unmix --help`: Built-in help
- `tru-ols args`: Show this reference from CLI
- README.md: Quick start guide
