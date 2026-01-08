use plotters::{
    prelude::*,
    style::colors::colormaps::{BlackWhite, Bone, MandelbrotHSL, ViridisRGB, VulcanoHSL},
};

pub enum ColorMaps {
    Viridis(ViridisRGB),
    Bone(Bone),
    Mandelbrot(MandelbrotHSL),
    BlackWhite(BlackWhite),
    Volcano(VulcanoHSL),
}

impl Clone for ColorMaps {
    fn clone(&self) -> Self {
        match self {
            ColorMaps::Viridis(_) => ColorMaps::Viridis(ViridisRGB),
            ColorMaps::Bone(_) => ColorMaps::Bone(Bone),
            ColorMaps::Mandelbrot(_) => ColorMaps::Mandelbrot(MandelbrotHSL),
            ColorMaps::BlackWhite(_) => ColorMaps::BlackWhite(BlackWhite),
            ColorMaps::Volcano(_) => ColorMaps::Volcano(VulcanoHSL),
        }
    }
}
impl ColorMaps {
    pub fn map(&self, value: f32) -> RGBColor {
        match self {
            ColorMaps::Viridis(c) => c.get_color(value),
            ColorMaps::Bone(c) => c.get_color(value),
            ColorMaps::Mandelbrot(c) => convert_hsl_to_rgb(c.get_color(value)),
            ColorMaps::BlackWhite(c) => c.get_color(value),
            ColorMaps::Volcano(c) => convert_hsl_to_rgb(c.get_color(value)),
        }
    }
}
impl std::fmt::Debug for ColorMaps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorMaps::Viridis(_) => write!(f, "Viridis"),
            ColorMaps::Bone(_) => write!(f, "Bone"),
            ColorMaps::Mandelbrot(_) => write!(f, "Mandelbrot"),
            ColorMaps::BlackWhite(_) => write!(f, "BlackWhite"),
            ColorMaps::Volcano(_) => write!(f, "Volcano"),
        }
    }
}
impl Default for ColorMaps {
    fn default() -> Self {
        ColorMaps::Viridis(ViridisRGB)
    }
}

fn convert_hsl_to_rgb(hsl: HSLColor) -> RGBColor {
    let (r, g, b) = hsl.rgb();
    RGBColor(r, g, b)
}

// Define your custom color map
pub struct CustomColorMap;

macro_rules! def_linear_colormap{
    ($color_scale_name:ident, $color_type:ident, $doc:expr, $(($($color_value:expr),+)),*) => {
        #[doc = $doc]
        pub struct $color_scale_name;

        impl $color_scale_name {
            // const COLORS: [$color_type; $number_colors] = [$($color_type($($color_value),+)),+];
            // const COLORS: [$color_type; $crate::count!($(($($color_value:expr),+))*)] = [$($color_type($($color_value),+)),+];
            const COLORS: [$color_type; $crate::count!($(($($color_value:expr),+))*)] = $crate::define_colors_from_list_of_values_or_directly!{$color_type, $(($($color_value),+)),*};
        }

        $crate::implement_linear_interpolation_color_map!{$color_scale_name, $color_type}
    };
    ($color_scale_name:ident, $color_type:ident, $doc:expr, $($color_complete:tt),+) => {
        #[doc = $doc]
        pub struct $color_scale_name;

        impl $color_scale_name {
            const COLORS: [$color_type; $crate::count!($($color_complete)*)] = $crate::define_colors_from_list_of_values_or_directly!{$($color_complete),+};
        }

        $crate::implement_linear_interpolation_color_map!{$color_scale_name, $color_type}
    }
}

#[macro_export]
#[doc(hidden)]
/// Implements the [ColorMap] trait on a given color scale.
macro_rules! implement_linear_interpolation_color_map {
    ($color_scale_name:ident, $color_type:ident) => {
        impl<FloatType: std::fmt::Debug + num_traits::Float + num_traits::FromPrimitive + num_traits::ToPrimitive>
            ColorMap<$color_type, FloatType> for $color_scale_name
        {
            fn get_color_normalized(
                &self,
                h: FloatType,
                min: FloatType,
                max: FloatType,
            ) -> $color_type {
                let (
                    relative_difference,
                    index_lower,
                    index_upper
                ) = calculate_relative_difference_index_lower_upper(
                    h,
                    min,
                    max,
                    Self::COLORS.len()
                );
                // Interpolate the final color linearly
                $crate::calculate_new_color_value!(
                    relative_difference,
                    Self::COLORS,
                    index_upper,
                    index_lower,
                    $color_type
                )
            }
        }

        impl $color_scale_name {
            #[doc = "Get color value from `"]
            #[doc = stringify!($color_scale_name)]
            #[doc = "` by supplying a parameter 0.0 <= h <= 1.0"]
            pub fn get_color<FloatType: std::fmt::Debug + num_traits::Float + num_traits::FromPrimitive + num_traits::ToPrimitive>(
                h: FloatType,
            ) -> $color_type {
                let color_scale = $color_scale_name {};
                color_scale.get_color(h)
            }

            #[doc = "Get color value from `"]
            #[doc = stringify!($color_scale_name)]
            #[doc = "` by supplying lower and upper bounds min, max and a parameter h where min <= h <= max"]
            pub fn get_color_normalized<
                FloatType: std::fmt::Debug + num_traits::Float + num_traits::FromPrimitive + num_traits::ToPrimitive,
            >(
                h: FloatType,
                min: FloatType,
                max: FloatType,
            ) -> $color_type {
                let color_scale = $color_scale_name {};
                color_scale.get_color_normalized(h, min, max)
            }
        }
    };
}

pub fn calculate_relative_difference_index_lower_upper<
    FloatType: num_traits::Float + num_traits::FromPrimitive + num_traits::ToPrimitive,
>(
    h: FloatType,
    min: FloatType,
    max: FloatType,
    n_steps: usize,
) -> (FloatType, usize, usize) {
    // Ensure that we do have a value in bounds
    let h = num_traits::clamp(h, min, max);
    // Next calculate a normalized value between 0.0 and 1.0
    let t = (h - min) / (max - min);
    let approximate_index = t
        * (FloatType::from_usize(n_steps).expect("should be able to get a float type from usize")
            - FloatType::one())
        .max(FloatType::zero());
    // Calculate which index are the two most nearest of the supplied value
    let index_lower = approximate_index
        .floor()
        .to_usize()
        .expect("should be able to get the lower index");
    let index_upper = approximate_index
        .ceil()
        .to_usize()
        .expect("should be able to get the upper index");
    // Calculate the relative difference, ie. is the actual value more towards the color of index_upper or index_lower?
    let relative_difference = approximate_index.ceil() - approximate_index;
    (relative_difference, index_lower, index_upper)
}

/// Converts a given color identifier and a sequence of colors to an array of them.
macro_rules! define_colors_from_list_of_values_or_directly{
    ($color_type:ident, $(($($color_value:expr),+)),+) => {
        [$($color_type($($color_value),+)),+]
    };
    ($($color_complete:tt),+) => {
        [$($color_complete),+]
    };
}
