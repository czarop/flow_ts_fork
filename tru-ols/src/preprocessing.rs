//! Preprocessing functions for TRU-OLS algorithm.
//!
//! This module handles the analysis of unstained control data to determine
//! cutoff thresholds and calculate the nonspecific observation.

use crate::error::TruOlsError;
use faer::{Col, ColRef, Mat, MatRef};

/// Solve a linear system Ax = b, using least squares for overdetermined systems.
///
/// For overdetermined systems (nrows > ncols), uses QR-based least squares.
/// For square systems, uses LU decomposition.
///
/// # Arguments
/// * `a` - Coefficient matrix (nrows × ncols)
/// * `b` - Right-hand side vector (length = nrows)
///
/// # Returns
/// Solution vector x (length = ncols)
pub(crate) fn solve_linear_system(
    a: MatRef<'_, f64>,
    b: ColRef<'_, f64>,
) -> Result<Col<f64>, TruOlsError> {
    let nrows = a.nrows();
    let ncols = a.ncols();

    if nrows < ncols {
        return Err(TruOlsError::LinearAlgebra(
            "Underdetermined systems are not supported".to_string(),
        ));
    }

    #[cfg(feature = "blas")]
    {
        use faer_ext::IntoNdarray;
        use ndarray_linalg::LeastSquaresSvd;

        let a_ndarray = a.into_ndarray().to_owned();
        let b_ndarray = ndarray::Array1::from_vec(b.as_slice().to_vec());
        let x = a_ndarray
            .least_squares(&b_ndarray)
            .map_err(|e| TruOlsError::LinearAlgebra(format!("BLAS solve failed: {}", e)))?;
        Ok(Col::from_fn(ncols, |i| x.solution[i]))
    }

    #[cfg(not(feature = "blas"))]
    {
        use faer::linalg::solvers::{PartialPivLu, Qr};
        use faer::prelude::{Solve, SolveLstsq};

        let b_col = Mat::from_fn(nrows, 1, |i, _| b[i]);

        let x_faer = if nrows > ncols {
            let qr = Qr::new(a);
            qr.solve_lstsq(b_col.as_ref())
        } else {
            let lu = PartialPivLu::new(a);
            lu.solve(b_col.as_ref())
        };

        Ok(Col::from_fn(ncols, |i| x_faer[(i, 0)]))
    }
}

/// Calculates cutoff thresholds for each endmember based on unstained control data.
pub struct CutoffCalculator {
    cutoffs: Col<f64>,
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
        mixing_matrix: MatRef<'_, f64>,
        unstained_control: MatRef<'_, f64>,
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
            let observation = Col::from_fn(n_detectors, |i| unstained_control[(event_idx, i)]);
            let abundances = solve_linear_system(mixing_matrix, observation.as_ref())
                .map_err(|e| TruOlsError::LinearAlgebra(format!("Failed to solve: {}", e)))?;
            unmixed_abundances.push(
                (0..abundances.nrows()).map(|i| abundances[i]).collect(),
            );
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
            cutoffs: Col::from_fn(n_endmembers, |i| cutoffs[i]),
        })
    }

    /// Get the cutoff value for a specific endmember.
    pub fn get_cutoff(&self, endmember_idx: usize) -> f64 {
        self.cutoffs[endmember_idx]
    }

    /// Get all cutoff values.
    pub fn cutoffs(&self) -> &Col<f64> {
        &self.cutoffs
    }
}

/// Calculates the nonspecific observation from unstained control data.
pub struct NonspecificObservation {
    observation: Col<f64>,
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
        mixing_matrix: MatRef<'_, f64>,
        unstained_control: MatRef<'_, f64>,
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
        let mut mean_abundances = vec![0.0; n_endmembers];
        for event_idx in 0..n_events {
            let observation = Col::from_fn(n_detectors, |i| unstained_control[(event_idx, i)]);
            let abundances = solve_linear_system(mixing_matrix, observation.as_ref())
                .map_err(|e| TruOlsError::LinearAlgebra(format!("Failed to solve: {}", e)))?;

            for idx in 0..abundances.nrows() {
                let abundance = abundances[idx];
                if idx != autofluorescence_idx {
                    mean_abundances[idx] += abundance;
                }
            }
        }

        // Calculate mean (excluding autofluorescence)
        for i in 0..n_endmembers {
            mean_abundances[i] /= n_events as f64;
        }
        mean_abundances[autofluorescence_idx] = 0.0; // Ensure AF is zero

        let mean_col = Col::from_fn(n_endmembers, |i| mean_abundances[i]);

        // Calculate nonspecific observation: M · mean_abundances
        let observation = &mixing_matrix * &mean_col;

        Ok(Self {
            observation: Col::from_fn(n_detectors, |i| observation[i]),
        })
    }

    /// Get the nonspecific observation vector.
    pub fn observation(&self) -> &Col<f64> {
        &self.observation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use faer::mat;

    #[test]
    fn test_cutoff_calculation() {
        // Simple 2x2 mixing matrix
        let mixing_matrix = mat![[1.0, 0.1], [0.1, 1.0]];
        // Two unstained events
        let unstained = mat![[0.0, 0.0], [0.1, 0.1]];

        let calculator =
            CutoffCalculator::calculate(mixing_matrix.as_ref(), unstained.as_ref(), 0.995).unwrap();
        assert_eq!(calculator.cutoffs().nrows(), 2);
    }

    #[test]
    fn test_nonspecific_observation() {
        let mixing_matrix = mat![[1.0, 0.1], [0.1, 1.0]];
        let unstained = mat![[0.0, 0.0], [0.1, 0.1]];

        let nonspecific =
            NonspecificObservation::calculate(mixing_matrix.as_ref(), unstained.as_ref(), 0)
                .unwrap();
        assert_eq!(nonspecific.observation().nrows(), 2);
    }
}
