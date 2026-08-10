//! Authoritative conversions between simulator layout and output coordinates.

use crate::LayoutRect;

pub const COORDINATE_SPACE_VERSION: u32 = 2;
pub const PHYSICAL_PIXELS: &str = "physical_pixels";
pub const RENDERER_VIEWPORT_UNITS: &str = "renderer_viewport_units";
pub const WOW_SCREEN_UNITS: &str = "wow_screen_units";
pub const LOCAL_FRAME_UNITS: &str = "local_frame_units";
pub const PARENT_RELATIVE_UI_UNITS: &str = "parent_relative_ui_units";
pub const TOP_LEFT_ORIGIN: &str = "top_left";
pub const BOTTOM_LEFT_ORIGIN: &str = "bottom_left";

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Geometry {
    pub coordinate_space: &'static str,
    pub origin: &'static str,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Geometry {
    pub fn renderer(rect: LayoutRect) -> Self {
        Self {
            coordinate_space: RENDERER_VIEWPORT_UNITS,
            origin: TOP_LEFT_ORIGIN,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }

    pub fn physical(rect: LayoutRect, renderer_scale: f32) -> Self {
        let rect = renderer_rect_to_physical(rect, renderer_scale);
        Self {
            coordinate_space: PHYSICAL_PIXELS,
            origin: TOP_LEFT_ORIGIN,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

pub fn renderer_rect_to_physical(rect: LayoutRect, renderer_scale: f32) -> LayoutRect {
    LayoutRect {
        x: rect.x * renderer_scale,
        y: rect.y * renderer_scale,
        width: rect.width * renderer_scale,
        height: rect.height * renderer_scale,
    }
}

pub fn renderer_viewport_size(
    physical_width: f32,
    physical_height: f32,
    renderer_scale: f32,
) -> (f32, f32) {
    debug_assert!(renderer_scale.is_finite() && renderer_scale > 0.0);
    (
        physical_width / renderer_scale,
        physical_height / renderer_scale,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_geometry_converts_to_physical_pixels_once() {
        let logical = LayoutRect {
            x: 1351.02,
            y: 218.36,
            width: 33.92,
            height: 33.92,
        };
        let physical = renderer_rect_to_physical(logical, 0.53);

        assert!((physical.x - 716.0406).abs() < 0.001);
        assert!((physical.y - 115.7308).abs() < 0.001);
        assert!((physical.width - 17.9776).abs() < 0.001);
        assert!((physical.height - 17.9776).abs() < 0.001);
    }

    #[test]
    fn physical_stage_maps_to_renderer_viewport_units() {
        let (width, height) = renderer_viewport_size(2560.0, 1440.0, 0.53);
        assert!((width - 4830.1887).abs() < 0.001);
        assert!((height - 2716.9811).abs() < 0.001);
    }
}
