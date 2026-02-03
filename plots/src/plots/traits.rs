use std::f32::INFINITY;

use crate::options::PlotOptions;
use crate::render::RenderConfig;
use anyhow::Result;

/// Trait for plot types
///
/// This trait defines the interface that all plot types must implement.
/// Each plot type specifies its own options type and data type.
///
/// # Example
///
/// ```rust,no_run
/// use flow_plots::plots::traits::Plot;
/// use flow_plots::options::{PlotOptions, BasePlotOptions};
/// use flow_plots::render::RenderConfig;
/// use flow_plots::PlotBytes;
/// use anyhow::Result;
///
/// struct MyPlotOptions {
///     base: BasePlotOptions,
///     // ... your options
/// }
///
/// impl PlotOptions for MyPlotOptions {
///     fn base(&self) -> &BasePlotOptions { &self.base }
/// }
///
/// struct MyPlot;
///
/// impl Plot for MyPlot {
///     type Options = MyPlotOptions;
///     type Data = Vec<(f32, f32)>;
///
///     fn render(
///         &self,
///         data: Self::Data,
///         options: &Self::Options,
///         render_config: &mut RenderConfig,
///     ) -> Result<PlotBytes> {
///         // ... your rendering logic
///         Ok(vec![])
///     }
/// }
/// ```
pub trait Plot {
    /// The options type for this plot
    type Options: PlotOptions;

    /// The data type this plot accepts
    type Data;

    /// Render the plot with the given data and options
    ///
    /// # Arguments
    ///
    /// * `data` - The data to plot
    /// * `options` - Plot-specific options
    /// * `render_config` - Rendering configuration (progress callbacks, etc.)
    ///
    /// # Returns
    ///
    /// JPEG-encoded plot image bytes
    fn render(
        &self,
        data: Self::Data,
        options: &Self::Options,
        render_config: &mut RenderConfig,
    ) -> Result<crate::render::plothelper::PlotData>;
}

pub trait PlotDrawable {
    fn get_points(&self) -> Vec<(f32, f32)>;
    fn is_finalised(&self) -> bool;

    fn is_near_segment(
        &self,
        m: (f32, f32),
        a: (f32, f32),
        b: (f32, f32),
        tolerance: (f32, f32),
    ) -> Option<f32> {
        let (tol_x, tol_y) = tolerance;
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let length_sq = dx * dx + dy * dy;

        // 1. Find the nearest point on the segment
        let t_clamped = if length_sq == 0.0 {
            0.0
        } else {
            (((m.0 - a.0) * dx + (m.1 - a.1) * dy) / length_sq).clamp(0.0, 1.0)
        };

        let nearest_x = a.0 + t_clamped * dx;
        let nearest_y = a.1 + t_clamped * dy;

        // 2. Check the rectangular tolerance box
        let diff_x = (m.0 - nearest_x).abs();
        let diff_y = (m.1 - nearest_y).abs();

        if diff_x <= tol_x && diff_y <= tol_y {
            // 3. Return the actual Euclidean distance in data space
            let actual_dist = (diff_x.powi(2) + diff_y.powi(2)).sqrt();
            Some(actual_dist)
        } else {
            None
        }
    }
    fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32> {
        let points = self.get_points();
        let mut closest = INFINITY;
        for segment in points.windows(2) {
            let (p1, p2) = (segment[0], segment[1]);
            if let Some(dis) = self.is_near_segment(point, p1, p2, tolerance) {
                if dis < closest {
                    closest = dis;
                }
            }
        }
        if closest == INFINITY {
            return None;
        } else {
            return Some(closest);
        }
    }
}
