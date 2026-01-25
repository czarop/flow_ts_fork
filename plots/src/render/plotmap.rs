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
            return None;
        }

        // 3. Map relative 0..1 to data range
        // Note: Y is flipped because screen Y increases downwards
        let data_x = self.x_data_min + rel_x * (self.x_data_max - self.x_data_min);
        let data_y = self.y_data_max - rel_y * (self.y_data_max - self.y_data_min);

        Some((data_x, data_y))
    }
}

#[derive(Clone, PartialEq)]
pub struct PlotData {
    pub plot_map: PlotMapper,
    pub plot_bytes: crate::PlotBytes,
}
