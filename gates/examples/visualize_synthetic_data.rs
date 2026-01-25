//! Example: Visualize synthetic test data using flow-plots
//!
//! This example demonstrates the synthetic FCS file generation and creates
//! scatter plots for each test scenario.

use flow_fcs::Fcs;
use flow_plots::{DensityPlot, DensityPlotOptions, BasePlotOptions, AxisOptions, Plot};
use flow_plots::render::RenderConfig;
use std::fs;
use std::path::PathBuf;

// Import test helpers - copy the necessary parts inline for the example
mod test_helpers {
    pub use flow_fcs::{
        Fcs, Header, Metadata, Parameter, TransformType,
        file::AccessWrapper,
        parameter::ParameterMap,
    };
    use polars::prelude::*;
    use std::sync::Arc;
    use std::fs::File;
    use std::io::Write;

    #[derive(Debug, Clone, Copy)]
    pub enum TestScenario {
        SinglePopulation,
        MultiPopulation,
        WithDoublets,
        NoisyData,
    }

    pub fn create_synthetic_fcs(n_events: usize, scenario: TestScenario) -> Result<Fcs, Box<dyn std::error::Error>> {
        let temp_path = std::env::temp_dir().join(format!("test_fcs_{}.tmp", std::process::id()));
        {
            let mut f = File::create(&temp_path)?;
            f.write_all(b"test")?;
        }

        let (fsc_a, fsc_h, fsc_w, ssc_a, ssc_h) = match scenario {
            TestScenario::SinglePopulation => generate_single_population(n_events),
            TestScenario::MultiPopulation => generate_multi_population(n_events),
            TestScenario::WithDoublets => generate_with_doublets(n_events),
            TestScenario::NoisyData => generate_noisy_data(n_events),
        };

        let mut columns = Vec::new();
        columns.push(Column::new("FSC-A".into(), fsc_a));
        columns.push(Column::new("FSC-H".into(), fsc_h));
        columns.push(Column::new("FSC-W".into(), fsc_w));
        columns.push(Column::new("SSC-A".into(), ssc_a));
        columns.push(Column::new("SSC-H".into(), ssc_h));

        let df = DataFrame::new(columns).expect("Failed to create test DataFrame");

        let mut params = ParameterMap::default();
        params.insert("FSC-A".into(), Parameter::new(&1, "FSC-A", "FSC-A", &TransformType::Linear));
        params.insert("FSC-H".into(), Parameter::new(&2, "FSC-H", "FSC-H", &TransformType::Linear));
        params.insert("FSC-W".into(), Parameter::new(&3, "FSC-W", "FSC-W", &TransformType::Linear));
        params.insert("SSC-A".into(), Parameter::new(&4, "SSC-A", "SSC-A", &TransformType::Linear));
        params.insert("SSC-H".into(), Parameter::new(&5, "SSC-H", "SSC-H", &TransformType::Linear));

        Ok(Fcs {
            header: Header::new(),
            metadata: Metadata::new(),
            parameters: params,
            data_frame: Arc::new(df),
            file_access: AccessWrapper::new(temp_path.to_str().unwrap_or(""))?,
        })
    }

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
            let fsc_a_val = center_fsc + rng.gen_range(-spread..spread);
            fsc_a.push(fsc_a_val.max(0.0));
            fsc_h.push((fsc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
            fsc_w.push((fsc_a_val * 0.3 + rng.gen_range(-spread * 0.05..spread * 0.05)).max(0.0));
            let ssc_a_val = center_ssc + rng.gen_range(-spread * 0.8..spread * 0.8);
            ssc_a.push(ssc_a_val.max(0.0));
            ssc_h.push((ssc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
        }
        (fsc_a, fsc_h, fsc_w, ssc_a, ssc_h)
    }

    fn generate_multi_population(n_events: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let center1_fsc: f32 = 30000.0;
        let center1_ssc: f32 = 20000.0;
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
        
        for _ in 0..n_pop1 {
            let fsc_a_val = center1_fsc + rng.gen_range(-spread..spread);
            fsc_a.push(fsc_a_val.max(0.0));
            fsc_h.push((fsc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
            fsc_w.push((fsc_a_val * 0.3 + rng.gen_range(-spread * 0.05..spread * 0.05)).max(0.0));
            let ssc_a_val = center1_ssc + rng.gen_range(-spread * 0.8..spread * 0.8);
            ssc_a.push(ssc_a_val.max(0.0));
            ssc_h.push((ssc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
        }
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

    fn generate_with_doublets(n_events: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let center_fsc: f32 = 50000.0;
        let center_ssc: f32 = 30000.0;
        let spread: f32 = 10000.0;
        let n_doublets = n_events / 10;
        let n_singlets = n_events - n_doublets;
        
        let mut fsc_a = Vec::with_capacity(n_events);
        let mut fsc_h = Vec::with_capacity(n_events);
        let mut fsc_w = Vec::with_capacity(n_events);
        let mut ssc_a = Vec::with_capacity(n_events);
        let mut ssc_h = Vec::with_capacity(n_events);
        
        for _ in 0..n_singlets {
            let fsc_a_val = center_fsc + rng.gen_range(-spread..spread);
            fsc_a.push(fsc_a_val.max(0.0));
            fsc_h.push((fsc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
            fsc_w.push((fsc_a_val * 0.3 + rng.gen_range(-spread * 0.05..spread * 0.05)).max(0.0));
            let ssc_a_val = center_ssc + rng.gen_range(-spread * 0.8..spread * 0.8);
            ssc_a.push(ssc_a_val.max(0.0));
            ssc_h.push((ssc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
        }
        for _ in 0..n_doublets {
            let fsc_a_val = center_fsc * 1.8 + rng.gen_range(-spread..spread);
            fsc_a.push(fsc_a_val.max(0.0));
            fsc_h.push((fsc_a_val * 0.6 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
            fsc_w.push((fsc_a_val * 0.4 + rng.gen_range(-spread * 0.05..spread * 0.05)).max(0.0));
            let ssc_a_val = center_ssc * 1.5 + rng.gen_range(-spread * 0.8..spread * 0.8);
            ssc_a.push(ssc_a_val.max(0.0));
            ssc_h.push((ssc_a_val * 0.95 + rng.gen_range(-spread * 0.1..spread * 0.1)).max(0.0));
        }
        (fsc_a, fsc_h, fsc_w, ssc_a, ssc_h)
    }

    fn generate_noisy_data(n_events: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let center_fsc: f32 = 50000.0;
        let center_ssc: f32 = 30000.0;
        let spread: f32 = 20000.0;
        
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
}
use test_helpers::{create_synthetic_fcs, TestScenario};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    let output_dir = PathBuf::from("examples/synthetic_plots");
    fs::create_dir_all(&output_dir)?;

    // Generate and plot each scenario
    let scenarios = vec![
        (TestScenario::SinglePopulation, "single_population"),
        (TestScenario::MultiPopulation, "multi_population"),
        (TestScenario::WithDoublets, "with_doublets"),
        (TestScenario::NoisyData, "noisy_data"),
    ];

    for (scenario, name) in scenarios {
        println!("Generating plot for: {}", name);
        
        // Create synthetic FCS file
        let fcs = create_synthetic_fcs(5000, scenario)?;
        
        // Extract FSC-A and SSC-A data
        let fsc_a = fcs.get_parameter_events_slice("FSC-A")?;
        let ssc_a = fcs.get_parameter_events_slice("SSC-A")?;
        
        // Create plot data pairs
        let plot_data: Vec<(f32, f32)> = fsc_a
            .iter()
            .zip(ssc_a.iter())
            .map(|(&x, &y)| (x, y))
            .collect();
        
        // Create plot options
        let x_axis = AxisOptions::new()
            .range(0.0..=100_000.0)
            .label("FSC-A".to_string())
            .build()?;
        
        let y_axis = AxisOptions::new()
            .range(0.0..=100_000.0)
            .label("SSC-A".to_string())
            .build()?;
        
        let base = BasePlotOptions::new()
            .width(800u32)
            .height(600u32)
            .title(format!("Synthetic Data: {}", name.replace("_", " ")))
            .build()?;
        
        let plot_options = DensityPlotOptions::new()
            .base(base)
            .x_axis(x_axis)
            .y_axis(y_axis)
            .build()?;
        
        // Render plot
        let plot = DensityPlot::new();
        let mut render_config = RenderConfig::default();
        let image_bytes = plot.render(plot_data, &plot_options, &mut render_config)?;
        
        // Save to file
        let output_path = output_dir.join(format!("{}.png", name));
        fs::write(&output_path, image_bytes)?;
        println!("  Saved to: {}", output_path.display());
    }
    
    println!("\nAll plots generated successfully!");
    Ok(())
}
