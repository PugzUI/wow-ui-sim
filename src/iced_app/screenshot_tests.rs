use super::app::{App, AppInit};
use super::screenshot::write_visualizer_manifest;
use crate::lua_api::WowLuaEnv;
use crate::render::{GlyphAtlas, WowFontSystem};
use crate::screen::ScreenKind;
use crate::texture::TextureManager;
use std::cell::RefCell;
use std::rc::Rc;

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
        Some("__SCALPEL_VISUALIZER__"),
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest).expect("manifest should exist"))
            .expect("manifest should be valid JSON");
    assert_eq!(payload["stage"]["width"], 2560.0);
    assert_eq!(payload["stage"]["height"], 1440.0);
    assert!((payload["stage"]["ui_scale"].as_f64().unwrap() - 0.53).abs() < 0.0001);
    assert_eq!(payload["stage"]["weak_auras_options_open"], false);
    assert_eq!(payload["elvui_frames"][0]["name"], "ElvUF_Player");
    assert!((payload["elvui_frames"][0]["width"].as_f64().unwrap() - 106.0).abs() < 0.001);
    assert!((payload["elvui_frames"][0]["height"].as_f64().unwrap() - 21.2).abs() < 0.001);
    assert_eq!(
        payload["elvui_frames"][0]["anchors"][0]["relative_to"],
        "UIParent"
    );
    assert_eq!(payload["elvui_frames"][0]["anchors"][0]["x_offset"], 10.0);
    assert_eq!(payload["elvui_frames"][0]["anchors"][0]["y_offset"], 20.0);
}
