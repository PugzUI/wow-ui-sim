//! IPC screenshot rendering for the running app.

use std::collections::HashSet;
use std::path::Path;

use crate::iced_app::frame_collect::SCALPEL_VISUALIZER_ROOT;
use crate::lua_api::WowLuaEnv;
use crate::lua_server::Response as LuaResponse;

use super::app::App;
const CROP_FORMAT_EXAMPLE: &str = "700x150+400+650";

#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamFrameResult {
    pub output: String,
    pub stage_width: u32,
    pub stage_height: u32,
    pub width: u32,
    pub height: u32,
    pub render_ms: f64,
    pub encode_ms: f64,
    pub total_ms: f64,
    pub sim_time: f64,
    pub cached_strata: bool,
    pub dirty_strata_mask: u16,
}

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
        // Screenshot sizing fires the same display/scale events as a real
        // window resize. The visualizer's Options fixture intentionally
        // restores the real options state after those events so the manifest
        // describes the stage that agents actually inspect, not a transient
        // closed frame.
        if manifest.is_some() {
            let env = self.env.borrow();
            if let Err(error) = env.exec(
                "if WeakAuras and WeakAuras.ScalpelPreviewEnsureOptions then \
                 WeakAuras.ScalpelPreviewEnsureOptions() end",
            ) {
                eprintln!("[visualizer] failed to restore Options after sizing: {error}");
            }
        }
        let output_path = Path::new(output).to_path_buf();
        let batch = self.build_screenshot_batch(width, height, filter);
        let mut tex_mgr = self.texture_manager.borrow_mut();
        let mut renderer = self.headless_renderer.borrow_mut();
        let renderer = renderer.get_or_insert_with(crate::render::headless::HeadlessRenderer::new);
        let (glyph_upload, glyph_size, glyph_revision) = self.glyph_upload_for(renderer);
        let glyph_data = glyph_upload
            .as_ref()
            .map(|data| (data.as_slice(), glyph_size));
        let img = renderer.render_to_image_with_glyph_revision(
            &batch,
            &mut tex_mgr,
            width,
            height,
            glyph_data,
            glyph_revision,
        );
        let img = match maybe_crop_image(img, crop) {
            Ok(img) => img,
            Err(e) => return LuaResponse::Error(e),
        };

        if let Err(e) = save_screenshot(&img, &output_path) {
            return LuaResponse::Error(format!("Failed to save screenshot: {}", e));
        }
        // Sizing and rendering can dispatch the same UI events as a real
        // resize. Reassert the real Options state after the render as well,
        // before writing the authoritative manifest and returning control.
        if manifest.is_some() {
            let env = self.env.borrow();
            if let Err(error) = env.exec(
                "if WeakAuras and WeakAuras.ScalpelPreviewEnsureOptions then \
                 WeakAuras.ScalpelPreviewEnsureOptions() end",
            ) {
                eprintln!("[visualizer] failed to restore Options after render: {error}");
            }
        }
        if let Some(manifest) = manifest {
            write_visualizer_manifest(
                &self.env.borrow(),
                manifest,
                width,
                height,
                crate::render::texture::UI_SCALE,
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

    /// Render the current native stage into a reduced interactive frame.
    ///
    /// The stage remains at its authoritative physical size; only the output
    /// target is reduced. Exact screenshots and manifests continue to use
    /// `render_screenshot` at 2560x1440.
    pub(crate) fn render_stream_frame(
        &mut self,
        output: &str,
        width: u32,
        height: u32,
        quality: f32,
    ) -> Result<StreamFrameResult, String> {
        let started = std::time::Instant::now();
        let stage = self.screen_size.get();
        let stage_width = stage.width.round().max(1.0) as u32;
        let stage_height = stage.height.round().max(1.0) as u32;
        let dirty_strata_mask = self.strata_dirty.get();
        if dirty_strata_mask != 0 {
            let _ = self.get_or_rebuild_quads(stage);
        }
        let cached_strata = self.cached_strata_quads.borrow().clone();
        let cached_strata_ready = cached_strata.iter().any(Option::is_some);
        let stream_filter =
            crate::render::texture::visualizer_mode().then_some(SCALPEL_VISUALIZER_ROOT);
        let fallback_batch = (!cached_strata_ready)
            .then(|| self.build_screenshot_batch(stage_width, stage_height, stream_filter));
        let render_started = std::time::Instant::now();
        let image = {
            let mut tex_mgr = self.texture_manager.borrow_mut();
            let mut renderer = self.headless_renderer.borrow_mut();
            let renderer =
                renderer.get_or_insert_with(crate::render::headless::HeadlessRenderer::new);
            let (glyph_upload, glyph_size, glyph_revision) = self.glyph_upload_for(renderer);
            let glyph_data = glyph_upload
                .as_ref()
                .map(|data| (data.as_slice(), glyph_size));
            if cached_strata_ready {
                renderer.render_cached_strata_to_image(
                    &cached_strata,
                    &mut tex_mgr,
                    stage_width,
                    stage_height,
                    width,
                    height,
                    glyph_data,
                    glyph_revision,
                )
            } else {
                renderer.render_scaled_to_image(
                    fallback_batch.as_ref().expect("fallback batch built"),
                    &mut tex_mgr,
                    stage_width,
                    stage_height,
                    width,
                    height,
                    glyph_data,
                )
            }
        };
        let render_ms = render_started.elapsed().as_secs_f64() * 1000.0;
        let encode_started = std::time::Instant::now();
        let output_path = Path::new(output);
        save_stream_frame(&image, output_path, quality)?;
        let encode_ms = encode_started.elapsed().as_secs_f64() * 1000.0;
        let sim_time = self
            .env
            .borrow()
            .eval::<f64>("return GetTime()")
            .unwrap_or_default();
        Ok(StreamFrameResult {
            output: output_path.to_string_lossy().into_owned(),
            stage_width,
            stage_height,
            width,
            height,
            render_ms,
            encode_ms,
            total_ms: started.elapsed().as_secs_f64() * 1000.0,
            sim_time,
            cached_strata: cached_strata_ready,
            dirty_strata_mask,
        })
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

    fn glyph_upload_for(
        &self,
        renderer: &crate::render::headless::HeadlessRenderer,
    ) -> (Option<Vec<u8>>, u32, u64) {
        let glyph_atlas = self.glyph_atlas.borrow();
        let revision = glyph_atlas.revision();
        let (data, size, _) = glyph_atlas.texture_data();
        let upload = renderer.needs_glyph_upload(revision).then(|| data.to_vec());
        (upload, size, revision)
    }

    fn build_screenshot_batch(
        &self,
        width: u32,
        height: u32,
        filter: Option<&str>,
    ) -> crate::render::QuadBatch {
        let mut glyph_atlas = self.glyph_atlas.borrow_mut();
        glyph_atlas.advance_generation();
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
    }
}

fn visualizer_stage_json(
    env: &WowLuaEnv,
    width: u32,
    height: u32,
    renderer_scale: f32,
) -> serde_json::Value {
    use crate::render::coordinates::{
        BOTTOM_LEFT_ORIGIN, COORDINATE_SPACE_VERSION, LOCAL_FRAME_UNITS, PARENT_RELATIVE_UI_UNITS,
        PHYSICAL_PIXELS, RENDERER_VIEWPORT_UNITS, TOP_LEFT_ORIGIN, WOW_SCREEN_UNITS,
        renderer_viewport_size,
    };

    let mut stage = env
        .runtime_screen_metrics()
        .ok()
        .and_then(|metrics| serde_json::to_value(metrics).ok())
        .unwrap_or_else(|| {
            serde_json::json!({
                "width": width,
                "height": height,
                "physical_width": width,
                "physical_height": height,
                "ui_scale": renderer_scale,
                "probe_error": "runtime screen metrics unavailable"
            })
        });
    let (renderer_width, renderer_height) =
        renderer_viewport_size(width as f32, height as f32, renderer_scale);
    let logical_width = stage
        .get("screen_width")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(renderer_width as f64);
    let logical_height = stage
        .get("screen_height")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(renderer_height as f64);

    if let Some(object) = stage.as_object_mut() {
        object.insert(
            "coordinate_space_version".into(),
            COORDINATE_SPACE_VERSION.into(),
        );
        object.insert("physical_width".into(), width.into());
        object.insert("physical_height".into(), height.into());
        object.insert("logical_width".into(), logical_width.into());
        object.insert("logical_height".into(), logical_height.into());
        object.insert("renderer_viewport_width".into(), renderer_width.into());
        object.insert("renderer_viewport_height".into(), renderer_height.into());
        object.insert("renderer_scale".into(), renderer_scale.into());
        object.insert(
            "coordinate_spaces".into(),
            serde_json::json!({
                "physical": {"id": PHYSICAL_PIXELS, "origin": TOP_LEFT_ORIGIN, "y_direction": "down"},
                "logical": {"id": WOW_SCREEN_UNITS, "origin": BOTTOM_LEFT_ORIGIN, "y_direction": "up"},
                "renderer": {"id": RENDERER_VIEWPORT_UNITS, "origin": TOP_LEFT_ORIGIN, "y_direction": "down"},
                "local_frame": {"id": LOCAL_FRAME_UNITS, "origin": "frame_anchor_dependent"},
                "parent_anchor": {"id": PARENT_RELATIVE_UI_UNITS, "y_direction": "up"}
            }),
        );
    }
    stage
}

pub fn write_visualizer_manifest(
    env: &WowLuaEnv,
    path: &str,
    width: u32,
    height: u32,
    renderer_scale: f32,
    filter: Option<&str>,
    requested_ids: &[String],
) {
    let stage = visualizer_stage_json(env, width, height, renderer_scale);
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
            let mut geometry = frame_geometry_json(&state, frame, name, rect, renderer_scale);
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
        let mut geometry = frame_geometry_json(&state, frame, name, rect, renderer_scale);
        insert_region_metadata(
            &mut geometry,
            display_id,
            frame,
            name.starts_with("ScalpelVisualizer_"),
        );
        regions.push(geometry);
    }

    for requested in resolved {
        append_requested_region_subtree(
            &state,
            &requested,
            width,
            height,
            renderer_scale,
            &mut regions,
        );
    }

    regions.sort_by_key(manifest_sort_key);
    elvui_frames.sort_by_key(manifest_sort_key);
    let payload = serde_json::json!({
        "coordinate_space_version": crate::render::coordinates::COORDINATE_SPACE_VERSION,
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
    renderer_scale: f32,
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
        let mut geometry = frame_geometry_json(state, frame, &synthetic_name, rect, renderer_scale);
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
    renderer_scale: f32,
) -> serde_json::Value {
    use crate::render::coordinates::{
        COORDINATE_SPACE_VERSION, Geometry, LOCAL_FRAME_UNITS, PARENT_RELATIVE_UI_UNITS,
        PHYSICAL_PIXELS, TOP_LEFT_ORIGIN,
    };

    let logical_geometry = Geometry::renderer(rect);
    let physical_geometry = Geometry::physical(rect, renderer_scale);
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
                "coordinate_space": PARENT_RELATIVE_UI_UNITS,
                "x_direction": "right",
                "y_direction": "up",
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
        "coordinate_space_version": COORDINATE_SPACE_VERSION,
        "coordinate_space": PHYSICAL_PIXELS,
        "origin": TOP_LEFT_ORIGIN,
        "native_frame_id": frame.id,
        "visible": frame.visible,
        "effective_visible": state.widgets.is_ancestor_visible(frame.id),
        "parent": parent,
        "parent_native_frame_id": frame.parent_id,
        "x": physical_geometry.x,
        "y": physical_geometry.y,
        "width": physical_geometry.width,
        "height": physical_geometry.height,
        "logical_geometry": logical_geometry,
        "physical_geometry": physical_geometry,
        "local_geometry": {
            "coordinate_space": LOCAL_FRAME_UNITS,
            "width": frame.width,
            "height": frame.height,
        },
        "renderer_scale": renderer_scale,
        "scale": frame.scale,
        "local_scale": frame.scale,
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

fn save_stream_frame(image: &image::RgbaImage, output: &Path, quality: f32) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = output.with_extension("stream.tmp");
    match output.extension().and_then(|value| value.to_str()) {
        Some("jpg" | "jpeg") => {
            let rgb = image::DynamicImage::ImageRgba8(image.clone()).to_rgb8();
            let file = std::fs::File::create(&temporary).map_err(|error| error.to_string())?;
            let mut writer = std::io::BufWriter::new(file);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut writer,
                quality.round().clamp(1.0, 100.0) as u8,
            );
            encoder
                .encode(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|error| error.to_string())?;
        }
        _ => {
            let encoder = webp::Encoder::from_rgba(image.as_raw(), image.width(), image.height());
            let encoded = encoder.encode(quality);
            std::fs::write(&temporary, &*encoded).map_err(|error| error.to_string())?;
        }
    }
    if output.exists() {
        std::fs::remove_file(output).map_err(|error| error.to_string())?;
    }
    std::fs::rename(&temporary, output).map_err(|error| error.to_string())
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
