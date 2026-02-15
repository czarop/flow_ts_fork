//! Options for spectral signature plots

use crate::options::{AxisOptions, BasePlotOptions, PlotOptions};
use derive_builder::Builder;

/// Options for spectral signature plots
#[derive(Builder, Debug, Clone)]
#[builder(pattern = "owned")]
pub struct SpectralSignaturePlotOptions {
    /// Base plot options (layout, dimensions, etc.)
    #[builder(default)]
    pub base: BasePlotOptions,

    /// X-axis configuration (detector channels)
    #[builder(default)]
    pub x_axis: Option<AxisOptions>,

    /// Y-axis configuration (normalized intensity 0.0-1.0)
    #[builder(default)]
    pub y_axis: Option<AxisOptions>,

    /// Line color (default: blue)
    #[builder(default = "String::from(\"#1f77b4\")")]
    pub line_color: String,

    /// Line width (default: 2.0)
    #[builder(default = "2.0")]
    pub line_width: f64,

    /// Show grid (default: true)
    #[builder(default = "true")]
    pub show_grid: bool,
}

impl PlotOptions for SpectralSignaturePlotOptions {
    fn base(&self) -> &BasePlotOptions {
        &self.base
    }
}

impl SpectralSignaturePlotOptions {
    /// Create a new builder for SpectralSignaturePlotOptions
    pub fn new() -> SpectralSignaturePlotOptionsBuilder {
        SpectralSignaturePlotOptionsBuilder::default()
    }
}
