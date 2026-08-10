#![cfg(feature = "gui")]

#[allow(dead_code)]
mod common;
#[path = "render_order_support.rs"]
mod render_order_support;

use image::RgbaImage;
use wow_ui_sim::iced_app::write_visualizer_manifest;
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
    let batch = render_order_support::build_screenshot_like_batch(env, width, height, None);
    render_to_image(
        &batch,
        &mut render_order_support::make_texture_manager(),
        width,
        height,
        None,
    )
}

fn assert_axis_near(actual: u32, expected: f64, label: &str) {
    assert!(
        (actual as f64 - expected).abs() <= 2.0,
        "{label}: raster={actual}, manifest={expected}"
    );
}

#[test]
fn manifest_physical_geometry_localizes_native_texture_pixels_at_visualizer_scale() {
    let xdg_runtime = tempfile::tempdir().expect("XDG runtime dir");
    // This dedicated test binary owns both environment variables before any
    // simulator renderer state is initialized.
    unsafe {
        std::env::set_var("XDG_RUNTIME_DIR", xdg_runtime.path());
        std::env::set_var("WOW_SIM_VISUALIZER", "1");
    }
    if common::try_create_gpu_device().is_none() {
        eprintln!("Skipping Visualizer raster localization test: no adapter available");
        return;
    }
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_ui_scale(0.53).expect("UI scale should apply");
    env.exec(
        r#"
        WeakAuras = WeakAuras or {}
        local region = CreateFrame("Frame", nil, UIParent)
        region:SetSize(64, 48)
        region:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 40, -30)
        region:Show()

        local texture = region:CreateTexture(nil, "ARTWORK")
        texture:SetAllPoints(region)
        texture:SetColorTexture(1, 1, 1, 1)
        texture:Show()

        WeakAuras.GetRegion = function(id)
            if id == "Coordinate Aura" then return region end
            return nil
        end
        "#,
    )
    .expect("native region fixture");

    let width = 320;
    let height = 180;
    let image = rasterize(&env, width, height);
    assert_eq!(image.dimensions(), (width, height));
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("native-manifest.json");
    write_visualizer_manifest(
        &env,
        manifest_path.to_str().expect("UTF-8 manifest path"),
        width,
        height,
        0.53,
        None,
        &["Coordinate Aura".to_string()],
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).expect("manifest should exist"))
            .expect("manifest JSON");
    assert!(
        manifest["unresolved_requested_ids"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let texture = manifest["regions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|region| region["region_type"] == "Texture")
        .expect("owned Texture record");
    let physical = &texture["physical_geometry"];
    let x = physical["x"].as_f64().unwrap();
    let y = physical["y"].as_f64().unwrap();
    let right = x + physical["width"].as_f64().unwrap();
    let bottom = y + physical["height"].as_f64().unwrap();
    let bounds = lit_pixel_bounds(&image).expect("native Texture should produce pixels");
    assert_axis_near(bounds.0, x.floor(), "left");
    assert_axis_near(bounds.1, y.floor(), "top");
    assert_axis_near(bounds.2, right.ceil() - 1.0, "right");
    assert_axis_near(bounds.3, bottom.ceil() - 1.0, "bottom");

    assert_eq!(texture["coordinate_space"], "physical_pixels");
    assert_eq!(texture["x"], physical["x"]);
    assert_eq!(texture["y"], physical["y"]);
    assert_eq!(texture["width"], physical["width"]);
    assert_eq!(texture["height"], physical["height"]);
}
