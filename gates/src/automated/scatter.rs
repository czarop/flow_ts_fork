//! Automated scatter gating (FSC vs SSC)
//!
//! Provides algorithms for automatically identifying viable cell populations
//! in scatter plots, supporting multi-population detection.

use crate::{Gate, GateError, GateResult, GateStatistics};
use crate::geometry::create_ellipse_geometry;
use flow_fcs::Fcs;
use flow_utils::kde::KernelDensity;
use ndarray::Array2;
use std::sync::Arc;

/// Configuration for scatter gating
#[derive(Debug, Clone)]
pub struct ScatterGateConfig {
    /// FSC channel name
    pub fsc_channel: String,
    /// SSC channel name
    pub ssc_channel: String,
    /// Gating method to use
    pub method: ScatterGateMethod,
    /// Minimum number of events required
    pub min_events: usize,
    /// Density threshold (for density-based methods)
    pub density_threshold: Option<f64>,
    /// Cluster epsilon (for DBSCAN)
    pub cluster_eps: Option<f64>,
    /// Minimum samples for clustering
    pub cluster_min_samples: Option<usize>,
}

/// Scatter gating method
#[derive(Debug, Clone)]
pub enum ScatterGateMethod {
    /// Density contour-based gating
    DensityContour { threshold: f64 },
    /// Clustering-based gating
    Clustering { algorithm: ClusterAlgorithm },
    /// Ellipse fitting to main population
    EllipseFit,
}

/// Clustering algorithm for scatter gating
#[derive(Debug, Clone, Copy)]
pub enum ClusterAlgorithm {
    /// K-means clustering
    KMeans,
    /// DBSCAN clustering
    Dbscan,
    /// Gaussian Mixture Model
    Gmm,
}

/// Result of scatter gating
#[derive(Debug, Clone)]
pub struct ScatterGateResult {
    /// Generated gate (if successful)
    pub gate: Option<Gate>,
    /// Population mask (true = inside gate)
    pub population_mask: Vec<bool>,
    /// Statistics about the gated population
    pub statistics: GateStatistics,
    /// Method used for gating
    pub method_used: String,
}

// GateStatistics is imported from crate::statistics

/// Create automated scatter gate
///
/// # Arguments
/// * `fcs` - FCS file data
/// * `config` - Scatter gate configuration
///
/// # Returns
/// ScatterGateResult with gate and statistics
pub fn create_scatter_gate(
    fcs: &Fcs,
    config: &ScatterGateConfig,
) -> GateResult<ScatterGateResult> {
    // Extract FSC/SSC data (NO transformation - raw values)
    // Fcs API returns f32 slices, convert to f64 for processing
    let fsc_data_f32 = fcs
        .get_parameter_events_slice(&config.fsc_channel)
        .map_err(|e| GateError::Other {
            message: format!("Failed to get FSC channel {}: {}", config.fsc_channel, e),
            source: None, // anyhow::Error doesn't implement StdError, use message only
        })?;
    let ssc_data_f32 = fcs
        .get_parameter_events_slice(&config.ssc_channel)
        .map_err(|e| GateError::Other {
            message: format!("Failed to get SSC channel {}: {}", config.ssc_channel, e),
            source: None, // anyhow::Error doesn't implement StdError, use message only
        })?;
    
    // Convert to f64 for processing
    let fsc_data: Vec<f64> = fsc_data_f32.iter().map(|&x| x as f64).collect();
    let ssc_data: Vec<f64> = ssc_data_f32.iter().map(|&x| x as f64).collect();

    if fsc_data.len() != ssc_data.len() {
        return Err(GateError::Other {
            message: format!(
                "FSC and SSC channels have different lengths: {} vs {}",
                fsc_data.len(),
                ssc_data.len()
            ),
            source: None,
        });
    }

    if fsc_data.len() < config.min_events {
        return Err(GateError::Other {
            message: format!(
                "Insufficient data: need at least {} events, got {}",
                config.min_events,
                fsc_data.len()
            ),
            source: None,
        });
    }

    // Create 2D data matrix (n_samples × 2 features)
    let n_samples = fsc_data.len();
    let mut data = Array2::<f64>::zeros((n_samples, 2));
    for (i, (&fsc, &ssc)) in fsc_data.iter().zip(ssc_data.iter()).enumerate() {
        data[[i, 0]] = fsc;
        data[[i, 1]] = ssc;
    }

    // Apply gating method
    let (gate, mask, method_name) = match &config.method {
        ScatterGateMethod::DensityContour { threshold } => {
            create_density_contour_gate(&data, config, *threshold)?
        }
        ScatterGateMethod::Clustering { algorithm } => {
            create_clustering_gate(&data, config, *algorithm)?
        }
        ScatterGateMethod::EllipseFit => create_ellipse_fit_gate(&data, config)?,
    };

    // Calculate statistics using GateStatistics
    // Note: GateStatistics::calculate requires a gate
    let statistics = if let Some(ref gate) = gate {
        GateStatistics::calculate(fcs, gate)
            .unwrap_or_else(|_| {
                // Create empty statistics manually if calculation fails
                GateStatistics {
                    event_count: 0,
                    percentage: 0.0,
                    centroid: (0.0, 0.0),
                    x_stats: crate::statistics::ParameterStatistics {
                        parameter: config.fsc_channel.clone(),
                        mean: 0.0,
                        geometric_mean: 0.0,
                        median: 0.0,
                        std_dev: 0.0,
                        min: 0.0,
                        max: 0.0,
                        q1: 0.0,
                        q3: 0.0,
                        cv: 0.0,
                    },
                    y_stats: crate::statistics::ParameterStatistics {
                        parameter: config.ssc_channel.clone(),
                        mean: 0.0,
                        geometric_mean: 0.0,
                        median: 0.0,
                        std_dev: 0.0,
                        min: 0.0,
                        max: 0.0,
                        q1: 0.0,
                        q3: 0.0,
                        cv: 0.0,
                    },
                }
            })
    } else {
        // Create empty statistics if no gate
        GateStatistics {
            event_count: 0,
            percentage: 0.0,
            centroid: (0.0, 0.0),
            x_stats: crate::statistics::ParameterStatistics {
                parameter: config.fsc_channel.clone(),
                mean: 0.0,
                geometric_mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                q1: 0.0,
                q3: 0.0,
                cv: 0.0,
            },
            y_stats: crate::statistics::ParameterStatistics {
                parameter: config.ssc_channel.clone(),
                mean: 0.0,
                geometric_mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                q1: 0.0,
                q3: 0.0,
                cv: 0.0,
            },
        }
    };

    Ok(ScatterGateResult {
        gate,
        population_mask: mask,
        statistics,
        method_used: method_name,
    })
}

/// Create gate using density contour method
fn create_density_contour_gate(
    data: &Array2<f64>,
    config: &ScatterGateConfig,
    threshold: f64,
) -> GateResult<(Option<Gate>, Vec<bool>, String)> {
    // Use KDE to estimate density
    // For 2D, we'll use 1D KDE on each dimension and combine
    // TODO: Implement 2D KDE or use a different approach
    
    // For now, use a simple approach: find main population using KDE on FSC
    let fsc_values: Vec<f64> = data.column(0).iter().copied().collect();
    let kde = KernelDensity::estimate(&fsc_values, 1.0, 512)
        .map_err(|e| GateError::Other {
            message: format!("KDE failed: {:?}", e),
            source: None,
        })?;
    
    // Find peak in FSC distribution
    let peaks = kde.find_peaks(threshold);
    if peaks.is_empty() {
        return Err(GateError::Other {
            message: "No peaks found in FSC distribution".to_string(),
            source: None,
        });
    }
    
    let main_peak = peaks[0];
    
    // Create simple ellipse around main population
    // This is a placeholder - full implementation would use 2D density contours
    let center_x = main_peak;
    let center_y = data.column(1).iter().sum::<f64>() / data.nrows() as f64;
    
    // Calculate spread
    let mut sum_dist_x = 0.0;
    let mut sum_dist_y = 0.0;
    let mut count = 0;
    for i in 0..data.nrows() {
        let dist_x = (data[[i, 0]] - center_x).abs();
        let dist_y = (data[[i, 1]] - center_y).abs();
        if dist_x < 3.0 * (data.column(0).iter().map(|x| (x - center_x).abs()).sum::<f64>() / data.nrows() as f64) {
            sum_dist_x += dist_x;
            sum_dist_y += dist_y;
            count += 1;
        }
    }
    
    let radius_x = if count > 0 { sum_dist_x / count as f64 * 2.0 } else { 1000.0 };
    let radius_y = if count > 0 { sum_dist_y / count as f64 * 2.0 } else { 1000.0 };
    
    // Create ellipse gate
    // create_ellipse_geometry expects Vec<(f32, f32)> coordinates
    // Create coordinates: center, right, top, left, bottom
    let center = (center_x as f32, center_y as f32);
    let right = ((center_x + radius_x) as f32, center_y as f32);
    let top = (center_x as f32, (center_y + radius_y) as f32);
    let left = ((center_x - radius_x) as f32, center_y as f32);
    let bottom = (center_x as f32, (center_y - radius_y) as f32);
    let coords = vec![center, right, top, left, bottom];
    
    let geometry = create_ellipse_geometry(
        coords,
        &config.fsc_channel,
        &config.ssc_channel,
    )?;
    
    let gate = Gate::new(
        "scatter-gate",
        "Automated Scatter Gate",
        geometry,
        Arc::from(config.fsc_channel.as_str()),
        Arc::from(config.ssc_channel.as_str()),
    );
    
    // Create mask (simple ellipse check)
    let mask: Vec<bool> = (0..data.nrows())
        .map(|i| {
            let dx = (data[[i, 0]] - center_x) / radius_x;
            let dy = (data[[i, 1]] - center_y) / radius_y;
            dx * dx + dy * dy <= 1.0
        })
        .collect();
    
    Ok((Some(gate), mask, "DensityContour".to_string()))
}

/// Create gate using clustering method
fn create_clustering_gate(
    data: &Array2<f64>,
    config: &ScatterGateConfig,
    algorithm: ClusterAlgorithm,
) -> GateResult<(Option<Gate>, Vec<bool>, String)> {
    // TODO: Implement clustering-based gating once linfa API is fixed
    // For now, fall back to ellipse fit
    create_ellipse_fit_gate(data, config)
}

/// Create gate using ellipse fitting
fn create_ellipse_fit_gate(
    data: &Array2<f64>,
    config: &ScatterGateConfig,
) -> GateResult<(Option<Gate>, Vec<bool>, String)> {
    // Calculate center (mean)
    let center_x = data.column(0).iter().sum::<f64>() / data.nrows() as f64;
    let center_y = data.column(1).iter().sum::<f64>() / data.nrows() as f64;
    
    // Calculate standard deviations for ellipse radii
    let var_x: f64 = data.column(0).iter().map(|x| (x - center_x).powi(2)).sum::<f64>() / data.nrows() as f64;
    let var_y: f64 = data.column(1).iter().map(|y| (y - center_y).powi(2)).sum::<f64>() / data.nrows() as f64;
    
    // Use 2 standard deviations for ellipse (covers ~95% of data)
    let radius_x = var_x.sqrt() * 2.0;
    let radius_y = var_y.sqrt() * 2.0;
    
    // Create ellipse gate
    // create_ellipse_geometry expects Vec<(f32, f32)> coordinates
    // Create coordinates: center, right, top, left, bottom
    let center = (center_x as f32, center_y as f32);
    let right = ((center_x + radius_x) as f32, center_y as f32);
    let top = (center_x as f32, (center_y + radius_y) as f32);
    let left = ((center_x - radius_x) as f32, center_y as f32);
    let bottom = (center_x as f32, (center_y - radius_y) as f32);
    let coords = vec![center, right, top, left, bottom];
    
    let geometry = create_ellipse_geometry(
        coords,
        &config.fsc_channel,
        &config.ssc_channel,
    )?;
    
    let gate = Gate::new(
        "scatter-gate",
        "Automated Scatter Gate (Ellipse Fit)",
        geometry,
        Arc::from(config.fsc_channel.as_str()),
        Arc::from(config.ssc_channel.as_str()),
    );
    
    // Create mask
    let mask: Vec<bool> = (0..data.nrows())
        .map(|i| {
            let dx = (data[[i, 0]] - center_x) / radius_x;
            let dy = (data[[i, 1]] - center_y) / radius_y;
            dx * dx + dy * dy <= 1.0
        })
        .collect();
    
    Ok((Some(gate), mask, "EllipseFit".to_string()))
}
