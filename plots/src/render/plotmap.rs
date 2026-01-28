#[derive(Clone, Debug, PartialEq)] // Serialize so you can send it to Dioxus
pub struct PlotMapper {
    pub view_width: f32,
    pub view_height: f32,
    pub plot_left: f32,
    pub plot_top: f32,
    pub plot_width: f32,
    pub plot_height: f32,
    pub x_data_min: f32,
    pub x_data_max: f32,
    pub y_data_min: f32,
    pub y_data_max: f32,
}

impl PlotMapper {
    pub fn pixel_to_data(&self, click_x: f32, click_y: f32) -> Option<(f32, f32)> {
        // 1. Convert absolute click to relative plot-area click
        let rel_x = (click_x - self.plot_left) / self.plot_width;
        let rel_y = (click_y - self.plot_top) / self.plot_height;

        // 2. Check if the click was actually inside the plot (not on the margins)
        if rel_x < 0.0 || rel_x > 1.0 || rel_y < 0.0 || rel_y > 1.0 {
            println!(
                "Click outside plot area: rel_x = {}, rel_y = {}",
                rel_x, rel_y
            );
            return None;
        }

        // 3. Map relative 0..1 to data range
        // Note: Y is flipped because screen Y increases downwards
        let data_x = self.x_data_min + rel_x * (self.x_data_max - self.x_data_min);
        let data_y = self.y_data_max - rel_y * (self.y_data_max - self.y_data_min);

        Some((data_x, data_y))
    }

    pub fn data_to_pixel(&self, data_x: f32, data_y: f32) -> (f32, f32) {
        // 1. Map data values back to relative 0..1 range
        let rel_x = (data_x - self.x_data_min) / (self.x_data_max - self.x_data_min);
        // Note: Y is flipped again to account for screen coordinates
        let rel_y = (self.y_data_max - data_y) / (self.y_data_max - self.y_data_min);

        // 2. Map relative 0..1 to absolute plot-area pixels
        let click_x = self.plot_left + (rel_x * self.plot_width);
        let click_y = self.plot_top + (rel_y * self.plot_height);

        (click_x, click_y)
    }

    /// Transforms a batch of raw data coordinates into screen pixel coordinates.
    pub fn map_data_to_pixels(&self, data_points: &[(f32, f32)]) -> Vec<(f32, f32)> {
        data_points
            .iter()
            .map(|&(x, y)| self.data_to_pixel(x, y)) // Clean and reused!
            .collect()
    }

    /// Transforms a batch of screen pixels into raw data coordinates,
    /// skipping those outside the plot area.
    pub fn map_pixels_to_data(&self, pixel_points: &[(f32, f32)]) -> Vec<(f32, f32)> {
        pixel_points
            .iter()
            .filter_map(|&(px, py)| self.pixel_to_data(px, py)) // Reuses your bounds checking too!
            .collect()
    }
}

#[derive(Clone, PartialEq)]
pub struct PlotData {
    pub plot_map: PlotMapper,
    pub plot_bytes: crate::PlotBytes,
}
