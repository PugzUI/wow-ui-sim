use iced::{Point, Rectangle, Size};
use std::borrow::Cow;

use crate::atlas::{AtlasSliceMode, get_atlas_slice_info};
use crate::render::{BlendMode, QuadBatch};

use super::super::slice_render::{
    StretchSliceRender, TextureUvs, TexturedSlice, ThreeSliceRender, TileSliceRender,
    emit_stretch_slice_atlas, emit_three_slice_h_atlas, emit_tile_slice_atlas,
};
use super::super::statusbar::StatusBarFill;
use super::super::texture_gradient::frame_gradient;
use super::super::tiling::{emit_tiled_texture, has_uv_repeat};

/// Build quads for a Texture widget, optionally clipped by a StatusBar fill.
pub(crate) fn build_texture_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    bar_fill: Option<&StatusBarFill>,
    alpha: f32,
) {
    if let Some(ref ns) = f.nine_slice_atlas {
        super::super::nine_slice::emit_nine_slice_atlas(batch, bounds, ns, alpha);
        return;
    }

    let tint = resolve_tint(f, bar_fill, alpha);

    if let Some(color) = f.color_texture {
        let fill_bounds = apply_bar_fill(bounds, bar_fill);
        if let Some(ref grad) = f.gradient {
            push_gradient_quad(batch, fill_bounds, color, grad, tint);
        } else {
            batch.push_solid(
                fill_bounds,
                [
                    color.r * tint[0],
                    color.g * tint[1],
                    color.b * tint[2],
                    color.a * alpha,
                ],
            );
        }
        return;
    }

    let Some(tex_path) = resolve_texture_path(f) else {
        emit_bar_fill_fallback(batch, bar_fill, bounds, alpha);
        return;
    };
    emit_textured_quad(batch, bounds, f, bar_fill, tex_path.as_ref(), tint, alpha);
}

fn resolve_texture_path(f: &crate::widget::Frame) -> Option<Cow<'_, str>> {
    if let Some(path) = f.texture.as_deref() {
        return Some(Cow::Borrowed(path));
    }

    let file_data_id = u32::try_from(f.texture_file_data_id?).ok()?;
    let path = crate::manifest_interface_data::get_texture_path(file_data_id)?;
    Some(Cow::Owned(format!(
        "Interface\\{}",
        path.replace('/', "\\")
    )))
}

/// Compute the vertex color tint from vertex_color and bar fill override.
fn resolve_tint(
    f: &crate::widget::Frame,
    bar_fill: Option<&StatusBarFill>,
    alpha: f32,
) -> [f32; 4] {
    if let Some(fill) = bar_fill
        && let Some(c) = &fill.color
    {
        return [c.r, c.g, c.b, c.a * alpha];
    }
    let vc = f.vertex_color.as_ref();
    [
        vc.map_or(1.0, |c| c.r),
        vc.map_or(1.0, |c| c.g),
        vc.map_or(1.0, |c| c.b),
        vc.map_or(1.0, |c| c.a) * alpha,
    ]
}

/// Emit a solid color quad when no texture path exists but a bar fill has a color.
fn emit_bar_fill_fallback(
    batch: &mut QuadBatch,
    bar_fill: Option<&StatusBarFill>,
    bounds: Rectangle,
    alpha: f32,
) {
    if let Some(fill) = bar_fill
        && let Some(c) = &fill.color
    {
        let fill_bounds = apply_bar_fill(bounds, bar_fill);
        batch.push_solid(fill_bounds, [c.r, c.g, c.b, c.a * alpha]);
    }
}

/// Emit a gradient quad with per-vertex colors (VERTICAL or HORIZONTAL).
fn push_gradient_quad(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    base: crate::widget::Color,
    grad: &crate::widget::Gradient,
    tint: [f32; 4],
) {
    let min = tinted_gradient_color(base, grad.min_color, tint);
    let max = tinted_gradient_color(base, grad.max_color, tint);
    let colors = if grad.vertical {
        [max, max, min, min]
    } else {
        [min, max, max, min]
    };
    batch.push_gradient(bounds, colors);
}

fn tinted_gradient_color(
    base: crate::widget::Color,
    stop: crate::widget::Color,
    tint: [f32; 4],
) -> [f32; 4] {
    [
        base.r * stop.r * tint[0],
        base.g * stop.g * tint[1],
        base.b * stop.b * tint[2],
        base.a * stop.a * tint[3],
    ]
}

/// Emit a textured quad with atlas cropping, three-slice, tiling, rotation, desaturation.
fn emit_textured_quad(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    bar_fill: Option<&StatusBarFill>,
    tex_path: &str,
    tint: [f32; 4],
    alpha: f32,
) {
    if bar_fill.is_none()
        && let Some(uv4) = rotated_quad_uvs(f)
    {
        let sampled_uv4 = if has_out_of_range_uv4(f) {
            uv4.map(|[u, v]| [u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)])
        } else {
            uv4
        };
        let (effective_path, effective_uv4) =
            remap_atlas_crop_uv4(tex_path, sampled_uv4, f.atlas_tex_coords);
        // Sample clamped coordinates, but retain the native logical UVs for
        // ClampToBlack so only the tiny overscan outside the source becomes
        // transparent instead of filling the seam with edge texels.
        let source_uv4 = uv4;
        let vert_before = batch.vertices.len();
        if let Some(gradient) = frame_gradient(f, bounds) {
            if f.clamp_to_black {
                batch.push_textured_path_uv4_clamp_to_black(
                    bounds,
                    effective_uv4,
                    source_uv4,
                    &effective_path,
                    gradient.colors(bounds, tint),
                    f.blend_mode,
                );
            } else {
                batch.push_textured_path_uv4_colors(
                    bounds,
                    effective_uv4,
                    &effective_path,
                    gradient.colors(bounds, tint),
                    f.blend_mode,
                );
            }
        } else if f.clamp_to_black {
            batch.push_textured_path_uv4_clamp_to_black(
                bounds,
                effective_uv4,
                source_uv4,
                &effective_path,
                [tint; 4],
                f.blend_mode,
            );
        } else {
            batch.push_textured_path_uv4(
                bounds,
                effective_uv4,
                &effective_path,
                tint,
                f.blend_mode,
            );
        }
        finalize_textured_quad(batch, vert_before, f);
        return;
    }

    let fallback_tex_coords = if requires_axis_fallback(f) {
        Some((0.0, 1.0, 0.0, 1.0))
    } else {
        f.tex_coords
    };
    let (fill_bounds, fill_uvs) = apply_bar_fill_with_uvs(bounds, fallback_tex_coords, bar_fill);
    let (effective_path, effective_uvs) = remap_atlas_crop(tex_path, fill_uvs, f.atlas_tex_coords);
    let vert_before = batch.vertices.len();
    emit_texture_fill(
        batch,
        fill_bounds,
        effective_uvs,
        &effective_path,
        f,
        tint,
        alpha,
    );
    finalize_textured_quad(batch, vert_before, f);
}

/// Returns the 4 corner UVs (TL, TR, BR, BL) when the frame has an 8-arg
/// SetTexCoord that can't be represented as an axis-aligned rect. Slight
/// near-full overscan is retained for progress-layer corner mapping, while
const UV4_NEAR_FULL_TOLERANCE: f32 = 0.0;

fn has_out_of_range_uv4(f: &crate::widget::Frame) -> bool {
    f.tex_coords_quad
        .is_some_and(|raw| raw.iter().any(|&value| !(0.0..=1.0).contains(&value)))
}

fn requires_axis_fallback(f: &crate::widget::Frame) -> bool {
    let Some(raw) = f.tex_coords_quad else {
        return false;
    };
    has_out_of_range_uv4(f)
        && (!f.clamp_to_black
            || raw.iter().any(|&value| {
                !(-UV4_NEAR_FULL_TOLERANCE..=1.0 + UV4_NEAR_FULL_TOLERANCE).contains(&value)
            }))
}

fn rotated_quad_uvs(f: &crate::widget::Frame) -> Option<[[f32; 2]; 4]> {
    let raw = f.tex_coords_quad?;
    if requires_axis_fallback(f) {
        return None;
    }
    let tl = [raw[0], raw[1]];
    let bl = [raw[2], raw[3]];
    let tr = [raw[4], raw[5]];
    let br = [raw[6], raw[7]];
    let axis_aligned = (tl[0] - bl[0]).abs() < f32::EPSILON
        && (tr[0] - br[0]).abs() < f32::EPSILON
        && (tl[1] - tr[1]).abs() < f32::EPSILON
        && (bl[1] - br[1]).abs() < f32::EPSILON;
    if axis_aligned {
        return None;
    }
    Some([tl, tr, br, bl])
}

const ATLAS_FULL_BOUNDS_TOLERANCE: f32 = 0.001;

/// Apply atlas-slot cropping to 4-corner UVs. Returns the rewritten path
/// (with `@crop:` key when the texture is a sub-region) and corner UVs in
/// [0,1] of the slot's local space.
/// Bounds padded by less than the tolerance remain full texture; larger
/// directional padding is isolated through the crop-normalized path.
fn atlas_bounds_are_full((cl, cr, ct, cb): TextureUvs) -> bool {
    (-ATLAS_FULL_BOUNDS_TOLERANCE..=0.0).contains(&cl)
        && (1.0..=1.0 + ATLAS_FULL_BOUNDS_TOLERANCE).contains(&cr)
        && (-ATLAS_FULL_BOUNDS_TOLERANCE..=0.0).contains(&ct)
        && (1.0..=1.0 + ATLAS_FULL_BOUNDS_TOLERANCE).contains(&cb)
}

fn remap_atlas_crop_uv4(
    tex_path: &str,
    uv4: [[f32; 2]; 4],
    atlas_tex_coords: Option<TextureUvs>,
) -> (String, [[f32; 2]; 4]) {
    let Some((cl, cr, ct, cb)) = atlas_tex_coords else {
        return (tex_path.to_string(), uv4);
    };
    if atlas_bounds_are_full((cl, cr, ct, cb)) {
        return (tex_path.to_string(), uv4);
    }
    let crop_key = format!("{tex_path}@crop:{cl:.6},{cr:.6},{ct:.6},{cb:.6}");
    let cw = cr - cl;
    let ch = cb - ct;
    if cw <= 0.0 || ch <= 0.0 {
        return (crop_key, [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    }
    let remapped = uv4.map(|[u, v]| [(u - cl) / cw, (v - ct) / ch]);
    (crop_key, remapped)
}

fn emit_texture_fill(
    batch: &mut QuadBatch,
    fill_bounds: Rectangle,
    effective_uvs: Option<TextureUvs>,
    effective_path: &str,
    f: &crate::widget::Frame,
    tint: [f32; 4],
    alpha: f32,
) {
    let Some(uvs) = effective_uvs else {
        if let Some(gradient) = frame_gradient(f, fill_bounds) {
            batch.push_textured_path_colors(
                fill_bounds,
                &effective_path,
                gradient.colors(fill_bounds, tint),
                f.blend_mode,
            );
        } else {
            batch.push_textured_path(fill_bounds, &effective_path, tint, f.blend_mode);
        }
        return;
    };

    let texture = TexturedSlice {
        path: effective_path,
        uvs,
        tint,
        blend: f.blend_mode,
        gradient: frame_gradient(f, fill_bounds),
    };

    if emit_specialized_textured_fill(batch, fill_bounds, f, texture) {
        return;
    }

    emit_basic_textured_fill(batch, fill_bounds, texture, f, alpha);
}

fn emit_specialized_textured_fill(
    batch: &mut QuadBatch,
    fill_bounds: Rectangle,
    f: &crate::widget::Frame,
    texture: TexturedSlice<'_>,
) -> bool {
    if let Some(render) = stretch_slice_render(f, fill_bounds, texture) {
        emit_stretch_slice_atlas(batch, render);
        return true;
    }

    if let Some(render) = tile_slice_render(f, fill_bounds, texture) {
        emit_tile_slice_atlas(batch, render);
        return true;
    }

    if let Some(render) = three_slice_render(f, fill_bounds, texture) {
        emit_three_slice_h_atlas(batch, render);
        return true;
    }

    false
}

fn stretch_slice_render<'a>(
    f: &crate::widget::Frame,
    bounds: Rectangle,
    texture: TexturedSlice<'a>,
) -> Option<StretchSliceRender<'a>> {
    let atlas_name = f.atlas.as_deref()?;
    let slice = get_atlas_slice_info(atlas_name)?;
    if slice.mode != AtlasSliceMode::Stretch {
        return None;
    }

    if bounds.width <= (slice.left + slice.right) as f32
        || bounds.height <= (slice.top + slice.bottom) as f32
    {
        return None;
    }

    let atlas_info = crate::atlas::get_atlas_info(atlas_name)?;
    Some(StretchSliceRender {
        bounds,
        texture,
        left_px: slice.left as f32,
        top_px: slice.top as f32,
        right_px: slice.right as f32,
        bottom_px: slice.bottom as f32,
        atlas_width_px: atlas_info.width() as f32,
        atlas_height_px: atlas_info.height() as f32,
    })
}

fn tile_slice_render<'a>(
    f: &crate::widget::Frame,
    bounds: Rectangle,
    texture: TexturedSlice<'a>,
) -> Option<TileSliceRender<'a>> {
    let atlas_name = f.atlas.as_deref()?;
    let slice = get_atlas_slice_info(atlas_name)?;
    if slice.mode != AtlasSliceMode::Tile {
        return None;
    }

    if bounds.width < (slice.left + slice.right) as f32
        || bounds.height < (slice.top + slice.bottom) as f32
    {
        return None;
    }

    let atlas_info = crate::atlas::get_atlas_info(atlas_name)?;
    Some(TileSliceRender {
        bounds,
        texture,
        left_px: slice.left as f32,
        top_px: slice.top as f32,
        right_px: slice.right as f32,
        bottom_px: slice.bottom as f32,
        atlas_width_px: atlas_info.width() as f32,
        atlas_height_px: atlas_info.height() as f32,
    })
}

fn three_slice_render<'a>(
    f: &crate::widget::Frame,
    bounds: Rectangle,
    texture: TexturedSlice<'a>,
) -> Option<ThreeSliceRender<'a>> {
    let (left_cap_px, right_cap_px, atlas_width_px) = f.three_slice_h?;
    if bounds.width <= left_cap_px + right_cap_px {
        return None;
    }

    Some(ThreeSliceRender {
        bounds,
        texture,
        left_cap_px,
        right_cap_px,
        atlas_width_px,
    })
}

fn emit_basic_textured_fill(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    texture: TexturedSlice<'_>,
    f: &crate::widget::Frame,
    alpha: f32,
) {
    let uvs = uv_rect(texture.uvs);
    if !f.clamp_to_black && (f.horiz_tile || f.vert_tile || has_uv_repeat(f)) {
        emit_tiled_texture(batch, bounds, &uvs, texture.path, f, alpha);
        return;
    }

    let vert_before = batch.vertices.len();
    if let Some(gradient) = texture.gradient {
        batch.push_textured_path_uv_colors(
            bounds,
            uvs,
            texture.path,
            gradient.colors(bounds, texture.tint),
            texture.blend,
        );
    } else if f.clamp_to_black {
        let source_uvs = clamp_source_uv_rect(f, uvs);
        batch.push_textured_path_uv_clamp_to_black(
            bounds,
            uvs,
            source_uvs,
            texture.path,
            texture.tint,
            texture.blend,
        );
    } else {
        batch.push_textured_path_uv(bounds, uvs, texture.path, texture.tint, texture.blend);
    }
    if f.clamp_to_black && texture.gradient.is_some() {
        let source_uvs = clamp_source_uv_rect(f, uvs);
        let source_corners = [
            [source_uvs.x, source_uvs.y],
            [source_uvs.x + source_uvs.width, source_uvs.y],
            [
                source_uvs.x + source_uvs.width,
                source_uvs.y + source_uvs.height,
            ],
            [source_uvs.x, source_uvs.y + source_uvs.height],
        ];
        for (index, vertex) in batch.vertices[vert_before..].iter_mut().enumerate() {
            vertex.source_uv = source_corners[index % 4];
        }
    }
}

fn uv_rect((left, right, top, bottom): TextureUvs) -> Rectangle {
    Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top))
}
/// Tiny progress-texture overscan is a sampling guard, not visible content.
/// Treating it as a shader clip boundary makes the two rasterized triangles
/// disagree near their shared diagonal when their interpolated UVs re-enter
/// the valid range. Keep the sample UVs unchanged, but avoid carrying that
/// sub-pixel boundary into the fragment alpha test.
const CLAMP_NEAR_FULL_TOLERANCE: f32 = 0.002;

fn near_full_clamp_source_uv(f: &crate::widget::Frame) -> bool {
    let Some((left, right, top, bottom)) = f.local_tex_coords else {
        return false;
    };
    left >= -CLAMP_NEAR_FULL_TOLERANCE
        && right <= 1.0 + CLAMP_NEAR_FULL_TOLERANCE
        && top >= -CLAMP_NEAR_FULL_TOLERANCE
        && bottom <= 1.0 + CLAMP_NEAR_FULL_TOLERANCE
}

/// Preserve the logical UV boundary for ordinary clamp textures. A rejected
/// out-of-range UV4 is rendered through the axis-aligned fallback, so its
/// invalid source coordinates must not reintroduce diagonal shader clipping.
fn clamp_source_uv_rect(f: &crate::widget::Frame, fallback: Rectangle) -> Rectangle {
    if requires_axis_fallback(f) || near_full_clamp_source_uv(f) {
        return Rectangle::new(Point::ORIGIN, Size::new(1.0, 1.0));
    }
    f.local_tex_coords.map(uv_rect).unwrap_or(fallback)
}
fn finalize_textured_quad(batch: &mut QuadBatch, vert_before: usize, f: &crate::widget::Frame) {
    let rejected_uv4 = requires_axis_fallback(f);
    if !rejected_uv4 && f.rotation != 0.0 {
        apply_uv_rotation(batch, vert_before, f.rotation);
    }
    if !rejected_uv4 {
        apply_vertex_offsets(batch, vert_before, f.vertex_offsets);
    }
    if f.clamp_to_black {
        // Axis-aligned clamp quads preserve the raw logical local coordinates
        // for the boundary test. Rotated quads already carry their raw
        // per-corner coordinates and retain unit local UVs for geometry.
        if !rejected_uv4
            && f.tex_coords_quad.is_none()
            && f.rotation == 0.0
            && let Some((left, right, top, bottom)) = f.local_tex_coords
        {
            let local_uvs = [[left, top], [right, top], [right, bottom], [left, bottom]];
            for (vertex, local_uv) in batch.vertices[vert_before..].iter_mut().zip(local_uvs) {
                vertex.local_uv = local_uv;
            }
        }
        for vertex in &mut batch.vertices[vert_before..] {
            vertex.flags |= crate::render::shader::FLAG_CLAMP_TO_BLACK;
        }
    }
    if f.desaturated {
        apply_desaturate_flag(batch, vert_before);
    }
}

/// Apply WoW's per-corner UI offsets to emitted textured quads.
///
/// Native SetVertexOffset indices are UL, LL, UR, LR (1..4), while a
/// QuadBatch quad is emitted TL, TR, BR, BL. UI Y grows upward, so screen
/// coordinates invert the offset's Y component.
fn apply_vertex_offsets(
    batch: &mut QuadBatch,
    vert_before: usize,
    offsets: Option<[(f32, f32); 4]>,
) {
    let Some(offsets) = offsets else {
        return;
    };
    if offsets.iter().all(|&(x, y)| x == 0.0 && y == 0.0) {
        return;
    }
    // Native storage [TL, BL, TR, BR] -> batch order [TL, TR, BR, BL].
    const NATIVE_TO_BATCH: [usize; 4] = [0, 2, 3, 1];
    for quad in batch.vertices[vert_before..].chunks_exact_mut(4) {
        for (batch_vertex, native_index) in quad.iter_mut().zip(NATIVE_TO_BATCH) {
            let (dx, dy) = offsets[native_index];
            batch_vertex.position[0] += dx;
            batch_vertex.position[1] -= dy;
        }
    }
}

/// Apply StatusBar fill clipping to bounds.
fn apply_bar_fill(bounds: Rectangle, bar_fill: Option<&StatusBarFill>) -> Rectangle {
    let Some(fill) = bar_fill else { return bounds };
    let fill_width = bounds.width * fill.fraction;
    if fill.reverse {
        Rectangle::new(
            Point::new(bounds.x + bounds.width - fill_width, bounds.y),
            Size::new(fill_width, bounds.height),
        )
    } else {
        Rectangle::new(bounds.position(), Size::new(fill_width, bounds.height))
    }
}

/// Remap atlas sub-region textures: encode crop coords in path, remap UVs to [0,1].
pub(super) fn remap_atlas_crop(
    tex_path: &str,
    fill_uvs: Option<TextureUvs>,
    atlas_tex_coords: Option<TextureUvs>,
) -> (String, Option<TextureUvs>) {
    let Some((cl, cr, ct, cb)) = atlas_tex_coords else {
        return (tex_path.to_string(), fill_uvs);
    };
    if atlas_bounds_are_full((cl, cr, ct, cb)) {
        return (tex_path.to_string(), fill_uvs);
    }

    let crop_key = format!("{tex_path}@crop:{cl:.6},{cr:.6},{ct:.6},{cb:.6}");
    let remapped_uvs = fill_uvs.map(|(fl, fr, ft, fb)| {
        let cw = cr - cl;
        let ch = cb - ct;
        if cw <= 0.0 || ch <= 0.0 {
            return (0.0, 1.0, 0.0, 1.0);
        }
        (
            (fl - cl) / cw,
            (fr - cl) / cw,
            (ft - ct) / ch,
            (fb - ct) / ch,
        )
    });

    (crop_key, remapped_uvs)
}

/// Apply StatusBar fill clipping to bounds and UV coordinates.
fn apply_bar_fill_with_uvs(
    bounds: Rectangle,
    tex_coords: Option<TextureUvs>,
    bar_fill: Option<&StatusBarFill>,
) -> (Rectangle, Option<TextureUvs>) {
    let Some(fill) = bar_fill else {
        return (bounds, tex_coords);
    };
    let fill_bounds = apply_bar_fill(bounds, bar_fill);
    let (uv_left, uv_right, uv_top, uv_bottom) = tex_coords.unwrap_or((0.0, 1.0, 0.0, 1.0));
    let uv_range = uv_right - uv_left;
    let fill_uvs = if fill.reverse {
        (
            uv_left + uv_range * (1.0 - fill.fraction),
            uv_right,
            uv_top,
            uv_bottom,
        )
    } else {
        (
            uv_left,
            uv_left + uv_range * fill.fraction,
            uv_top,
            uv_bottom,
        )
    };
    (fill_bounds, Some(fill_uvs))
}

/// Rotate texture UV coordinates around their center for vertices added after `vert_before`.
fn apply_uv_rotation(batch: &mut QuadBatch, vert_before: usize, radians: f32) {
    let verts = &mut batch.vertices[vert_before..];
    if verts.len() < 4 {
        return;
    }
    let (sin_r, cos_r) = radians.sin_cos();
    for chunk in verts.chunks_exact_mut(4) {
        let cx = (chunk[0].tex_coords[0]
            + chunk[1].tex_coords[0]
            + chunk[2].tex_coords[0]
            + chunk[3].tex_coords[0])
            * 0.25;
        let cy = (chunk[0].tex_coords[1]
            + chunk[1].tex_coords[1]
            + chunk[2].tex_coords[1]
            + chunk[3].tex_coords[1])
            * 0.25;
        for v in chunk.iter_mut() {
            let du = v.tex_coords[0] - cx;
            let dv = v.tex_coords[1] - cy;
            v.tex_coords[0] = cx + du * cos_r - dv * sin_r;
            v.tex_coords[1] = cy + du * sin_r + dv * cos_r;
        }
    }
}

/// Apply the desaturation flag to vertices added after `vert_before`.
fn apply_desaturate_flag(batch: &mut QuadBatch, vert_before: usize) {
    use crate::render::shader::FLAG_DESATURATE;
    for v in &mut batch.vertices[vert_before..] {
        v.flags |= FLAG_DESATURATE;
    }
}

const DEFAULT_MINIMAP_MASK_TEXTURE: &str = r"Interface\HUD\UIMinimapMask";

/// Build quads for a Minimap widget - map texture clipped by the active minimap mask.
pub(crate) fn build_minimap_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    alpha: f32,
) {
    let vert_before = batch.vertices.len();
    batch.push_textured_path(
        bounds,
        r"Interface\AddOns\SimCommands\textures\minimap-placeholder",
        [1.0, 1.0, 1.0, alpha],
        BlendMode::Alpha,
    );
    let mask_texture = f
        .minimap_mask_texture
        .as_deref()
        .unwrap_or(DEFAULT_MINIMAP_MASK_TEXTURE);
    crate::iced_app::masking::apply_mask_path(batch, vert_before, bounds, mask_texture);
}
#[cfg(test)]
mod tests {
    use super::{
        BlendMode, DEFAULT_MINIMAP_MASK_TEXTURE, TexturedSlice, apply_vertex_offsets,
        build_minimap_quads, build_texture_quads, emit_texture_fill, remap_atlas_crop,
        remap_atlas_crop_uv4, stretch_slice_render, tile_slice_render,
    };
    use crate::atlas::get_render_atlas_info;
    use crate::iced_app::slice_render::{tile_slice_center_height, tile_slice_center_width};
    use crate::render::QuadBatch;
    use crate::render::shader::FLAG_CLAMP_TO_BLACK;
    use crate::widget::{Color, Frame, Gradient, WidgetType};
    use iced::{Point, Rectangle, Size};

    #[test]
    fn vertex_offsets_follow_native_corners_and_invert_screen_y() {
        let mut batch = QuadBatch::new();
        batch.push_solid(
            Rectangle::new(Point::new(10.0, 20.0), Size::new(30.0, 40.0)),
            [1.0; 4],
        );
        let original = batch
            .vertices
            .iter()
            .map(|v| v.position)
            .collect::<Vec<_>>();
        apply_vertex_offsets(
            &mut batch,
            0,
            Some([(1.0, 2.0), (3.0, 4.0), (5.0, 6.0), (7.0, 8.0)]),
        );
        assert_eq!(
            batch
                .vertices
                .iter()
                .map(|v| v.position)
                .collect::<Vec<_>>(),
            vec![[11.0, 18.0], [45.0, 14.0], [47.0, 52.0], [13.0, 56.0]],
        );

        let mut unchanged = QuadBatch::new();
        unchanged.push_solid(
            Rectangle::new(Point::new(10.0, 20.0), Size::new(30.0, 40.0)),
            [1.0; 4],
        );
        apply_vertex_offsets(&mut unchanged, 0, None);
        assert_eq!(
            unchanged
                .vertices
                .iter()
                .map(|v| v.position)
                .collect::<Vec<_>>(),
            original,
        );
    }

    fn texture_frame_with_atlas(name: &str) -> Frame {
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.atlas = Some(name.to_string());
        frame
    }

    fn render_texture_slice(name: &str) -> TexturedSlice<'static> {
        let lookup = get_render_atlas_info(name)
            .unwrap_or_else(|| panic!("missing render atlas info for {name}"));
        TexturedSlice {
            path: lookup.info.file,
            uvs: (
                lookup.info.left_tex_coord,
                lookup.info.right_tex_coord,
                lookup.info.top_tex_coord,
                lookup.info.bottom_tex_coord,
            ),
            tint: [1.0, 1.0, 1.0, 1.0],
            blend: BlendMode::Alpha,
            gradient: None,
        }
    }
    #[test]
    fn clamp_axis_crop_preserves_logical_source_uvs_for_shader_boundary() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.texture = Some(r"Interface\Textures\spinner".to_string());
        frame.clamp_to_black = true;
        frame.atlas_tex_coords = Some((0.25, 0.75, 0.125, 0.875));
        frame.local_tex_coords = Some((-0.001489, 1.001489, -0.001489, 1.001489));
        frame.tex_coords = Some((0.25, 0.75, 0.125, 0.875));

        build_texture_quads(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(200.0, 200.0)),
            &frame,
            None,
            1.0,
        );

        assert_eq!(batch.vertices.len(), 4);
        assert_eq!(
            batch
                .vertices
                .iter()
                .map(|v| v.position)
                .collect::<Vec<_>>(),
            vec![[0.0, 0.0], [200.0, 0.0], [200.0, 200.0], [0.0, 200.0],]
        );
        assert!(
            batch
                .vertices
                .iter()
                .all(|v| v.flags & FLAG_CLAMP_TO_BLACK != 0)
        );
        let expected_local_uvs = [
            [-0.001489, -0.001489],
            [1.001489, -0.001489],
            [1.001489, 1.001489],
            [-0.001489, 1.001489],
        ];
        assert_eq!(
            batch
                .vertices
                .iter()
                .map(|v| v.local_uv)
                .collect::<Vec<_>>(),
            expected_local_uvs
        );
        let expected_samples = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert_eq!(
            batch
                .vertices
                .iter()
                .map(|v| v.tex_coords)
                .collect::<Vec<_>>(),
            expected_samples
        );
        let expected_source_uvs = [
            [-0.001489, -0.001489],
            [1.001489, -0.001489],
            [1.001489, 1.001489],
            [-0.001489, 1.001489],
        ];
        assert_eq!(
            batch
                .vertices
                .iter()
                .map(|v| v.source_uv)
                .collect::<Vec<_>>(),
            expected_source_uvs
        );
        assert!(
            batch.texture_requests[0]
                .path
                .ends_with("@crop:0.250000,0.750000,0.125000,0.875000")
        );

        // Equivalent to the fragment shader's ClampToBlack boundary test.
        let shader_accepts =
            |uv: [f32; 2]| uv[0] >= 0.0 && uv[0] <= 1.0 && uv[1] >= 0.0 && uv[1] <= 1.0;
        assert!(!shader_accepts(batch.vertices[0].local_uv));
        assert!(shader_accepts([0.5, 0.5]));
        assert!(!shader_accepts(batch.vertices[0].source_uv));
        assert!(shader_accepts([0.5, 0.5]));
    }

    #[test]
    fn out_of_range_rotated_uvs_with_clamp_keep_axis_aligned_fallback() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.texture = Some(r"Interface\Textures\spinner".to_string());
        frame.clamp_to_black = true;
        frame.atlas_tex_coords = Some((0.25, 0.75, 0.125, 0.875));
        // SetTexCoord's native order is UL, LL, UR, LR; these coordinates
        // are outside the local domain and must use the safe fallback.
        let raw = [-0.2, 0.1, 0.1, 1.2, 1.2, -0.1, 0.8, 1.1];
        frame.local_tex_coords = Some((-0.2, 1.2, -0.1, 1.2));
        frame.tex_coords = Some((0.15, 0.85, 0.05, 1.025));
        frame.tex_coords_quad = Some(raw);
        frame.vertex_offsets = Some([(20.0, 30.0), (-20.0, 30.0), (-20.0, -30.0), (20.0, -30.0)]);

        build_texture_quads(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0)),
            &frame,
            None,
            1.0,
        );

        assert_eq!(batch.vertices.len(), 4);
        assert_eq!(batch.quad_count(), 1);
        assert_eq!(
            batch
                .vertices
                .iter()
                .map(|v| v.position)
                .collect::<Vec<_>>(),
            vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0],]
        );
        assert!(
            batch
                .vertices
                .iter()
                .all(|v| v.flags & FLAG_CLAMP_TO_BLACK != 0)
        );
        assert_eq!(
            batch
                .vertices
                .iter()
                .map(|v| v.tex_coords)
                .collect::<Vec<_>>(),
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        );
        assert_eq!(
            batch
                .vertices
                .iter()
                .map(|v| v.local_uv)
                .collect::<Vec<_>>(),
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        );
        assert_eq!(
            batch
                .vertices
                .iter()
                .map(|v| v.source_uv)
                .collect::<Vec<_>>(),
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        );
        assert_eq!(
            batch.texture_requests[0].path,
            r"Interface\Textures\spinner@crop:0.250000,0.750000,0.125000,0.875000"
        );
    }

    #[test]
    fn out_of_range_rotated_uvs_without_clamp_keep_axis_aligned_fallback() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.texture = Some(r"Interface\Textures\spinner".to_string());
        frame.tex_coords_quad = Some([-0.2, 0.1, 0.1, 1.2, 1.2, -0.1, 0.8, 1.1]);

        build_texture_quads(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0)),
            &frame,
            None,
            1.0,
        );

        assert_eq!(batch.vertices.len(), 4);
        assert_eq!(batch.vertices[0].tex_coords, [0.0, 0.0]);
        assert_eq!(batch.vertices[2].tex_coords, [1.0, 1.0]);
        assert!(
            batch
                .vertices
                .iter()
                .all(|v| v.flags & FLAG_CLAMP_TO_BLACK == 0)
        );
        assert!(batch.vertices.iter().all(|v| v.local_uv[0].abs() <= 1.0));
    }

    #[test]
    fn minimap_uses_default_mask_texture() {
        let mut batch = QuadBatch::new();
        let frame = Frame::new(WidgetType::Minimap, Some("Minimap".to_string()), None);
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(140.0, 140.0));

        build_minimap_quads(&mut batch, bounds, &frame, 1.0);

        assert_eq!(batch.mask_texture_requests.len(), 1);
        assert_eq!(
            batch.mask_texture_requests[0].path,
            DEFAULT_MINIMAP_MASK_TEXTURE
        );
        assert!(
            batch
                .vertices
                .iter()
                .all(|vertex| vertex.mask_tex_index == -2)
        );
    }

    #[test]
    fn minimap_respects_set_mask_texture_state() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Minimap, Some("Minimap".to_string()), None);
        frame.minimap_mask_texture = Some(r"Interface\CharacterFrame\TempPortraitAlphaMask".into());
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(140.0, 140.0));

        build_minimap_quads(&mut batch, bounds, &frame, 1.0);

        assert_eq!(batch.mask_texture_requests.len(), 1);
        assert_eq!(
            batch.mask_texture_requests[0].path,
            r"Interface\CharacterFrame\TempPortraitAlphaMask"
        );
    }

    #[test]
    fn textured_gradient_emits_per_vertex_colors() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.texture = Some(r"Interface\Textures\gradient".to_string());
        frame.gradient = Some(Gradient {
            vertical: true,
            min_color: Color::new(0.1, 0.2, 0.3, 0.4),
            max_color: Color::new(0.8, 0.7, 0.6, 0.9),
        });

        build_texture_quads(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(100.0, 40.0)),
            &frame,
            None,
            1.0,
        );

        assert_eq!(batch.vertices.len(), 4);
        assert_eq!(batch.texture_requests.len(), 1);
        assert_eq!(batch.vertices[0].color, [0.8, 0.7, 0.6, 0.9]);
        assert_eq!(batch.vertices[1].color, [0.8, 0.7, 0.6, 0.9]);
        assert_eq!(batch.vertices[2].color, [0.1, 0.2, 0.3, 0.4]);
        assert_eq!(batch.vertices[3].color, [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn tiled_textured_gradient_stays_continuous_across_tiles() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.texture = Some(r"Interface\Textures\tiled-gradient".to_string());
        frame.tex_coords = Some((0.0, 1.0, 0.0, 1.0));
        frame.horiz_tile = true;
        frame.width = 25.0;
        frame.height = 20.0;
        frame.gradient = Some(Gradient {
            vertical: false,
            min_color: Color::rgb(0.1, 0.1, 0.1),
            max_color: Color::rgb(0.9, 0.9, 0.9),
        });

        build_texture_quads(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(50.0, 20.0)),
            &frame,
            None,
            1.0,
        );

        assert_eq!(batch.vertices.len(), 8);
        assert_eq!(batch.texture_requests.len(), 2);
        assert!((batch.vertices[1].color[0] - 0.5).abs() < 0.0001);
        assert!((batch.vertices[4].color[0] - 0.5).abs() < 0.0001);
        assert_eq!(batch.vertices[6].color, [0.9, 0.9, 0.9, 1.0]);
    }

    #[test]
    fn remap_atlas_crop_rewrites_subregion_to_crop_key() {
        let (path, uvs) = remap_atlas_crop(
            r"Interface\Glues\CharacterSelect\Glues-AddOn-Icons",
            Some((0.25, 0.5, 0.125, 0.625)),
            Some((0.25, 0.5, 0.125, 0.625)),
        );

        assert_eq!(
            path,
            r"Interface\Glues\CharacterSelect\Glues-AddOn-Icons@crop:0.250000,0.500000,0.125000,0.625000"
        );
        assert_eq!(uvs, Some((0.0, 1.0, 0.0, 1.0)));
    }

    #[test]
    fn remap_atlas_crop_isolates_aura9_near_full_bounds() {
        let raw_uvs = Some((-1.224634, 2.224634, -1.224634, 2.224634));
        let bounds = (-0.001489, 1.0, -0.001489, 1.0);
        let (path, uvs) = remap_atlas_crop(r"Interface\Textures\Aura9", raw_uvs, Some(bounds));

        assert_eq!(
            path,
            r"Interface\Textures\Aura9@crop:-0.001489,1.000000,-0.001489,1.000000"
        );
        let Some((left, right, top, bottom)) = uvs else {
            panic!("near-full bounds must preserve remapped UVs");
        };
        let cw = bounds.1 - bounds.0;
        let ch = bounds.3 - bounds.2;
        assert!((left - (raw_uvs.unwrap().0 - bounds.0) / cw).abs() < 1e-6);
        assert!((right - (raw_uvs.unwrap().1 - bounds.0) / cw).abs() < 1e-6);
        assert!((top - (raw_uvs.unwrap().2 - bounds.2) / ch).abs() < 1e-6);
        assert!((bottom - (raw_uvs.unwrap().3 - bounds.2) / ch).abs() < 1e-6);
    }

    #[test]
    fn remap_atlas_crop_uv4_isolates_aura9_near_full_bounds() {
        let raw_uvs = [
            [-1.224634, -1.224634],
            [2.224634, -1.224634],
            [2.224634, 2.224634],
            [-1.224634, 2.224634],
        ];
        let bounds = (-0.001489, 1.0, -0.001489, 1.0);
        let (path, uvs) = remap_atlas_crop_uv4(r"Interface\Textures\Aura9", raw_uvs, Some(bounds));

        assert!(path.ends_with("@crop:-0.001489,1.000000,-0.001489,1.000000"));
        let cw = bounds.1 - bounds.0;
        let ch = bounds.3 - bounds.2;
        for (actual, [u, v]) in uvs.iter().zip(raw_uvs) {
            assert!((actual[0] - (u - bounds.0) / cw).abs() < 1e-6);
            assert!((actual[1] - (v - bounds.2) / ch).abs() < 1e-6);
        }
    }

    #[test]
    fn remap_atlas_crop_keeps_inset_and_outside_padding_cropped() {
        let inset = remap_atlas_crop(
            "atlas",
            Some((0.0, 1.0, 0.0, 1.0)),
            Some((0.001, 0.999, 0.001, 0.999)),
        );
        assert!(inset.0.contains("@crop:"));

        let outside = remap_atlas_crop(
            "atlas",
            Some((0.0, 1.0, 0.0, 1.0)),
            Some((-0.001501, 1.001501, -0.001501, 1.001501)),
        );
        assert!(outside.0.contains("@crop:"));
    }

    #[test]
    fn remap_atlas_crop_uv4_keeps_inset_and_outside_padding_cropped() {
        let raw_uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let inset = remap_atlas_crop_uv4("atlas", raw_uvs, Some((0.001, 0.999, 0.001, 0.999)));
        assert!(inset.0.contains("@crop:"));

        let outside = remap_atlas_crop_uv4(
            "atlas",
            raw_uvs,
            Some((-0.001501, 1.001501, -0.001501, 1.001501)),
        );
        assert!(outside.0.contains("@crop:"));
    }
    #[test]
    fn stretch_atlas_slices_emit_nine_quads() {
        let mut batch = QuadBatch::new();
        let mut frame = texture_frame_with_atlas("common-button-tertiary-normal");
        frame.gradient = Some(Gradient {
            vertical: false,
            min_color: Color::rgb(0.1, 0.1, 0.1),
            max_color: Color::rgb(0.9, 0.9, 0.9),
        });

        emit_texture_fill(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(160.0, 32.0)),
            Some((0.0, 1.0, 0.0, 1.0)),
            "stretch-path",
            &frame,
            [1.0, 1.0, 1.0, 1.0],
            1.0,
        );

        assert_eq!(batch.vertices.len(), 36);
        assert_eq!(batch.texture_requests.len(), 9);
        assert_eq!(batch.vertices[0].color, [0.1, 0.1, 0.1, 1.0]);
        assert_eq!(batch.vertices[33].color, [0.9, 0.9, 0.9, 1.0]);
    }

    #[test]
    fn tile_atlas_slices_collapse_unit_repeat_regions() {
        let mut batch = QuadBatch::new();
        let frame = texture_frame_with_atlas("questlog-frame");

        emit_texture_fill(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(120.0, 120.0)),
            Some((0.0, 1.0, 0.0, 1.0)),
            "tile-path@crop:0.001953,0.210938,0.076172,0.285156",
            &frame,
            [1.0, 1.0, 1.0, 1.0],
            1.0,
        );

        assert_eq!(batch.vertices.len(), 36);
        assert_eq!(batch.texture_requests.len(), 9);
        assert!(
            batch
                .texture_requests
                .iter()
                .all(|request| request.path.matches("@crop:").count() == 1),
            "tile atlas slices should flatten crop paths, got: {:?}",
            batch.texture_requests
        );
    }

    #[test]
    fn tile_slice_render_uses_logical_dimensions_for_2x_fallback_atlas() {
        let frame = texture_frame_with_atlas("questlog-frame");
        let texture = render_texture_slice("questlog-frame");
        let render = tile_slice_render(
            &frame,
            Rectangle::new(Point::ORIGIN, Size::new(314.0, 436.0)),
            texture,
        )
        .expect("questlog-frame should use tile slice rendering");

        assert_eq!(render.atlas_width_px, 107.0);
        assert_eq!(render.atlas_height_px, 107.0);
        assert_eq!(tile_slice_center_width(render), Some((208.0, 1.0)));
        assert_eq!(tile_slice_center_height(render), Some((330.0, 1.0)));
    }

    #[test]
    fn uv_repeat_texcoords_emit_tiled_quads_without_tile_flags() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.texture = Some(r"Interface\AddOns\Details\images\background".to_string());
        frame.tex_coords = Some((0.0, 2.109, 0.0, 0.872));
        frame.tex_coords_quad = Some([0.0, 0.0, 0.0, 0.872, 2.109, 0.0, 2.109, 0.872]);

        build_texture_quads(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(270.0, 112.0)),
            &frame,
            None,
            1.0,
        );

        assert_eq!(batch.vertices.len(), 12);
        assert_eq!(batch.texture_requests.len(), 3);
        assert!(
            batch
                .vertices
                .iter()
                .all(|vertex| vertex.local_uv[0] <= 1.0 && vertex.local_uv[1] <= 1.0),
            "UV-repeat quads must not sample beyond their atlas slot: {:?}",
            batch.vertices
        );
        assert_eq!(batch.vertices[1].position[0], 128.02277);
        assert_eq!(batch.vertices[5].position[0], 256.04553);
        assert_eq!(batch.vertices[9].position[0], 270.0);
    }

    #[test]
    fn stretch_slice_render_uses_logical_dimensions_for_2x_fallback_atlas() {
        let frame = texture_frame_with_atlas("common-button-tertiary-normal");
        let texture = render_texture_slice("common-button-tertiary-normal");
        let render = stretch_slice_render(
            &frame,
            Rectangle::new(Point::ORIGIN, Size::new(160.0, 32.0)),
            texture,
        )
        .expect("common-button-tertiary-normal should use stretch slice rendering");

        assert_eq!(render.atlas_width_px, 46.0);
        assert_eq!(render.atlas_height_px, 34.0);
    }
}
