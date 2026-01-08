use super::{PlotOptions, density::calculate_density_per_pixel, draw_plot};
use anyhow::Result;

pub enum PlotType {
    Dot,
    Density,
    Contour,
    Zebra,
    Histogram,
}

pub trait DensityPlottable {
    fn plot(&self, _data: Vec<(f32, f32)>, _options: Option<PlotOptions>) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}
impl DensityPlottable for PlotType {
    fn plot(&self, data: Vec<(f32, f32)>, options: Option<PlotOptions>) -> Result<Vec<u8>> {
        let options = options.unwrap_or_default();
        match self {
            PlotType::Density => {
                let density_start = std::time::Instant::now();
                // Use dimensions from options instead of hardcoded 400x400
                let raw_pixels = calculate_density_per_pixel(
                    &data[..],
                    options.width as usize,
                    options.height as usize,
                    &options,
                );
                eprintln!(
                    "  ├─ Density calculation: {:?} ({} pixels at {}x{})",
                    density_start.elapsed(),
                    raw_pixels.len(),
                    options.width,
                    options.height
                );

                let draw_start = std::time::Instant::now();
                let result = draw_plot(raw_pixels, &options);
                eprintln!("  └─ Draw + encode: {:?}", draw_start.elapsed());
                match result {
                    Ok((bytes, _, _, _)) => Ok(bytes),
                    Err(e) => Err(anyhow::anyhow!("failed to draw plot: {}", e)),
                }
            }
            _ => Err(anyhow::anyhow!("not a density plot")),
        }
    }
}
