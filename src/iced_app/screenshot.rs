//! IPC screenshot rendering for the running app.

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
        &self,
        output: &str,
        width: u32,
        height: u32,
        filter: Option<&str>,
        crop: Option<&str>,
        manifest: Option<&str>,
    ) -> LuaResponse {
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
            write_visualizer_manifest(&self.env.borrow(), manifest, width, height, filter);
        }

        LuaResponse::Output(format_screenshot_saved_message(
            &img,
            &output_path,
            width,
            height,
            crop.is_some(),
        ))
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

pub(crate) fn write_visualizer_manifest(
    env: &WowLuaEnv,
    path: &str,
    width: u32,
    height: u32,
    filter: Option<&str>,
) {
    let state = env.state().borrow();
    let visualizer_filter = filter == Some(SCALPEL_VISUALIZER_ROOT);
    let needle = filter.map(str::to_ascii_lowercase);
    let mut regions = Vec::new();
    for id in state.widgets.iter_ids() {
        let Some(frame) = state.widgets.get(id) else {
            continue;
        };
        let Some(name) = frame.name.as_deref() else {
            continue;
        };
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
        let rect =
            crate::layout::compute_frame_rect(&state.widgets, id, width as f32, height as f32);
        let parent = frame
            .parent_id
            .and_then(|parent_id| state.widgets.get(parent_id))
            .and_then(|parent| parent.name.clone());
        let display_id = name
            .strip_prefix("ScalpelVisualizer_")
            .or_else(|| name.strip_prefix("WeakAuras:"))
            .unwrap_or(name);
        regions.push(serde_json::json!({
            "display_id": display_id,
            "encoded_id": name.starts_with("ScalpelVisualizer_"),
            "region_type": format!("{:?}", frame.widget_type),
            "effective_visible": state.widgets.is_ancestor_visible(id),
            "parent": parent.as_ref().map(|value| value.trim_start_matches("WeakAuras:").to_string()),
            "is_root": parent.is_none(),
            "x": rect.x,
            "y": rect.y,
            "width": rect.width,
            "height": rect.height,
            "scale": frame.scale,
            "alpha": frame.alpha,
        }));
    }
    let payload = serde_json::json!({"regions": regions});
    let output = Path::new(path);
    if let Some(parent) = output.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(error) = std::fs::write(output, serde_json::to_vec_pretty(&payload).unwrap()) {
        eprintln!("[manifest] failed to write {}: {error}", output.display());
    }
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
