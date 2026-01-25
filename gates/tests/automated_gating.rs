//! Integration tests for automated gating

use flow_gates::automated::{
    create_preprocessing_gates, create_preprocessing_gates_interactive,
    DoubletGateConfig, DoubletMethod, PipelineBreakpoint, PreprocessingConfig,
    ScatterGateConfig, ScatterGateMethod, UserReview,
};
use flow_gates::automated::scatter::{create_scatter_gate, ClusterAlgorithm};
use flow_gates::automated::doublets::detect_doublets;
use flow_fcs::Fcs;
use std::path::PathBuf;

/// Helper function to create a simple test FCS file
/// This is a placeholder - in real tests, we'd load actual FCS files
fn create_test_fcs() -> Result<Fcs, Box<dyn std::error::Error>> {
    // TODO: Create or load a test FCS file
    // For now, this is a placeholder
    Err("Test FCS creation not yet implemented".into())
}

#[test]
#[ignore] // Ignore until we have test data
fn test_scatter_gating_ellipse_fit() {
    let fcs = create_test_fcs().unwrap();
    
    let config = ScatterGateConfig {
        fsc_channel: "FSC-A".to_string(),
        ssc_channel: "SSC-A".to_string(),
        method: ScatterGateMethod::EllipseFit,
        min_events: 100,
        density_threshold: None,
        cluster_eps: None,
        cluster_min_samples: None,
    };
    
    let result = create_scatter_gate(&fcs, &config).unwrap();
    
    assert!(result.gate.is_some());
    assert_eq!(result.method_used, "EllipseFit");
    assert!(!result.population_mask.is_empty());
}

#[test]
#[ignore]
fn test_scatter_gating_density_contour() {
    let fcs = create_test_fcs().unwrap();
    
    let config = ScatterGateConfig {
        fsc_channel: "FSC-A".to_string(),
        ssc_channel: "SSC-A".to_string(),
        method: ScatterGateMethod::DensityContour { threshold: 0.1 },
        min_events: 100,
        density_threshold: Some(0.1),
        cluster_eps: None,
        cluster_min_samples: None,
    };
    
    let result = create_scatter_gate(&fcs, &config).unwrap();
    
    assert!(result.gate.is_some());
    assert_eq!(result.method_used, "DensityContour");
}

#[test]
#[ignore]
fn test_doublet_detection_ratio_mad() {
    let fcs = create_test_fcs().unwrap();
    
    let config = DoubletGateConfig {
        channels: vec![("FSC-A".to_string(), "FSC-H".to_string())],
        method: DoubletMethod::RatioMAD { nmad: 4.0 },
        nmad: Some(4.0),
        density_threshold: None,
        cluster_eps: None,
        cluster_min_samples: None,
    };
    
    let result = detect_doublets(&fcs, &config).unwrap();
    
    assert!(!result.singlet_mask.is_empty());
    assert_eq!(result.statistics.method_used, "RatioMAD(nmad=4)");
}

#[test]
#[ignore]
fn test_doublet_detection_density_based() {
    let fcs = create_test_fcs().unwrap();
    
    let config = DoubletGateConfig {
        channels: vec![("FSC-A".to_string(), "FSC-H".to_string())],
        method: DoubletMethod::DensityBased { threshold: 0.1 },
        nmad: None,
        density_threshold: Some(0.1),
        cluster_eps: None,
        cluster_min_samples: None,
    };
    
    let result = detect_doublets(&fcs, &config).unwrap();
    
    assert!(!result.singlet_mask.is_empty());
    assert!(result.statistics.method_used.starts_with("DensityBased"));
}

#[test]
#[ignore]
fn test_preprocessing_pipeline() {
    let fcs = create_test_fcs().unwrap();
    
    let config = PreprocessingConfig {
        scatter_config: ScatterGateConfig {
            fsc_channel: "FSC-A".to_string(),
            ssc_channel: "SSC-A".to_string(),
            method: ScatterGateMethod::EllipseFit,
            min_events: 100,
            density_threshold: None,
            cluster_eps: None,
            cluster_min_samples: None,
        },
        doublet_config: DoubletGateConfig {
            channels: vec![("FSC-A".to_string(), "FSC-H".to_string())],
            method: DoubletMethod::RatioMAD { nmad: 4.0 },
            nmad: Some(4.0),
            density_threshold: None,
            cluster_eps: None,
            cluster_min_samples: None,
        },
    };
    
    let result = create_preprocessing_gates(&fcs, config).unwrap();
    
    assert!(result.scatter_gate.is_some() || result.doublet_gate.is_some());
}

#[test]
#[ignore]
fn test_interactive_pipeline() {
    let fcs = create_test_fcs().unwrap();
    
    let config = PreprocessingConfig {
        scatter_config: ScatterGateConfig {
            fsc_channel: "FSC-A".to_string(),
            ssc_channel: "SSC-A".to_string(),
            method: ScatterGateMethod::EllipseFit,
            min_events: 100,
            density_threshold: None,
            cluster_eps: None,
            cluster_min_samples: None,
        },
        doublet_config: DoubletGateConfig {
            channels: vec![("FSC-A".to_string(), "FSC-H".to_string())],
            method: DoubletMethod::RatioMAD { nmad: 4.0 },
            nmad: Some(4.0),
            density_threshold: None,
            cluster_eps: None,
            cluster_min_samples: None,
        },
    };
    
    // Test interactive pipeline with accept callback
    let result = create_preprocessing_gates_interactive(
        &fcs,
        config,
        |_breakpoint| UserReview::Accept,
    )
    .unwrap();
    
    assert!(result.scatter_gate.is_some() || result.doublet_gate.is_some());
}
