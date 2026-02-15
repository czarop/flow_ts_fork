//! Synthetic data test utilities and tests for TRU-OLS

use flow_tru_ols::{TruOls, UnmixingStrategy};
use ndarray::{Array2, array};
use rand::Rng;

/// Generate a synthetic mixing matrix with specified dimensions
pub fn generate_mixing_matrix(n_detectors: usize, n_endmembers: usize) -> Array2<f64> {
    let mut rng = rand::rng();
    let mut matrix = Array2::<f64>::zeros((n_detectors, n_endmembers));

    for i in 0..n_detectors {
        for j in 0..n_endmembers {
            // Create a mixing matrix with some structure
            // Diagonal elements are strongest, with some cross-talk
            if i == j {
                matrix[(i, j)] = 0.8 + rng.random_range(0.0..0.2);
            } else {
                matrix[(i, j)] = rng.random_range(0.0..0.1);
            }
        }
    }

    matrix
}

/// Generate synthetic observations from known abundances
pub fn generate_observations(
    mixing_matrix: &Array2<f64>,
    abundances: &Array2<f64>,
    noise_level: f64,
) -> Array2<f64> {
    let mut rng = rand::rng();
    let observations = mixing_matrix.dot(abundances);

    // Add noise
    let mut noisy_observations = observations.clone();
    for val in noisy_observations.iter_mut() {
        *val += rng.random_range(-noise_level..noise_level);
    }

    noisy_observations
}

/// Generate synthetic abundances matrix
pub fn generate_abundances(n_events: usize, n_endmembers: usize, sparsity: f64) -> Array2<f64> {
    let mut rng = rand::rng();
    let mut abundances = Array2::<f64>::zeros((n_events, n_endmembers));

    for i in 0..n_events {
        for j in 0..n_endmembers {
            if rng.random::<f64>() > sparsity {
                abundances[(i, j)] = rng.random_range(0.0..100.0);
            }
        }
    }

    abundances
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mixing_matrix() {
        let matrix = generate_mixing_matrix(4, 5);
        assert_eq!(matrix.nrows(), 4);
        assert_eq!(matrix.ncols(), 5);

        // Check that diagonal elements are larger
        for i in 0..4.min(5) {
            assert!(matrix[(i, i)] > 0.5);
        }
    }

    #[test]
    fn test_generate_observations() {
        let mixing_matrix = array![[1.0, 0.1], [0.1, 1.0]];
        let abundances = array![[10.0, 5.0], [20.0, 15.0]];
        let observations = generate_observations(&mixing_matrix, &abundances, 0.1);

        assert_eq!(observations.nrows(), 2);
        assert_eq!(observations.ncols(), 2);
    }

    #[test]
    fn test_tru_ols_synthetic() {
        // Create a simple synthetic dataset
        let mixing_matrix = array![[1.0, 0.1, 0.05], [0.1, 1.0, 0.1], [0.05, 0.1, 1.0]];
        let n_detectors = 3;
        let n_endmembers = 3;

        // Create unstained control (mostly zeros with small noise)
        let mut unstained = Array2::<f64>::zeros((100, n_detectors));
        let mut rng = rand::rng();
        for val in unstained.iter_mut() {
            *val = rng.random_range(-0.1..0.1);
        }

        // Create TRU-OLS instance
        let tru_ols = TruOls::new(mixing_matrix.clone(), unstained, 0).unwrap();

        // Create test observations
        let test_abundances = array![[10.0, 0.0, 0.0], [0.0, 20.0, 0.0], [5.0, 5.0, 0.0]];
        let observations = mixing_matrix.dot(&test_abundances.t());
        // Unmix
        let observations_transposed = observations.t().to_owned();
        let unmixed = tru_ols.unmix(&observations_transposed).unwrap();

        // Check that we got reasonable results
        assert_eq!(unmixed.nrows(), observations.nrows());
        assert_eq!(unmixed.ncols(), n_endmembers);

        // First event should have mostly first endmember
        assert!(unmixed[(0, 0)] > 5.0);
    }

    #[test]
    fn test_tru_ols_removes_irrelevant_endmembers() {
        // Test that TRU-OLS correctly identifies and removes irrelevant endmembers
        let mixing_matrix = array![[1.0, 0.1], [0.1, 1.0]];
        let mut unstained = Array2::<f64>::zeros((100, 2));
        let mut rng = rand::rng();
        for val in unstained.iter_mut() {
            *val = rng.random_range(-0.1..0.1);
        }

        let tru_ols = TruOls::new(mixing_matrix.clone(), unstained, 0).unwrap();

        // Create observation with only first endmember present
        let observation = array![[10.0, 1.0]]; // Only first detector has signal

        let unmixed = tru_ols.unmix(&observation).unwrap();

        // Second endmember should be zero or very small
        assert!(unmixed[(0, 1)].abs() < 1.0);
    }
}
