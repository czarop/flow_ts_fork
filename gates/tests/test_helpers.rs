//! Test helpers for automated gating tests
//!
//! Provides utilities to create synthetic FCS files with realistic scatter patterns
//! for testing automated gating algorithms.

use flow_fcs::{
    Fcs, Header, Metadata, Parameter, TransformType,
    file::AccessWrapper,
    parameter::ParameterMap,
};
use polars::prelude::*;
use std::sync::Arc;
use std::fs::File;
use std::io::Write;

/// Create a synthetic FCS file with realistic scatter patterns
///
/// # Arguments
/// * `n_events` - Number of events to generate
/// * `scenario` - Test scenario type
///
/// # Returns
/// Fcs struct with synthetic data
pub fn create_synthetic_fcs(n_events: usize, scenario: TestScenario) -> Result<Fcs, Box<dyn std::error::Error>> {
    // Create a temporary file for testing
    let temp_path = std::env::temp_dir().join(format!("test_fcs_{}.tmp", std::process::id()));
    {
        let mut f = File::create(&temp_path)?;
        f.write_all(b"test")?;
    }

    // Generate data based on scenario
    let (fsc_a, fsc_h, fsc_w, ssc_a, ssc_h) = match scenario {
        TestScenario::SinglePopulation => generate_single_population(n_events),
        TestScenario::MultiPopulation => generate_multi_population(n_events),
        TestScenario::WithDoublets => generate_with_doublets(n_events),
        TestScenario::NoisyData => generate_noisy_data(n_events),
        TestScenario::WithDebris => generate_with_debris(n_events),
    };

    // Create DataFrame with scatter channels
    let mut columns = Vec::new();
    columns.push(Column::new("FSC-A".into(), fsc_a));
    columns.push(Column::new("FSC-H".into(), fsc_h));
    columns.push(Column::new("FSC-W".into(), fsc_w));
    columns.push(Column::new("SSC-A".into(), ssc_a));
    columns.push(Column::new("SSC-H".into(), ssc_h));

    let df = DataFrame::new(columns).expect("Failed to create test DataFrame");

    // Create parameter map
    let mut params = ParameterMap::default();
    params.insert(
        "FSC-A".into(),
        Parameter::new(&1, "FSC-A", "FSC-A", &TransformType::Linear),
    );
    params.insert(
        "FSC-H".into(),
        Parameter::new(&2, "FSC-H", "FSC-H", &TransformType::Linear),
    );
    params.insert(
        "FSC-W".into(),
        Parameter::new(&3, "FSC-W", "FSC-W", &TransformType::Linear),
    );
    params.insert(
        "SSC-A".into(),
        Parameter::new(&4, "SSC-A", "SSC-A", &TransformType::Linear),
    );
    params.insert(
        "SSC-H".into(),
        Parameter::new(&5, "SSC-H", "SSC-H", &TransformType::Linear),
    );

    Ok(Fcs {
        header: Header::new(),
        metadata: Metadata::new(),
        parameters: params,
        data_frame: Arc::new(df),
        file_access: AccessWrapper::new(temp_path.to_str().unwrap_or(""))?,
    })
}

/// Test scenario types
#[derive(Debug, Clone, Copy)]
pub enum TestScenario {
    /// Single population scatter (ellipse fit test)
    SinglePopulation,
    /// Multi-population scatter (clustering test)
    MultiPopulation,
    /// Data with doublet patterns (doublet detection test)
    WithDoublets,
    /// Noisy data (edge case test)
    NoisyData,
}

/// Generate single population scatter data
///
/// Creates a tight cluster around a center point with Gaussian noise.
fn generate_single_population(n_events: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let center_fsc: f32 = 50000.0;
    let center_ssc: f32 = 30000.0;
    let spread: f32 = 10000.0;
    
    let mut fsc_a = Vec::with_capacity(n_events);
    let mut fsc_h = Vec::with_capacity(n_events);
    let mut fsc_w = Vec::with_capacity(n_events);
    let mut ssc_a = Vec::with_capacity(n_events);
    let mut ssc_h = Vec::with_capacity(n_events);
    
    for _ in 0..n_events {
        // Generate FSC-A with uniform noise around center
        let fsc_a_val = center_fsc + rng.gen_range(-spread..spread);
        fsc_a.push(fsc_a_val.max(0.0));
        
        // FSC-H is typically slightly less than FSC-A for singlets
        let fsc_h_val = fsc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1);
        fsc_h.push(fsc_h_val.max(0.0));
        
        // FSC-W is typically proportional to FSC-A
        let fsc_w_val = fsc_a_val * 0.3 + rng.gen_range(-spread * 0.05..spread * 0.05);
        fsc_w.push(fsc_w_val.max(0.0));
        
        // Generate SSC-A with uniform noise
        let ssc_a_val = center_ssc + rng.gen_range(-spread * 0.8..spread * 0.8);
        ssc_a.push(ssc_a_val.max(0.0));
        
        // SSC-H is typically slightly less than SSC-A
        let ssc_h_val = ssc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1);
        ssc_h.push(ssc_h_val.max(0.0));
    }
    
    (fsc_a, fsc_h, fsc_w, ssc_a, ssc_h)
}

/// Generate multi-population scatter data
///
/// Creates two distinct populations with different scatter characteristics.
fn generate_multi_population(n_events: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Population 1: smaller cells (lower FSC/SSC)
    let center1_fsc: f32 = 30000.0;
    let center1_ssc: f32 = 20000.0;
    
    // Population 2: larger cells (higher FSC/SSC)
    let center2_fsc: f32 = 70000.0;
    let center2_ssc: f32 = 50000.0;
    
    let spread: f32 = 8000.0;
    let n_pop1 = n_events / 2;
    let n_pop2 = n_events - n_pop1;
    
    let mut fsc_a = Vec::with_capacity(n_events);
    let mut fsc_h = Vec::with_capacity(n_events);
    let mut fsc_w = Vec::with_capacity(n_events);
    let mut ssc_a = Vec::with_capacity(n_events);
    let mut ssc_h = Vec::with_capacity(n_events);
    
    // Generate population 1
    for _ in 0..n_pop1 {
        let fsc_a_val = center1_fsc + rng.gen_range(-spread..spread);
        fsc_a.push(fsc_a_val.max(0.0));
        fsc_h.push((fsc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
        fsc_w.push((fsc_a_val * 0.3 + rng.gen_range(-spread * 0.05..spread * 0.05)).max(0.0));
        
        let ssc_a_val = center1_ssc + rng.gen_range(-spread * 0.8..spread * 0.8);
        ssc_a.push(ssc_a_val.max(0.0));
        ssc_h.push((ssc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
    }
    
    // Generate population 2
    for _ in 0..n_pop2 {
        let fsc_a_val = center2_fsc + rng.gen_range(-spread..spread);
        fsc_a.push(fsc_a_val.max(0.0));
        fsc_h.push((fsc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
        fsc_w.push((fsc_a_val * 0.3 + rng.gen_range(-spread * 0.05..spread * 0.05)).max(0.0));
        
        let ssc_a_val = center2_ssc + rng.gen_range(-spread * 0.8..spread * 0.8);
        ssc_a.push(ssc_a_val.max(0.0));
        ssc_h.push((ssc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
    }
    
    (fsc_a, fsc_h, fsc_w, ssc_a, ssc_h)
}

/// Generate data with doublet patterns
///
/// Creates singlet population plus doublet population with higher FSC-A/FSC-H ratios.
fn generate_with_doublets(n_events: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let center_fsc: f32 = 50000.0;
    let center_ssc: f32 = 30000.0;
    let spread: f32 = 10000.0;
    
    // ~10% doublets
    let n_doublets = n_events / 10;
    let n_singlets = n_events - n_doublets;
    
    let mut fsc_a = Vec::with_capacity(n_events);
    let mut fsc_h = Vec::with_capacity(n_events);
    let mut fsc_w = Vec::with_capacity(n_events);
    let mut ssc_a = Vec::with_capacity(n_events);
    let mut ssc_h = Vec::with_capacity(n_events);
    
    // Generate singlets (normal FSC-A/FSC-H ratio ~1.0)
    for _ in 0..n_singlets {
        let fsc_a_val = center_fsc + rng.gen_range(-spread..spread);
        fsc_a.push(fsc_a_val.max(0.0));
        fsc_h.push((fsc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
        fsc_w.push((fsc_a_val * 0.3 + rng.gen_range(-spread * 0.05..spread * 0.05)).max(0.0));
        
        let ssc_a_val = center_ssc + rng.gen_range(-spread * 0.8..spread * 0.8);
        ssc_a.push(ssc_a_val.max(0.0));
        ssc_h.push((ssc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
    }
    
    // Generate doublets (higher FSC-A/FSC-H ratio ~1.5-2.0)
    for _ in 0..n_doublets {
        let fsc_a_val = center_fsc * 1.8 + rng.gen_range(-spread..spread);
        fsc_a.push(fsc_a_val.max(0.0));
        // Doublets have FSC-H that doesn't scale as much, creating higher ratio
        fsc_h.push((fsc_a_val * 0.6 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
        fsc_w.push((fsc_a_val * 0.4 + rng.gen_range(-spread * 0.05..spread * 0.05)).max(0.0));
        
        let ssc_a_val = center_ssc * 1.5 + rng.gen_range(-spread * 0.8..spread * 0.8);
        ssc_a.push(ssc_a_val.max(0.0));
        ssc_h.push((ssc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
    }
    
    (fsc_a, fsc_h, fsc_w, ssc_a, ssc_h)
}

/// Generate noisy data (edge case)
///
/// Creates data with high noise and less clear patterns.
fn generate_noisy_data(n_events: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let center_fsc: f32 = 50000.0;
    let center_ssc: f32 = 30000.0;
    let spread: f32 = 20000.0; // Higher spread for noise
    
    let mut fsc_a = Vec::with_capacity(n_events);
    let mut fsc_h = Vec::with_capacity(n_events);
    let mut fsc_w = Vec::with_capacity(n_events);
    let mut ssc_a = Vec::with_capacity(n_events);
    let mut ssc_h = Vec::with_capacity(n_events);
    
    for _ in 0..n_events {
        let fsc_a_val = center_fsc + rng.gen_range(-spread..spread);
        fsc_a.push(fsc_a_val.max(0.0));
        fsc_h.push((fsc_a_val * 0.9 + rng.gen_range(-spread * 0.2..spread * 0.2)).max(0.0));
        fsc_w.push((fsc_a_val * 0.3 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
        
        let ssc_a_val = center_ssc + rng.gen_range(-spread * 0.8..spread * 0.8);
        ssc_a.push(ssc_a_val.max(0.0));
        ssc_h.push((ssc_a_val * 0.9 + rng.gen_range(-spread * 0.2..spread * 0.2)).max(0.0));
    }
    
    (fsc_a, fsc_h, fsc_w, ssc_a, ssc_h)
}
