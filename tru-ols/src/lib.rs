//! TRU-OLS (Truncated ReUnmixing Ordinary Least Squares) algorithm for flow cytometry unmixing.
//!
//! This crate implements the TRU-OLS algorithm, which reduces the variance of unmixed
//! abundance distributions by removing irrelevant endmembers (dyes) from the mixing matrix
//! on a per-event basis.
//!
//! # Overview
//!
//! TRU-OLS is a variant of stepwise regression that uses unstained control data to determine
//! which endmembers are relevant for each event. By unmixing each event with only its relevant
//! endmembers, the algorithm reduces variance and improves separation between populations.
//!
//! # Basic Usage
//!
//! ```no_run
//! use flow_tru_ols::{TruOls, UnmixingStrategy};
//! use ndarray::Array2;
//!
//! // Create a TRU-OLS instance
//! let mut tru_ols = TruOls::new(mixing_matrix, unstained_control);
//!
//! // Configure the algorithm
//! tru_ols.set_cutoff_percentile(0.995); // 99.5th percentile
//! tru_ols.set_strategy(UnmixingStrategy::Zero);
//!
//! // Unmix a dataset
//! let unmixed = tru_ols.unmix(&dataset)?;
//! # Ok::<(), anyhow::Error>(())
//! ```

pub mod error;
pub mod preprocessing;
pub mod unmixing;

#[cfg(feature = "flow-fcs")]
pub mod fcs_integration;

#[cfg(all(feature = "flow-fcs", feature = "plotting"))]
pub mod plotting;

pub use error::TruOlsError;
pub use preprocessing::{CutoffCalculator, NonspecificObservation};
pub use unmixing::{TruOls, UnmixingStrategy};

#[cfg(feature = "flow-fcs")]
pub use fcs_integration::{TruOlsUnmixing, extract_detector_data};

#[cfg(all(feature = "flow-fcs", feature = "plotting"))]
pub use plotting::{plot_abundance_distribution, plot_unmixed_comparison, plot_ucm_comparison};
