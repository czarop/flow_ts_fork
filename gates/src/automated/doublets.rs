//! Enhanced doublet detection
//!
//! Provides multiple methods for detecting doublet events in flow cytometry data,
//! including the original peacoqc-rs method and improved density-based approaches.

use crate::{GateError, GateResult};
use flow_fcs::Fcs;
use flow_utils::kde::KernelDensity;

/// Configuration for doublet detection
#[derive(Debug, Clone)]
pub struct DoubletGateConfig {
    /// Channel pairs for doublet detection
    /// Each pair is (area_channel, height_channel) or (area_channel, width_channel)
    pub channels: Vec<(String, String)>,
    /// Detection method to use
    pub method: DoubletMethod,
    /// Number of MADs above median (for MAD-based methods)
    pub nmad: Option<f64>,
    /// Density threshold (for density-based methods)
    pub density_threshold: Option<f64>,
    /// Cluster epsilon (for DBSCAN)
    pub cluster_eps: Option<f64>,
    /// Minimum samples for clustering
    pub cluster_min_samples: Option<usize>,
}

/// Doublet detection method
#[derive(Debug, Clone)]
pub enum DoubletMethod {
    /// Ratio-based MAD method (peacoqc-rs approach)
    RatioMAD { nmad: f64 },
    /// Density-based detection using KDE
    DensityBased { threshold: f64 },
    /// Clustering-based detection
    Clustering { eps: f64, min_samples: usize },
    /// Hybrid approach combining multiple methods
    Hybrid,
}

/// Result of doublet detection
#[derive(Debug, Clone)]
pub struct DoubletGateResult {
    /// Exclusion gate for doublets (if generated)
    pub exclusion_gate: Option<Gate>,
    /// Singlet mask (true = singlet, false = doublet)
    pub singlet_mask: Vec<bool>,
    /// Doublet mask (true = doublet, false = singlet)
    pub doublet_mask: Vec<bool>,
    /// Statistics about doublet detection
    pub statistics: DoubletStatistics,
}

/// Statistics for doublet detection
#[derive(Debug, Clone)]
pub struct DoubletStatistics {
    /// Number of singlets detected
    pub n_singlets: usize,
    /// Number of doublets detected
    pub n_doublets: usize,
    /// Percentage of doublets
    pub doublet_percentage: f64,
    /// Method used
    pub method_used: String,
}

/// Detect doublets using specified method
///
/// # Arguments
/// * `fcs` - FCS file data
/// * `config` - Doublet detection configuration
///
/// # Returns
/// DoubletGateResult with masks and statistics
pub fn detect_doublets(
    fcs: &Fcs,
    config: &DoubletGateConfig,
) -> GateResult<DoubletGateResult> {
    if config.channels.is_empty() {
        return Err(GateError::Other {
            message: "No channel pairs specified for doublet detection".to_string(),
            source: None,
        });
    }

    // Use first channel pair for now (can extend to multiple pairs)
    let (area_channel, height_channel) = &config.channels[0];

    // Extract channel data (NO transformation - raw values)
    // Fcs API returns f32 slices, convert to f64 for processing
    let area_data_f32 = fcs
        .get_parameter_events_slice(area_channel)
        .map_err(|e| GateError::Other {
            message: format!("Failed to get area channel {}: {}", area_channel, e),
            source: None, // anyhow::Error doesn't implement StdError, use message only
        })?;
    let height_data_f32 = fcs
        .get_parameter_events_slice(height_channel)
        .map_err(|e| GateError::Other {
            message: format!("Failed to get height channel {}: {}", height_channel, e),
            source: None, // anyhow::Error doesn't implement StdError, use message only
        })?;
    
    // Convert to f64 for processing
    let area_data: Vec<f64> = area_data_f32.iter().map(|&x| x as f64).collect();
    let height_data: Vec<f64> = height_data_f32.iter().map(|&x| x as f64).collect();

    if area_data.len() != height_data.len() {
        return Err(GateError::Other {
            message: format!(
                "Area and height channels have different lengths: {} vs {}",
                area_data.len(),
                height_data.len()
            ),
            source: None,
        });
    }

    // Apply detection method
    let (singlet_mask, method_name) = match &config.method {
        DoubletMethod::RatioMAD { nmad } => {
            detect_ratio_mad(&area_data, &height_data, *nmad)?
        }
        DoubletMethod::DensityBased { threshold } => {
            detect_density_based(&area_data, &height_data, *threshold)?
        }
        DoubletMethod::Clustering { eps, min_samples } => {
            detect_clustering(&area_data, &height_data, *eps, *min_samples)?
        }
        DoubletMethod::Hybrid => {
            // Combine multiple methods
            let mad_result = detect_ratio_mad(&area_data, &height_data, config.nmad.unwrap_or(4.0))?;
            let density_result = detect_density_based(&area_data, &height_data, config.density_threshold.unwrap_or(0.1))?;
            
            // Intersection: both methods must agree it's a singlet
            let combined_mask: Vec<bool> = mad_result.0
                .iter()
                .zip(density_result.0.iter())
                .map(|(&a, &b)| a && b)
                .collect();
            
            (combined_mask, "Hybrid".to_string())
        }
    };

    let doublet_mask: Vec<bool> = singlet_mask.iter().map(|&x| !x).collect();
    let n_doublets = doublet_mask.iter().filter(|&&x| x).count();
    let n_singlets = singlet_mask.len() - n_doublets;
    let doublet_percentage = (n_doublets as f64 / singlet_mask.len() as f64) * 100.0;

    let statistics = DoubletStatistics {
        n_singlets,
        n_doublets,
        doublet_percentage,
        method_used: method_name,
    };

    // Generate exclusion gate (optional - can be None for now)
    let exclusion_gate = None; // TODO: Generate polygon gate for doublet region

    Ok(DoubletGateResult {
        exclusion_gate,
        singlet_mask,
        doublet_mask,
        statistics,
    })
}

/// Detect doublets using ratio-based MAD method (peacoqc-rs approach)
fn detect_ratio_mad(
    area_data: &[f64],
    height_data: &[f64],
    nmad: f64,
) -> GateResult<(Vec<bool>, String)> {
    // Calculate ratios
    let ratios: Vec<f64> = area_data
        .iter()
        .zip(height_data.iter())
        .map(|(&a, &h)| a / (1e-10 + h))
        .collect();

    // Calculate median and MAD
    let mut sorted_ratios = ratios.clone();
    sorted_ratios.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let median = if sorted_ratios.len() % 2 == 0 {
        (sorted_ratios[sorted_ratios.len() / 2 - 1] + sorted_ratios[sorted_ratios.len() / 2]) / 2.0
    } else {
        sorted_ratios[sorted_ratios.len() / 2]
    };

    // Calculate MAD (median absolute deviation)
    let deviations: Vec<f64> = ratios.iter().map(|&r| (r - median).abs()).collect();
    let mut sorted_deviations = deviations.clone();
    sorted_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    let mad = if sorted_deviations.len() % 2 == 0 {
        (sorted_deviations[sorted_deviations.len() / 2 - 1] + sorted_deviations[sorted_deviations.len() / 2]) / 2.0
    } else {
        sorted_deviations[sorted_deviations.len() / 2]
    };

    // Scaled MAD (R's default: constant = 1.4826)
    let scaled_mad = mad * 1.4826;
    let threshold = median + nmad * scaled_mad;

    // Create mask (singlets have ratio < threshold)
    let mask: Vec<bool> = ratios.iter().map(|&r| r < threshold).collect();

    Ok((mask, format!("RatioMAD(nmad={})", nmad)))
}

/// Detect doublets using density-based method
fn detect_density_based(
    area_data: &[f64],
    height_data: &[f64],
    threshold: f64,
) -> GateResult<(Vec<bool>, String)> {
    // Calculate ratios
    let ratios: Vec<f64> = area_data
        .iter()
        .zip(height_data.iter())
        .map(|(&a, &h)| a / (1e-10 + h))
        .collect();

    // Use KDE to estimate density of ratios
    let kde = KernelDensity::estimate(&ratios, 1.0, 512)
        .map_err(|e| GateError::Other {
            message: format!("KDE failed: {:?}", e),
            source: None,
        })?;

    // Find peak (main population)
    let peaks = kde.find_peaks(threshold);
    if peaks.is_empty() {
        return Err(GateError::Other {
            message: "No peaks found in ratio distribution".to_string(),
            source: None,
        });
    }

    let main_peak = peaks[0];

    // Calculate spread around peak
    let mut distances: Vec<f64> = ratios.iter().map(|&r| (r - main_peak).abs()).collect();
    distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    // Use 95th percentile as threshold
    let threshold_idx = (distances.len() as f64 * 0.95) as usize;
    let threshold_dist = distances[threshold_idx.min(distances.len() - 1)];

    // Create mask (singlets are within threshold distance of peak)
    let mask: Vec<bool> = ratios
        .iter()
        .map(|&r| (r - main_peak).abs() <= threshold_dist)
        .collect();

    Ok((mask, format!("DensityBased(threshold={})", threshold)))
}

/// Detect doublets using clustering method
fn detect_clustering(
    _area_data: &[f64],
    _height_data: &[f64],
    _eps: f64,
    _min_samples: usize,
) -> GateResult<(Vec<bool>, String)> {
    // TODO: Implement clustering-based detection once linfa API is fixed
    // For now, fall back to ratio MAD
    Err(GateError::Other {
        message: "Clustering-based doublet detection not yet implemented (pending linfa API fix)".to_string(),
        source: None,
    })
}
