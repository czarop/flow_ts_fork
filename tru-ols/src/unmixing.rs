//! TRU-OLS unmixing implementation.
//!
//! This module contains the main TRU-OLS algorithm that performs per-event
//! unmixing with iterative endmember removal.

use crate::error::TruOlsError;
use crate::preprocessing::{CutoffCalculator, NonspecificObservation};
use ndarray::{Array1, Array2, Axis};
use ndarray_linalg::Solve;

/// Strategy for handling irrelevant endmember abundances.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnmixingStrategy {
    /// Set irrelevant abundances to zero.
    Zero,
    /// Map irrelevant abundances to match unstained control distribution (UCM).
    UnstainedControlMapping,
}

/// Main TRU-OLS unmixing algorithm.
pub struct TruOls {
    mixing_matrix: Array2<f64>,
    cutoffs: Array1<f64>,
    nonspecific_observation: Array1<f64>,
    autofluorescence_idx: usize,
    strategy: UnmixingStrategy,
}

impl TruOls {
    /// Create a new TRU-OLS instance.
    ///
    /// # Arguments
    /// * `mixing_matrix` - The full mixing matrix (detectors × endmembers)
    /// * `unstained_control` - Unstained control observations (events × detectors)
    /// * `autofluorescence_idx` - Index of the autofluorescence endmember
    ///
    /// # Returns
    /// Configured TRU-OLS instance with default settings (99.5th percentile cutoff, Zero strategy)
    pub fn new(
        mixing_matrix: Array2<f64>,
        unstained_control: Array2<f64>,
        autofluorescence_idx: usize,
    ) -> Result<Self, TruOlsError> {
        let cutoffs = CutoffCalculator::calculate(&mixing_matrix, &unstained_control, 0.995)?;
        let nonspecific = NonspecificObservation::calculate(
            &mixing_matrix,
            &unstained_control,
            autofluorescence_idx,
        )?;

        Ok(Self {
            mixing_matrix,
            cutoffs: cutoffs.cutoffs().clone(),
            nonspecific_observation: nonspecific.observation().clone(),
            autofluorescence_idx,
            strategy: UnmixingStrategy::Zero,
        })
    }

    /// Set the cutoff percentile (default: 0.995).
    ///
    /// This will recalculate cutoffs from the unstained control.
    pub fn set_cutoff_percentile(
        &mut self,
        percentile: f64,
        unstained_control: &Array2<f64>,
    ) -> Result<(), TruOlsError> {
        let cutoffs = CutoffCalculator::calculate(&self.mixing_matrix, unstained_control, percentile)?;
        self.cutoffs = cutoffs.cutoffs().clone();
        Ok(())
    }

    /// Set the unmixing strategy.
    pub fn set_strategy(&mut self, strategy: UnmixingStrategy) {
        self.strategy = strategy;
    }

    /// Unmix a single event.
    ///
    /// # Arguments
    /// * `observation` - Detector outputs for a single event (length = n_detectors)
    ///
    /// # Returns
    /// * `relevant_abundances` - Abundances for endmembers that survived TRU-OLS
    /// * `relevant_indices` - Indices of relevant endmembers in the original mixing matrix
    /// * `irrelevant_abundances` - Abundances for removed endmembers (before removal)
    /// * `irrelevant_indices` - Indices of irrelevant endmembers
    pub fn unmix_event(
        &self,
        observation: &Array1<f64>,
    ) -> Result<
        (
            Array1<f64>,
            Vec<usize>,
            Vec<(usize, f64)>,
        ),
        TruOlsError,
    > {
        let n_detectors = self.mixing_matrix.nrows();
        if observation.len() != n_detectors {
            return Err(TruOlsError::DimensionMismatch {
                expected: n_detectors,
                actual: observation.len(),
            });
        }

        // Subtract nonspecific observation
        let adjusted_observation = observation - &self.nonspecific_observation;

        // Start with full mixing matrix
        let mut current_matrix = self.mixing_matrix.clone();
        let mut current_indices: Vec<usize> = (0..self.mixing_matrix.ncols()).collect();
        let mut irrelevant_abundances: Vec<(usize, f64)> = Vec::new();

        // Iterative unmixing with endmember removal
        loop {
            // Unmix with current matrix
            let abundances = current_matrix
                .solve(&adjusted_observation)
                .map_err(|e| TruOlsError::LinearAlgebra(format!("Failed to solve: {}", e)))?;

            // Find irrelevant endmembers (below cutoff, excluding autofluorescence)
            let mut to_remove = Vec::new();
            for (local_idx, &global_idx) in current_indices.iter().enumerate() {
                if global_idx == self.autofluorescence_idx {
                    continue; // Never remove autofluorescence
                }

                if abundances[local_idx] < self.cutoffs[global_idx] {
                    to_remove.push((local_idx, global_idx, abundances[local_idx]));
                }
            }

            // If no endmembers to remove, we're done
            if to_remove.is_empty() {
                return Ok((abundances, current_indices, irrelevant_abundances));
            }

            // Store irrelevant abundances before removal
            for (_, global_idx, abundance) in &to_remove {
                irrelevant_abundances.push((*global_idx, *abundance));
            }

            // Build list of columns to keep
            let to_remove_local_indices: std::collections::HashSet<usize> =
                to_remove.iter().map(|(local_idx, _, _)| *local_idx).collect();
            let keep_indices: Vec<usize> = (0..current_matrix.ncols())
                .filter(|idx| !to_remove_local_indices.contains(idx))
                .collect();

            // Remove columns from matrix using select
            current_matrix = current_matrix.select(Axis(1), &keep_indices);

            // Update indices to match
            current_indices = keep_indices.iter().map(|&idx| current_indices[idx]).collect();

            // Safety check: ensure we don't remove all endmembers
            if current_matrix.ncols() == 0 {
                return Err(TruOlsError::AllEndmembersRemoved { event_index: 0 });
            }
        }
    }

    /// Unmix an entire dataset.
    ///
    /// # Arguments
    /// * `dataset` - Observations for all events (events × detectors)
    ///
    /// # Returns
    /// Full unmixed abundances matrix (events × endmembers) with irrelevant abundances
    /// set according to the configured strategy
    pub fn unmix(&self, dataset: &Array2<f64>) -> Result<Array2<f64>, TruOlsError> {
        let n_events = dataset.nrows();
        let n_endmembers = self.mixing_matrix.ncols();
        let n_detectors = self.mixing_matrix.nrows();

        if dataset.ncols() != n_detectors {
            return Err(TruOlsError::DimensionMismatch {
                expected: n_detectors,
                actual: dataset.ncols(),
            });
        }

        // Initialize result matrix with zeros
        let mut result = Array2::<f64>::zeros((n_events, n_endmembers));

        // Use parallel processing for large datasets
        // Threshold: use parallel for datasets with >10k events
        const PARALLEL_THRESHOLD: usize = 10_000;
        
        if n_events > PARALLEL_THRESHOLD {
            use rayon::prelude::*;
            
            // Process events in parallel
            let results: Result<Vec<_>, _> = (0..n_events)
                .into_par_iter()
                .map(|event_idx| {
                    let observation = dataset.row(event_idx);
                    self.unmix_event(&observation.to_owned())
                        .map(|(relevant_abundances, relevant_indices, _)| {
                            (event_idx, relevant_abundances, relevant_indices)
                        })
                })
                .collect();

            // Fill in results
            for res in results? {
                let (event_idx, relevant_abundances, relevant_indices) = res;
                for (local_idx, &global_idx) in relevant_indices.iter().enumerate() {
                    result[(event_idx, global_idx)] = relevant_abundances[local_idx];
                }
            }
        } else {
            // Sequential processing for smaller datasets (lower overhead)
            for event_idx in 0..n_events {
                let observation = dataset.row(event_idx);
                let (relevant_abundances, relevant_indices, _) =
                    self.unmix_event(&observation.to_owned())?;

                // Fill in relevant abundances
                for (local_idx, &global_idx) in relevant_indices.iter().enumerate() {
                    result[(event_idx, global_idx)] = relevant_abundances[local_idx];
                }
            }
        }

        // Handle irrelevant abundances according to strategy
        // Note: UCM strategy would need unstained control data, which is not available here
        // This is handled at a higher level in the FCS integration
        match self.strategy {
            UnmixingStrategy::Zero => {
                // Already zero from initialization
            }
            UnmixingStrategy::UnstainedControlMapping => {
                // UCM requires unstained control data - handled in fcs_integration
                // For now, values remain zero
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_unmix_event() {
        let mixing_matrix = array![[1.0, 0.1], [0.1, 1.0]];
        let unstained = array![[0.0, 0.0], [0.1, 0.1]];

        let tru_ols = TruOls::new(mixing_matrix, unstained, 0).unwrap();
        let observation = array![1.0, 1.0];

        let (relevant, relevant_indices, irrelevant) =
            tru_ols.unmix_event(&observation).unwrap();
        assert!(!relevant_indices.is_empty());
    }
}
