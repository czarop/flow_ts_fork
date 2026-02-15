# Research Notes: Automated Scatter and Doublet Gating

## Overview

This document summarizes the research and implementation decisions for automated scatter and doublet gating in flow cytometry data.

## Scatter Gating (FSC vs SSC)

### Requirements

- **No transformation**: Use raw FSC/SSC values (transformation only for fluorescent parameters)
- **Multi-population support**: Detect multiple cell populations for future autofluorescence signature detection
- **Performance priority**: Gating steps prioritize performance over accuracy

### Implemented Methods

1. **Ellipse Fit**: 
   - Calculates mean and standard deviation
   - Creates ellipse covering ~95% of data (2 standard deviations)
   - Fast and simple

2. **Density Contour**:
   - Uses KDE to estimate density
   - Finds peaks in FSC distribution
   - Creates gate around main population
   - More sophisticated but still fast (FFT-accelerated KDE)

3. **Clustering** (placeholder):
   - K-means, DBSCAN, GMM support planned
   - Pending DBSCAN API fix
   - Will enable true multi-population detection

### Future Enhancements

- Full 2D KDE for density contours
- Multi-population detection using clustering
- Adaptive threshold selection
- Integration with autofluorescence signature detection

## Doublet Detection

### Requirements

- **No transformation**: Use raw FSC/SSC values
- **Multiple channel pairs**: Support FSC-A/FSC-H, FSC-W/FSC-H, SSC-A/SSC-H
- **Performance priority**: Fast detection for preprocessing
- **Backward compatibility**: Include peacoqc-rs method for comparison

### Implemented Methods

1. **Ratio MAD** (peacoqc-rs compatible):
   - Calculates area/height ratio
   - Uses median + nMAD threshold
   - Fast and proven method
   - Default: 4.0 nMAD

2. **Density-Based**:
   - Uses KDE on ratio distribution
   - Identifies outliers from main peak
   - More adaptive than fixed threshold

3. **Hybrid**:
   - Combines multiple methods
   - Intersection approach (both must agree)
   - More conservative but potentially more accurate

4. **Clustering** (pending):
   - DBSCAN to separate singlet/doublet clusters
   - Pending linfa API fix

### Comparison Framework

- Head-to-head comparison of methods
- Agreement matrix calculation
- Performance metrics
- Method recommendation based on agreement and performance

## Performance vs Accuracy Tradeoffs

### Gating Steps (Performance Priority)

- **Scatter gating**: <100ms target for typical datasets (<100k events)
- **Doublet detection**: <50ms target
- KDE uses FFT acceleration for O(n log n) performance
- Clustering methods use optimized linfa implementations

### Unmixing Steps (Accuracy Priority)

- Single-stain control processing: Accuracy > performance
- Peak detection: Accuracy > performance
- Mixing matrix generation: Accuracy > performance

## Algorithm Parameters

### Scatter Gating

- **Min events**: Default 1000 (configurable)
- **Density threshold**: Default 0.1 (for density contour)
- **Cluster parameters**: Configurable (eps, min_samples)

### Doublet Detection

- **nMAD**: Default 4.0 (peacoqc-rs compatible)
- **Density threshold**: Default 0.1
- **Channel pairs**: Configurable (default: FSC-A/FSC-H)

## Known Limitations

1. **DBSCAN**: Temporarily disabled due to linfa-clustering API issue
2. **Multi-population**: Placeholder implementation (needs full clustering support)
3. **2D KDE**: Currently uses 1D KDE on FSC (2D implementation needed for better density contours)
4. **Gate generation for doublets**: Masks available but polygon gates not yet generated

## Future Work

1. Fix DBSCAN API integration
2. Implement full 2D KDE for density contours
3. Complete multi-population scatter gating
4. Generate polygon gates for doublet exclusion
5. Benchmark against manual gating
6. Performance optimization for large datasets (>1M events)
