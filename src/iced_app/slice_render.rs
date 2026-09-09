//! Slice rendering: three-slice, stretch (nine-slice), and tile-slice emission.
//!
//! Extracted from quad_builders_textures.rs to keep file sizes manageable.

use iced::{Point, Rectangle, Size};

use crate::render::{BlendMode, QuadBatch};

use super::texture_gradient::TextureGradient;
use super::tiling::{
    GridTileStrip, HorizTileStrip, VertTileStrip, emit_grid_tiles, emit_horiz_tiles,
    emit_vert_tiles,
};

pub(super) type TextureUvs = (f32, f32, f32, f32);

#[derive(Clone, Copy)]
struct RectGrid {
    x: [f32; 3],
    y: [f32; 3],
    w: [f32; 3],
    h: [f32; 3],
}

#[derive(Clone, Copy)]
pub(super) struct TexturedSlice<'a> {
    pub path: &'a str,
    pub uvs: TextureUvs,
    pub tint: [f32; 4],
    pub blend: BlendMode,
    pub gradient: Option<TextureGradient>,
}

#[derive(Clone, Copy)]
pub(super) struct ThreeSliceRender<'a> {
    pub bounds: Rectangle,
    pub texture: TexturedSlice<'a>,
    pub left_cap_px: f32,
    pub right_cap_px: f32,
    pub atlas_width_px: f32,
}

#[derive(Clone, Copy)]
pub(super) struct StretchSliceRender<'a> {
    pub bounds: Rectangle,
    pub texture: TexturedSlice<'a>,
    pub left_px: f32,
    pub top_px: f32,
    pub right_px: f32,
    pub bottom_px: f32,
    pub atlas_width_px: f32,
    pub atlas_height_px: f32,
}

#[derive(Clone, Copy)]
pub(super) struct TileSliceRender<'a> {
    pub bounds: Rectangle,
    pub texture: TexturedSlice<'a>,
    pub left_px: f32,
    pub top_px: f32,
    pub right_px: f32,
    pub bottom_px: f32,
    pub atlas_width_px: f32,
    pub atlas_height_px: f32,
}

#[derive(Clone, Copy)]
enum TileSlicePattern {
    Horizontal { tile_w: f32 },
    Vertical { tile_h: f32 },
    Grid { tile_w: f32, tile_h: f32 },
}

#[derive(Clone, Copy)]
struct TilePatternEmit<'a> {
    texture: TexturedSlice<'a>,
    bounds: Rectangle,
    full_uvs: &'a Rectangle,
    path: &'a str,
}

impl TileSlicePattern {
    fn is_valid(self) -> bool {
        match self {
            Self::Horizontal { tile_w } => tile_w > 0.0,
            Self::Vertical { tile_h } => tile_h > 0.0,
            Self::Grid { tile_w, tile_h } => tile_w > 0.0 && tile_h > 0.0,
        }
    }

    fn emit_collapsed_repeat(
        self,
        batch: &mut QuadBatch,
        texture: TexturedSlice<'_>,
        bounds: Rectangle,
        source_uvs: Rectangle,
    ) -> bool {
        match self {
            Self::Horizontal { tile_w } if tile_w <= 1.0 => {
                push_cropped_slice_quad(batch, texture, bounds, source_uvs);
                true
            }
            Self::Vertical { tile_h } if tile_h <= 1.0 => {
                push_cropped_slice_quad(batch, texture, bounds, source_uvs);
                true
            }
            Self::Grid { tile_w, tile_h } if tile_w <= 1.0 && tile_h <= 1.0 => {
                push_cropped_slice_quad(batch, texture, bounds, source_uvs);
                true
            }
            Self::Grid { tile_w, tile_h } if tile_w <= 1.0 => {
                emit_vert_tiled_slice(batch, texture, bounds, source_uvs, tile_h);
                true
            }
            Self::Grid { tile_w, tile_h } if tile_h <= 1.0 => {
                emit_horiz_tiled_slice(batch, texture, bounds, source_uvs, tile_w);
                true
            }
            _ => false,
        }
    }

    fn emit_tiles(
        self,
        batch: &mut QuadBatch,
        texture: TexturedSlice<'_>,
        bounds: Rectangle,
        full_uvs: &Rectangle,
        path: &str,
    ) {
        let emit = TilePatternEmit {
            texture,
            bounds,
            full_uvs,
            path,
        };

        match self {
            Self::Horizontal { tile_w } => emit_pattern_horiz_tiles(batch, emit, tile_w),
            Self::Vertical { tile_h } => emit_pattern_vert_tiles(batch, emit, tile_h),
            Self::Grid { tile_w, tile_h } => emit_pattern_grid_tiles(batch, emit, tile_w, tile_h),
        }
    }
}

fn emit_pattern_horiz_tiles(batch: &mut QuadBatch, emit: TilePatternEmit<'_>, tile_w: f32) {
    emit_horiz_tiles(
        batch,
        HorizTileStrip {
            bounds: emit.bounds,
            uvs: emit.full_uvs,
            tex_path: emit.path,
            tile_w,
            tint: emit.texture.tint,
            blend: emit.texture.blend,
            gradient: emit.texture.gradient,
        },
    );
}

fn emit_pattern_vert_tiles(batch: &mut QuadBatch, emit: TilePatternEmit<'_>, tile_h: f32) {
    emit_vert_tiles(
        batch,
        VertTileStrip {
            bounds: emit.bounds,
            uvs: emit.full_uvs,
            tex_path: emit.path,
            tile_h,
            tint: emit.texture.tint,
            blend: emit.texture.blend,
            gradient: emit.texture.gradient,
        },
    );
}

fn emit_pattern_grid_tiles(
    batch: &mut QuadBatch,
    emit: TilePatternEmit<'_>,
    tile_w: f32,
    tile_h: f32,
) {
    emit_grid_tiles(
        batch,
        GridTileStrip {
            bounds: emit.bounds,
            uvs: emit.full_uvs,
            tex_path: emit.path,
            tile_w,
            tile_h,
            tint: emit.texture.tint,
            blend: emit.texture.blend,
            gradient: emit.texture.gradient,
        },
    );
}

/// Render an atlas texture as 3 horizontal slices (left cap, stretched middle, right cap).
pub(super) fn emit_three_slice_h_atlas(batch: &mut QuadBatch, render: ThreeSliceRender<'_>) {
    push_textured_rects(batch, render.texture, three_slice_rects(render));
}

fn three_slice_rects(render: ThreeSliceRender<'_>) -> [(Rectangle, Rectangle); 3] {
    let dst = three_slice_dst_rects(render);
    let src = three_slice_src_rects(render);
    [(dst[0], src[0]), (dst[1], src[1]), (dst[2], src[2])]
}

fn three_slice_dst_rects(render: ThreeSliceRender<'_>) -> [Rectangle; 3] {
    let bounds = render.bounds;
    let (mid_x, mid_w, right_x) = three_slice_dst_spans(render);

    [
        rect(bounds.x, bounds.y, render.left_cap_px, bounds.height),
        rect(mid_x, bounds.y, mid_w, bounds.height),
        rect(right_x, bounds.y, render.right_cap_px, bounds.height),
    ]
}

fn three_slice_src_rects(render: ThreeSliceRender<'_>) -> [Rectangle; 3] {
    let (left_uv, right_uv, top_uv, bottom_uv) = render.texture.uvs;
    let (uv_w, uv_h) = (right_uv - left_uv, bottom_uv - top_uv);
    let (left_cap_uv_end, right_cap_uv_start) =
        three_slice_cap_uvs(render, left_uv, right_uv, uv_w);

    [
        rect(left_uv, top_uv, left_cap_uv_end - left_uv, uv_h),
        rect(
            left_cap_uv_end,
            top_uv,
            right_cap_uv_start - left_cap_uv_end,
            uv_h,
        ),
        rect(
            right_cap_uv_start,
            top_uv,
            right_uv - right_cap_uv_start,
            uv_h,
        ),
    ]
}

fn three_slice_cap_uvs(
    render: ThreeSliceRender<'_>,
    left_uv: f32,
    right_uv: f32,
    uv_w: f32,
) -> (f32, f32) {
    let left_frac = render.left_cap_px / render.atlas_width_px;
    let right_frac = render.right_cap_px / render.atlas_width_px;
    (left_uv + left_frac * uv_w, right_uv - right_frac * uv_w)
}

fn three_slice_dst_spans(render: ThreeSliceRender<'_>) -> (f32, f32, f32) {
    let bounds = render.bounds;
    let mid_x = bounds.x + render.left_cap_px;
    let mid_w = bounds.width - render.left_cap_px - render.right_cap_px;
    let right_x = bounds.x + bounds.width - render.right_cap_px;
    (mid_x, mid_w, right_x)
}

pub(super) fn emit_stretch_slice_atlas(batch: &mut QuadBatch, render: StretchSliceRender<'_>) {
    push_textured_rects(batch, render.texture, stretch_slice_rects(render));
}

pub(super) fn emit_tile_slice_atlas(batch: &mut QuadBatch, render: TileSliceRender<'_>) {
    emit_tile_slice_corners(batch, render);
    emit_tile_slice_edges(batch, render);
    emit_tile_slice_center(batch, render);
}

fn emit_tile_slice_corners(batch: &mut QuadBatch, render: TileSliceRender<'_>) {
    push_cropped_slice_rects(batch, render.texture, tile_slice_corner_rects(render));
}

fn emit_tile_slice_edges(batch: &mut QuadBatch, render: TileSliceRender<'_>) {
    emit_tile_slice_horiz_edges(batch, render);
    emit_tile_slice_vert_edges(batch, render);
}

fn emit_tile_slice_center(batch: &mut QuadBatch, render: TileSliceRender<'_>) {
    let Some((center_dst_w, center_src_w)) = tile_slice_center_width(render) else {
        return;
    };
    let Some((center_dst_h, center_src_h)) = tile_slice_center_height(render) else {
        return;
    };

    let center_uvs = atlas_subregion_uvs(
        render.texture,
        render.atlas_width_px,
        render.atlas_height_px,
        render.left_px,
        render.top_px,
        center_src_w,
        center_src_h,
    );
    let center_bounds = Rectangle::new(
        Point::new(
            render.bounds.x + render.left_px,
            render.bounds.y + render.top_px,
        ),
        Size::new(center_dst_w, center_dst_h),
    );
    emit_grid_tiled_slice(
        batch,
        render.texture,
        center_bounds,
        center_uvs,
        center_src_w,
        center_src_h,
    );
}

fn tile_slice_corner_rects(render: TileSliceRender<'_>) -> [(Rectangle, Rectangle); 4] {
    [
        tile_slice_corner_rect(render, 0.0, 0.0, render.left_px, render.top_px),
        tile_slice_corner_rect(
            render,
            render.atlas_width_px - render.right_px,
            0.0,
            render.right_px,
            render.top_px,
        ),
        tile_slice_corner_rect(
            render,
            0.0,
            render.atlas_height_px - render.bottom_px,
            render.left_px,
            render.bottom_px,
        ),
        tile_slice_corner_rect(
            render,
            render.atlas_width_px - render.right_px,
            render.atlas_height_px - render.bottom_px,
            render.right_px,
            render.bottom_px,
        ),
    ]
}

fn tile_slice_corner_rect(
    render: TileSliceRender<'_>,
    source_x: f32,
    source_y: f32,
    width: f32,
    height: f32,
) -> (Rectangle, Rectangle) {
    let dst_x = render.bounds.x + source_x.min(render.left_px);
    let dst_y = render.bounds.y + source_y.min(render.top_px);
    let dst_x = if source_x > render.left_px {
        render.bounds.x + render.bounds.width - width
    } else {
        dst_x
    };
    let dst_y = if source_y > render.top_px {
        render.bounds.y + render.bounds.height - height
    } else {
        dst_y
    };
    (
        rect(dst_x, dst_y, width, height),
        atlas_subregion_uvs(
            render.texture,
            render.atlas_width_px,
            render.atlas_height_px,
            source_x,
            source_y,
            width,
            height,
        ),
    )
}

fn emit_tile_slice_horiz_edges(batch: &mut QuadBatch, render: TileSliceRender<'_>) {
    let Some((center_dst_w, center_src_w)) = tile_slice_center_width(render) else {
        return;
    };

    emit_tile_slice_horiz_edge(
        batch,
        render,
        0.0,
        render.bounds.y,
        render.top_px,
        center_dst_w,
        center_src_w,
    );
    emit_tile_slice_horiz_edge(
        batch,
        render,
        render.atlas_height_px - render.bottom_px,
        render.bounds.y + render.bounds.height - render.bottom_px,
        render.bottom_px,
        center_dst_w,
        center_src_w,
    );
}

fn emit_tile_slice_horiz_edge(
    batch: &mut QuadBatch,
    render: TileSliceRender<'_>,
    source_y: f32,
    dst_y: f32,
    height: f32,
    center_dst_w: f32,
    center_src_w: f32,
) {
    let edge_uvs = atlas_subregion_uvs(
        render.texture,
        render.atlas_width_px,
        render.atlas_height_px,
        render.left_px,
        source_y,
        center_src_w,
        height,
    );
    let edge_bounds = rect(
        render.bounds.x + render.left_px,
        dst_y,
        center_dst_w,
        height,
    );
    emit_horiz_tiled_slice(batch, render.texture, edge_bounds, edge_uvs, center_src_w);
}

fn emit_tile_slice_vert_edges(batch: &mut QuadBatch, render: TileSliceRender<'_>) {
    let Some((center_dst_h, center_src_h)) = tile_slice_center_height(render) else {
        return;
    };

    let left_uvs = atlas_subregion_uvs(
        render.texture,
        render.atlas_width_px,
        render.atlas_height_px,
        0.0,
        render.top_px,
        render.left_px,
        center_src_h,
    );
    let left_bounds = Rectangle::new(
        Point::new(render.bounds.x, render.bounds.y + render.top_px),
        Size::new(render.left_px, center_dst_h),
    );
    emit_vert_tiled_slice(batch, render.texture, left_bounds, left_uvs, center_src_h);

    let right_uvs = atlas_subregion_uvs(
        render.texture,
        render.atlas_width_px,
        render.atlas_height_px,
        render.atlas_width_px - render.right_px,
        render.top_px,
        render.right_px,
        center_src_h,
    );
    let right_bounds = Rectangle::new(
        Point::new(
            render.bounds.x + render.bounds.width - render.right_px,
            render.bounds.y + render.top_px,
        ),
        Size::new(render.right_px, center_dst_h),
    );
    emit_vert_tiled_slice(batch, render.texture, right_bounds, right_uvs, center_src_h);
}

pub(super) fn tile_slice_center_width(render: TileSliceRender<'_>) -> Option<(f32, f32)> {
    let dst_w = render.bounds.width - render.left_px - render.right_px;
    let src_w = render.atlas_width_px - render.left_px - render.right_px;
    (dst_w > 0.0 && src_w > 0.0).then_some((dst_w, src_w))
}

pub(super) fn tile_slice_center_height(render: TileSliceRender<'_>) -> Option<(f32, f32)> {
    let dst_h = render.bounds.height - render.top_px - render.bottom_px;
    let src_h = render.atlas_height_px - render.top_px - render.bottom_px;
    (dst_h > 0.0 && src_h > 0.0).then_some((dst_h, src_h))
}

fn stretch_slice_rects(render: StretchSliceRender<'_>) -> [(Rectangle, Rectangle); 9] {
    let dst = stretch_slice_dst_grid(render);
    let src = stretch_slice_src_grid(render);
    grid_rect_pairs(dst, src)
}

fn stretch_slice_dst_grid(render: StretchSliceRender<'_>) -> RectGrid {
    let bounds = render.bounds;
    let left_w = render.left_px;
    let right_w = render.right_px;
    let top_h = render.top_px;
    let bottom_h = render.bottom_px;
    RectGrid {
        x: [
            bounds.x,
            bounds.x + left_w,
            bounds.x + bounds.width - right_w,
        ],
        y: [
            bounds.y,
            bounds.y + top_h,
            bounds.y + bounds.height - bottom_h,
        ],
        w: [left_w, bounds.width - left_w - right_w, right_w],
        h: [top_h, bounds.height - top_h - bottom_h, bottom_h],
    }
}

fn stretch_slice_src_grid(render: StretchSliceRender<'_>) -> RectGrid {
    let (left_uv, right_uv, top_uv, bottom_uv) = render.texture.uvs;
    let uv_w = right_uv - left_uv;
    let uv_h = bottom_uv - top_uv;
    let left_u = (render.left_px / render.atlas_width_px) * uv_w;
    let right_u = (render.right_px / render.atlas_width_px) * uv_w;
    let top_v = (render.top_px / render.atlas_height_px) * uv_h;
    let bottom_v = (render.bottom_px / render.atlas_height_px) * uv_h;

    let u1 = left_uv + left_u;
    let u2 = right_uv - right_u;
    let v1 = top_uv + top_v;
    let v2 = bottom_uv - bottom_v;
    RectGrid {
        x: [left_uv, u1, u2],
        y: [top_uv, v1, v2],
        w: [u1 - left_uv, u2 - u1, right_uv - u2],
        h: [v1 - top_uv, v2 - v1, bottom_uv - v2],
    }
}

fn grid_rect_pairs(dst: RectGrid, src: RectGrid) -> [(Rectangle, Rectangle); 9] {
    [
        (grid_rect(dst, 0, 0), grid_rect(src, 0, 0)),
        (grid_rect(dst, 1, 0), grid_rect(src, 1, 0)),
        (grid_rect(dst, 2, 0), grid_rect(src, 2, 0)),
        (grid_rect(dst, 0, 1), grid_rect(src, 0, 1)),
        (grid_rect(dst, 1, 1), grid_rect(src, 1, 1)),
        (grid_rect(dst, 2, 1), grid_rect(src, 2, 1)),
        (grid_rect(dst, 0, 2), grid_rect(src, 0, 2)),
        (grid_rect(dst, 1, 2), grid_rect(src, 1, 2)),
        (grid_rect(dst, 2, 2), grid_rect(src, 2, 2)),
    ]
}

fn grid_rect(grid: RectGrid, col: usize, row: usize) -> Rectangle {
    rect(grid.x[col], grid.y[row], grid.w[col], grid.h[row])
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
    Rectangle::new(Point::new(x, y), Size::new(width, height))
}

fn push_textured_rects<const N: usize>(
    batch: &mut QuadBatch,
    texture: TexturedSlice<'_>,
    rects: [(Rectangle, Rectangle); N],
) {
    for (dst, src) in rects {
        push_textured_slice_quad(batch, texture, dst, src, texture.path);
    }
}

fn push_textured_slice_quad(
    batch: &mut QuadBatch,
    texture: TexturedSlice<'_>,
    bounds: Rectangle,
    uvs: Rectangle,
    path: &str,
) {
    if let Some(gradient) = texture.gradient {
        batch.push_textured_path_uv_colors(
            bounds,
            uvs,
            path,
            gradient.colors(bounds, texture.tint),
            texture.blend,
        );
    } else {
        batch.push_textured_path_uv(bounds, uvs, path, texture.tint, texture.blend);
    }
}

fn push_cropped_slice_quad(
    batch: &mut QuadBatch,
    texture: TexturedSlice<'_>,
    bounds: Rectangle,
    source_uvs: Rectangle,
) {
    if bounds.width <= 0.0
        || bounds.height <= 0.0
        || source_uvs.width <= 0.0
        || source_uvs.height <= 0.0
    {
        return;
    }

    let (path, full_uvs) = crop_flattened_subregion(texture.path, source_uvs);
    push_textured_slice_quad(batch, texture, bounds, full_uvs, &path);
}

fn push_cropped_slice_rects<const N: usize>(
    batch: &mut QuadBatch,
    texture: TexturedSlice<'_>,
    rects: [(Rectangle, Rectangle); N],
) {
    for (bounds, source_uvs) in rects {
        push_cropped_slice_quad(batch, texture, bounds, source_uvs);
    }
}

macro_rules! tiled_slice_emitter {
    ($name:ident, $tile_w:ident => $pattern:expr) => {
        fn $name(
            batch: &mut QuadBatch,
            texture: TexturedSlice<'_>,
            bounds: Rectangle,
            source_uvs: Rectangle,
            $tile_w: f32,
        ) {
            emit_tiled_slice(batch, texture, bounds, source_uvs, $pattern);
        }
    };
    ($name:ident, $tile_w:ident, $tile_h:ident => $pattern:expr) => {
        fn $name(
            batch: &mut QuadBatch,
            texture: TexturedSlice<'_>,
            bounds: Rectangle,
            source_uvs: Rectangle,
            $tile_w: f32,
            $tile_h: f32,
        ) {
            emit_tiled_slice(batch, texture, bounds, source_uvs, $pattern);
        }
    };
}

tiled_slice_emitter!(emit_horiz_tiled_slice, tile_w => TileSlicePattern::Horizontal { tile_w });
tiled_slice_emitter!(emit_vert_tiled_slice, tile_h => TileSlicePattern::Vertical { tile_h });
tiled_slice_emitter!(emit_grid_tiled_slice, tile_w, tile_h => TileSlicePattern::Grid { tile_w, tile_h });

fn emit_tiled_slice(
    batch: &mut QuadBatch,
    texture: TexturedSlice<'_>,
    bounds: Rectangle,
    source_uvs: Rectangle,
    pattern: TileSlicePattern,
) {
    if bounds.width <= 0.0
        || bounds.height <= 0.0
        || !pattern.is_valid()
        || source_uvs.width <= 0.0
        || source_uvs.height <= 0.0
    {
        return;
    }

    if pattern.emit_collapsed_repeat(batch, texture, bounds, source_uvs) {
        return;
    }

    let (path, full_uvs) = crop_flattened_subregion(texture.path, source_uvs);
    pattern.emit_tiles(batch, texture, bounds, &full_uvs, &path);
}

pub(super) fn atlas_subregion_uvs(
    texture: TexturedSlice<'_>,
    atlas_width_px: f32,
    atlas_height_px: f32,
    left_px: f32,
    top_px: f32,
    width_px: f32,
    height_px: f32,
) -> Rectangle {
    let (uv_left, uv_right, uv_top, uv_bottom) = texture.uvs;
    let uv_w = uv_right - uv_left;
    let uv_h = uv_bottom - uv_top;
    Rectangle::new(
        Point::new(
            uv_left + (left_px / atlas_width_px) * uv_w,
            uv_top + (top_px / atlas_height_px) * uv_h,
        ),
        Size::new(
            (width_px / atlas_width_px) * uv_w,
            (height_px / atlas_height_px) * uv_h,
        ),
    )
}

pub(super) fn crop_flattened_subregion(tex_path: &str, sub_uvs: Rectangle) -> (String, Rectangle) {
    if is_full_uv_rect(sub_uvs) {
        return (tex_path.to_string(), full_uv_rect());
    }

    let (base_path, parent_uvs) = decode_crop_path(tex_path).unwrap_or((tex_path, full_uv_rect()));
    let left = parent_uvs.x + sub_uvs.x * parent_uvs.width;
    let right = parent_uvs.x + (sub_uvs.x + sub_uvs.width) * parent_uvs.width;
    let top = parent_uvs.y + sub_uvs.y * parent_uvs.height;
    let bottom = parent_uvs.y + (sub_uvs.y + sub_uvs.height) * parent_uvs.height;
    (
        format!("{base_path}@crop:{left:.6},{right:.6},{top:.6},{bottom:.6}"),
        full_uv_rect(),
    )
}

pub(super) fn decode_crop_path(path: &str) -> Option<(&str, Rectangle)> {
    let crop_start = path.find("@crop:")?;
    let base_path = &path[..crop_start];
    let coords: Vec<f32> = path[crop_start + 6..]
        .split(',')
        .filter_map(|part| part.parse().ok())
        .collect();
    if coords.len() != 4 {
        return None;
    }

    Some((
        base_path,
        Rectangle::new(
            Point::new(coords[0], coords[2]),
            Size::new(coords[1] - coords[0], coords[3] - coords[2]),
        ),
    ))
}

pub(super) fn full_uv_rect() -> Rectangle {
    Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0))
}

pub(super) fn is_full_uv_rect(rect: Rectangle) -> bool {
    rect.x.abs() < 0.001
        && rect.y.abs() < 0.001
        && (rect.width - 1.0).abs() < 0.001
        && (rect.height - 1.0).abs() < 0.001
}
