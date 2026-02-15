//! Clustering algorithms module
//!
//! Provides K-means, DBSCAN, and Gaussian Mixture Model clustering.

mod kmeans;
mod dbscan;
mod gmm;

pub use kmeans::{KMeans, KMeansConfig, KMeansResult};
pub use dbscan::{Dbscan, DbscanConfig, DbscanResult};
pub use gmm::{Gmm, GmmConfig, GmmResult};

use thiserror::Error;

/// Error type for clustering operations
#[derive(Error, Debug)]
pub enum ClusteringError {
    #[error("Empty data")]
    EmptyData,
    #[error("Insufficient data: need at least {min} points, got {actual}")]
    InsufficientData { min: usize, actual: usize },
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Parameter validation failed: {0}")]
    ValidationFailed(String),
    #[error("Clustering failed: {0}")]
    ClusteringFailed(String),
}

pub type ClusteringResult<T> = Result<T, ClusteringError>;
