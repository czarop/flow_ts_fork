//! FCS integration for TRU-OLS unmixing.
//!
//! This module provides integration between TRU-OLS and the `Fcs` struct from `flow-fcs`.
//! It enables TRU-OLS unmixing directly on FCS file data structures.

#[cfg(feature = "flow-fcs")]
use crate::error::TruOlsError;
#[cfg(feature = "flow-fcs")]
use crate::unmixing::{TruOls, UnmixingStrategy};
#[cfg(feature = "flow-fcs")]
use ndarray::Array2;
#[cfg(feature = "flow-fcs")]
use flow_fcs::{Fcs, EventDataFrame};
#[cfg(feature = "flow-fcs")]

/// Extract detector data from Fcs DataFrame and convert to Array2<f64>.
///
/// This function extracts detector channel data from an Fcs struct and converts
/// it from f32 to f64 for use with TRU-OLS algorithm.
///
/// # Arguments
/// * `fcs` - The Fcs struct containing the data
/// * `detector_names` - Names of detector channels to extract (must exist in FCS file)
///
/// # Returns
/// Matrix of shape (events × detectors) with f64 values
///
/// # Errors
/// Returns error if any detector name is not found or data cannot be extracted
#[cfg(feature = "flow-fcs")]
pub fn extract_detector_data(
    fcs: &Fcs,
    detector_names: &[&str],
) -> Result<Array2<f64>, TruOlsError> {
    let n_events = fcs.get_event_count_from_dataframe();
    let n_detectors = detector_names.len();

    if n_detectors == 0 {
        return Err(TruOlsError::InsufficientData(
            "At least one detector must be specified".to_string(),
        ));
    }

    // Extract data for each detector
    let mut detector_data: Vec<Vec<f64>> = Vec::with_capacity(n_detectors);
    for &detector_name in detector_names {
        let f32_slice = fcs
            .get_parameter_events_slice(detector_name)
            .map_err(|e| TruOlsError::InsufficientData(format!(
                "Detector '{}' not found: {}",
                detector_name, e
            )))?;

        // Convert f32 to f64 efficiently
        let f64_vec: Vec<f64> = f32_slice.iter().map(|&x| x as f64).collect();
        detector_data.push(f64_vec);
    }

    // Build Array2 from column vectors (transpose to get events × detectors)
    let mut result = Array2::<f64>::zeros((n_events, n_detectors));
    for (detector_idx, data) in detector_data.iter().enumerate() {
        for (event_idx, &value) in data.iter().enumerate() {
            result[(event_idx, detector_idx)] = value;
        }
    }

    Ok(result)
}

/// Extension trait for Fcs to enable TRU-OLS unmixing.
#[cfg(feature = "flow-fcs")]
pub trait TruOlsUnmixing {
    /// Apply TRU-OLS unmixing to FCS data.
    ///
    /// This method performs TRU-OLS unmixing on the FCS data, returning a new
    /// DataFrame with unmixed endmember abundances. Only performs unmixing -
    /// no compensation or transformation is applied.
    ///
    /// # Arguments
    /// * `unstained_control` - Fcs struct containing unstained control data
    /// * `mixing_matrix` - Mixing matrix (detectors × endmembers) as f64
    /// * `detector_names` - Names of detector channels in the mixing matrix
    /// * `endmember_names` - Names of endmembers (dyes) in the mixing matrix
    /// * `autofluorescence_name` - Name of the autofluorescence endmember
    /// * `strategy` - Optional strategy for handling irrelevant abundances (default: Zero)
    ///
    /// # Returns
    /// New EventDataFrame with columns for each endmember containing unmixed abundances
    ///
    /// # Errors
    /// Returns error if data cannot be extracted, mixing matrix dimensions don't match,
    /// or unmixing fails
    fn apply_tru_ols_unmixing(
        &self,
        unstained_control: &Fcs,
        mixing_matrix: Array2<f64>,
        detector_names: &[&str],
        endmember_names: &[&str],
        autofluorescence_name: &str,
        strategy: Option<UnmixingStrategy>,
    ) -> Result<EventDataFrame, TruOlsError>;
}

#[cfg(feature = "flow-fcs")]
impl TruOlsUnmixing for Fcs {
    fn apply_tru_ols_unmixing(
        &self,
        unstained_control: &Fcs,
        mixing_matrix: Array2<f64>,
        detector_names: &[&str],
        endmember_names: &[&str],
        autofluorescence_name: &str,
        strategy: Option<UnmixingStrategy>,
    ) -> Result<EventDataFrame, TruOlsError> {
        use polars::prelude::{Column, DataFrame};
        use std::sync::Arc;

        // Validate dimensions
        let n_detectors = mixing_matrix.nrows();
        let n_endmembers = mixing_matrix.ncols();

        if detector_names.len() != n_detectors {
            return Err(TruOlsError::DimensionMismatch {
                expected: n_detectors,
                actual: detector_names.len(),
            });
        }

        if endmember_names.len() != n_endmembers {
            return Err(TruOlsError::DimensionMismatch {
                expected: n_endmembers,
                actual: endmember_names.len(),
            });
        }

        // Find autofluorescence index
        let autofluorescence_idx = endmember_names
            .iter()
            .position(|&name| name == autofluorescence_name)
            .ok_or_else(|| TruOlsError::InsufficientData(format!(
                "Autofluorescence endmember '{}' not found in endmember names",
                autofluorescence_name
            )))?;

        // Extract detector data from both files
        let stained_data = extract_detector_data(self, detector_names)?;
        let unstained_data = extract_detector_data(unstained_control, detector_names)?;

        // Create TRU-OLS instance
        let mut tru_ols = TruOls::new(mixing_matrix, unstained_data, autofluorescence_idx)?;

        // Set strategy if provided
        if let Some(s) = strategy {
            tru_ols.set_strategy(s);
        }

        // Perform unmixing
        let unmixed_abundances = tru_ols.unmix(&stained_data)?;

        // Convert results back to DataFrame
        let n_events = unmixed_abundances.nrows();
        let mut columns: Vec<Column> = Vec::with_capacity(n_endmembers);

        for (endmember_idx, &endmember_name) in endmember_names.iter().enumerate() {
            // Extract column from unmixed abundances and convert to f32
            let f64_values: Vec<f64> = (0..n_events)
                .map(|event_idx| unmixed_abundances[(event_idx, endmember_idx)])
                .collect();
            
            // Convert to f32 for DataFrame
            let f32_values: Vec<f32> = f64_values.iter().map(|&x| x as f32).collect();
            
            let column = Column::new(endmember_name.into(), f32_values);
            columns.push(column);
        }

        let df = DataFrame::new(columns)
            .map_err(|e| TruOlsError::InsufficientData(format!(
                "Failed to create DataFrame: {}",
                e
            )))?;

        Ok(Arc::new(df))
    }
}

#[cfg(test)]
#[cfg(feature = "flow-fcs")]
mod tests {
    use super::*;
    use ndarray::array;
    use flow_fcs::{Header, Metadata, Parameter, TransformType, file::AccessWrapper, parameter::ParameterMap};
    use polars::{frame::DataFrame, prelude::Column};
    use std::sync::Arc;

    /// Helper function to create a test Fcs struct with detector data
    fn create_test_fcs() -> Result<Fcs, Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::Write;

        // Create a temporary file for testing
        let temp_path = std::env::temp_dir().join("test_tru_ols_fcs.tmp");
        {
            let mut f = File::create(&temp_path)?;
            f.write_all(b"test")?;
        }

        // Create test DataFrame with detector channels
        let mut columns = Vec::new();
        columns.push(Column::new(
            "FL1-A".into(),
            vec![100.0f32, 200.0, 300.0, 400.0, 500.0],
        ));
        columns.push(Column::new(
            "FL2-A".into(),
            vec![50.0f32, 150.0, 250.0, 350.0, 450.0],
        ));
        columns.push(Column::new(
            "FL3-A".into(),
            vec![10.0f32, 20.0, 30.0, 40.0, 50.0],
        ));

        let df = DataFrame::new(columns).expect("Failed to create test DataFrame");

        // Create parameter map
        let mut params = ParameterMap::default();
        params.insert(
            "FL1-A".into(),
            Parameter::new(&1, "FL1-A", "FL1-A", &TransformType::Linear),
        );
        params.insert(
            "FL2-A".into(),
            Parameter::new(&2, "FL2-A", "FL2-A", &TransformType::Linear),
        );
        params.insert(
            "FL3-A".into(),
            Parameter::new(&3, "FL3-A", "FL3-A", &TransformType::Linear),
        );

        Ok(Fcs {
            header: Header::new(),
            metadata: Metadata::new(),
            parameters: params,
            data_frame: Arc::new(df),
            file_access: AccessWrapper::new(temp_path.to_str().unwrap_or(""))?,
        })
    }

    #[test]
    fn test_extract_detector_data_success() {
        let fcs = create_test_fcs().expect("Failed to create test FCS");
        
        let detector_names = &["FL1-A", "FL2-A", "FL3-A"];
        let result = extract_detector_data(&fcs, detector_names);
        
        assert!(result.is_ok(), "Should successfully extract detector data");
        let data = result.unwrap();
        
        // Check dimensions: 5 events × 3 detectors
        assert_eq!(data.nrows(), 5, "Should have 5 events");
        assert_eq!(data.ncols(), 3, "Should have 3 detectors");
        
        // Check first event values
        assert!((data[(0, 0)] - 100.0).abs() < 1e-6, "First detector, first event should be 100.0");
        assert!((data[(0, 1)] - 50.0).abs() < 1e-6, "Second detector, first event should be 50.0");
        assert!((data[(0, 2)] - 10.0).abs() < 1e-6, "Third detector, first event should be 10.0");
        
        // Check last event values
        assert!((data[(4, 0)] - 500.0).abs() < 1e-6, "First detector, last event should be 500.0");
        assert!((data[(4, 1)] - 450.0).abs() < 1e-6, "Second detector, last event should be 450.0");
        assert!((data[(4, 2)] - 50.0).abs() < 1e-6, "Third detector, last event should be 50.0");
    }

    #[test]
    fn test_extract_detector_data_missing_detector() {
        let fcs = create_test_fcs().expect("Failed to create test FCS");
        
        let detector_names = &["FL1-A", "NonExistent"];
        let result = extract_detector_data(&fcs, detector_names);
        
        assert!(result.is_err(), "Should error on missing detector");
        match result.unwrap_err() {
            TruOlsError::InsufficientData(msg) => {
                assert!(msg.contains("NonExistent"), "Error message should mention missing detector");
            }
            _ => panic!("Should return InsufficientData error"),
        }
    }

    #[test]
    fn test_extract_detector_data_empty_list() {
        let fcs = create_test_fcs().expect("Failed to create test FCS");
        
        let detector_names: &[&str] = &[];
        let result = extract_detector_data(&fcs, detector_names);
        
        assert!(result.is_err(), "Should error on empty detector list");
        match result.unwrap_err() {
            TruOlsError::InsufficientData(msg) => {
                assert!(msg.contains("At least one detector"), "Error message should mention requirement");
            }
            _ => panic!("Should return InsufficientData error"),
        }
    }

    #[test]
    fn test_extract_detector_data_subset() {
        let fcs = create_test_fcs().expect("Failed to create test FCS");
        
        // Extract only first two detectors
        let detector_names = &["FL1-A", "FL2-A"];
        let result = extract_detector_data(&fcs, detector_names);
        
        assert!(result.is_ok(), "Should successfully extract subset of detectors");
        let data = result.unwrap();
        
        assert_eq!(data.nrows(), 5, "Should have 5 events");
        assert_eq!(data.ncols(), 2, "Should have 2 detectors");
        
        // Verify values
        assert!((data[(0, 0)] - 100.0).abs() < 1e-6);
        assert!((data[(0, 1)] - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_f32_to_f64_conversion_precision() {
        // Test that f32 to f64 conversion preserves precision
        let f32_data = vec![1.0f32, 2.5, 3.14159, -1.0, 0.0, 1e-6];
        let f64_data: Vec<f64> = f32_data.iter().map(|&x| x as f64).collect();
        
        assert_eq!(f64_data.len(), f32_data.len());
        for (f32_val, f64_val) in f32_data.iter().zip(f64_data.iter()) {
            // f32 to f64 conversion should be exact
            assert_eq!(*f64_val, *f32_val as f64);
        }
    }

    #[test]
    fn test_extract_detector_data_ordering() {
        let fcs = create_test_fcs().expect("Failed to create test FCS");
        
        // Extract detectors in different order
        let detector_names = &["FL3-A", "FL1-A", "FL2-A"];
        let result = extract_detector_data(&fcs, detector_names);
        
        assert!(result.is_ok(), "Should successfully extract detectors in different order");
        let data = result.unwrap();
        
        // Check that ordering is preserved (FL3-A should be first column)
        assert!((data[(0, 0)] - 10.0).abs() < 1e-6, "First column should be FL3-A (10.0)");
        assert!((data[(0, 1)] - 100.0).abs() < 1e-6, "Second column should be FL1-A (100.0)");
        assert!((data[(0, 2)] - 50.0).abs() < 1e-6, "Third column should be FL2-A (50.0)");
    }

    #[test]
    fn test_apply_tru_ols_unmixing_basic() {
        let stained_fcs = create_test_fcs().expect("Failed to create stained FCS");
        let unstained_fcs = create_test_fcs().expect("Failed to create unstained FCS");
        
        // Create a simple mixing matrix: 3 detectors × 2 endmembers
        // Identity-like matrix for simplicity
        let mixing_matrix = array![
            [0.9, 0.1],
            [0.1, 0.9],
            [0.05, 0.05]
        ];
        
        let detector_names = &["FL1-A", "FL2-A", "FL3-A"];
        let endmember_names = &["Dye1", "Autofluorescence"];
        let autofluorescence = "Autofluorescence";
        
        let result = stained_fcs.apply_tru_ols_unmixing(
            &unstained_fcs,
            mixing_matrix,
            detector_names,
            endmember_names,
            autofluorescence,
            None, // Use default strategy
        );
        
        assert!(result.is_ok(), "Should successfully apply TRU-OLS unmixing");
        let unmixed_df = result.unwrap();
        
        // Check that DataFrame has correct number of columns (endmembers)
        assert_eq!(unmixed_df.width(), 2, "Should have 2 endmember columns");
        assert_eq!(unmixed_df.height(), 5, "Should have 5 events");
        
        // Check that columns exist
        let col_names: Vec<String> = unmixed_df.get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(col_names.contains(&"Dye1".to_string()), "Should have Dye1 column");
        assert!(col_names.contains(&"Autofluorescence".to_string()), "Should have Autofluorescence column");
    }

    #[test]
    fn test_apply_tru_ols_unmixing_dimension_mismatch() {
        let stained_fcs = create_test_fcs().expect("Failed to create stained FCS");
        let unstained_fcs = create_test_fcs().expect("Failed to create unstained FCS");
        
        // Create mixing matrix with wrong dimensions
        let mixing_matrix = array![
            [0.9, 0.1],
            [0.1, 0.9]
        ]; // 2×2 matrix, but we have 3 detectors
        
        let detector_names = &["FL1-A", "FL2-A", "FL3-A"]; // 3 detectors
        let endmember_names = &["Dye1", "Autofluorescence"];
        let autofluorescence = "Autofluorescence";
        
        let result = stained_fcs.apply_tru_ols_unmixing(
            &unstained_fcs,
            mixing_matrix,
            detector_names,
            endmember_names,
            autofluorescence,
            None,
        );
        
        assert!(result.is_err(), "Should error on dimension mismatch");
        match result.unwrap_err() {
            TruOlsError::DimensionMismatch { expected, actual } => {
                assert_eq!(expected, 2, "Expected 2 rows in mixing matrix");
                assert_eq!(actual, 3, "Actual 3 detectors provided");
            }
            _ => panic!("Should return DimensionMismatch error"),
        }
    }

    #[test]
    fn test_apply_tru_ols_unmixing_missing_autofluorescence() {
        let stained_fcs = create_test_fcs().expect("Failed to create stained FCS");
        let unstained_fcs = create_test_fcs().expect("Failed to create unstained FCS");
        
        let mixing_matrix = array![
            [0.9, 0.1],
            [0.1, 0.9],
            [0.05, 0.05]
        ];
        
        let detector_names = &["FL1-A", "FL2-A", "FL3-A"];
        let endmember_names = &["Dye1", "Dye2"]; // No Autofluorescence!
        let autofluorescence = "Autofluorescence";
        
        let result = stained_fcs.apply_tru_ols_unmixing(
            &unstained_fcs,
            mixing_matrix,
            detector_names,
            endmember_names,
            autofluorescence,
            None,
        );
        
        assert!(result.is_err(), "Should error on missing autofluorescence");
        match result.unwrap_err() {
            TruOlsError::InsufficientData(msg) => {
                assert!(msg.contains("Autofluorescence"), "Error message should mention autofluorescence");
            }
            _ => panic!("Should return InsufficientData error"),
        }
    }
}
