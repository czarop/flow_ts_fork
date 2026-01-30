#[derive(Clone, PartialEq)]
pub struct PlotHelper {
    pub x_data_min: f32,
    pub x_data_max: f32,
    pub y_data_min: f32,
    pub y_data_max: f32,
}

#[derive(Clone, PartialEq)]
pub struct PlotData {
    pub plot_helper: PlotHelper,
    pub plot_bytes: crate::PlotBytes,
}
