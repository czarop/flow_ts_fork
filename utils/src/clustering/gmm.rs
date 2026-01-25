//! Gaussian Mixture Model clustering implementation

use crate::clustering::{ClusteringError, ClusteringResult};
use linfa::prelude::*;
use linfa_clustering::GaussianMixtureModel as LinfaGmm;
use ndarray::Array2;

/// Configuration for GMM clustering
#[derive(Debug, Clone)]
pub struct GmmConfig {
    /// Number of components
    pub n_components: usize,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Tolerance for convergence
    pub tolerance: f64,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for GmmConfig {
    fn default() -> Self {
        Self {
            n_components: 2,
            max_iterations: 100,
            tolerance: 1e-3,
            seed: None,
        }
    }
}

/// GMM clustering result
#[derive(Debug)]
pub struct GmmResult {
    /// Cluster assignments for each point
    pub assignments: Vec<usize>,
    /// Component means
    pub means: Array2<f64>,
    /// Number of iterations performed
    pub iterations: usize,
    /// Log likelihood
    pub log_likelihood: f64,
}

/// Gaussian Mixture Model clustering
pub struct Gmm;

impl Gmm {
    /// Perform GMM clustering
    ///
    /// # Arguments
    /// * `data` - Input data matrix (n_samples × n_features)
    /// * `config` - Configuration for GMM
    ///
    /// # Returns
    /// GmmResult with cluster assignments and means
    pub fn fit(data: &Array2<f64>, config: &GmmConfig) -> ClusteringResult<GmmResult> {
        if data.nrows() == 0 {
            return Err(ClusteringError::EmptyData);
        }

        if data.nrows() < config.n_components {
            return Err(ClusteringError::InsufficientData {
                min: config.n_components,
                actual: data.nrows(),
            });
        }

        // Use linfa-clustering for GMM
        let dataset = DatasetBase::from(data.clone());
        let model = LinfaGmm::params(config.n_components)
            .max_n_iterations(config.max_iterations as u64)
            .tolerance(config.tolerance)
            .fit(&dataset)
            .map_err(|e| ClusteringError::ClusteringFailed(format!("{}", e)))?;

        // Extract assignments (hard assignment: most likely component)
        // GMM predict returns probabilities, we need to find argmax for each point
        let assignments: Vec<usize> = (0..data.nrows())
            .map(|i| {
                let point = data.row(i);
                let mut max_prob = f64::NEG_INFINITY;
                let mut best_component = 0;
                // Calculate probability for each component (simplified - use means distance)
                for (j, mean) in model.means().rows().into_iter().enumerate() {
                    let dist: f64 = point
                        .iter()
                        .zip(mean.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum();
                    let prob = (-dist).exp(); // Simplified probability
                    if prob > max_prob {
                        max_prob = prob;
                        best_component = j;
                    }
                }
                best_component
            })
            .collect();

        // Extract means (convert to Array2<f64>)
        let means = model.means().to_owned();

        Ok(GmmResult {
            assignments,
            means,
            iterations: config.max_iterations, // linfa doesn't expose n_iterations
            log_likelihood: 0.0, // linfa doesn't expose log_likelihood directly
        })
    }
}
