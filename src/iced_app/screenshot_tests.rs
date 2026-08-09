use super::app::{App, AppInit};
use super::screenshot::write_visualizer_manifest;
use crate::lua_api::WowLuaEnv;
use crate::render::{GlyphAtlas, WowFontSystem};
use crate::screen::ScreenKind;
use crate::texture::TextureManager;
use std::cell::RefCell;
use std::rc::Rc;

fn assert_physical_geometry_matches(region: &serde_json::Value) {
    let logical = &region["logical_geometry"];
    let physical = &region["physical_geometry"];
    let scale = region["renderer_scale"]
        .as_f64()
        .expect("renderer scale should be numeric");

    for field in ["x", "y", "width", "height"] {
        let logical_value = logical[field]
            .as_f64()
            .unwrap_or_else(|| panic!("logical {field} should be numeric"));
        let physical_value = physical[field]
            .as_f64()
            .unwrap_or_else(|| panic!("physical {field} should be numeric"));
        assert!(
            (physical_value - logical_value * scale).abs() < 0.001,
            "physical {field} must equal logical {field} times renderer scale"
        );
        assert_eq!(region[field], physical[field]);
    }
}

fn build_test_app() -> App {
    let env = Rc::new(RefCell::new(
        WowLuaEnv::new().expect("Lua environment should initialize"),
    ));
    env.borrow().set_screen_mode(ScreenKind::Game);
    env.borrow().set_screen_size(1024.0, 768.0);
    let (_cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
    let (_lua_tx, lua_rx) = std::sync::mpsc::channel();
    App::build_app(AppInit {
        env,
        log_messages: Vec::new(),
        texture_manager: Rc::new(RefCell::new(TextureManager::new())),
        font_system: Rc::new(RefCell::new(WowFontSystem::new())),
        glyph_atlas: Rc::new(RefCell::new(GlyphAtlas::new())),
        cmd_rx,
        lua_rx,
        debug_borders: false,
        debug_anchors: false,
        saved_vars: None,
        config: crate::config::SimConfig::default(),
    })
}
#[test]
fn configure_screenshot_runtime_updates_authoritative_metrics() {
    let mut app = build_test_app();

    app.configure_screenshot_runtime(2560, 1440, Some(0.53))
        .expect("runtime geometry should apply");

    let metrics = app
        .env
        .borrow()
        .runtime_screen_metrics()
        .expect("runtime probes should be readable");
    assert_eq!(metrics.physical_width, 2560.0);
    assert_eq!(metrics.physical_height, 1440.0);
    assert!((metrics.ui_parent_effective_scale - 0.53).abs() < 0.0001);
    assert!((metrics.screen_width - 2560.0 / 0.53).abs() < 0.01);
    assert!((metrics.screen_height - 1440.0 / 0.53).abs() < 0.01);
    assert!((metrics.ui_parent_width - metrics.screen_width).abs() < 0.01);
    assert!((metrics.ui_parent_height - metrics.screen_height).abs() < 0.01);
    assert_eq!(app.screen_size.get(), iced::Size::new(2560.0, 1440.0));
}

#[test]
fn visualizer_manifest_records_runtime_stage_probes() {
    let mut app = build_test_app();
    app.configure_screenshot_runtime(2560, 1440, Some(0.53))
        .expect("runtime geometry should apply");
    app.env
        .borrow()
        .exec(
            r#"
            local frame = CreateFrame("Frame", "ElvUF_Player", UIParent)
            frame:SetSize(200, 40)
            frame:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 10, 20)
            frame:Show()
            "#,
        )
        .expect("ElvUI frame fixture should initialize");
    let temp = tempfile::tempdir().expect("tempdir should initialize");
    let manifest = temp.path().join("native-manifest.json");
    write_visualizer_manifest(
        &app.env.borrow(),
        manifest.to_str().expect("manifest path should be UTF-8"),
        2560,
        1440,
        0.53,
        Some("__SCALPEL_VISUALIZER__"),
        &[],
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest).expect("manifest should exist"))
            .expect("manifest should be valid JSON");
    assert_eq!(payload["coordinate_space_version"], 2);
    assert_eq!(payload["stage"]["width"], 2560.0);
    assert_eq!(payload["stage"]["height"], 1440.0);
    assert_eq!(payload["stage"]["physical_width"], 2560);
    assert_eq!(payload["stage"]["physical_height"], 1440);
    assert!((payload["stage"]["ui_scale"].as_f64().unwrap() - 0.53).abs() < 0.0001);
    assert!((payload["stage"]["renderer_scale"].as_f64().unwrap() - 0.53).abs() < 0.0001);
    assert!((payload["stage"]["logical_width"].as_f64().unwrap() - 2560.0 / 0.53).abs() < 0.01);
    assert!((payload["stage"]["logical_height"].as_f64().unwrap() - 1440.0 / 0.53).abs() < 0.01);
    assert_eq!(
        payload["stage"]["coordinate_spaces"]["physical"]["id"],
        "physical_pixels"
    );
    assert_eq!(
        payload["stage"]["coordinate_spaces"]["logical"]["id"],
        "wow_screen_units"
    );
    assert_eq!(payload["stage"]["weak_auras_options_open"], false);
    let frame = &payload["elvui_frames"][0];
    assert_eq!(frame["name"], "ElvUF_Player");
    assert_eq!(frame["coordinate_space"], "physical_pixels");
    assert_eq!(
        frame["logical_geometry"]["coordinate_space"],
        "renderer_viewport_units"
    );
    assert!((frame["logical_geometry"]["width"].as_f64().unwrap() - 106.0).abs() < 0.001);
    assert!((frame["physical_geometry"]["width"].as_f64().unwrap() - 56.18).abs() < 0.001);
    assert_eq!(frame["visible"], true);
    assert_physical_geometry_matches(frame);
    assert_eq!(frame["anchors"][0]["relative_to"], "UIParent");
    assert_eq!(frame["anchors"][0]["x_offset"], 10.0);
    assert_eq!(frame["anchors"][0]["y_offset"], 20.0);
}

#[test]
fn manifest_resolves_exact_unnamed_weakauras_region_without_marker_heuristics() {
    let mut app = build_test_app();
    app.configure_screenshot_runtime(2560, 1440, Some(0.53))
        .expect("runtime geometry should apply");
    let display_id = "Actual Aura";
    app.env
        .borrow()
        .exec(
            r#"
            WeakAuras = WeakAuras or {}
            local actual = CreateFrame("Frame", nil, UIParent)
            actual:SetSize(64, 48)
            actual:SetPoint("CENTER", UIParent, "CENTER", 140, 70)
            actual:Show()
            local label = actual:CreateFontString(nil, "OVERLAY")
            label:SetText("OK")
            label:SetPoint("CENTER", actual, "CENTER", 0, 0)
            label:Show()
            local overlap = CreateFrame("Frame", "OverlappingUnrelated", UIParent)
            overlap:SetSize(64, 48)
            overlap:SetPoint("CENTER", UIParent, "CENTER", 140, 70)
            overlap:Show()
            WeakAuras.GetRegion = function(id)
                if id == "Actual Aura" then return actual end
                return nil
            end
            "#,
        )
        .expect("WeakAuras region fixture should initialize");
    let actual_frame_id = app
        .env
        .borrow()
        .weak_aura_region_frame_id(display_id)
        .expect("region lookup should execute")
        .expect("region should resolve");
    let temp = tempfile::tempdir().expect("tempdir should initialize");
    let manifest = temp.path().join("actual-region.json");
    write_visualizer_manifest(
        &app.env.borrow(),
        manifest.to_str().expect("manifest path should be UTF-8"),
        2560,
        1440,
        0.53,
        None,
        &[display_id.to_string(), "Missing Aura".to_string()],
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest).expect("manifest should exist"))
            .expect("manifest should be valid JSON");
    assert_eq!(payload["manifest_scope"], "requested_weakauras");
    assert!(payload["render_scope"].is_null());
    assert_eq!(
        payload["unresolved_requested_ids"],
        serde_json::json!(["Missing Aura"])
    );
    let regions = payload["regions"].as_array().unwrap();
    assert_eq!(regions.len(), 2);
    let root = regions
        .iter()
        .find(|region| region["native_region_root"] == true)
        .expect("exact root should be present");
    let label = regions
        .iter()
        .find(|region| region["native_region_descendant"] == true)
        .expect("owned FontString should be present");
    assert_eq!(root["display_id"], display_id);
    assert_eq!(root["native_frame_id"], actual_frame_id);
    assert_eq!(root["owner_native_frame_id"], actual_frame_id);
    assert_eq!(root["native_region_depth"], 0);
    assert_eq!(root["marker"], false);
    assert_eq!(root["resolved_via"], "WeakAuras.GetRegion");
    assert_ne!(root["name"], "OverlappingUnrelated");
    assert_eq!(root["coordinate_space_version"], 2);
    assert_eq!(root["coordinate_space"], "physical_pixels");
    assert_eq!(
        root["logical_geometry"]["coordinate_space"],
        "renderer_viewport_units"
    );
    assert!((root["logical_geometry"]["width"].as_f64().unwrap() - 64.0 * 0.53).abs() < 0.001);
    assert!((root["logical_geometry"]["height"].as_f64().unwrap() - 48.0 * 0.53).abs() < 0.001);
    assert!((root["width"].as_f64().unwrap() - 64.0 * 0.53 * 0.53).abs() < 0.001);
    assert!((root["height"].as_f64().unwrap() - 48.0 * 0.53 * 0.53).abs() < 0.001);
    assert!((root["effective_scale"].as_f64().unwrap() - 0.53).abs() < 0.001);
    assert_physical_geometry_matches(root);
    assert_eq!(label["display_id"], display_id);
    assert_eq!(label["owner_native_frame_id"], actual_frame_id);
    assert_eq!(label["native_region_depth"], 1);
    assert_eq!(label["native_region_path"], "root.0");
    assert_eq!(label["resolved_via"], "WeakAuras.GetRegion subtree");
    assert_eq!(label["visual_leaf"], true);
    assert_eq!(label["region_type"], "FontString");
    assert_eq!(label["text"], "OK");
    assert_eq!(label["visible"], true);
    assert_physical_geometry_matches(label);
    assert_ne!(label["name"], "OverlappingUnrelated");
}

#[test]
fn weak_aura_region_lookup_escapes_display_ids() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        WeakAuras = WeakAuras or {}
        local actual = CreateFrame("Frame", nil, UIParent)
        WeakAuras.GetRegion = function(id)
            if id == "Aüra \\\"one\\\"\\\\path" then return actual end
            return nil
        end
        "#,
    )
    .expect("lookup fixture should initialize");

    assert!(
        env.weak_aura_region_frame_id("Aüra \\\"one\\\"\\\\path")
            .expect("lookup should execute")
            .is_some()
    );
}
