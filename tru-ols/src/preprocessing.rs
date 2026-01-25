//! Preprocessing functions for TRU-OLS algorithm.
//!
//! This module handles the analysis of unstained control data to determine
//! cutoff thresholds and calculate the nonspecific observation.

use crate::error::TruOlsError;
use ndarray::{Array1, Array2};
use ndarray_linalg::Solve;

/// Calculates cutoff thresholds for each endmember based on unstained control data.
pub struct CutoffCalculator {
    cutoffs: Array1<f64>,
}

impl CutoffCalculator {
    /// Calculate cutoff thresholds from unstained control data.
    ///
    /// # Arguments
    /// * `mixing_matrix` - The full mixing matrix (detectors × endmembers)
    /// * `unstained_control` - Unstained control observations (events × detectors)
    /// * `percentile` - Percentile to use for cutoff (e.g., 0.995 for 99.5th percentile)
    ///
    /// # Returns
    /// Vector of cutoff values, one per endmember
    pub fn calculate(
        mixing_matrix: &Array2<f64>,
        unstained_control: &Array2<f64>,
        percentile: f64,
    ) -> Result<Self, TruOlsError> {
        if !(0.0..=1.0).contains(&percentile) {
            return Err(TruOlsError::InvalidPercentile(percentile));
        }

        let n_detectors = mixing_matrix.nrows();
        let n_endmembers = mixing_matrix.ncols();
        let n_events = unstained_control.nrows();

        if unstained_control.ncols() != n_detectors {
            return Err(TruOlsError::DimensionMismatch {
                expected: n_detectors,
                actual: unstained_control.ncols(),
            });
        }

        if n_events == 0 {
            return Err(TruOlsError::InsufficientData(
                "Unstained control must contain at least one event".to_string(),
            ));
        }

        // Unmix each event in the unstained control
        let mut unmixed_abundances: Vec<Vec<f64>> = Vec::with_capacity(n_events);
        for event_idx in 0..n_events {
            let observation = unstained_control.row(event_idx);
            let abundances = mixing_matrix
                .solve(&observation.to_owned())
                .map_err(|e| TruOlsError::LinearAlgebra(format!("Failed to solve: {}", e)))?;
            unmixed_abundances.push(abundances.to_vec());
        }

        // Calculate percentile for each endmember
        let mut cutoffs = Vec::with_capacity(n_endmembers);
        for endmember_idx in 0..n_endmembers {
            let mut values: Vec<f64> = unmixed_abundances
                .iter()
                .map(|abundances| abundances[endmember_idx])
                .collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let percentile_idx = ((values.len() - 1) as f64 * percentile).round() as usize;
            let cutoff = values[percentile_idx.min(values.len() - 1)];
            cutoffs.push(cutoff);
        }

        Ok(Self {
            cutoffs: Array1::from(cutoffs),
        })
    }

    /// Get the cutoff value for a specific endmember.
    pub fn get_cutoff(&self, endmember_idx: usize) -> f64 {
        self.cutoffs[endmember_idx]
    }

    /// Get all cutoff values.
    pub fn cutoffs(&self) -> &Array1<f64> {
        &self.cutoffs
    }
}

/// Calculates the nonspecific observation from unstained control data.
pub struct NonspecificObservation {
    observation: Array1<f64>,
}

impl NonspecificObservation {
    /// Calculate the nonspecific observation.
    ///
    /// This represents the expected "background" signal from nonspecific binding/noise.
    /// It is calculated as: `o⃗NS = M · E[α⃗c-NoAuto]`
    ///
    /// # Arguments
    /// * `mixing_matrix` - The full mixing matrix (detectors × endmembers)
    /// * `unstained_control` - Unstained control observations (events × detectors)
    /// * `autofluorescence_idx` - Index of the autofluorescence endmember (excluded from mean)
    pub fn calculate(
        mixing_matrix: &Array2<f64>,
        unstained_control: &Array2<f64>,
        autofluorescence_idx: usize,
    ) -> Result<Self, TruOlsError> {
        let n_detectors = mixing_matrix.nrows();
        let n_endmembers = mixing_matrix.ncols();

        if autofluorescence_idx >= n_endmembers {
            return Err(TruOlsError::NoAutofluorescenceEndmember);
        }

        if unstained_control.ncols() != n_detectors {
            return Err(TruOlsError::DimensionMismatch {
                expected: n_detectors,
                actual: unstained_control.ncols(),
            });
        }

        let n_events = unstained_control.nrows();
        if n_events == 0 {
            return Err(TruOlsError::InsufficientData(
                "Unstained control must contain at least one event".to_string(),
            ));
        }

        // Unmix each event and calculate mean abundances (excluding autofluorescence)
        let mut mean_abundances = Array1::<f64>::zeros(n_endmembers);
        for event_idx in 0..n_events {
            let observation = unstained_control.row(event_idx);
            let abundances = mixing_matrix
                .solve(&observation.to_owned())
                .map_err(|e| TruOlsError::LinearAlgebra(format!("Failed to solve: {}", e)))?;

            for (idx, &abundance) in abundances.iter().enumerate() {
                if idx != autofluorescence_idx {
                    mean_abundances[idx] += abundance;
                }
            }
        }

        // Calculate mean (excluding autofluorescence)
        mean_abundances /= n_events as f64;
        mean_abundances[autofluorescence_idx] = 0.0; // Ensure AF is zero

        // Calculate nonspecific observation: M · mean_abundances
        let observation = mixing_matrix.dot(&mean_abundances);

        Ok(Self { observation })
    }

    /// Get the nonspecific observation vector.
    pub fn observation(&self) -> &Array1<f64> {
        &self.observation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_cutoff_calculation() {
        // Simple 2x2 mixing matrix
        let mixing_matrix = array![[1.0, 0.1], [0.1, 1.0]];
        // Two unstained events
        let unstained = array![[0.0, 0.0], [0.1, 0.1]];

        let calculator = CutoffCalculator::calculate(&mixing_matrix, &unstained, 0.995).unwrap();
        assert_eq!(calculator.cutoffs().len(), 2);
    }

    #[test]
    fn test_nonspecific_observation() {
        let mixing_matrix = array![[1.0, 0.1], [0.1, 1.0]];
        let unstained = array![[0.0, 0.0], [0.1, 0.1]];

        let nonspecific = NonspecificObservation::calculate(&mixing_matrix, &unstained, 0).unwrap();
        assert_eq!(nonspecific.observation().len(), 2);
    }
}
