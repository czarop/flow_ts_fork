//! # flow-utils
//!
//! Shared algorithms and utilities for flow cytometry crates.
//!
//! This crate provides high-performance implementations of common algorithms used across
//! multiple flow cytometry crates, including:
//!
//! - **Kernel Density Estimation (KDE)**: FFT-accelerated KDE with GPU support
//! - **Clustering**: K-means, DBSCAN, Gaussian Mixture Model
//! - **PCA**: Principal Component Analysis for dimensionality reduction
//!
//! ## Features
//!
//! - `gpu`: Enable GPU acceleration for KDE (requires burn and cubecl)

pub mod kde;
pub mod clustering;
pub mod pca;
pub mod common;

pub use kde::{KernelDensity, KernelDensity2D, KdeError, KdeResult};
pub use clustering::{
    KMeans, KMeansConfig, KMeansResult,
    Dbscan, DbscanConfig, DbscanResult,
    Gmm, GmmConfig, GmmResult,
    ClusteringError, ClusteringResult,
};
pub use pca::{Pca, PcaError, PcaResult};
