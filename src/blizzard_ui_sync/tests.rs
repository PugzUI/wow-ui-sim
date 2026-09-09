use super::{manifest_entries, manifest_entry_fdid, manifest_entry_is_allowed_unmapped};
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "wow-ui-sim-blizzard-ui-sync-{label}-{}-{unique}",
        std::process::id()
    ))
}

#[test]
fn default_cache_addons_path_is_profile_scoped_addons_root() {
    let path = super::default_cache_addons_path().expect("cache path");

    assert!(
        path.ends_with(PathBuf::from(crate::client_profile::ACTIVE.cache_subdir()).join("AddOns")),
        "cache path should end with profile/AddOns, got {}",
        path.display()
    );
}

#[test]
#[cfg(feature = "casc")]
fn fdid_extraction_uses_local_casc_before_cdn() {
    let out_path = PathBuf::from("Interface/AddOns/Test.lua");
    let calls = RefCell::new(Vec::new());

    let extracted = super::extract_fdid_with_cdn_fallback(
        42,
        &out_path,
        |fdid, path| {
            calls
                .borrow_mut()
                .push(format!("local:{fdid}:{}", path.display()));
            Ok(true)
        },
        |fdid, path| {
            calls
                .borrow_mut()
                .push(format!("cdn:{fdid}:{}", path.display()));
            Ok(true)
        },
    )
    .expect("extract");

    assert!(extracted);
    assert_eq!(
        calls.into_inner(),
        vec!["local:42:Interface/AddOns/Test.lua"]
    );
}

#[test]
#[cfg(feature = "casc")]
fn fdid_extraction_uses_cdn_after_local_casc_miss() {
    let out_path = PathBuf::from("Interface/AddOns/Test.lua");
    let calls = RefCell::new(Vec::new());

    let extracted = super::extract_fdid_with_cdn_fallback(
        42,
        &out_path,
        |fdid, path| {
            calls
                .borrow_mut()
                .push(format!("local:{fdid}:{}", path.display()));
            Ok(false)
        },
        |fdid, path| {
            calls
                .borrow_mut()
                .push(format!("cdn:{fdid}:{}", path.display()));
            Ok(true)
        },
    )
    .expect("extract");

    assert!(extracted);
    assert_eq!(
        calls.into_inner(),
        vec![
            "local:42:Interface/AddOns/Test.lua",
            "cdn:42:Interface/AddOns/Test.lua"
        ]
    );
}

#[test]
fn complete_marker_writes_profile_provenance() {
    let root = unique_temp_dir("provenance");

    super::write_complete_marker(&root).expect("write complete marker");

    let provenance =
        std::fs::read_to_string(root.join(super::PROVENANCE_FILE)).expect("read provenance");
    assert!(provenance.contains(&format!(
        "profile={}",
        crate::client_profile::ACTIVE.cache_subdir()
    )));
    assert!(provenance.contains("source=casc-local-or-cdn"));
    assert!(provenance.contains("fallback=none"));
    std::fs::remove_dir_all(root).expect("remove cache root");
}

#[test]
fn manifest_preserves_blizzard_addon_case() {
    let first = manifest_entries()
        .next()
        .expect("manifest should not be empty");
    assert!(first.starts_with("Blizzard_"));
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_manifest_includes_ptr_only_aura_container() {
    let manifest: Vec<_> = manifest_entries().collect();

    assert!(manifest.contains(&"Blizzard_AuraContainer/Blizzard_AuraContainer.toc"));
}

#[test]
#[cfg(feature = "client-retail")]
fn retail_manifest_excludes_ptr_only_aura_container() {
    let manifest: Vec<_> = manifest_entries().collect();

    assert!(!manifest.contains(&"Blizzard_AuraContainer/Blizzard_AuraContainer.toc"));
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_aura_container_resolves_through_limited_listfile() {
    let entry = "Blizzard_AuraContainer/Blizzard_AuraContainer.toc";

    assert_eq!(manifest_entry_fdid(entry), Some(8154511));
    assert!(!manifest_entry_is_allowed_unmapped(entry));
}

#[test]
fn manifest_entries_resolve_through_limited_listfile() {
    let missing: Vec<_> = manifest_entries()
        .filter(|entry| manifest_entry_fdid(entry).is_none())
        .filter(|entry| !manifest_entry_is_allowed_unmapped(entry))
        .take(10)
        .collect();
    assert!(
        missing.is_empty(),
        "unmapped Blizzard UI files: {missing:?}"
    );
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_sync_manifest_excludes_legacy_profile_entries() {
    let active: Vec<_> = super::sync_manifest_entries().collect();

    assert!(!active.contains(&"Blizzard_ActionBar/Classic/ActionButtonTemplate.xml"));
    assert!(!active.contains(&"Blizzard_UnitFrame/Mists/ShardBar.lua"));
    assert!(!active.contains(&"Blizzard_ChatFrame/Wrath/ChatConfigFrame.lua"));
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_sync_manifest_excludes_removed_world_map_entries() {
    let active: Vec<_> = super::sync_manifest_entries().collect();

    assert!(!active.contains(&"Blizzard_WorldMap/Blizzard_WorldMapTooltip.xml"));
    assert!(!active.contains(&"Blizzard_WorldMap/WM_InvasionDataProvider.lua"));
    assert!(!active.contains(&"Blizzard_WorldMap/WM_InvasionDataProvider.xml"));
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_achievement_shared_sources_are_required_and_synced() {
    let manifest: std::collections::HashSet<_> = manifest_entries().collect();
    for entry in [
        "Blizzard_AchievementUI/Classic/Blizzard_AchievementUI_Shared.lua",
        "Blizzard_AchievementUI/Classic/Localization.lua",
    ] {
        assert!(
            manifest.contains(entry),
            "Achievement TOC source missing: {entry}"
        );
        assert!(super::required_profile_cache_entries().contains(&entry));
        assert!(
            manifest_entry_fdid(entry).is_some(),
            "Achievement source needs a CASC ID: {entry}"
        );
    }
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_required_cache_entries_are_in_manifest() {
    let manifest: std::collections::HashSet<_> = manifest_entries().collect();

    for entry in super::required_profile_cache_entries() {
        assert!(
            manifest.contains(entry),
            "Mists cache-required file must be synced by the Blizzard UI manifest: {entry}"
        );
    }
}
#[test]
#[cfg(feature = "client-mists")]
fn mists_cache_is_incomplete_when_required_profile_files_are_missing() {
    let root = unique_temp_dir("mists-required-files");
    std::fs::create_dir_all(&root).expect("create cache root");

    assert!(
        !super::cache_has_required_profile_files(&root),
        "Mists cache marker must not be trusted when profile-required TOC files are absent"
    );

    std::fs::remove_dir_all(root).expect("remove cache root");
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_cache_rejects_old_classic_action_button_template() {
    let root = unique_temp_dir("mists-action-button-template");
    write_mists_required_cache_entries(&root);

    let action_button_template = root.join("Blizzard_ActionBar/Classic/ActionButtonTemplate.xml");
    std::fs::write(&action_button_template, "placeholder").expect("write placeholder");
    assert!(
        !super::cache_has_required_profile_files(&root),
        "Mists cache marker must not be trusted when ActionButtonTemplate.xml is the old Classic Era variant"
    );

    std::fs::write(
            action_button_template,
            r#"<CheckButton name="ActionBarButtonTemplate"><Cooldown parentKey="chargeCooldown"/></CheckButton>"#,
        )
        .expect("write Mists-compatible action button template");
    assert!(
        super::cache_has_required_profile_files(&root),
        "Mists cache should be complete when required files exist and ActionButtonTemplate.xml defines ActionBarButtonTemplate"
    );

    std::fs::remove_dir_all(root).expect("remove cache root");
}

#[test]
#[cfg(feature = "client-mists")]
fn mists_cache_rejects_mainline_nameplates_toc_without_game_type_gates() {
    let root = unique_temp_dir("mists-nameplates-toc");
    write_mists_required_cache_entries(&root);

    let nameplates_toc = root.join("Blizzard_NamePlates/Blizzard_NamePlates.toc");
    std::fs::write(&nameplates_toc, "Blizzard_ClassNameplateBar.lua\n")
        .expect("write ungated nameplates toc");
    assert!(
        !super::cache_has_required_profile_files(&root),
        "Mists cache marker must not be trusted when Blizzard_NamePlates.toc would load Mainline class bar files"
    );

    std::fs::write(
        nameplates_toc,
        "Mainline\\Blizzard_ClassNameplateBar.lua [AllowLoadGameType mainline]\n",
    )
    .expect("write Mists-compatible nameplates toc");
    assert!(
        super::cache_has_required_profile_files(&root),
        "Mists cache should be complete when Blizzard_NamePlates.toc preserves Mainline game-type gates"
    );

    std::fs::remove_dir_all(root).expect("remove cache root");
}

#[cfg(feature = "client-mists")]
include!("../blizzard_ui_sync_mists_test_fixture.rs");
