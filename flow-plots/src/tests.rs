#[test]
fn test_scaling_to_plot_pixels() {
    let mut options = ColoredDensityPlotOptions::default();

    // Handles normal (square) scaling
    assert_eq!(
        Some((80, 160)),
        scale_to_plot_pixels(&40_000f32, &80_000f32, &options)
    );

    // Handles non-square aspect ratios
    options.width = 400;
    options.height = 800;
    assert_eq!(
        Some((80, 320)),
        scale_to_plot_pixels(&40_000f32, &80_000f32, &options)
    );

    // Handles negative bounds
    options = ColoredDensityPlotOptions::default();
    options.plot_min_x = -200_000f32;
    assert_eq!(
        Some((240, 160)),
        scale_to_plot_pixels(&40_000f32, &80_000f32, &options)
    );

    // Returns None if the point is outside the plot bounds (above max)
    options = ColoredDensityPlotOptions::default();
    assert_eq!(
        None,
        scale_to_plot_pixels(&999_999f32, &80_000f32, &options)
    );
}

fn scale_to_plot_pixels(
    x: &f32,
    y: &f32,
    options: &ColoredDensityPlotOptions,
) -> Option<(usize, usize)> {
    let ColoredDensityPlotOptions {
        width,
        height,
        plot_min_x,
        plot_max_x,
        plot_min_y,
        plot_max_y,
        ..
    } = options;

    if *x > *plot_max_x || *x < *plot_min_x || *y > *plot_max_y || *y < *plot_min_y {
        return None;
    }

    let scale_x = *width as f32 / (plot_max_x - plot_min_x);
    let scale_y = *height as f32 / (plot_max_y - plot_min_y);

    let pixel_x = ((x - plot_min_x) * scale_x).floor() as usize;
    let pixel_y = ((y - plot_min_y) * scale_y).floor() as usize;

    Some((pixel_x, pixel_y))
}
fn scale_from_plot_pixels(
    x: &usize,
    y: &usize,
    options: &ColoredDensityPlotOptions,
) -> Option<(f32, f32)> {
    let ColoredDensityPlotOptions {
        width,
        height,
        plot_min_x,
        plot_max_x,
        plot_min_y,
        plot_max_y,
        ..
    } = options;

    let scale_x = (plot_max_x - plot_min_x) / *width as f32;
    let scale_y = (plot_max_y - plot_min_y) / *height as f32;

    let x = *x as f32 * scale_x + plot_min_x;
    let y = *y as f32 * scale_y + plot_min_y;

    Some((x, y))
}
