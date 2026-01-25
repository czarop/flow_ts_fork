# Automated Gating Module

This module provides automated gate generation for common flow cytometry preprocessing steps.

## Features

- **Automated Scatter Gating**: Identify viable cell populations in FSC vs SSC plots
  - Density contour method
  - Ellipse fitting
  - Clustering-based (pending linfa API fix)
  - Multi-population support

- **Enhanced Doublet Detection**: Detect doublet events using multiple methods
  - Ratio-based MAD (peacoqc-rs compatible)
  - Density-based detection using KDE
  - Clustering-based (pending linfa API fix)
  - Hybrid approach combining multiple methods

- **Interactive Pipelines**: Support for user review and breakpoints
  - Fully automated mode
  - Semi-automated mode with user review at each step

## Usage

### Scatter Gating

```rust
use flow_gates::automated::{ScatterGateConfig, ScatterGateMethod, create_scatter_gate};

let config = ScatterGateConfig {
    fsc_channel: "FSC-A".to_string(),
    ssc_channel: "SSC-A".to_string(),
    method: ScatterGateMethod::EllipseFit,
    min_events: 1000,
    density_threshold: None,
    cluster_eps: None,
    cluster_min_samples: None,
};

let result = create_scatter_gate(&fcs, &config)?;
```

### Doublet Detection

```rust
use flow_gates::automated::{DoubletGateConfig, DoubletMethod, detect_doublets};

let config = DoubletGateConfig {
    channels: vec![
        ("FSC-A".to_string(), "FSC-H".to_string()),
        ("FSC-W".to_string(), "FSC-H".to_string()),
    ],
    method: DoubletMethod::RatioMAD { nmad: 4.0 },
    nmad: Some(4.0),
    density_threshold: None,
    cluster_eps: None,
    cluster_min_samples: None,
};

let result = detect_doublets(&fcs, &config)?;
```

### Preprocessing Pipeline

```rust
use flow_gates::automated::{PreprocessingConfig, create_preprocessing_gates};

let config = PreprocessingConfig {
    scatter_config: scatter_config,
    doublet_config: doublet_config,
};

let gates = create_preprocessing_gates(&fcs, config)?;
```

## Algorithm Details

### Scatter Gating

- **Ellipse Fit**: Calculates mean and standard deviation, creates ellipse covering ~95% of data
- **Density Contour**: Uses KDE to estimate density, finds peaks, creates gate around main population
- **Clustering**: Uses K-means/DBSCAN/GMM to identify cell populations (pending linfa API fix)

### Doublet Detection

- **Ratio MAD**: Calculates area/height ratio, uses median + nMAD threshold (peacoqc-rs compatible)
- **Density-Based**: Uses KDE on ratio distribution, identifies outliers from main peak
- **Clustering**: Uses DBSCAN to separate singlet/doublet clusters (pending linfa API fix)

## Performance Notes

- Performance is prioritized over accuracy for gating steps
- KDE uses FFT acceleration for O(n log n) performance
- Clustering methods use linfa-clustering for optimized implementations

## Known Limitations

- Clustering-based methods pending linfa API fix (DBSCAN trait bounds)
- Multi-population scatter gating is a placeholder (needs full implementation)
- Gate generation for doublets is not yet implemented (masks are available)
