//! Tiled texture rendering — horizontal, vertical, and grid tiling.

use crate::render::{BlendMode, QuadBatch};
use iced::{Point, Rectangle, Size};

use super::texture_gradient::{TextureGradient, frame_gradient};

pub(super) fn crop_path_for_subregion(tex_path: &str, uvs: &Rectangle) -> (String, Rectangle) {
    let is_full = (uvs.x).abs() < 0.001
        && (uvs.y).abs() < 0.001
        && (uvs.width - 1.0).abs() < 0.001
        && (uvs.height - 1.0).abs() < 0.001;
    if is_full {
        return (tex_path.to_string(), *uvs);
    }
    let left = uvs.x;
    let right = uvs.x + uvs.width;
    let top = uvs.y;
    let bottom = uvs.y + uvs.height;
    let crop_key = format!("{tex_path}@crop:{left:.6},{right:.6},{top:.6},{bottom:.6}");
    let full_uvs = Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0));
    (crop_key, full_uvs)
}

#[derive(Debug, Clone, Copy)]
enum TileDir {
    Horizontal,
    Vertical,
    Grid,
}

struct UvRepeatInfo {
    u_min: f32,
    u_max: f32,
    v_min: f32,
    v_max: f32,
    repeat_x: f32,
    repeat_y: f32,
    dir: TileDir,
    rotated: bool,
}

struct UvRepeatMetrics {
    u_min: f32,
    u_max: f32,
    v_min: f32,
    v_max: f32,
    repeat_x: f32,
    repeat_y: f32,
}

impl UvRepeatMetrics {
    fn into_info(self, dir: TileDir, rotated: bool) -> UvRepeatInfo {
        UvRepeatInfo {
            u_min: self.u_min,
            u_max: self.u_max,
            v_min: self.v_min,
            v_max: self.v_max,
            repeat_x: self.repeat_x,
            repeat_y: self.repeat_y,
            dir,
            rotated,
        }
    }
}

struct UvRepeatEdges {
    any_x_repeats: bool,
    left_y_repeats: bool,
    right_y_repeats: bool,
    bottom_y_repeats: bool,
    top_y_repeats: bool,
}

/// Compute the natural pixel size of one tile. For atlas-backed textures this
/// is the atlas slot's source size, so `horizTile`/`vertTile` repeats at the
/// texture's authored width/height instead of stretching one quad to fill the
/// whole frame.
fn tile_dimensions(f: &crate::widget::Frame, uv_w: f32, uv_h: f32) -> (f32, f32) {
    if let Some(atlas_name) = f.atlas.as_deref()
        && let Some(info) = crate::atlas::get_atlas_info(atlas_name)
    {
        return (info.width() as f32, info.height() as f32);
    }

    let tile_w = if f.width > 0.0 {
        f.width
    } else {
        (uv_w * 128.0).max(8.0)
    };
    let tile_h = if f.height > 0.0 {
        f.height
    } else {
        (uv_h * 128.0).max(8.0)
    };
    (tile_w, tile_h)
}

/// Analyze raw 8-arg SetTexCoord values to determine tiling parameters.
///
/// BackdropTemplateMixin encodes repeat counts as UV values >1.0.
/// TopEdge/BottomEdge: Y coords on left corners have repeats, UV is rotated.
/// LeftEdge/RightEdge: Y coords on bottom corners have repeats, UV is standard.
fn analyze_uv_repeat(raw: &[f32; 8]) -> UvRepeatInfo {
    let metrics = uv_repeat_metrics(raw);
    let edges = uv_repeat_edges(raw, &metrics);

    if is_rotated_horizontal_repeat(&edges) {
        return metrics.into_info(TileDir::Horizontal, true);
    }

    if is_standard_vertical_repeat(&edges) {
        return metrics.into_info(TileDir::Vertical, false);
    }

    if is_standard_horizontal_repeat(&edges) {
        return metrics.into_info(TileDir::Horizontal, false);
    }

    metrics.into_info(TileDir::Grid, false)
}

fn uv_repeat_metrics(raw: &[f32; 8]) -> UvRepeatMetrics {
    let [ul_x, ul_y, ll_x, ll_y, ur_x, ur_y, lr_x, lr_y] = *raw;

    let max_x = ul_x.max(ll_x).max(ur_x).max(lr_x);
    let max_y = ul_y.max(ll_y).max(ur_y).max(lr_y);

    UvRepeatMetrics {
        u_min: ul_x.min(ll_x).min(ur_x).min(lr_x),
        u_max: max_x.min(1.0),
        v_min: ul_y.min(ll_y).min(ur_y).min(lr_y),
        v_max: max_y.min(1.0),
        repeat_x: max_x.max(1.0),
        repeat_y: max_y.max(1.0),
    }
}

fn uv_repeat_edges(raw: &[f32; 8], metrics: &UvRepeatMetrics) -> UvRepeatEdges {
    let [_ul_x, ul_y, _ll_x, ll_y, _ur_x, ur_y, _lr_x, lr_y] = *raw;

    UvRepeatEdges {
        any_x_repeats: metrics.repeat_x > 1.0,
        left_y_repeats: ul_y > 1.0 || ll_y > 1.0,
        right_y_repeats: ur_y > 1.0 || lr_y > 1.0,
        bottom_y_repeats: ll_y > 1.0 || lr_y > 1.0,
        top_y_repeats: ul_y > 1.0 || ur_y > 1.0,
    }
}

fn is_rotated_horizontal_repeat(edges: &UvRepeatEdges) -> bool {
    (edges.left_y_repeats ^ edges.right_y_repeats) && !edges.any_x_repeats
}

fn is_standard_vertical_repeat(edges: &UvRepeatEdges) -> bool {
    (edges.bottom_y_repeats ^ edges.top_y_repeats) && !edges.any_x_repeats
}

fn is_standard_horizontal_repeat(edges: &UvRepeatEdges) -> bool {
    edges.any_x_repeats && !edges.top_y_repeats && !edges.bottom_y_repeats
}

fn uv_repeat_tile_size(
    f: &crate::widget::Frame,
    bounds: Rectangle,
    info: &UvRepeatInfo,
) -> (f32, f32) {
    if let Some(atlas_name) = f.atlas.as_deref()
        && let Some(info) = crate::atlas::get_atlas_info(atlas_name)
    {
        return (info.width() as f32, info.height() as f32);
    }

    let repeat_w = bounds.width / info.repeat_x.max(1.0);
    let repeat_h = bounds.height / info.repeat_y.max(1.0);
    if f.width > 1.0 && f.height > 1.0 {
        (
            repeat_w.min(f.width).max(1.0),
            repeat_h.min(f.height).max(1.0),
        )
    } else if f.height > 1.0 {
        (
            repeat_w.min(f.height).max(1.0),
            repeat_h.min(f.height).max(1.0),
        )
    } else if f.width > 1.0 {
        (
            repeat_w.min(f.width).max(1.0),
            repeat_h.min(f.width).max(1.0),
        )
    } else {
        (repeat_w.max(1.0), repeat_h.max(1.0))
    }
}

fn frame_tint(f: &crate::widget::Frame, alpha: f32) -> [f32; 4] {
    let vc = f.vertex_color.as_ref();
    [
        vc.map_or(1.0, |c| c.r),
        vc.map_or(1.0, |c| c.g),
        vc.map_or(1.0, |c| c.b),
        vc.map_or(1.0, |c| c.a) * alpha,
    ]
}

struct StandardTileConfig {
    cropped_path: String,
    cropped_uvs: Rectangle,
    tile_w: f32,
    tile_h: f32,
    tint: [f32; 4],
    gradient: Option<TextureGradient>,
}

/// Emit tiled texture quads (horizontal, vertical, or both).
pub(super) fn emit_tiled_texture(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    uvs: &Rectangle,
    tex_path: &str,
    f: &crate::widget::Frame,
    alpha: f32,
) {
    if emit_uv_repeat_if_needed(batch, bounds, tex_path, f, alpha) {
        return;
    }

    let config = standard_tile_config(tex_path, uvs, f, alpha, bounds);
    emit_standard_tiled_texture(batch, bounds, &config, f);
}

pub(super) fn has_uv_repeat(f: &crate::widget::Frame) -> bool {
    f.tex_coords_quad
        .is_some_and(|raw| raw.iter().any(|&value| value > 1.0))
}

fn emit_uv_repeat_if_needed(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    tex_path: &str,
    f: &crate::widget::Frame,
    alpha: f32,
) -> bool {
    let Some(raw) = &f.tex_coords_quad else {
        return false;
    };
    if !has_uv_repeat(f) {
        return false;
    }
    emit_uv_repeat_tiled(batch, bounds, raw, tex_path, f, alpha);
    true
}

fn standard_tile_config(
    tex_path: &str,
    uvs: &Rectangle,
    f: &crate::widget::Frame,
    alpha: f32,
    gradient_bounds: Rectangle,
) -> StandardTileConfig {
    let (cropped_path, cropped_uvs) = crop_path_for_subregion(tex_path, uvs);
    let (tile_w, tile_h) = tile_dimensions(f, cropped_uvs.width, cropped_uvs.height);

    StandardTileConfig {
        cropped_path,
        cropped_uvs,
        tile_w,
        tile_h,
        tint: frame_tint(f, alpha),
        gradient: frame_gradient(f, gradient_bounds),
    }
}

fn emit_standard_tiled_texture(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    config: &StandardTileConfig,
    f: &crate::widget::Frame,
) {
    if f.horiz_tile && !f.vert_tile {
        emit_standard_horiz_tiles(batch, bounds, config, f.blend_mode);
        return;
    }

    if f.vert_tile && !f.horiz_tile {
        emit_standard_vert_tiles(batch, bounds, config, f.blend_mode);
        return;
    }

    emit_standard_grid_tiles(batch, bounds, config, f.blend_mode);
}

fn emit_standard_horiz_tiles(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    config: &StandardTileConfig,
    blend: BlendMode,
) {
    emit_horiz_tiles(
        batch,
        HorizTileStrip {
            bounds,
            uvs: &config.cropped_uvs,
            tex_path: &config.cropped_path,
            tile_w: config.tile_w,
            tint: config.tint,
            blend,
            gradient: config.gradient,
        },
    );
}

fn emit_standard_vert_tiles(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    config: &StandardTileConfig,
    blend: BlendMode,
) {
    emit_vert_tiles(
        batch,
        VertTileStrip {
            bounds,
            uvs: &config.cropped_uvs,
            tex_path: &config.cropped_path,
            tile_h: config.tile_h,
            tint: config.tint,
            blend,
            gradient: config.gradient,
        },
    );
}

fn emit_standard_grid_tiles(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    config: &StandardTileConfig,
    blend: BlendMode,
) {
    emit_grid_tiles(
        batch,
        GridTileStrip {
            bounds,
            uvs: &config.cropped_uvs,
            tex_path: &config.cropped_path,
            tile_w: config.tile_w,
            tile_h: config.tile_h,
            tint: config.tint,
            blend,
            gradient: config.gradient,
        },
    );
}

/// Handle UV-based repeat tiling from BackdropTemplateMixin.
fn emit_uv_repeat_tiled(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    raw: &[f32; 8],
    tex_path: &str,
    f: &crate::widget::Frame,
    alpha: f32,
) {
    let tint = frame_tint(f, alpha);
    let gradient = frame_gradient(f, bounds);
    let info = analyze_uv_repeat(raw);
    let tile_size = uv_repeat_tile_size(f, bounds, &info);

    if info.rotated {
        emit_rotated_horiz_tiles(
            batch,
            RotatedHorizTileStrip {
                bounds,
                info: &info,
                tex_path,
                tile_w: tile_size.0,
                tint,
                blend: f.blend_mode,
                gradient,
            },
        );
        return;
    }

    emit_standard_uv_repeat_tiles(
        batch,
        bounds,
        tex_path,
        &info,
        tile_size,
        tint,
        f.blend_mode,
        gradient,
    );
}

fn emit_standard_uv_repeat_tiles(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    tex_path: &str,
    info: &UvRepeatInfo,
    tile_size: (f32, f32),
    tint: [f32; 4],
    blend: BlendMode,
    gradient: Option<TextureGradient>,
) {
    let (cropped_path, cropped_uvs) = uv_repeat_region(tex_path, info);
    match info.dir {
        TileDir::Vertical => {
            emit_vert_tiles(
                batch,
                VertTileStrip {
                    bounds,
                    uvs: &cropped_uvs,
                    tex_path: &cropped_path,
                    tile_h: tile_size.1,
                    tint,
                    blend,
                    gradient,
                },
            );
        }
        _ => {
            emit_grid_tiles(
                batch,
                GridTileStrip {
                    bounds,
                    uvs: &cropped_uvs,
                    tex_path: &cropped_path,
                    tile_w: tile_size.0,
                    tile_h: tile_size.1,
                    tint,
                    blend,
                    gradient,
                },
            );
        }
    }
}

fn uv_repeat_region(tex_path: &str, info: &UvRepeatInfo) -> (String, Rectangle) {
    let base_uvs = Rectangle::new(
        Point::new(info.u_min, info.v_min),
        Size::new(info.u_max - info.u_min, info.v_max - info.v_min),
    );
    crop_path_for_subregion(tex_path, &base_uvs)
}

/// Emit horizontally tiled quads with rotated UV mapping (U→vertical, V→horizontal).
/// Used for BackdropTemplateMixin TopEdge/BottomEdge where V maps to screen horizontal.
fn emit_rotated_horiz_tiles(batch: &mut QuadBatch, strip: RotatedHorizTileStrip<'_>) {
    // Crop the UV sub-region into its own atlas slot to prevent bilinear bleed.
    let sub_uvs = Rectangle::new(
        Point::new(strip.info.u_min, strip.info.v_min),
        Size::new(
            strip.info.u_max - strip.info.u_min,
            strip.info.v_max - strip.info.v_min,
        ),
    );
    let (cropped_path, _) = crop_path_for_subregion(strip.tex_path, &sub_uvs);
    // After cropping, UVs are remapped to full [0,1] range within the cropped slot.
    let u_min = 0.0_f32;
    let u_max = 1.0_f32;
    let v_start = 0.0_f32;
    let v_range = 1.0_f32;

    let mut x = strip.bounds.x;
    while x < strip.bounds.x + strip.bounds.width {
        let w = (strip.bounds.x + strip.bounds.width - x).min(strip.tile_w);
        let tile_bounds = Rectangle::new(
            Point::new(x, strip.bounds.y),
            Size::new(w, strip.bounds.height),
        );
        let v_extent = if w < strip.tile_w {
            v_range * (w / strip.tile_w)
        } else {
            v_range
        };
        // Rotated: U maps to screen Y (top→bottom), V maps to screen X (left→right)
        let uvs = [
            [u_min, v_start + v_extent], // TL: top of strip, right side of V tile
            [u_min, v_start],            // TR: top of strip, left side of V tile
            [u_max, v_start],            // BR: bottom of strip, left side of V tile
            [u_max, v_start + v_extent], // BL: bottom of strip, right side of V tile
        ];
        if let Some(gradient) = strip.gradient {
            batch.push_textured_path_uv4_colors(
                tile_bounds,
                uvs,
                &cropped_path,
                gradient.colors(tile_bounds, strip.tint),
                strip.blend,
            );
        } else {
            batch.push_textured_path_uv4(tile_bounds, uvs, &cropped_path, strip.tint, strip.blend);
        }
        x += strip.tile_w;
    }
}

struct RotatedHorizTileStrip<'a> {
    bounds: Rectangle,
    info: &'a UvRepeatInfo,
    tex_path: &'a str,
    tile_w: f32,
    tint: [f32; 4],
    blend: BlendMode,
    gradient: Option<TextureGradient>,
}

/// Emit horizontally tiled texture quads.
pub(super) struct HorizTileStrip<'a> {
    pub(super) bounds: Rectangle,
    pub(super) uvs: &'a Rectangle,
    pub(super) tex_path: &'a str,
    pub(super) tile_w: f32,
    pub(super) tint: [f32; 4],
    pub(super) blend: BlendMode,
    pub(super) gradient: Option<TextureGradient>,
}

fn push_tiled_quad(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    uvs: Rectangle,
    path: &str,
    tint: [f32; 4],
    blend: BlendMode,
    gradient: Option<TextureGradient>,
) {
    if let Some(gradient) = gradient {
        batch.push_textured_path_uv_colors(bounds, uvs, path, gradient.colors(bounds, tint), blend);
    } else {
        batch.push_textured_path_uv(bounds, uvs, path, tint, blend);
    }
}

pub(super) fn emit_horiz_tiles(batch: &mut QuadBatch, strip: HorizTileStrip<'_>) {
    if strip.tile_w <= 1.0 {
        push_tiled_quad(
            batch,
            strip.bounds,
            *strip.uvs,
            strip.tex_path,
            strip.tint,
            strip.blend,
            strip.gradient,
        );
        return;
    }

    let mut x = strip.bounds.x;
    while x < strip.bounds.x + strip.bounds.width {
        let w = (strip.bounds.x + strip.bounds.width - x).min(strip.tile_w);
        let tile_bounds = Rectangle::new(
            Point::new(x, strip.bounds.y),
            Size::new(w, strip.bounds.height),
        );
        let uv_w = if w < strip.tile_w {
            strip.uvs.width * (w / strip.tile_w)
        } else {
            strip.uvs.width
        };
        let tile_uvs = Rectangle::new(strip.uvs.position(), Size::new(uv_w, strip.uvs.height));
        push_tiled_quad(
            batch,
            tile_bounds,
            tile_uvs,
            strip.tex_path,
            strip.tint,
            strip.blend,
            strip.gradient,
        );
        x += strip.tile_w;
    }
}

/// Emit vertically tiled texture quads.
pub(super) struct VertTileStrip<'a> {
    pub(super) bounds: Rectangle,
    pub(super) uvs: &'a Rectangle,
    pub(super) tex_path: &'a str,
    pub(super) tile_h: f32,
    pub(super) tint: [f32; 4],
    pub(super) blend: BlendMode,
    pub(super) gradient: Option<TextureGradient>,
}

pub(super) fn emit_vert_tiles(batch: &mut QuadBatch, strip: VertTileStrip<'_>) {
    let mut y = strip.bounds.y;
    while y < strip.bounds.y + strip.bounds.height {
        let h = (strip.bounds.y + strip.bounds.height - y).min(strip.tile_h);
        let tile_bounds = Rectangle::new(
            Point::new(strip.bounds.x, y),
            Size::new(strip.bounds.width, h),
        );
        let uv_h = if h < strip.tile_h {
            strip.uvs.height * (h / strip.tile_h)
        } else {
            strip.uvs.height
        };
        let tile_uvs = Rectangle::new(strip.uvs.position(), Size::new(strip.uvs.width, uv_h));
        push_tiled_quad(
            batch,
            tile_bounds,
            tile_uvs,
            strip.tex_path,
            strip.tint,
            strip.blend,
            strip.gradient,
        );
        y += strip.tile_h;
    }
}

pub(super) struct GridTileStrip<'a> {
    pub(super) bounds: Rectangle,
    pub(super) uvs: &'a Rectangle,
    pub(super) tex_path: &'a str,
    pub(super) tile_w: f32,
    pub(super) tile_h: f32,
    pub(super) tint: [f32; 4],
    pub(super) blend: BlendMode,
    pub(super) gradient: Option<TextureGradient>,
}

/// Emit grid-tiled texture quads (both horizontal and vertical).
pub(super) fn emit_grid_tiles(batch: &mut QuadBatch, strip: GridTileStrip<'_>) {
    let mut y = strip.bounds.y;
    while y < strip.bounds.y + strip.bounds.height {
        let h = (strip.bounds.y + strip.bounds.height - y).min(strip.tile_h);
        let mut x = strip.bounds.x;
        while x < strip.bounds.x + strip.bounds.width {
            let w = (strip.bounds.x + strip.bounds.width - x).min(strip.tile_w);
            let tile_bounds = Rectangle::new(Point::new(x, y), Size::new(w, h));
            let uv_w = if w < strip.tile_w {
                strip.uvs.width * (w / strip.tile_w)
            } else {
                strip.uvs.width
            };
            let uv_h = if h < strip.tile_h {
                strip.uvs.height * (h / strip.tile_h)
            } else {
                strip.uvs.height
            };
            let tile_uvs = Rectangle::new(strip.uvs.position(), Size::new(uv_w, uv_h));
            push_tiled_quad(
                batch,
                tile_bounds,
                tile_uvs,
                strip.tex_path,
                strip.tint,
                strip.blend,
                strip.gradient,
            );
            x += strip.tile_w;
        }
        y += strip.tile_h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_repeat_region_crops_to_repeat_strip() {
        let info = UvRepeatInfo {
            u_min: 0.25,
            u_max: 0.75,
            v_min: 0.4,
            v_max: 1.0,
            repeat_x: 1.0,
            repeat_y: 1.0,
            dir: TileDir::Grid,
            rotated: false,
        };

        let (path, uvs) = uv_repeat_region("Interface/Test", &info);

        assert_eq!(
            path,
            "Interface/Test@crop:0.250000,0.750000,0.400000,1.000000"
        );
        assert_eq!(uvs, Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0)));
    }

    #[test]
    fn standard_tile_config_crops_subregions_and_uses_fallback_sizes() {
        let frame = crate::widget::Frame {
            width: 0.0,
            height: 0.0,
            vertex_color: Some(crate::widget::Color {
                r: 0.25,
                g: 0.5,
                b: 0.75,
                a: 0.8,
            }),
            ..Default::default()
        };
        let uvs = Rectangle::new(Point::new(0.25, 0.5), Size::new(0.5, 0.25));

        let config = standard_tile_config(
            "Interface/Test",
            &uvs,
            &frame,
            0.5,
            Rectangle::new(Point::ORIGIN, Size::new(128.0, 128.0)),
        );

        assert_eq!(
            config.cropped_path,
            "Interface/Test@crop:0.250000,0.750000,0.500000,0.750000"
        );
        assert_eq!(
            config.cropped_uvs,
            Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0))
        );
        assert_eq!(config.tile_w, 128.0);
        assert_eq!(config.tile_h, 128.0);
        assert_eq!(config.tint, [0.25, 0.5, 0.75, 0.4]);
    }

    #[test]
    fn standard_tile_config_preserves_one_pixel_atlas_strip_size() {
        let frame = crate::widget::Frame {
            width: 1.0,
            height: 42.0,
            ..Default::default()
        };
        let uvs = Rectangle::new(Point::new(0.0, 0.003906), Size::new(0.015625, 0.164063));

        let config = standard_tile_config(
            "Interface/FrameGeneral/UIFrameTabs",
            &uvs,
            &frame,
            1.0,
            Rectangle::new(Point::ORIGIN, Size::new(1.0, 42.0)),
        );

        assert_eq!(config.tile_w, 1.0);
        assert_eq!(config.tile_h, 42.0);
    }

    #[test]
    fn horizontal_atlas_tiles_use_source_size() {
        let frame = crate::widget::Frame {
            atlas: Some("_128-RedButton-Center".to_string()),
            horiz_tile: true,
            ..Default::default()
        };
        let uvs = Rectangle::new(Point::new(0.0, 0.000488), Size::new(0.125, 0.0625));

        let config = standard_tile_config(
            "Interface\\buttons\\128redbutton",
            &uvs,
            &frame,
            1.0,
            Rectangle::new(Point::ORIGIN, Size::new(128.0, 128.0)),
        );

        assert_eq!(config.tile_w, 64.0);
        assert_eq!(config.tile_h, 128.0);
        assert_eq!(
            config.cropped_path,
            "Interface\\buttons\\128redbutton@crop:0.000000,0.125000,0.000488,0.062988"
        );
    }

    #[test]
    fn horizontal_one_pixel_repeat_collapses_to_single_quad() {
        let mut batch = QuadBatch::new();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(14.0, 42.0));
        let uvs = Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0));

        emit_horiz_tiles(
            &mut batch,
            HorizTileStrip {
                bounds,
                uvs: &uvs,
                tex_path: "Interface/FrameGeneral/UIFrameTabs@crop:0.000000,0.015625,0.003906,0.167969",
                tile_w: 1.0,
                tint: [1.0, 1.0, 1.0, 1.0],
                blend: BlendMode::Alpha,
                gradient: None,
            },
        );

        assert_eq!(batch.vertices.len(), 4);
        assert_eq!(batch.texture_requests.len(), 1);
        assert_eq!(batch.vertices[0].position, [0.0, 0.0]);
        assert_eq!(batch.vertices[1].position, [14.0, 0.0]);
    }

    #[test]
    fn uv_repeat_atlas_tiles_use_source_size() {
        let mut batch = QuadBatch::new();
        let mut frame = crate::widget::Frame {
            atlas: Some("_128-RedButton-Center".to_string()),
            width: 256.0,
            height: 256.0,
            ..Default::default()
        };
        frame.tex_coords_quad = Some([0.0, 0.0, 0.0, 2.0, 1.0, 0.0, 1.0, 0.0]);

        emit_uv_repeat_tiled(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(128.0, 20.0)),
            frame.tex_coords_quad.as_ref().expect("tex coords"),
            "Interface\\buttons\\128redbutton",
            &frame,
            1.0,
        );

        assert_eq!(batch.vertices.len(), 8);
        assert_eq!(batch.vertices[0].position, [0.0, 0.0]);
        assert_eq!(batch.vertices[1].position, [64.0, 0.0]);
        assert_eq!(batch.vertices[4].position, [64.0, 0.0]);
        assert_eq!(batch.vertices[5].position, [128.0, 0.0]);
    }
}
