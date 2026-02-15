//! Signal heatmap and normalized spectral signature plot generation
//!
//! These functions generate visualizations for flow cytometry spectral signatures.
//! They return plot bytes rather than writing files directly, allowing callers
//! to handle file I/O as needed.

use anyhow::{Context, Result};
use flow_fcs::{Fcs, TransformType, Transformable};
use std::collections::HashMap;

use crate::colormap::ColorMaps;
use crate::plots::Plot;

/// Helper function to calculate geometric mean of positive values
fn calculate_geometric_mean(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }

    // Filter to positive values only
    let positive_values: Vec<f32> = values.iter().filter(|&&v| v > 0.0).copied().collect();

    if positive_values.is_empty() {
        return None;
    }

    // Calculate geometric mean: exp(mean(ln(values)))
    let log_sum: f64 = positive_values.iter().map(|&v| (v as f64).ln()).sum();
    let n = positive_values.len() as f64;
    Some((log_sum / n).exp() as f32)
}

/// Helper function to calculate median
fn _calculate_median(values: &[f32]) -> f32 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Sort channels by laser type, then wavelength
/// Order: UV > V > B > YG > R, then by wavelength within each group
fn sort_channels_by_laser_and_wavelength(channels: &mut [String]) {
    // Extract laser type and wavelength from channel name
    // Format examples: "UV379-A", "V660-A", "B510-A", "YG585-A", "UV446-A"
    fn get_laser_order(channel: &str) -> (u8, u32) {
        let upper = channel.to_uppercase();

        // Determine laser type order: UV=1, V=2, B=3, YG=4, R=5, others=99
        let laser_order = if upper.starts_with("UV") {
            1
        } else if upper.starts_with("V") && !upper.starts_with("UV") {
            2
        } else if upper.starts_with("B") {
            3
        } else if upper.starts_with("YG") {
            4
        } else if upper.starts_with("R") {
            5
        } else {
            99
        };

        // Extract wavelength number (digits after laser prefix)
        let wavelength = if upper.starts_with("UV") {
            // Extract number after "UV"
            upper[2..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(9999)
        } else if upper.starts_with("YG") {
            // Extract number after "YG"
            upper[2..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(9999)
        } else if upper.starts_with("V") || upper.starts_with("B") || upper.starts_with("R") {
            // Extract number after single letter
            upper[1..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(9999)
        } else {
            9999
        };

        (laser_order, wavelength)
    }

    channels.sort_by(|a, b| {
        let (order_a, wave_a) = get_laser_order(a);
        let (order_b, wave_b) = get_laser_order(b);

        // First sort by laser type
        match order_a.cmp(&order_b) {
            std::cmp::Ordering::Equal => {
                // Then by wavelength
                wave_a.cmp(&wave_b)
            }
            other => other,
        }
    });
}

/// Generate a heatmap visualization of signal intensity across channels
///
/// Shows a density distribution of events across intensity levels for each channel.
/// Each channel is a vertical column where color represents the density of events
/// at each intensity level (y-axis). This creates a 1D vertical distribution showing
/// where events cluster in intensity space.
///
/// Returns JPEG-encoded bytes rather than writing to a file.
pub fn generate_signal_heatmap(
    _signature_name: &str,
    detector_names: &[String],
    raw_signals: &HashMap<String, f32>,
    fcs_file_path: Option<&std::path::Path>,
    colormap: Option<ColorMaps>,
    unstained_medians: Option<&HashMap<String, f32>>,
    positive_medians: Option<&HashMap<String, f32>>,
    positive_geometric_means: Option<&HashMap<String, f32>>,
) -> Result<Vec<u8>> {
    // Use provided colormap or default to Spectral
    let colormap = colormap.unwrap_or(ColorMaps::Spectral);
    use image::RgbImage;
    use plotters::prelude::*;

    // Sort channels by laser type, then wavelength (UV > V > B > YG > R)
    let mut sorted_detector_names = detector_names.to_vec();
    sort_channels_by_laser_and_wavelength(&mut sorted_detector_names);

    let width = 1600u32;
    let height = 600u32;
    let margin = 80u32;
    let x_label_area_size = 100u32;
    let y_label_area_size = 80u32;

    // Number of intensity bins for y-axis
    let n_y_bins = 200;

    // Arcsinh cofactor for transformation (typical value for modern instruments)
    let arcsinh_cofactor = 200.0f32;
    let arcsinh_transform = TransformType::Arcsinh {
        cofactor: arcsinh_cofactor,
    };

    // Read event data from FCS file if provided, otherwise use synthetic distribution
    let channel_densities: Vec<Vec<f32>>;
    let y_min: f32;
    let y_max: f32;
    let max_density: f32;

    if let Some(fcs_path) = fcs_file_path {
        // Read actual event data from FCS file
        let fcs = Fcs::open(fcs_path.to_str().ok_or_else(|| {
            anyhow::anyhow!(
                "FCS file path contains invalid UTF-8: {}",
                fcs_path.display()
            )
        })?)
        .with_context(|| format!("Failed to read FCS file: {}", fcs_path.display()))?;

        // Determine y-axis range from actual data AFTER arcsinh transformation
        let mut global_min = f32::MAX;
        let mut global_max = f32::MIN;
        let mut global_max_raw = f32::MIN; // Track raw max for capping

        // First pass: find min/max across all channels after arcsinh transformation
        for det_name in &sorted_detector_names {
            if let Ok(series) = fcs.data_frame.column(det_name) {
                if let Ok(f32_vals) = series.f32() {
                    for val_opt in f32_vals.iter() {
                        if let Some(val) = val_opt {
                            let transformed_val = arcsinh_transform.transform(&val);
                            global_min = global_min.min(transformed_val);
                            global_max = global_max.max(transformed_val);
                            global_max_raw = global_max_raw.max(val); // Track raw max
                        }
                    }
                }
            }
        }

        // Cap y-axis at ~5e6 in original signal space (unless data exceeds it)
        let max_signal_cap = 5_000_000.0f32;
        let cap_transformed = arcsinh_transform.transform(&max_signal_cap);

        // Use the larger of: actual max or cap (both in transformed space)
        let effective_max = if global_max_raw > max_signal_cap {
            // Data exceeds cap, use actual transformed max
            global_max
        } else {
            // Use cap in transformed space
            cap_transformed
        };

        y_min = 0.0f32.max(global_min * 0.9); // Slight margin below
        y_max = effective_max * 1.1; // 10% margin above
        let y_bin_size = (y_max - y_min) / n_y_bins as f32;

        eprintln!(
            "Y-axis range: [{:.3}, {:.3}], bin_size={:.6}, n_bins={}",
            y_min, y_max, y_bin_size, n_y_bins
        );

        // Create density bins for each channel (in sorted order)
        let mut densities: Vec<Vec<f32>> = Vec::new();

        for det_name in &sorted_detector_names {
            let mut density = vec![0.0f32; n_y_bins];

            if let Ok(series) = fcs.data_frame.column(det_name) {
                if let Ok(f32_vals) = series.f32() {
                    // Bin events by intensity AFTER arcsinh transformation
                    // Only count events that fall within the y-axis range
                    let mut event_count = 0u32;
                    let mut bins_used = std::collections::HashSet::new();

                    for val_opt in f32_vals.iter() {
                        if let Some(val) = val_opt {
                            let transformed_val = arcsinh_transform.transform(&val);
                            // Only bin if within the valid range
                            if transformed_val >= y_min && transformed_val <= y_max {
                                let bin_idx = (((transformed_val - y_min) / y_bin_size) as usize)
                                    .min(n_y_bins - 1);
                                // Ensure bin index is valid
                                if bin_idx < n_y_bins {
                                    density[bin_idx] += 1.0;
                                    bins_used.insert(bin_idx);
                                    event_count += 1;
                                }
                            }
                        }
                    }

                    // Debug: Print binning statistics
                    eprintln!(
                        "Channel {}: {} events binned into {} unique bins (out of {} total bins)",
                        det_name,
                        event_count,
                        bins_used.len(),
                        n_y_bins
                    );

                    // Ensure bins are truly zero if no events were found
                    if event_count == 0 {
                        // All bins should remain 0.0 (already initialized)
                        eprintln!("  Warning: No events found for channel {}", det_name);
                    }
                }
            }

            densities.push(density);
        }

        // Apply logarithmic transformation to density values (same as regular density plots)
        // Adding 1.0 before log to avoid log(0) = -Infinity
        let mut max_log_density = 0.0f32;
        for density in &mut densities {
            for count in density.iter_mut() {
                if *count > 0.0 {
                    *count = (*count + 1.0).log10();
                    max_log_density = max_log_density.max(*count);
                }
            }
        }

        // Ensure max is at least 1.0 (for log10(1.0 + 1.0) = log10(2.0) ≈ 0.301)
        max_log_density = max_log_density.max(1.0);

        channel_densities = densities;
        max_density = max_log_density;
    } else {
        // Fallback: use synthetic distribution based on raw_signals
        let max_signal = raw_signals.values().fold(0.0f32, |a, &b| a.max(b)).max(1.0);
        let max_signal_cap = 5_000_000.0f32;

        // Cap at 5e6 unless data exceeds it
        let capped_signal = if max_signal > max_signal_cap {
            max_signal
        } else {
            max_signal_cap
        };

        // Transform the capped signal
        let capped_transformed = arcsinh_transform.transform(&capped_signal);

        y_min = 0.0f32;
        y_max = capped_transformed * 1.1;
        let y_bin_size = (y_max - y_min) / n_y_bins as f32;

        let mut densities: Vec<Vec<f32>> = Vec::new();

        for det_name in &sorted_detector_names {
            let signal = raw_signals.get(det_name).copied().unwrap_or(0.0);
            let mut density = vec![0.0f32; n_y_bins];

            if signal > 0.0 {
                let std_dev = signal * 0.1;
                let mean = signal;

                for bin_idx in 0..n_y_bins {
                    let y_center = y_min + (bin_idx as f32 + 0.5) * y_bin_size;
                    let diff = (y_center - mean) / std_dev;
                    let density_value = (-0.5 * diff * diff).exp();
                    density[bin_idx] = density_value;
                }
            } else {
                let baseline_signal = 100.0;
                let std_dev = baseline_signal * 0.1;

                for bin_idx in 0..n_y_bins {
                    let y_center = y_min + (bin_idx as f32 + 0.5) * y_bin_size;
                    if y_center < baseline_signal * 2.0 {
                        let diff = (y_center - baseline_signal) / std_dev;
                        let density_value = (-0.5 * diff * diff).exp();
                        density[bin_idx] = density_value;
                    }
                }
            }

            densities.push(density);
        }

        // Apply logarithmic transformation to density values (same as regular density plots)
        // Adding 1.0 before log to avoid log(0) = -Infinity
        let mut max_log_density = 0.0f32;
        for density in &mut densities {
            for count in density.iter_mut() {
                if *count > 0.0 {
                    *count = (*count + 1.0).log10();
                    max_log_density = max_log_density.max(*count);
                }
            }
        }

        // Ensure max is at least 1.0 (for log10(1.0 + 1.0) = log10(2.0) ≈ 0.301)
        max_log_density = max_log_density.max(1.0);

        channel_densities = densities;
        max_density = max_log_density;
    }

    let y_bin_size = (y_max - y_min) / n_y_bins as f32;

    let mut pixel_buffer = vec![255; (width * height * 3) as usize];

    {
        let backend = BitMapBackend::with_buffer(&mut pixel_buffer, (width, height));
        let root = backend.into_drawing_area();
        root.fill(&WHITE)
            .map_err(|e| anyhow::anyhow!("failed to fill plot background: {e}"))?;

        let x_min = -0.5f32;
        let x_max = sorted_detector_names.len() as f32 - 0.5;

        let mut chart = ChartBuilder::on(&root)
            .margin(margin)
            .x_label_area_size(x_label_area_size)
            .y_label_area_size(y_label_area_size)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .map_err(|e| anyhow::anyhow!("failed to build chart: {e}"))?;

        // Configure mesh with channel names (sorted)
        let channel_names_clone = sorted_detector_names.clone();
        let formatter = move |x: &f32| -> String {
            let idx = x.round() as usize;
            if idx < channel_names_clone.len() {
                channel_names_clone[idx].clone()
            } else {
                format!("{:.0}", x)
            }
        };

        chart
            .configure_mesh()
            .x_max_light_lines(4)
            .y_max_light_lines(4)
            .x_labels(sorted_detector_names.len().min(20))
            .y_labels(10)
            .x_label_formatter(&formatter)
            .x_desc("Channel")
            .y_desc("Signal Intensity (arcsinh transformed)")
            .draw()
            .map_err(|e| anyhow::anyhow!("failed to draw mesh: {e}"))?;

        // Draw density heatmap for each channel
        let column_width = 0.8;

        for (idx, density) in channel_densities.iter().enumerate() {
            let x_center = idx as f32;
            let x_start = x_center - column_width / 2.0;
            let x_end = x_center + column_width / 2.0;

            // Draw each y-bin as a rectangle with color based on density
            // Only draw bins that have events to show white space where no events exist
            let mut rectangles_to_draw = Vec::new();

            for (bin_idx, &density_value) in density.iter().enumerate() {
                // Strictly skip empty bins - this creates white space
                if density_value <= 0.0 {
                    continue;
                }

                let y_bottom = y_min + bin_idx as f32 * y_bin_size;
                let y_top = y_min + (bin_idx + 1) as f32 * y_bin_size;

                // Density values are already log-transformed (log10)
                // Normalize to 0-1 range for colormap
                let normalized_log_density = if max_density > 0.0 {
                    (density_value / max_density).min(1.0).max(0.0)
                } else {
                    0.0
                };

                // Invert the scale so high density maps to red (1.0) and low density maps to blue (0.0)
                let inverted_density = 1.0 - normalized_log_density;

                // Map density to color using the provided colormap
                let color = colormap.map(inverted_density);

                rectangles_to_draw.push(Rectangle::new(
                    [(x_start, y_bottom), (x_end, y_top)],
                    color.filled(),
                ));
            }

            // Draw all rectangles for this channel at once
            if !rectangles_to_draw.is_empty() {
                chart
                    .draw_series(rectangles_to_draw.into_iter())
                    .map_err(|e| anyhow::anyhow!("failed to draw heatmap column: {e}"))?;
            }
        }

        // Draw unstained medians overlay line if provided (dashed, semi-opaque grey)
        if let Some(unstained) = unstained_medians {
            // Create line series: (channel_index, transformed_unstained_median)
            let unstained_overlay_points: Vec<(f32, f32)> = sorted_detector_names
                .iter()
                .enumerate()
                .filter_map(|(idx, det_name)| {
                    unstained.get(det_name).map(|&median| {
                        // Transform unstained median using same arcsinh transform
                        let transformed_median = arcsinh_transform.transform(&median);
                        (idx as f32, transformed_median)
                    })
                })
                .collect();

            if !unstained_overlay_points.is_empty() {
                use plotters::prelude::PathElement;
                // Semi-opaque grey color (RGB: 128, 128, 128 with alpha ~0.7)
                // Note: plotters RGBColor doesn't support alpha directly, so we use a lighter grey
                let unstained_color = plotters::style::RGBColor(180, 180, 180); // Light grey

                // Draw continuous dashed line across all segments
                // Break each segment into dash-gap pattern
                if unstained_overlay_points.len() > 1 {
                    const DASH_LENGTH: f32 = 0.15; // Length of each dash (in data space)
                    const GAP_LENGTH: f32 = 0.1; // Length of each gap (in data space)

                    for i in 0..unstained_overlay_points.len() - 1 {
                        let start = unstained_overlay_points[i];
                        let end = unstained_overlay_points[i + 1];

                        // Calculate segment length and direction
                        let dx = end.0 - start.0;
                        let dy = end.1 - start.1;
                        let segment_length = (dx * dx + dy * dy).sqrt();

                        if segment_length > 0.0 {
                            // Unit vector along the segment
                            let ux = dx / segment_length;
                            let uy = dy / segment_length;

                            // Draw dashes along the segment
                            let mut current_pos = 0.0;
                            while current_pos < segment_length {
                                let dash_start_x = start.0 + ux * current_pos;
                                let dash_start_y = start.1 + uy * current_pos;

                                let dash_end_pos = (current_pos + DASH_LENGTH).min(segment_length);
                                let dash_end_x = start.0 + ux * dash_end_pos;
                                let dash_end_y = start.1 + uy * dash_end_pos;

                                // Draw this dash
                                chart
                                    .draw_series(std::iter::once(PathElement::new(
                                        vec![
                                            (dash_start_x, dash_start_y),
                                            (dash_end_x, dash_end_y),
                                        ],
                                        unstained_color.stroke_width(2),
                                    )))
                                    .map_err(|e| {
                                        anyhow::anyhow!(
                                            "failed to draw unstained overlay dash: {e}"
                                        )
                                    })?;

                                // Move to next dash position (skip gap)
                                current_pos += DASH_LENGTH + GAP_LENGTH;
                            }
                        }
                    }
                }
            }
        }

        // Draw positive medians and geometric means overlay lines with markers
        // Show both if available, with different markers and colors
        let mut legend_items = Vec::new();

        // Draw geometric mean overlay (preferred, better for log-normal distributions)
        if let Some(positive_geo) = positive_geometric_means {
            // Create line series: (channel_index, transformed_geometric_mean)
            let geo_overlay_points: Vec<(f32, f32)> = sorted_detector_names
                .iter()
                .enumerate()
                .filter_map(|(idx, det_name)| {
                    positive_geo
                        .get(det_name)
                        .map(|&transformed_value| (idx as f32, transformed_value))
                })
                .collect();

            if !geo_overlay_points.is_empty() {
                use plotters::prelude::LineSeries;
                // Use orange for geometric mean, semi-transparent
                let geo_color = plotters::style::RGBAColor(255, 165, 0, 0.7); // Orange, semi-transparent
                let geo_color_opaque = plotters::style::RGBColor(255, 165, 0); // Opaque for legend

                // Draw solid line connecting all points
                chart
                    .draw_series(LineSeries::new(
                        geo_overlay_points.iter().copied(),
                        geo_color.stroke_width(2),
                    ))
                    .map_err(|e| {
                        anyhow::anyhow!("failed to draw geometric mean overlay line: {e}")
                    })?;

                // Draw square markers at each channel
                chart
                    .draw_series(geo_overlay_points.iter().map(|(x, y)| {
                        Rectangle::new(
                            [(x - 0.08, y - 0.12), (x + 0.08, y + 0.12)],
                            geo_color.filled(),
                        )
                    }))
                    .map_err(|e| anyhow::anyhow!("failed to draw geometric mean markers: {e}"))?;

                legend_items.push(("Geometric Mean", geo_color_opaque));
            }
        }

        // Draw median overlay (if geometric mean not available, or show both)
        if let Some(positive_med) = positive_medians {
            // Create line series: (channel_index, transformed_median)
            let median_overlay_points: Vec<(f32, f32)> = sorted_detector_names
                .iter()
                .enumerate()
                .filter_map(|(idx, det_name)| {
                    positive_med
                        .get(det_name)
                        .map(|&transformed_value| (idx as f32, transformed_value))
                })
                .collect();

            if !median_overlay_points.is_empty() {
                use plotters::prelude::PathElement;
                // Use blue for median, semi-transparent
                let median_color = plotters::style::RGBAColor(0, 0, 255, 0.7); // Blue, semi-transparent
                let median_color_opaque = plotters::style::RGBColor(0, 0, 255); // Opaque for legend

                // Draw dashed line connecting all points (to distinguish from geometric mean)
                if median_overlay_points.len() > 1 {
                    const DASH_LENGTH: f32 = 0.15;
                    const GAP_LENGTH: f32 = 0.1;

                    for i in 0..median_overlay_points.len() - 1 {
                        let start = median_overlay_points[i];
                        let end = median_overlay_points[i + 1];

                        let dx = end.0 - start.0;
                        let dy = end.1 - start.1;
                        let segment_length = (dx * dx + dy * dy).sqrt();

                        if segment_length > 0.0 {
                            let ux = dx / segment_length;
                            let uy = dy / segment_length;

                            let mut current_pos = 0.0;
                            while current_pos < segment_length {
                                let dash_start_x = start.0 + ux * current_pos;
                                let dash_start_y = start.1 + uy * current_pos;

                                let dash_end_pos = (current_pos + DASH_LENGTH).min(segment_length);
                                let dash_end_x = start.0 + ux * dash_end_pos;
                                let dash_end_y = start.1 + uy * dash_end_pos;

                                chart
                                    .draw_series(std::iter::once(PathElement::new(
                                        vec![
                                            (dash_start_x, dash_start_y),
                                            (dash_end_x, dash_end_y),
                                        ],
                                        median_color.stroke_width(2),
                                    )))
                                    .map_err(|e| {
                                        anyhow::anyhow!("failed to draw median overlay dash: {e}")
                                    })?;

                                current_pos += DASH_LENGTH + GAP_LENGTH;
                            }
                        }
                    }
                }

                // Draw circle markers at each channel
                chart
                    .draw_series(
                        median_overlay_points
                            .iter()
                            .map(|(x, y)| Circle::new((*x, *y), 4, median_color.filled())),
                    )
                    .map_err(|e| anyhow::anyhow!("failed to draw median markers: {e}"))?;

                legend_items.push(("Median", median_color_opaque));
            }
        }

        // Draw legend in top-right corner
        if !legend_items.is_empty() {
            let plotting_area = chart.plotting_area();
            let (_x_range, _y_range) = plotting_area.get_pixel_range();

            // Position legend in top-right corner (in data coordinates)
            let legend_x_data = sorted_detector_names.len() as f32
                - 0.5
                - (sorted_detector_names.len() as f32 * 0.15);
            let legend_y_start = y_max - (y_max - y_min) * 0.15;

            for (i, (label, color)) in legend_items.iter().enumerate() {
                let legend_y_data = legend_y_start - (i as f32 * (y_max - y_min) * 0.08);

                // Draw marker in data coordinates
                if *label == "Geometric Mean" {
                    // Square for geometric mean
                    chart
                        .draw_series(std::iter::once(Rectangle::new(
                            [
                                (legend_x_data - 0.08, legend_y_data - 0.12),
                                (legend_x_data + 0.08, legend_y_data + 0.12),
                            ],
                            color.filled(),
                        )))
                        .map_err(|e| anyhow::anyhow!("failed to draw legend marker: {e}"))?;
                } else {
                    // Circle for median
                    chart
                        .draw_series(std::iter::once(Circle::new(
                            (legend_x_data, legend_y_data),
                            4,
                            color.filled(),
                        )))
                        .map_err(|e| anyhow::anyhow!("failed to draw legend marker: {e}"))?;
                }

                // Draw label text
                let label_x = legend_x_data + 0.15;
                chart
                    .draw_series(std::iter::once(Text::new(
                        label.to_string(),
                        (label_x, legend_y_data),
                        ("sans-serif", 12).into_font().color(color),
                    )))
                    .map_err(|e| anyhow::anyhow!("failed to draw legend label: {e}"))?;
            }
        }
    }

    let img: RgbImage = image::ImageBuffer::from_vec(width, height, pixel_buffer)
        .ok_or_else(|| anyhow::anyhow!("plot image buffer had unexpected size"))?;

    let mut encoded_data = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded_data, 85);
    encoder
        .encode(img.as_raw(), width, height, image::ExtendedColorType::Rgb8)
        .map_err(|e| anyhow::anyhow!("failed to JPEG encode plot: {e}"))?;

    Ok(encoded_data)
}

/// Generate normalized spectral signature line plot
///
/// Shows the normalized signature (0-1 range) as a line plot connecting peaks across channels.
/// If detector_signals is empty, calculates normalized signature from FCS file.
///
/// Returns JPEG-encoded bytes rather than writing to a file.
pub fn generate_normalized_spectral_signature_plot(
    signature_name: &str,
    detector_names: &[String],
    detector_signals: &HashMap<String, f64>,
    fcs_file_path: Option<&std::path::Path>,
) -> Result<Vec<u8>> {
    // Sort channels by laser type, then wavelength (UV > V > B > YG > R)
    let mut sorted_detector_names = detector_names.to_vec();
    sort_channels_by_laser_and_wavelength(&mut sorted_detector_names);

    // Create normalized signature data: (channel_index, normalized_intensity)
    let spectrum_data: Vec<(usize, f64)> = if detector_signals.is_empty() && fcs_file_path.is_some()
    {
        // Calculate normalized signature from FCS data
        let fcs_path = fcs_file_path.ok_or_else(|| anyhow::anyhow!("FCS file path is None"))?;
        let fcs = Fcs::open(fcs_path.to_str().ok_or_else(|| {
            anyhow::anyhow!(
                "FCS file path contains invalid UTF-8: {}",
                fcs_path.display()
            )
        })?)
        .with_context(|| "Failed to read FCS file for normalized signature")?;

        let arcsinh_cofactor = 200.0f32;
        let arcsinh_transform = TransformType::Arcsinh {
            cofactor: arcsinh_cofactor,
        };

        // Calculate geometric means after arcsinh transformation (in sorted order)
        // Geometric mean is better for log-normal distributions common in flow cytometry
        let mut transformed_geometric_means = HashMap::new();
        for det_name in &sorted_detector_names {
            if let Ok(series) = fcs.data_frame.column(det_name) {
                if let Ok(f32_vals) = series.f32() {
                    let transformed_values: Vec<f32> = f32_vals
                        .iter()
                        .filter_map(|v| v.map(|x| arcsinh_transform.transform(&x)))
                        .collect();

                    if let Some(geo_mean) = calculate_geometric_mean(&transformed_values) {
                        transformed_geometric_means.insert(det_name.clone(), geo_mean);
                    }
                }
            }
        }

        // Normalize by max
        let max_signal = transformed_geometric_means
            .values()
            .fold(0.0f32, |a, &b| a.max(b));

        sorted_detector_names
            .iter()
            .enumerate()
            .map(|(idx, det_name)| {
                let normalized = if max_signal > 0.0 {
                    transformed_geometric_means
                        .get(det_name)
                        .copied()
                        .unwrap_or(0.0)
                        / max_signal
                } else {
                    0.0
                };
                (idx, normalized as f64)
            })
            .collect()
    } else {
        // Use provided signature (in sorted order)
        sorted_detector_names
            .iter()
            .enumerate()
            .map(|(idx, det_name)| {
                let normalized = detector_signals.get(det_name).copied().unwrap_or(0.0);
                (idx, normalized)
            })
            .collect()
    };

    // Pass channel names to the plot renderer (sorted)
    let channel_names = sorted_detector_names;

    let mut render_config = crate::render::RenderConfig::default();
    let plot = crate::plots::SpectralSignaturePlot::new();

    let base_opts = crate::options::BasePlotOptions::new()
        .width(1600u32)
        .height(600u32)
        .title(format!(
            "Normalized Spectral Signature - {}",
            signature_name
        ))
        .build()?;

    let options = crate::options::SpectralSignaturePlotOptions::new()
        .base(base_opts)
        .x_axis(Some(
            crate::options::AxisOptions::new()
                .label("Detector Channel".to_string())
                .build()?,
        ))
        .y_axis(Some(
            crate::options::AxisOptions::new()
                .label("Normalized Intensity (0.0 to 1.0)".to_string())
                .build()?,
        ))
        .line_color("#1f77b4".to_string())
        .line_width(2.5)
        .show_grid(true)
        .build()?;

    plot.render((spectrum_data, channel_names), &options, &mut render_config)
}
