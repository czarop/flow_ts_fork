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
//! use faer::mat;
//!
//! // Mixing matrix (detectors × endmembers), unstained control (events × detectors)
//! let mixing_matrix = mat![
//!     [0.9, 0.1],
//!     [0.1, 0.9],
//!     [0.05, 0.05],
//! ];
//! let unstained_control = mat![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
//! let dataset = mat![[100.0, 50.0, 10.0], [200.0, 150.0, 20.0]];
//!
//! // Create a TRU-OLS instance (autofluorescence is endmember index 1)
//! let mut tru_ols = TruOls::new(mixing_matrix, unstained_control.clone(), 1)?;
//!
//! // Configure the algorithm
//! tru_ols.set_cutoff_percentile(0.995, unstained_control.as_ref())?;
//! tru_ols.set_strategy(UnmixingStrategy::Zero);
//!
//! // Unmix a dataset
//! let unmixed = tru_ols.unmix(dataset.as_ref())?;
//! # Ok::<(), flow_tru_ols::TruOlsError>(())
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
