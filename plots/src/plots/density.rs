use crate::density_calc::calculate_density_per_pixel;
use crate::options::{DensityPlotOptions, PlotOptions};
use crate::plots::traits::Plot;
use crate::render::RenderConfig;
use crate::render::plotters_backend::render_pixels;
use anyhow::Result;

/// Density plot implementation
///
/// Creates a 2D density plot from (x, y) coordinate pairs by binning
/// data points into pixels and coloring by density.
///
/// # Example
///
/// ```rust,no_run
/// use flow_plots::plots::density::DensityPlot;
/// use flow_plots::options::DensityPlotOptions;
/// use flow_plots::render::RenderConfig;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let plot = DensityPlot::new();
/// let options = DensityPlotOptions::new()
///     .width(800)
///     .height(600)
///     .build()?;
/// let data: Vec<(f32, f32)> = vec![(100.0, 200.0), (150.0, 250.0)];
/// let mut render_config = RenderConfig::default();
/// let bytes = plot.render(data, &options, &mut render_config)?;
/// # Ok(())
/// # }
/// ```
pub struct DensityPlot;

impl DensityPlot {
    /// Create a new DensityPlot instance
    pub fn new() -> Self {
        Self
    }

    /// Render multiple density plots in batch
    ///
    /// High-level convenience method that handles both density calculation
    /// and rendering. For apps that want to orchestrate rendering themselves,
    /// use `calculate_density_per_pixel_batch` instead.
    ///
    /// # Arguments
    /// * `requests` - Vector of (data, options) tuples
    /// * `render_config` - Rendering configuration (shared across all plots)
    ///
    /// # Returns
    /// Vector of plot bytes, one per request
    pub fn render_batch(
        &self,
        requests: &[(Vec<(f32, f32)>, DensityPlotOptions)],
        render_config: &mut RenderConfig,
        gates: Option<&[Option<&[&dyn super::traits::PlotDrawable]>]>,
        gate_colours: Option<&[Option<&[u8]>]>,
    ) -> Result<Vec<crate::render::plothelper::PlotData>> {
        use crate::density_calc::calculate_density_per_pixel_batch;

        if let Some(gates) = gates {
            if gates.len() != requests.len() {
                return Err(anyhow::anyhow!(
                    "Number of gates ({}) does not match number of requests ({})",
                    gates.len(),
                    requests.len()
                ));
            }
            if let Some(gate_colors) = gate_colours {
                if gate_colors.len() != requests.len() {
                    return Err(anyhow::anyhow!(
                        "Number of gate colors ({}) does not match number of requests ({})",
                        gate_colors.len(),
                        requests.len()
                    ));
                }
            }
        }

        // Calculate density for all plots
        let raw_pixels_batch = calculate_density_per_pixel_batch(requests);

        // Render each plot
        let mut results = Vec::with_capacity(requests.len());
        for (i, raw_pixels) in raw_pixels_batch.iter().enumerate() {
            let gate_set = gates.and_then(|g| g[i]);
            let gate_colours = gate_colours.and_then(|gc| gc[i]);
            let bytes = render_pixels(raw_pixels.clone(), &requests[i].1, render_config, gate_set, gate_colours)?;
            results.push(bytes);
        }
        Ok(results)
    }
}

impl Plot for DensityPlot {
    type Options = DensityPlotOptions;
    type Data = std::sync::Arc<Vec<(f32, f32)>>;

    fn render(
        &self,
        data: Self::Data,
        options: &Self::Options,
        render_config: &mut RenderConfig,
        gates: Option<&[&dyn super::traits::PlotDrawable]>,
        gate_colours: Option<&[u8]>,
    ) -> Result<crate::render::plothelper::PlotData> {
        let density_start = std::time::Instant::now();

        // Calculate density per pixel
        let base = options.base();
        let raw_pixels = calculate_density_per_pixel(
            &data[..],
            base.width as usize,
            base.height as usize,
            options,
        );

        eprintln!(
            "  ├─ Density calculation: {:?} ({} pixels at {}x{})",
            density_start.elapsed(),
            raw_pixels.len(),
            base.width,
            base.height
        );

        let draw_start = std::time::Instant::now();
        let result = render_pixels(raw_pixels, options, render_config, gates, gate_colours);
        eprintln!("  └─ Draw + encode: {:?}", draw_start.elapsed());

        result
    }
}
