//! Mists action and bag layout, loaded from the active Blizzard profile.
#![cfg(feature = "client-mists")]

use crate::common;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn build_mists_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Mists Lua runtime should initialize");
    env.set_screen_size(1024.0, 768.0);
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Mists Blizzard cache should be available");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("{name} should load: {error}"));
        if name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

#[test]
fn mists_action_buttons_and_bag_slots_use_classic_layout() {
    let env = common::lock_env(build_mists_env);
    let result: String = env.eval(r#"
        assert(MainMenuBar and MainMenuBar:IsVisible(), "Mists main menu bar missing or hidden")
        assert(MainMenuBarArtFrame, "Mists action bar art frame missing")
        for i = 1, 12 do
            local button = _G["ActionButton" .. i]
            assert(button and button:IsVisible(), "Mists action button missing or hidden: " .. i)
        end
        local backpack = MainMenuBarBackpackButton
        assert(backpack and backpack:IsVisible(), "Mists backpack missing or hidden")
        assert(backpack:GetParent() == MainMenuBarArtFrame, "Mists backpack parent changed")
        assert(backpack:GetWidth() == 30 and backpack:GetHeight() == 30, "Mists backpack size changed")
        local previous = backpack
        for i = 0, 3 do
            local bag = _G["CharacterBag" .. i .. "Slot"]
            assert(bag and bag:IsVisible(), "Mists bag missing or hidden: " .. i)
            assert(bag:GetParent() == MainMenuBarArtFrame, "Mists bag parent changed: " .. i)
            local point, relative, relativePoint, x, y = bag:GetPoint(1)
            assert(point == "RIGHT" and relative == previous and relativePoint == "LEFT"
                and x == -2 and y == 0, "Mists bag anchor changed: " .. i)
            local left, bottom, width, height = bag:GetRect()
            local previousLeft, previousBottom, _, previousHeight = previous:GetRect()
            assert(width == 30 and height == 30, "Mists bag size changed: " .. i)
            assert(math.abs(left + width - (previousLeft - 2)) < 0.1, "Mists bag spacing changed: " .. i)
            assert(math.abs(bottom + height / 2 - (previousBottom + previousHeight / 2)) < 0.1,
                "Mists bag alignment changed: " .. i)
            previous = bag
        end
        assert(CharacterReagentBag0Slot == nil, "Retail reagent slot leaked into Mists")
        return "ok"
    "#).expect("Mists action and bag layout should match its real XML");
    assert_eq!(result, "ok");
    assert!(env.state().borrow().lua_errors.is_empty(), "Mists startup Lua errors: {:?}",
        env.state().borrow().lua_errors);
}
