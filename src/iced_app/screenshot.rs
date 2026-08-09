//! IPC screenshot rendering for the running app.

use std::collections::HashSet;
use std::path::Path;

use crate::iced_app::frame_collect::SCALPEL_VISUALIZER_ROOT;
use crate::lua_api::WowLuaEnv;
use crate::lua_server::Response as LuaResponse;
use crate::render::GlyphAtlas;
use crate::render::headless::render_to_image;

use super::app::App;
const CROP_FORMAT_EXAMPLE: &str = "700x150+400+650";

impl App {
    /// Render a screenshot from the live app state and save to disk.
    pub(crate) fn render_screenshot(
        &mut self,
        output: &str,
        width: u32,
        height: u32,
        ui_scale: Option<f32>,
        filter: Option<&str>,
        crop: Option<&str>,
        manifest: Option<&str>,
        requested_ids: &[String],
    ) -> LuaResponse {
        if let Err(error) = self.configure_screenshot_runtime(width, height, ui_scale) {
            return LuaResponse::Error(error);
        }
        let output_path = Path::new(output).to_path_buf();
        let (batch, mut glyph_atlas) = self.build_screenshot_batch(width, height, filter);
        let glyph_data = glyph_atlas_data(&mut glyph_atlas);
        let mut tex_mgr = self.texture_manager.borrow_mut();
        let img = render_to_image(&batch, &mut tex_mgr, width, height, glyph_data);
        let img = match maybe_crop_image(img, crop) {
            Ok(img) => img,
            Err(e) => return LuaResponse::Error(e),
        };

        if let Err(e) = save_screenshot(&img, &output_path) {
            return LuaResponse::Error(format!("Failed to save screenshot: {}", e));
        }
        if let Some(manifest) = manifest {
            write_visualizer_manifest(
                &self.env.borrow(),
                manifest,
                width,
                height,
                filter,
                requested_ids,
            );
        }

        LuaResponse::Output(format_screenshot_saved_message(
            &img,
            &output_path,
            width,
            height,
            crop.is_some(),
        ))
    }

    pub(super) fn configure_screenshot_runtime(
        &mut self,
        width: u32,
        height: u32,
        ui_scale: Option<f32>,
    ) -> Result<(), String> {
        let size = iced::Size::new(width as f32, height as f32);
        self.screen_size.set(size);
        {
            let env = self.env.borrow();
            if let Some(scale) = ui_scale {
                env.set_ui_scale(scale).map_err(|error| error.to_string())?;
            }
            env.set_screen_size(size.width, size.height);
        }
        *self.cached_hittable.borrow_mut() = None;
        self.mark_all_strata_dirty();
        self.flush_post_script_updates();
        Ok(())
    }

    fn build_screenshot_batch(
        &self,
        width: u32,
        height: u32,
        filter: Option<&str>,
    ) -> (crate::render::QuadBatch, GlyphAtlas) {
        let mut glyph_atlas = GlyphAtlas::new();
        let batch = {
            let env = self.env.borrow();
            let mut fs = self.font_system.borrow_mut();
            let buckets = {
                let mut state = env.state().borrow_mut();
                super::tooltip::update_tooltip_sizes(&mut state, &mut fs);
                state.ensure_layout_rects();
                let _ = state.get_strata_buckets();
                state.strata_buckets.as_ref().unwrap().clone()
            };
            let state = env.state().borrow();
            let tooltip_data = super::tooltip::collect_tooltip_data(&state);
            super::build_quad_batch_for_registry_with_quest_blobs(
                super::RegistryQuadBatchParams::new(
                    &state.widgets,
                    (width as f32, height as f32),
                    &buckets,
                )
                .root_name(filter)
                .pressed_frame(self.pressed_frame)
                .hovered_frame(self.hovered_frame)
                .text_ctx(Some((&mut fs, &mut glyph_atlas)))
                .message_frames(Some(&state.message_frames))
                .tooltip_data(Some(&tooltip_data))
                .quest_blobs(Some(&state.quest_blobs)),
            )
        };
        (batch, glyph_atlas)
    }
}

pub fn write_visualizer_manifest(
    env: &WowLuaEnv,
    path: &str,
    width: u32,
    height: u32,
    filter: Option<&str>,
    requested_ids: &[String],
) {
    let stage = env
        .runtime_screen_metrics()
        .ok()
        .and_then(|metrics| serde_json::to_value(metrics).ok())
        .unwrap_or_else(|| {
            serde_json::json!({
                "width": width,
                "height": height,
                "probe_error": "runtime screen metrics unavailable"
            })
        });
    let (resolved, unresolved_requested_ids) =
        resolve_requested_weakauras_regions(env, requested_ids);
    let state = env.state().borrow();
    let visualizer_filter = filter == Some(SCALPEL_VISUALIZER_ROOT);
    let needle = filter.map(str::to_ascii_lowercase);
    let mut regions = Vec::new();
    let mut elvui_frames = Vec::new();

    for id in state.widgets.iter_ids() {
        let Some(frame) = state.widgets.get(id) else {
            continue;
        };
        let Some(name) = frame.name.as_deref() else {
            continue;
        };
        let rect =
            crate::layout::compute_frame_rect(&state.widgets, id, width as f32, height as f32);
        if is_elvui_frame_name(name) {
            let mut geometry = frame_geometry_json(&state, frame, name, rect);
            if let Some(object) = geometry.as_object_mut() {
                object.insert(
                    "frame_type".into(),
                    format!("{:?}", frame.widget_type).into(),
                );
            }
            elvui_frames.push(geometry);
        }
        if !requested_ids.is_empty() {
            continue;
        }
        let selected = if visualizer_filter {
            name.starts_with("WeakAuras:") || name.starts_with("ScalpelVisualizer_")
        } else {
            needle
                .as_deref()
                .is_none_or(|value| name.to_ascii_lowercase().contains(value))
        };
        if !selected {
            continue;
        }
        let display_id = name
            .strip_prefix("ScalpelVisualizer_")
            .or_else(|| name.strip_prefix("WeakAuras:"))
            .unwrap_or(name);
        let mut geometry = frame_geometry_json(&state, frame, name, rect);
        insert_region_metadata(
            &mut geometry,
            display_id,
            frame,
            name.starts_with("ScalpelVisualizer_"),
        );
        regions.push(geometry);
    }

    for requested in resolved {
        append_requested_region_subtree(&state, &requested, width, height, &mut regions);
    }

    regions.sort_by_key(manifest_sort_key);
    elvui_frames.sort_by_key(manifest_sort_key);
    let payload = serde_json::json!({
        "stage": stage,
        "render_scope": filter,
        "manifest_scope": if requested_ids.is_empty() { "render_filter" } else { "requested_weakauras" },
        "requested_ids": requested_ids,
        "unresolved_requested_ids": unresolved_requested_ids,
        "regions": regions,
        "elvui_frames": elvui_frames,
    });
    let output = Path::new(path);
    if let Some(parent) = output.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(error) = std::fs::write(output, serde_json::to_vec_pretty(&payload).unwrap()) {
        eprintln!("[manifest] failed to write {}: {error}", output.display());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedWeakAuraRegion {
    display_id: String,
    frame_id: u64,
}

fn resolve_requested_weakauras_regions(
    env: &WowLuaEnv,
    requested_ids: &[String],
) -> (Vec<ResolvedWeakAuraRegion>, Vec<String>) {
    let mut resolved = Vec::new();
    let mut unresolved = Vec::new();
    for display_id in requested_ids {
        match env.weak_aura_region_frame_id(display_id) {
            Ok(Some(frame_id)) => resolved.push(ResolvedWeakAuraRegion {
                display_id: display_id.clone(),
                frame_id,
            }),
            Ok(None) | Err(_) => unresolved.push(display_id.clone()),
        }
    }
    (resolved, unresolved)
}

fn append_requested_region_subtree(
    state: &crate::lua_api::state::SimState,
    requested: &ResolvedWeakAuraRegion,
    width: u32,
    height: u32,
    regions: &mut Vec<serde_json::Value>,
) {
    let mut stack = vec![(requested.frame_id, 0_u32, "root".to_string())];
    let mut visited = HashSet::new();
    while let Some((frame_id, depth, path)) = stack.pop() {
        if !visited.insert(frame_id) {
            continue;
        }
        let Some(frame) = state.widgets.get(frame_id) else {
            continue;
        };
        let rect = crate::layout::compute_frame_rect(
            &state.widgets,
            frame_id,
            width as f32,
            height as f32,
        );
        let synthetic_name = frame.name.clone().unwrap_or_else(|| {
            format!(
                "WeakAuras:{}#{}:{}",
                requested.display_id, requested.frame_id, path
            )
        });
        let mut geometry = frame_geometry_json(state, frame, &synthetic_name, rect);
        insert_region_metadata(&mut geometry, &requested.display_id, frame, false);
        if let Some(object) = geometry.as_object_mut() {
            object.insert("owner_native_frame_id".into(), requested.frame_id.into());
            object.insert("native_region_depth".into(), depth.into());
            object.insert("native_region_path".into(), path.clone().into());
            object.insert("native_region_root".into(), (depth == 0).into());
            object.insert("native_region_descendant".into(), (depth > 0).into());
            object.insert(
                "resolved_via".into(),
                if depth == 0 {
                    "WeakAuras.GetRegion"
                } else {
                    "WeakAuras.GetRegion subtree"
                }
                .into(),
            );
            object.insert(
                "visual_leaf".into(),
                matches!(
                    frame.widget_type,
                    crate::widget::WidgetType::Texture
                        | crate::widget::WidgetType::FontString
                        | crate::widget::WidgetType::Line
                )
                .into(),
            );
        }
        regions.push(geometry);
        for (index, child_id) in frame.children.iter().copied().enumerate().rev() {
            stack.push((child_id, depth + 1, format!("{path}.{index}")));
        }
    }
}

fn frame_geometry_json(
    state: &crate::lua_api::state::SimState,
    frame: &crate::widget::Frame,
    name: &str,
    rect: crate::LayoutRect,
) -> serde_json::Value {
    let parent = frame
        .parent_id
        .and_then(|parent_id| state.widgets.get(parent_id))
        .and_then(|parent| parent.name.clone());
    let anchors = frame
        .anchors
        .iter()
        .map(|anchor| {
            let relative_to = anchor
                .relative_to_id
                .and_then(|target_id| state.widgets.get(target_id as u64))
                .and_then(|target| target.name.clone())
                .or_else(|| anchor.relative_to.clone());
            serde_json::json!({
                "point": format!("{:?}", anchor.point),
                "relative_to": relative_to,
                "relative_native_frame_id": anchor.relative_to_id,
                "relative_point": format!("{:?}", anchor.relative_point),
                "x_offset": anchor.x_offset,
                "y_offset": anchor.y_offset,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "name": name,
        "coordinate_space": "physical_pixels",
        "native_frame_id": frame.id,
        "effective_visible": state.widgets.is_ancestor_visible(frame.id),
        "parent": parent,
        "parent_native_frame_id": frame.parent_id,
        "x": rect.x,
        "y": rect.y,
        "width": rect.width,
        "height": rect.height,
        "scale": frame.scale,
        "effective_scale": frame.effective_scale,
        "alpha": frame.alpha,
        "effective_alpha": frame.effective_alpha,
        "anchors": anchors,
        "text": frame.text,
        "texture": frame.texture,
        "texture_file_data_id": frame.texture_file_data_id,
        "atlas": frame.atlas,
        "font": frame.font,
        "font_size": frame.font_size,
        "draw_layer": format!("{:?}", frame.draw_layer),
        "draw_sub_layer": frame.draw_sub_layer,
    })
}

fn insert_region_metadata(
    geometry: &mut serde_json::Value,
    display_id: &str,
    frame: &crate::widget::Frame,
    marker: bool,
) {
    if let Some(object) = geometry.as_object_mut() {
        object.insert("display_id".into(), display_id.into());
        object.insert("marker".into(), marker.into());
        object.insert("native_region_root".into(), false.into());
        object.insert(
            "region_type".into(),
            format!("{:?}", frame.widget_type).into(),
        );
        object.insert("is_root".into(), frame.parent_id.is_none().into());
    }
}

fn manifest_sort_key(value: &serde_json::Value) -> String {
    let owner = value
        .get("display_id")
        .or_else(|| value.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let depth = value
        .get("native_region_depth")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    let path = value
        .get("native_region_path")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let frame_id = value
        .get("native_frame_id")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    format!("{owner}\u{0}{depth:08}\u{0}{path}\u{0}{frame_id:020}")
}

fn is_elvui_frame_name(name: &str) -> bool {
    name.starts_with("ElvUI") || name.starts_with("ElvUF_") || name.starts_with("ElvAB_")
}

fn glyph_atlas_data(glyph_atlas: &mut GlyphAtlas) -> Option<(&[u8], u32)> {
    if glyph_atlas.is_dirty() {
        let (data, size, _) = glyph_atlas.texture_data();
        Some((data, size))
    } else {
        None
    }
}

fn maybe_crop_image(img: image::RgbaImage, crop: Option<&str>) -> Result<image::RgbaImage, String> {
    match crop {
        Some(crop_str) => apply_crop(img, crop_str),
        None => Ok(img),
    }
}

fn format_screenshot_saved_message(
    img: &image::RgbaImage,
    output_path: &Path,
    width: u32,
    height: u32,
    cropped: bool,
) -> String {
    let size_label = if cropped {
        format!(
            "{}x{} (cropped from {}x{})",
            img.width(),
            img.height(),
            width,
            height
        )
    } else {
        format!("{}x{}", width, height)
    };
    format!(
        "Saved {} screenshot to {}",
        size_label,
        output_path.display()
    )
}

/// Parse a crop string in WxH+X+Y format (e.g., "700x150+400+650").
/// Returns (width, height, x, y) or None if the format is invalid.
fn parse_crop(s: &str) -> Option<(u32, u32, u32, u32)> {
    let (dims, rest) = s.split_once('+')?;
    let (x_str, y_str) = rest.split_once('+')?;
    let (w_str, h_str) = dims.split_once('x')?;
    let w = w_str.parse().ok()?;
    let h = h_str.parse().ok()?;
    let x = x_str.parse().ok()?;
    let y = y_str.parse().ok()?;
    Some((w, h, x, y))
}

/// Apply crop to an image, returning an error string on invalid input.
fn apply_crop(img: image::RgbaImage, crop_str: &str) -> Result<image::RgbaImage, String> {
    use image::GenericImageView;
    let (cw, ch, cx, cy) = parse_crop(crop_str).ok_or_else(|| {
        format!("Invalid crop format '{crop_str}', expected WxH+X+Y (e.g., {CROP_FORMAT_EXAMPLE})")
    })?;
    if cx + cw > img.width() || cy + ch > img.height() {
        return Err(format!(
            "Crop region {}x{}+{}+{} exceeds image bounds {}x{}",
            cw,
            ch,
            cx,
            cy,
            img.width(),
            img.height()
        ));
    }
    Ok(img.view(cx, cy, cw, ch).to_image())
}

/// Save screenshots as WebP for streaming or lossless PNG for capture.
fn save_screenshot(img: &image::RgbaImage, output: &Path) -> Result<(), String> {
    if output.extension().and_then(|value| value.to_str()) == Some("png") {
        img.save_with_format(output, image::ImageFormat::Png)
            .map_err(|e| e.to_string())
    } else {
        let output = output.with_extension("webp");
        let encoder = webp::Encoder::from_rgba(img.as_raw(), img.width(), img.height());
        let mem = encoder.encode(65.0);
        std::fs::write(&output, &*mem).map_err(|e| e.to_string())
    }
}
