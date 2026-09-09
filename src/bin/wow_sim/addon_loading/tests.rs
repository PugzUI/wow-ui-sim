use super::*;
use std::path::{Path, PathBuf};

fn write_addon(root: &Path, name: &str) -> PathBuf {
    write_addon_with_toc(
        root,
        name,
        &format!(
            "## Interface: {}\nmain.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    )
}

fn write_addon_with_toc(root: &Path, name: &str, toc: &str) -> PathBuf {
    let addon_dir = root.join(name);
    std::fs::create_dir_all(&addon_dir).expect("create addon dir");
    let toc_path = addon_dir.join(format!("{name}.toc"));
    std::fs::write(&toc_path, toc).expect("write toc");
    std::fs::write(addon_dir.join("main.lua"), "").expect("write lua");
    toc_path
}

fn write_addon_with_lua(root: &Path, name: &str, metadata: &str, lua: &str) -> PathBuf {
    let addon_dir = root.join(name);
    std::fs::create_dir_all(&addon_dir).expect("create addon dir");
    let toc_path = addon_dir.join(format!("{name}.toc"));
    let toc = format!(
        "## Interface: {}\n{}main.lua\n",
        wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION,
        metadata
    );
    std::fs::write(&toc_path, toc).expect("write toc");
    std::fs::write(addon_dir.join("main.lua"), lua).expect("write lua");
    toc_path
}

#[test]
fn admin_allowlist_is_weakauras_family_and_pugzui_only() {
    assert_eq!(
        ADMIN_TEST_ADDONS,
        &[
            "WeakAuras",
            "WeakAurasOptions",
            "WeakAurasTemplates",
            "WeakAurasArchive",
            "WeakAurasModelPaths",
            "PugzUI",
        ]
    );
}

#[test]
fn admin_discovery_registers_load_on_demand_companions() {
    let temp = tempfile::tempdir().expect("tempdir");
    let toc = write_addon_with_toc(temp.path(), "WeakAurasOptions", "## LoadOnDemand: 1\n");

    assert!(scan_addons(temp.path(), &[], ScreenKind::Game).is_empty());
    assert_eq!(
        scan_addons_inner(temp.path(), &[], ScreenKind::Game, true),
        vec![("WeakAurasOptions".to_string(), toc)]
    );
}

#[test]
fn lua_errors_reports_enabled_addon_with_missing_required_dependency() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dependent_toc = write_addon_with_lua(
        temp.path(),
        "DependentAddon",
        "## Dependencies: MissingRequiredAddon\n",
        "_G.DependentAddonLoaded = true\n",
    );
    let addons = vec![("DependentAddon".to_string(), dependent_toc)];
    let env = WowLuaEnv::new().expect("create Lua env");
    let mut saved_vars = None;
    let mut stats = LoadStats::default();

    load_discovered_addons(&env, &addons, &mut saved_vars, None, &mut stats);

    let state = env.state().borrow();
    assert_eq!(state.lua_errors.len(), 1);
    assert_eq!(
        state.lua_errors[0],
        "DependentAddon missing required TOC dependencies: MissingRequiredAddon"
    );
    assert_eq!(state.lua_error_counts.get(&state.lua_errors[0]), Some(&1));
    assert_eq!(state.lua_error_records.len(), 1);
    assert_eq!(
        state.lua_error_records[0].addon_name.as_deref(),
        Some("DependentAddon")
    );
    assert!(
        !state
            .addons
            .iter()
            .find(|addon| addon.folder_name == "DependentAddon")
            .expect("registered dependent addon")
            .loaded,
        "addon with a missing required dependency should not load"
    );
    drop(state);
    let dependent_loaded: Option<bool> = env
        .eval("return DependentAddonLoaded")
        .expect("read global");
    assert_eq!(dependent_loaded, None);
}

#[test]
fn earlier_addon_can_see_later_addon_metadata_before_later_loads() {
    let temp = tempfile::tempdir().expect("tempdir");
    let early_toc = write_addon_with_lua(
        temp.path(),
        "EarlyAddon",
        "",
        r#"
            _G.EarlyAddonSawLaterDisplay = false
            _G.LaterAddonWasLoadedDuringEarlyScan = _G.LaterAddonLoaded == true
            for i = 1, C_AddOns.GetNumAddOns() do
                if C_AddOns.GetAddOnMetadata(i, "X-BugGrabber-Display") == "LaterDisplay" then
                    _G.EarlyAddonSawLaterDisplay = true
                end
            end
        "#,
    );
    let later_toc = write_addon_with_lua(
        temp.path(),
        "LaterAddon",
        "## X-BugGrabber-Display: LaterDisplay\n",
        "_G.LaterAddonLoaded = true\n",
    );
    let addons = vec![
        ("EarlyAddon".to_string(), early_toc),
        ("LaterAddon".to_string(), later_toc),
    ];
    let env = WowLuaEnv::new().expect("create Lua env");
    let mut saved_vars = None;
    let mut stats = LoadStats::default();

    load_discovered_addons(&env, &addons, &mut saved_vars, None, &mut stats);

    let saw_later: bool = env
        .eval("return EarlyAddonSawLaterDisplay")
        .expect("read early scan result");
    let later_loaded_during_scan: bool = env
        .eval("return LaterAddonWasLoadedDuringEarlyScan")
        .expect("read early load result");
    let later_loaded_after_all: bool = env
        .eval("return LaterAddonLoaded == true")
        .expect("read later load result");

    assert!(
        saw_later,
        "metadata for all discovered addons should be available before any addon Lua runs"
    );
    assert!(
        !later_loaded_during_scan,
        "pre-registration must not execute later addon files early"
    );
    assert!(
        later_loaded_after_all,
        "later addon should still load in normal order"
    );
}

#[test]
fn scan_addon_paths_merges_roots_and_keeps_first_duplicate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let sim_root = temp.path().join("sim");
    let wow_root = temp.path().join("wow");
    let sim_shared_toc = write_addon(&sim_root, "SharedAddon");
    write_addon(&sim_root, "SimulatorOnly");
    write_addon(&wow_root, "SharedAddon");
    write_addon(&wow_root, "WowOnly");

    let addons = scan_addon_paths(&[sim_root.clone(), wow_root], &[], ScreenKind::Game);
    let names: Vec<_> = addons.iter().map(|(name, _)| name.as_str()).collect();
    let shared_toc = addons
        .iter()
        .find(|(name, _)| name == "SharedAddon")
        .map(|(_, toc)| toc)
        .expect("shared addon should be present");

    assert_eq!(names, ["SharedAddon", "SimulatorOnly", "WowOnly"]);
    assert_eq!(
        shared_toc, &sim_shared_toc,
        "the first addon root should win duplicate addon names"
    );
}

#[test]
fn scan_addons_skips_out_of_date_interfaces_by_default() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_addon_with_toc(
        temp.path(),
        "CurrentAddon",
        &format!(
            "## Interface: {}\nmain.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    );
    write_addon_with_toc(temp.path(), "OldAddon", "## Interface: 120001\nmain.lua\n");

    let addons = scan_addons(temp.path(), &[], ScreenKind::Game);
    let names: Vec<_> = addons.iter().map(|(name, _)| name.as_str()).collect();

    assert_eq!(names, ["CurrentAddon"]);
}

#[test]
#[cfg(feature = "client-mists")]
fn scan_addons_accepts_mists_interface_version() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_addon_with_toc(temp.path(), "ElvUI", "## Interface: 50504\nmain.lua\n");

    let addons = scan_addons(temp.path(), &[], ScreenKind::Game);
    let names: Vec<_> = addons.iter().map(|(name, _)| name.as_str()).collect();

    assert_eq!(names, ["ElvUI"]);
}
