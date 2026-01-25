//! DBSCAN clustering implementation

use crate::clustering::{ClusteringError, ClusteringResult};
use linfa::prelude::*;
use linfa_clustering::Dbscan as LinfaDbscan;
use ndarray::Array2;

/// Configuration for DBSCAN clustering
#[derive(Debug, Clone)]
pub struct DbscanConfig {
    /// Maximum distance between samples for one to be considered in the neighborhood of the other
    pub eps: f64,
    /// Minimum number of samples in a neighborhood for a point to be a core point
    pub min_samples: usize,
}

impl Default for DbscanConfig {
    fn default() -> Self {
        Self {
            eps: 0.5,
            min_samples: 5,
        }
    }
}

/// DBSCAN clustering result
#[derive(Debug)]
pub struct DbscanResult {
    /// Cluster assignments for each point (-1 indicates noise/outlier)
    pub assignments: Vec<i32>,
    /// Number of clusters found
    pub n_clusters: usize,
    /// Number of noise points
    pub n_noise: usize,
}

/// DBSCAN clustering
pub struct Dbscan;

impl Dbscan {
    /// Perform DBSCAN clustering
    ///
    /// # Arguments
    /// * `data` - Input data matrix (n_samples × n_features)
    /// * `config` - Configuration for DBSCAN
    ///
    /// # Returns
    /// DbscanResult with cluster assignments
    pub fn fit(data: &Array2<f64>, config: &DbscanConfig) -> ClusteringResult<DbscanResult> {
        if data.nrows() == 0 {
            return Err(ClusteringError::EmptyData);
        }

        // Use linfa-clustering for DBSCAN
        // Note: linfa DBSCAN params only takes min_points, eps is set via tolerance
        // Array2 implements Records trait, so we can use DatasetBase::from
        // For ndarray 0.16 compatibility, ensure we're using the right version
        let dataset: DatasetBase<ndarray::ArrayBase<ndarray::OwnedRepr<f64>, ndarray::Dim<[usize; 2]>>, ()> = 
            DatasetBase::from(data.clone());
        let model = LinfaDbscan::params(config.min_samples)
            .tolerance(config.eps) // eps is set via tolerance
            .fit(&dataset)
            .map_err(|e| ClusteringError::ClusteringFailed(format!("{}", e)))?;

        // Extract assignments (linfa uses i32, -1 for noise)
        let assignments: Vec<i32> = model
            .labels()
            .iter()
            .map(|&label| label as i32)
            .collect();

        // Count clusters and noise
        let n_clusters = assignments
            .iter()
            .filter(|&&a| a >= 0)
            .map(|&a| a as usize)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        let n_noise = assignments.iter().filter(|&&a| a == -1).count();

        Ok(DbscanResult {
            assignments,
            n_clusters,
            n_noise,
        })
    }
}
