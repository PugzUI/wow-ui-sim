#![cfg(feature = "gui")]

use crate::common;
#[path = "render_order_support.rs"]
mod render_order_support;

use image::RgbaImage;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::headless::render_to_image;

fn lit_pixel_bounds(image: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let mut bounds = (u32::MAX, u32::MAX, 0, 0);
    let mut found = false;
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0[..3].iter().any(|channel| *channel > 24) {
            found = true;
            bounds.0 = bounds.0.min(x);
            bounds.1 = bounds.1.min(y);
            bounds.2 = bounds.2.max(x);
            bounds.3 = bounds.3.max(y);
        }
    }
    found.then_some(bounds)
}

fn rasterize(env: &WowLuaEnv, width: u32, height: u32) -> RgbaImage {
    let (batch, glyph_pixels, glyph_size) =
        render_order_support::build_screenshot_like_batch_with_glyph_atlas(
            env, width, height, None,
        );
    render_to_image(
        &batch,
        &mut render_order_support::make_texture_manager(),
        width,
        height,
        Some((&glyph_pixels, glyph_size)),
    )
}

#[test]
fn lua_fontstring_under_unnamed_parent_rasterizes_and_follows_parent_movement() {
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping FontString raster test: no adapter available");
        return;
    }

    let env = WowLuaEnv::new().expect("Lua environment");
    let width = 320;
    let height = 180;
    env.set_screen_size(width as f32, height as f32);
    env.exec(
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(180, 40)
        parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 20, -20)

        local label = parent:CreateFontString(nil, "OVERLAY")
        assert(label:SetFont("Fonts\\FRIZQT__.TTF", 24, ""))
        label:SetText("FontString raster")
        label:SetTextColor(1, 1, 1, 1)
        label:SetAlpha(1)
        label:SetPoint("LEFT", parent, "LEFT", 4, 0)
        _G.FontStringRasterParent = parent
        _G.FontStringRasterLabel = label
        "#,
    )
    .expect("FontString setup");

    let before = rasterize(&env, width, height);
    let before_bounds = lit_pixel_bounds(&before).expect("FontString should produce raster pixels");
    let label_id = {
        let state = env.state().borrow();
        state
            .widgets
            .iter_ids()
            .find(|id| {
                state.widgets.get(*id).is_some_and(|frame| {
                    frame.widget_type == wow_ui_sim::widget::WidgetType::FontString
                        && frame.text.as_deref() == Some("FontString raster")
                })
            })
            .expect("unnamed FontString should be registered")
    };
    let _ = env.state().borrow().widgets.take_render_dirty_with_ids();

    env.exec(
        r#"
        FontStringRasterParent:ClearAllPoints()
        FontStringRasterParent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -90)
        "#,
    )
    .expect("parent movement");

    env.state().borrow_mut().ensure_layout_rects();
    let (_, dirty_ids) = env.state().borrow().widgets.take_render_dirty_with_ids();
    assert!(
        dirty_ids.is_some_and(|ids| ids.contains(&label_id)),
        "moving an ancestor must invalidate the FontString snapshot whose quads bake absolute geometry"
    );

    let after = rasterize(&env, width, height);
    let after_bounds =
        lit_pixel_bounds(&after).expect("moved FontString should produce raster pixels");
    assert!(
        after_bounds.0 >= before_bounds.0 + 70 && after_bounds.1 >= before_bounds.1 + 60,
        "FontString pixels should follow unnamed parent movement: before={before_bounds:?} after={after_bounds:?}"
    );
}
