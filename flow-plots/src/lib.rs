pub mod cache;
pub mod colormap;
pub mod density;
pub mod executor;
pub mod plot_types;

use colormap::ColorMaps;
use core::f32;
use flow_fcs::transform::{Formattable, TransformType, Transformable};
use flow_fcs::{file::Fcs, parameter::Parameter};
use image::RgbImage;
use plotters::{
    backend::BitMapBackend, chart::ChartBuilder, prelude::IntoDrawingArea, style::WHITE,
    style::colors::colormaps::ViridisRGB,
};
use std::{ops::RangeInclusive, range::Range};

pub type PlotBytes = Vec<u8>;
pub type PlotRange = RangeInclusive<f32>;
/// TODO: extract into separate types of options per plot, and one general options struct;
///
#[derive(Clone)]
pub struct PlotOptions {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub colormap: ColorMaps,
    pub plot_range_x: PlotRange,
    pub plot_range_y: PlotRange,
    pub x_label_transform: TransformType,
    pub y_label_transform: TransformType,
    pub x_axis_label: Option<String>,
    pub y_axis_label: Option<String>,
    pub margin: u32,
    pub x_label_area_size: u32,
    pub y_label_area_size: u32,
}
impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            title: "Density Plot".to_string(),
            width: 400,
            height: 400,
            colormap: ColorMaps::Viridis(ViridisRGB),
            plot_range_x: 0f32..=200_000f32,
            plot_range_y: 0f32..=200_000f32,
            x_label_transform: TransformType::default(),
            y_label_transform: TransformType::default(),
            x_axis_label: None,
            y_axis_label: None,
            margin: 10,
            x_label_area_size: 50,
            y_label_area_size: 50,
        }
    }
}
impl PlotOptions {
    pub fn new(
        fcs: &Fcs,
        x_parameter: &Parameter,
        y_parameter: &Parameter,
        width: Option<u32>,
        height: Option<u32>,
        colormap: Option<ColorMaps>,
        plot_range_x: Option<PlotRange>,
        plot_range_y: Option<PlotRange>,
        x_label_transform: Option<TransformType>,
        y_label_transform: Option<TransformType>,
        // defaults: Option<impl PlotOptionsDefaults>,
    ) -> anyhow::Result<Self> {
        let default = Self::default();

        // Determine plot ranges (avoid panics so backend failures remain enumerable).
        let plot_range_x = match plot_range_x {
            Some(r) => r,
            None => match x_parameter.channel_name.as_ref() {
                name if name.contains("FSC") || name.contains("SSC") => {
                    default.plot_range_x.clone()
                }
                name if name.contains("Time") => {
                    let time_values = fcs.get_events_view_for_parameter(x_parameter)?;
                    let time_max = time_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                    0f32..=time_max
                }
                _ => {
                    let raw_view = fcs.get_events_view_for_parameter(x_parameter)?;
                    let transformed = raw_view
                        .iter()
                        .map(|&v| x_parameter.transform.transform(&v))
                        .collect::<Vec<_>>();
                    get_percentile_bounds(&transformed, 0.01, 0.99)
                }
            },
        };

        let plot_range_y = match plot_range_y {
            Some(r) => r,
            None => match y_parameter.channel_name.as_ref() {
                name if name.contains("FSC") || name.contains("SSC") => {
                    default.plot_range_y.clone()
                }
                name if name.contains("Time") => {
                    let time_values = fcs.get_events_view_for_parameter(y_parameter)?;
                    let time_max = time_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                    0f32..=time_max
                }
                _ => {
                    let raw_view = fcs.get_events_view_for_parameter(y_parameter)?;
                    let transformed = raw_view
                        .iter()
                        .map(|&v| y_parameter.transform.transform(&v))
                        .collect::<Vec<_>>();
                    get_percentile_bounds(&transformed, 0.01, 0.99)
                }
            },
        };

        let x_label_transform =
            x_label_transform.unwrap_or_else(|| match x_parameter.channel_name.as_ref() {
                name if name.contains("FSC") || name.contains("SSC") => TransformType::Linear,
                _ => TransformType::default(),
            });

        let y_label_transform =
            y_label_transform.unwrap_or_else(|| match y_parameter.channel_name.as_ref() {
                name if name.contains("FSC") || name.contains("SSC") => TransformType::Linear,
                _ => TransformType::default(),
            });

        let title = fcs.get_fil_keyword()?.to_string();

        Ok(Self {
            title,
            width: width.unwrap_or(default.width),
            height: height.unwrap_or(default.height),
            colormap: colormap.unwrap_or(default.colormap),
            plot_range_x,
            plot_range_y,
            x_label_transform,
            y_label_transform,
            x_axis_label: None,
            y_axis_label: None,
            margin: 10,
            x_label_area_size: 50,
            y_label_area_size: 50,
        })
    }
}

use crate::plotting::density::RawPixelData;
type PlotMargin = u32;
type PlotXLabelAreaSize = u32;
type PlotYLabelAreaSize = u32;
pub fn draw_plot(
    pixels: Vec<RawPixelData>,
    options: &PlotOptions,
) -> anyhow::Result<(
    PlotBytes,
    PlotMargin,
    PlotXLabelAreaSize,
    PlotYLabelAreaSize,
)> {
    crate::plotting::executor::with_render_lock(|| {
        let PlotOptions {
            title: _,
            width,
            height,
            colormap: _,
            plot_range_x,
            plot_range_y,
            x_label_transform,
            y_label_transform,
            x_axis_label,
            y_axis_label,
            margin,
            x_label_area_size,
            y_label_area_size,
        } = options;

        let setup_start = std::time::Instant::now();
        // Use RGB buffer (3 bytes per pixel) since we'll encode to JPEG which doesn't support alpha
        let mut pixel_buffer = vec![255; (width * height * 3) as usize];

        let (plot_x_range, plot_y_range, x_spec, y_spec) = {
            // let backend = BitMapBackend::new("./test.png", (width, height));
            let backend = BitMapBackend::with_buffer(&mut pixel_buffer, (*width, *height));
            let root = backend.into_drawing_area();
            root.fill(&WHITE)
                .map_err(|e| anyhow::anyhow!("failed to fill plot background: {e}"))?;

            // Create appropriate ranges based on transform types
            let (x_spec, y_spec) = create_axis_specs(
                plot_range_x,
                plot_range_y,
                x_label_transform,
                y_label_transform,
            )?;

            let mut chart = ChartBuilder::on(&root)
                .margin(*margin)
                .x_label_area_size(*x_label_area_size)
                .y_label_area_size(*y_label_area_size)
                .build_cartesian_2d(x_spec.start..x_spec.end, y_spec.start..y_spec.end)?;

            // Clone transforms to avoid lifetime issues with closures
            let x_transform_clone = x_label_transform.clone();
            let y_transform_clone = y_label_transform.clone();

            // Create owned closures for formatters
            let x_formatter = move |x: &f32| -> String { x_transform_clone.format(x) };
            let y_formatter = move |y: &f32| -> String { y_transform_clone.format(y) };

            let mut mesh = chart.configure_mesh();
            mesh.x_max_light_lines(4)
                .y_max_light_lines(4)
                .x_labels(10)
                .y_labels(10)
                .x_label_formatter(&x_formatter)
                .y_label_formatter(&y_formatter);

            // Add axis labels if provided
            if let Some(x_label) = x_axis_label {
                mesh.x_desc(x_label);
            }
            if let Some(y_label) = y_axis_label {
                mesh.y_desc(y_label);
            }

            let mesh_start = std::time::Instant::now();
            mesh.draw()
                .map_err(|e| anyhow::anyhow!("failed to draw plot mesh: {e}"))?;
            eprintln!("    ├─ Mesh drawing: {:?}", mesh_start.elapsed());

            // Get the plotting area bounds (we'll use these after Plotters releases the buffer)
            let plotting_area = chart.plotting_area();
            let (plot_x_range, plot_y_range) = plotting_area.get_pixel_range();

            root.present()
                .map_err(|e| anyhow::anyhow!("failed to present plotters buffer: {e}"))?;

            (plot_x_range, plot_y_range, x_spec, y_spec)
        }; // End Plotters scope - pixel_buffer is now released and we can write to it

        // DIRECT PIXEL BUFFER WRITING - 10-50x faster than Plotters series rendering
        // Now that Plotters has released pixel_buffer, we can write directly
        let series_start = std::time::Instant::now();

        let plot_x_start = plot_x_range.start as f32;
        let plot_y_start = plot_y_range.start as f32;
        let plot_width = (plot_x_range.end - plot_x_range.start) as f32;
        let plot_height = (plot_y_range.end - plot_y_range.start) as f32;

        // Calculate scale factors from data coordinates to screen pixels
        let data_width = x_spec.end - x_spec.start;
        let data_height = y_spec.end - y_spec.start;

        // Write each pixel directly to the buffer
        for pixel in &pixels {
            let data_x = pixel.x;
            let data_y = pixel.y;

            // Transform data coordinates to screen pixel coordinates
            let rel_x = (data_x - x_spec.start) / data_width;
            let rel_y = (y_spec.end - data_y) / data_height; // Flip Y (screen coords go down)

            let screen_x = (plot_x_start + rel_x * plot_width) as i32;
            let screen_y = (plot_y_start + rel_y * plot_height) as i32;

            // Bounds check
            if screen_x >= plot_x_range.start
                && screen_x < plot_x_range.end
                && screen_y >= plot_y_range.start
                && screen_y < plot_y_range.end
            {
                let px = screen_x as u32;
                let py = screen_y as u32;

                // Write to pixel buffer (RGB format - 3 bytes per pixel)
                let idx = ((py * *width + px) * 3) as usize;

                if idx + 2 < pixel_buffer.len() {
                    pixel_buffer[idx] = pixel.r;
                    pixel_buffer[idx + 1] = pixel.g;
                    pixel_buffer[idx + 2] = pixel.b;
                }
            }
        }

        eprintln!(
            "    ├─ Direct pixel writing: {:?} ({} pixels)",
            series_start.elapsed(),
            pixels.len()
        );
        eprintln!("    ├─ Total plotting: {:?}", setup_start.elapsed());

        let img_start = std::time::Instant::now();
        let img: RgbImage = image::ImageBuffer::from_vec(*width, *height, pixel_buffer)
            .ok_or_else(|| anyhow::anyhow!("plot image buffer had unexpected size"))?;
        eprintln!("    ├─ Image buffer conversion: {:?}", img_start.elapsed());

        let encode_start = std::time::Instant::now();

        // Pre-allocate Vec with estimated JPEG size
        // RGB buffer is (width * height * 3) bytes
        // JPEG at quality 85 typically compresses to ~10-15% of raw size for density plots
        let raw_size = (width * height * 3) as usize;
        let estimated_jpeg_size = raw_size / 8; // Conservative estimate (~12.5% of raw)
        let mut encoded_data = Vec::with_capacity(estimated_jpeg_size);

        // JPEG encoding is faster and produces smaller files for density plots
        // Quality 85 provides good visual quality with ~2x smaller file size vs PNG
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded_data, 85);
        encoder
            .encode(
                img.as_raw(),
                *width,
                *height,
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| anyhow::anyhow!("failed to JPEG encode plot: {e}"))?;
        eprintln!("    └─ JPEG encoding: {:?}", encode_start.elapsed());

        // Return the JPEG-encoded bytes directly
        Ok((
            encoded_data,
            *margin,
            *x_label_area_size,
            *y_label_area_size,
        ))
    })
}

/// Streaming version of draw_plot that emits progress events during pixel rendering
/// Streams pixel chunks during buffer writing phase for progressive rendering
pub fn draw_plot_streaming(
    pixels: Vec<RawPixelData>,
    options: &PlotOptions,
    channel: Option<&tauri::ipc::Channel<crate::commands::PlotProgressEvent>>,
) -> anyhow::Result<PlotBytes> {
    draw_plot_streaming_with_chunk_size(pixels, options, channel, 1000)
}

pub fn draw_plot_streaming_with_chunk_size(
    pixels: Vec<RawPixelData>,
    options: &PlotOptions,
    channel: Option<&tauri::ipc::Channel<crate::commands::PlotProgressEvent>>,
    chunk_size: usize,
) -> anyhow::Result<PlotBytes> {
    crate::plotting::executor::with_render_lock(|| {
        let PlotOptions {
            title: _,
            width,
            height,
            colormap: _,
            plot_range_x,
            plot_range_y,
            x_label_transform,
            y_label_transform,
            x_axis_label,
            y_axis_label,
            margin,
            x_label_area_size,
            y_label_area_size,
        } = options;

        let setup_start = std::time::Instant::now();
        // Use RGB buffer (3 bytes per pixel) since we'll encode to JPEG which doesn't support alpha
        let mut pixel_buffer = vec![255; (width * height * 3) as usize];

        let (plot_x_range, plot_y_range, x_spec, y_spec) = {
            // let backend = BitMapBackend::new("./test.png", (width, height));
            let backend = BitMapBackend::with_buffer(&mut pixel_buffer, (*width, *height));
            let root = backend.into_drawing_area();
            root.fill(&WHITE)
                .map_err(|e| anyhow::anyhow!("failed to fill plot background: {e}"))?;

            // Create appropriate ranges based on transform types
            let (x_spec, y_spec) = create_axis_specs(
                plot_range_x,
                plot_range_y,
                x_label_transform,
                y_label_transform,
            )?;

            let mut chart = ChartBuilder::on(&root)
                .margin(*margin)
                .x_label_area_size(*x_label_area_size)
                .y_label_area_size(*y_label_area_size)
                .build_cartesian_2d(x_spec.start..x_spec.end, y_spec.start..y_spec.end)?;

            // Clone transforms to avoid lifetime issues with closures
            let x_transform_clone = x_label_transform.clone();
            let y_transform_clone = y_label_transform.clone();

            // Create owned closures for formatters
            let x_formatter = move |x: &f32| -> String { x_transform_clone.format(x) };
            let y_formatter = move |y: &f32| -> String { y_transform_clone.format(y) };

            let mut mesh = chart.configure_mesh();
            mesh.x_max_light_lines(4)
                .y_max_light_lines(4)
                .x_labels(10)
                .y_labels(10)
                .x_label_formatter(&x_formatter)
                .y_label_formatter(&y_formatter);

            // Add axis labels if provided
            if let Some(x_label) = x_axis_label {
                mesh.x_desc(x_label);
            }
            if let Some(y_label) = y_axis_label {
                mesh.y_desc(y_label);
            }

            let mesh_start = std::time::Instant::now();
            mesh.draw()
                .map_err(|e| anyhow::anyhow!("failed to draw plot mesh: {e}"))?;
            eprintln!("    ├─ Mesh drawing: {:?}", mesh_start.elapsed());

            // Get the plotting area bounds (we'll use these after Plotters releases the buffer)
            let plotting_area = chart.plotting_area();
            let (plot_x_range, plot_y_range) = plotting_area.get_pixel_range();

            root.present()
                .map_err(|e| anyhow::anyhow!("failed to present plotters buffer: {e}"))?;

            (plot_x_range, plot_y_range, x_spec, y_spec)
        }; // End Plotters scope - pixel_buffer is now released and we can write to it

        // DIRECT PIXEL BUFFER WRITING - 10-50x faster than Plotters series rendering
        // Now that Plotters has released pixel_buffer, we can write directly
        let series_start = std::time::Instant::now();

        let plot_x_start = plot_x_range.start as f32;
        let plot_y_start = plot_y_range.start as f32;
        let plot_width = (plot_x_range.end - plot_x_range.start) as f32;
        let plot_height = (plot_y_range.end - plot_y_range.start) as f32;

        // Calculate scale factors from data coordinates to screen pixels
        let data_width = x_spec.end - x_spec.start;
        let data_height = y_spec.end - y_spec.start;

        // Stream pixel chunks during rendering using configurable chunk size
        let mut pixel_count = 0;
        let total_pixels = pixels.len();

        // Write each pixel directly to the buffer
        for pixel in &pixels {
            let data_x = pixel.x;
            let data_y = pixel.y;

            // Transform data coordinates to screen pixel coordinates
            let rel_x = (data_x - x_spec.start) / data_width;
            let rel_y = (y_spec.end - data_y) / data_height; // Flip Y (screen coords go down)

            let screen_x = (plot_x_start + rel_x * plot_width) as i32;
            let screen_y = (plot_y_start + rel_y * plot_height) as i32;

            // Bounds check
            if screen_x >= plot_x_range.start
                && screen_x < plot_x_range.end
                && screen_y >= plot_y_range.start
                && screen_y < plot_y_range.end
            {
                let px = screen_x as u32;
                let py = screen_y as u32;

                // Write to pixel buffer (RGB format - 3 bytes per pixel)
                let idx = ((py * *width + px) * 3) as usize;

                if idx + 2 < pixel_buffer.len() {
                    pixel_buffer[idx] = pixel.r;
                    pixel_buffer[idx + 1] = pixel.g;
                    pixel_buffer[idx + 2] = pixel.b;
                }
            }

            pixel_count += 1;

            // Emit progress every chunk_size pixels
            if pixel_count % chunk_size == 0 || pixel_count == total_pixels {
                let percent = (pixel_count as f32 / total_pixels as f32) * 100.0;

                // Create a small sample of pixels for this chunk (for visualization)
                let chunk_start = (pixel_count - chunk_size.min(pixel_count)).max(0);
                let chunk_end = pixel_count;
                let chunk_pixels: Vec<crate::plotting::density::RawPixelData> = pixels
                    .iter()
                    .skip(chunk_start)
                    .take(chunk_end - chunk_start)
                    .map(|p| crate::plotting::density::RawPixelData {
                        x: p.x,
                        y: p.y,
                        r: p.r,
                        g: p.g,
                        b: p.b,
                    })
                    .collect();

                if let Some(ch) = channel {
                    if let Err(e) = ch.send(crate::commands::PlotProgressEvent::Progress {
                        pixels: chunk_pixels,
                        percent,
                    }) {
                        eprintln!("⚠️ Failed to send progress event: {}", e);
                    }
                }
            }
        }

        eprintln!(
            "    ├─ Direct pixel writing: {:?} ({} pixels)",
            series_start.elapsed(),
            pixels.len()
        );
        eprintln!("    ├─ Total plotting: {:?}", setup_start.elapsed());

        let img_start = std::time::Instant::now();
        let img: RgbImage = image::ImageBuffer::from_vec(*width, *height, pixel_buffer)
            .ok_or_else(|| anyhow::anyhow!("plot image buffer had unexpected size"))?;
        eprintln!("    ├─ Image buffer conversion: {:?}", img_start.elapsed());

        let encode_start = std::time::Instant::now();

        // Pre-allocate Vec with estimated JPEG size
        // RGB buffer is (width * height * 3) bytes
        // JPEG at quality 85 typically compresses to ~10-15% of raw size for density plots
        let raw_size = (width * height * 3) as usize;
        let estimated_jpeg_size = raw_size / 8; // Conservative estimate (~12.5% of raw)
        let mut encoded_data = Vec::with_capacity(estimated_jpeg_size);

        // JPEG encoding is faster and produces smaller files for density plots
        // Quality 85 provides good visual quality with ~2x smaller file size vs PNG
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded_data, 85);
        encoder
            .encode(
                img.as_raw(),
                *width,
                *height,
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| anyhow::anyhow!("failed to JPEG encode plot: {e}"))?;
        eprintln!("    └─ JPEG encoding: {:?}", encode_start.elapsed());

        // Return the JPEG-encoded bytes directly
        Ok(encoded_data)
    })
}

// Create appropriate axis specifications with nice bounds and labels
pub fn create_axis_specs(
    plot_range_x: &RangeInclusive<f32>,
    plot_range_y: &RangeInclusive<f32>,
    x_transform: &TransformType,
    y_transform: &TransformType,
) -> anyhow::Result<(Range<f32>, Range<f32>)> {
    // For linear scales, use nice number bounds
    // For arcsinh, ensure we use proper transformed bounds
    let x_spec = match x_transform {
        TransformType::Linear => {
            let min = plot_range_x.start();
            let max = plot_range_x.end();
            let (nice_min, nice_max) = nice_bounds(*min, *max);
            nice_min..nice_max
        }
        TransformType::Arcsinh { cofactor: _ } => {
            // Keep the transformed range but we'll format nicely in the formatter
            *plot_range_x.start()..*plot_range_x.end()
        }
    };

    let y_spec = match y_transform {
        TransformType::Linear => {
            let min = plot_range_y.start();
            let max = plot_range_y.end();
            let (nice_min, nice_max) = nice_bounds(*min, *max);
            nice_min..nice_max
        }
        TransformType::Arcsinh { cofactor: _ } => {
            // Keep the transformed range but we'll format nicely in the formatter
            *plot_range_y.start()..*plot_range_y.end()
        }
    };

    Ok((x_spec.into(), y_spec.into()))
}

pub fn get_percentile_bounds(
    values: &[f32],
    percentile_low: f32,
    percentile_high: f32,
) -> PlotRange {
    let mut sorted_values = values.to_vec();
    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let low_index = (percentile_low * sorted_values.len() as f32).floor() as usize;
    let high_index = (percentile_high * sorted_values.len() as f32).ceil() as usize;

    // Ensure indices are within bounds
    let low_index = low_index.clamp(0, sorted_values.len() - 1);
    let high_index = high_index.clamp(0, sorted_values.len() - 1);

    let low_value = sorted_values[low_index];
    let high_value = sorted_values[high_index];

    // Round to nice numbers
    let min_bound = nearest_nice_number(low_value, RoundingDirection::Down);
    let max_bound = nearest_nice_number(high_value, RoundingDirection::Up);

    min_bound..=max_bound
}

fn nice_bounds(min: f32, max: f32) -> (f32, f32) {
    if min.is_infinite() || max.is_infinite() || min.is_nan() || max.is_nan() {
        return (0.0, 1.0); // Fallback for invalid ranges
    }

    let range = max - min;
    if range == 0.0 {
        return (min - 0.5, min + 0.5); // Handle single-point case
    }

    // Find nice step size
    let step_size = 10_f32.powf((range.log10()).floor());
    let nice_min = (min / step_size).floor() * step_size;
    let nice_max = (max / step_size).ceil() * step_size;

    (nice_min, nice_max)
}

enum RoundingDirection {
    Up,
    Down,
}

fn nearest_nice_number(value: f32, direction: RoundingDirection) -> f32 {
    // Handle edge cases
    if value == 0.0 {
        return 0.0;
    }

    let abs_value = value.abs();
    let exponent = abs_value.log10().floor() as i32;
    let factor = 10f32.powi(exponent);

    // Find nearest nice number based on direction
    let nice_value = match direction {
        RoundingDirection::Up => {
            let mantissa = (abs_value / factor).ceil();
            if mantissa <= 1.0 {
                1.0 * factor
            } else if mantissa <= 2.0 {
                2.0 * factor
            } else if mantissa <= 5.0 {
                5.0 * factor
            } else {
                10.0 * factor
            }
        }
        RoundingDirection::Down => {
            let mantissa = (abs_value / factor).floor();
            if mantissa >= 5.0 {
                5.0 * factor
            } else if mantissa >= 2.0 {
                2.0 * factor
            } else if mantissa >= 1.0 {
                1.0 * factor
            } else {
                0.5 * factor
            }
        }
    };

    // Preserve sign
    if value.is_sign_negative() {
        -nice_value
    } else {
        nice_value
    }
}
