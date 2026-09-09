#![cfg(feature = "client-mists")]

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;
use wow_ui_sim::xml::{FrameXml, XmlElement};

type MistsStartupApiShape = (
    i32,
    i32,
    String,
    i32,
    i32,
    bool,
    i32,
    i32,
    i32,
    i32,
    i32,
    String,
);

fn blizzard_ui_dir() -> std::path::PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn mists_lua_source(relative_path: &str) -> String {
    std::fs::read_to_string(blizzard_ui_dir().join(relative_path)).unwrap_or_else(|error| {
        panic!("Mists Lua source {relative_path} should be readable: {error}")
    })
}

fn mists_money_input_frame_xml_path() -> std::path::PathBuf {
    blizzard_ui_dir().join("Blizzard_MoneyFrame/Classic/MoneyInputFrame.xml")
}

fn mists_blizzard_toc(addon: &str, toc_name: &str) -> std::path::PathBuf {
    blizzard_ui_dir().join(addon).join(toc_name)
}

fn load_mists_money_frame_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    let fonts_toc = mists_blizzard_toc("Blizzard_Fonts_Shared", "Blizzard_Fonts_Shared.toc");
    load_addon(&env.loader_env(), &fonts_toc).expect("Blizzard_Fonts_Shared should load");

    let toc_path = mists_blizzard_toc("Blizzard_MoneyFrame", "Blizzard_MoneyFrame_Classic.toc");
    load_addon(&env.loader_env(), &toc_path).expect("Blizzard_MoneyFrame should load");
    env
}

fn find_top_level_frame<'a>(elements: &'a [XmlElement], name: &str) -> &'a FrameXml {
    elements
        .iter()
        .find_map(|element| match element {
            XmlElement::Frame(frame) if frame.name.as_deref() == Some(name) => Some(frame),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected top-level frame template {name}"))
}

#[test]
fn mists_bootstrap_reports_pandaria_as_the_current_classic_expansion() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (i32, bool, bool, bool, bool) = env
        .eval(
            r#"
            return GetExpansionLevel(),
                ClassicExpansionAtLeast(LE_EXPANSION_MISTS_OF_PANDARIA),
                ClassicExpansionAtMost(LE_EXPANSION_MISTS_OF_PANDARIA),
                ClassicExpansionAtLeast(5),
                ClassicExpansionAtMost(LE_EXPANSION_CATACLYSM)
            "#,
        )
        .expect("Mists expansion helpers should be callable");

    assert_eq!(
        result,
        (4, true, true, false, false),
        "Mists Classic should report MoP as the current classic expansion"
    );
}

#[test]
fn mists_bootstrap_exposes_classic_addon_compatibility_globals() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let build_info: (String, String, String, i32) = env
        .eval(
            r#"
            local version, build, date, interface = GetBuildInfo()
            return version, build, date, interface
            "#,
        )
        .expect("Mists build info should be callable");

    assert_eq!(
        build_info,
        (
            "5.5.4".to_string(),
            "68042".to_string(),
            "Jun 2 2026".to_string(),
            50504,
        ),
        "Mists build info should match the pinned 5.5.4 Blizzard UI source"
    );

    let result: (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            return GetRaidDifficultyID() == 14,
                GetLegacyRaidDifficultyID() == 3,
                type(GetTimePreciseSec()) == "number",
                Enum.ItemQuality.Good == Enum.ItemQuality.Uncommon,
                type(time({ year = "2026", month = "5", day = "16", hour = "12", min = "30", sec = "45" })) == "number"
            "#,
        )
        .expect("Mists addon compatibility globals should be callable");

    assert_eq!(
        result,
        (true, true, true, true, true,),
        "Mists should expose classic-era globals used by installed addons"
    );
}

#[test]
fn mists_rotation_animation_getters_round_trip_rotation_state() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let result: (f64, String, f64, f64) = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "TestMistsRotationGetters", UIParent)
            local ag = f:CreateAnimationGroup()
            local anim = ag:CreateAnimation("Rotation")
            anim:SetDegrees(135)
            anim:SetOrigin("TOPLEFT", 4, -7)
            local point, x, y = anim:GetOrigin()
            return anim:GetDegrees(), point, x, y
            "#,
        )
        .expect("rotation animation getters should be callable in Mists");

    assert_eq!(result, (135.0, "TOPLEFT".to_string(), 4.0, -7.0));
}

#[test]
fn mists_bootstrap_exposes_round_helper_for_addon_migrations() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (f64, f64, f64) = env
        .eval(
            r#"
            return Round(2.6), Round(2.34, 1), Round(-2.6)
            "#,
        )
        .expect("Mists Round helper should be callable");

    assert_eq!(
        result,
        (3.0, 2.3, -3.0),
        "Mists should expose the global Round helper used by addon migrations"
    );
}

#[test]
fn mists_bootstrap_exposes_degree_tangent_and_preserves_existing_global() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    env.exec(include_str!("../src/mists/compat_bootstrap.lua"))
        .expect("Mists compatibility bootstrap should load");

    let baseline: (f64, f64) = env
        .eval(
            r#"
            return tan(45), tan(180)
            "#,
        )
        .expect("Mists degree tangent helper should be callable");

    env.exec(
        r#"
        local existing = function(x)
            return x * 10
        end
        _G.__mists_existing_tan = existing
        _G.tan = existing
        "#,
    )
    .expect("existing tangent sentinel should be installable");
    env.exec(include_str!("../src/mists/compat_bootstrap.lua"))
        .expect("Mists compatibility bootstrap should be rerunnable");

    let preserved: (f64, bool) = env
        .eval(
            r#"
            return tan(7), tan == _G.__mists_existing_tan
            "#,
        )
        .expect("preserved tangent helper should be callable");

    assert!(
        (baseline.0 - 1.0).abs() < 1e-12,
        "tan should interpret 45 as degrees: {}",
        baseline.0
    );
    assert!(
        baseline.1.abs() < 1e-12,
        "tan should interpret 180 as degrees: {}",
        baseline.1
    );
    assert_eq!(
        preserved,
        (70.0, true),
        "bootstrap should preserve an existing global tan"
    );
}

#[test]
fn mists_bootstrap_exposes_legacy_table_helpers_and_degree_atan2_without_runtime_providers() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    env.exec(
        r#"
        _G.tinsert = nil
        _G.tremove = nil
        _G.wipe = nil
        _G.atan2 = nil
        if table ~= nil then
            table.wipe = nil
        end
        "#,
    )
    .expect("Mists runtime providers should be removable in the disposable environment");
    env.exec(include_str!("../src/mists/compat_bootstrap.lua"))
        .expect("Mists compatibility bootstrap should load without runtime providers");

    let behavior: (i32, i32, i32, f64, f64, f64, bool) = env
        .eval(
            r#"
            local values = { "a", "b" }
            tinsert(values, "c")
            local removed = tremove(values, 1)
            local wiped = { answer = 42 }
            local returned = wipe(wiped)
            return #values, removed == "a" and values[1] == "b" and values[2] == "c" and 2 or 0,
                next(wiped) == nil and returned == wiped and 1 or 0,
                atan2(1, 2), atan2(1, 0), atan2(0, -1), type(tinsert) == "function"
            "#,
        )
        .expect("Mists legacy table and angle helpers should be callable");

    assert_eq!(behavior.0, 2);
    assert_eq!(
        behavior.1, 2,
        "tinsert/tremove should delegate to table APIs"
    );
    assert_eq!(
        behavior.2, 1,
        "wipe fallback should clear and return its input table"
    );
    assert!(
        (behavior.3 - 26.56505117707799).abs() < 1e-12,
        "atan2 should use y,x order and return degrees: {}",
        behavior.3
    );
    assert!(
        (behavior.4 - 90.0).abs() < 1e-12,
        "atan2 should return degrees: {}",
        behavior.4
    );
    assert!(
        (behavior.5 - 180.0).abs() < 1e-12,
        "atan2 should preserve the signed quadrant: {}",
        behavior.5
    );
    assert!(behavior.6);
}

#[test]
fn mists_bootstrap_preserves_existing_legacy_helper_sentinels() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    env.exec(
        r#"
        local existing_tinsert = function() return "tinsert sentinel" end
        local existing_tremove = function() return "tremove sentinel" end
        local existing_wipe = function() return "wipe sentinel" end
        local existing_atan2 = function() return "atan2 sentinel" end
        _G.__mists_existing_tinsert = existing_tinsert
        _G.__mists_existing_tremove = existing_tremove
        _G.__mists_existing_wipe = existing_wipe
        _G.__mists_existing_atan2 = existing_atan2
        _G.tinsert = existing_tinsert
        _G.tremove = existing_tremove
        _G.wipe = existing_wipe
        _G.atan2 = existing_atan2
        "#,
    )
    .expect("existing legacy helper sentinels should be installable");
    env.exec(include_str!("../src/mists/compat_bootstrap.lua"))
        .expect("Mists compatibility bootstrap should be rerunnable");

    let preserved: (bool, bool, bool, bool) = env
        .eval(
            r#"
            return tinsert == __mists_existing_tinsert,
                tremove == __mists_existing_tremove,
                wipe == __mists_existing_wipe,
                atan2 == __mists_existing_atan2
            "#,
        )
        .expect("preexisting legacy helper sentinels should remain callable");
    assert_eq!(preserved, (true, true, true, true));
}

#[test]
fn mists_bootstrap_exposes_guarded_reverse_ipairs_iterator() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    env.exec(include_str!("../src/mists/compat_bootstrap.lua"))
        .expect("Mists compatibility bootstrap should load");

    let baseline: (String, i32) = env
        .eval(
            r#"
            local values = { "a", "b", "c" }
            local order = {}
            for index, value in ipairs_reverse(values) do
                order[#order + 1] = index .. value
            end

            local emptyCount = 0
            for _ in ipairs_reverse({}) do
                emptyCount = emptyCount + 1
            end

            return table.concat(order, ","), emptyCount
            "#,
        )
        .expect("Mists reverse ipairs helper should be callable");

    env.exec(
        r#"
        local existing = function(tbl)
            local function Enumerator(list, index)
                index = index - 1
                local value = list[index]
                if value ~= nil then
                    return index, value
                end
            end
            return Enumerator, tbl, #tbl + 1
        end
        _G.__mists_existing_ipairs_reverse = existing
        _G.ipairs_reverse = existing
        "#,
    )
    .expect("existing reverse ipairs sentinel should be installable");
    env.exec(include_str!("../src/mists/compat_bootstrap.lua"))
        .expect("Mists compatibility bootstrap should be rerunnable");

    let preserved: (String, i32, bool) = env
        .eval(
            r#"
            local values = { "a", "b", "c" }
            local order = {}
            for index, value in ipairs_reverse(values) do
                order[#order + 1] = index .. value
            end

            local emptyCount = 0
            for _ in ipairs_reverse({}) do
                emptyCount = emptyCount + 1
            end

            return table.concat(order, ","), emptyCount,
                ipairs_reverse == _G.__mists_existing_ipairs_reverse
            "#,
        )
        .expect("preserved reverse ipairs helper should be callable");

    assert_eq!(baseline, ("3c,2b,1a".to_string(), 0));
    assert_eq!(
        preserved,
        ("3c,2b,1a".to_string(), 0, true),
        "reverse ipairs should iterate densely and preserve an existing global"
    );
}

#[test]
fn mists_bootstrap_exposes_raid_marker_system_probe() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let enabled: bool = env
        .eval("return IsRaidMarkerSystemEnabled()")
        .expect("Mists raid marker system probe should be callable");

    assert!(
        !enabled,
        "headless simulator should leave raid marker placement disabled by default"
    );
}

#[test]
fn mists_toc_game_token_resolves_to_mists_subdirectory() {
    let toc = TocFile::parse(
        std::path::Path::new("Blizzard_CharacterFrame"),
        r#"
        ## Interface: 50503
        [Game]\PaperDollFrameUtil.lua [AllowLoadGameType cata, mists]
        "#,
    );

    assert_eq!(
        toc.files,
        vec![std::path::PathBuf::from("Mists/PaperDollFrameUtil.lua")],
        "Mists TOC [Game] token should select the Mists source variant"
    );
}

#[test]
fn mists_loads_money_frame_before_uipanels_game() {
    let addons = wow_ui_sim::loader::discover_blizzard_addons_for_screen(
        &blizzard_ui_dir(),
        ScreenKind::Game,
    );
    let money_frame_index = addons
        .iter()
        .position(|(name, _)| name == "Blizzard_MoneyFrame")
        .expect("Mists game startup should include Blizzard_MoneyFrame");
    let ui_panels_index = addons
        .iter()
        .position(|(name, _)| name == "Blizzard_UIPanels_Game")
        .expect("Mists game startup should include Blizzard_UIPanels_Game");

    assert!(
        money_frame_index < ui_panels_index,
        "Blizzard_MoneyFrame must load before Blizzard_UIPanels_Game so MoneyInputFrameTemplate exists before TradeFrame.xml instantiates TradePlayerInputMoneyFrame; indexes were MoneyFrame={money_frame_index}, UIPanels={ui_panels_index}"
    );
}

#[test]
fn mists_money_input_template_xml_wires_gold_silver_copper_parent_keys() {
    let ui = wow_ui_sim::xml::parse_xml_file(&mists_money_input_frame_xml_path())
        .expect("Mists MoneyInputFrame.xml should parse");
    let template = find_top_level_frame(&ui.elements, "MoneyInputFrameTemplate");

    let child_keys: Vec<&str> = template
        .all_frame_elements()
        .into_iter()
        .filter_map(|(frame, _tag)| frame.parent_key.as_deref())
        .collect();

    assert!(
        child_keys.contains(&"gold"),
        "MoneyInputFrameTemplate should wire the gold edit box via parentKey"
    );
    assert!(
        child_keys.contains(&"silver"),
        "MoneyInputFrameTemplate should wire the silver edit box via parentKey"
    );
    assert!(
        child_keys.contains(&"copper"),
        "MoneyInputFrameTemplate should wire the copper edit box via parentKey"
    );
}

#[test]
fn mists_money_input_template_runtime_syncs_coin_parent_keys() {
    let env = load_mists_money_frame_env();

    let result: (String, String, String, String, String, String) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "MoneyInputFrameParentKeyProbe", UIParent, "MoneyInputFrameTemplate")
            return type(frame.gold),
                type(frame.silver),
                type(frame.copper),
                frame.gold:GetName(),
                frame.silver:GetName(),
                frame.copper:GetName()
            "#,
        )
        .expect("MoneyInputFrameTemplate should instantiate under CreateFrame");

    assert_eq!(
        result,
        (
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "MoneyInputFrameParentKeyProbeGold".to_string(),
            "MoneyInputFrameParentKeyProbeSilver".to_string(),
            "MoneyInputFrameParentKeyProbeCopper".to_string()
        ),
        "MoneyInputFrameTemplate inheritance and parentKey sync should publish gold/silver/copper children; missing copper is not a MoneyFrame API-state issue"
    );
}

#[test]
fn mists_trade_player_input_money_frame_widget_has_copper_child() {
    let env = load_mists_money_frame_env();

    let result: (String, String, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "TradePlayerInputMoneyFrame", UIParent, "MoneyInputFrameTemplate")
            return type(frame.copper),
                TradePlayerInputMoneyFrameCopper:GetName(),
                frame.copper == TradePlayerInputMoneyFrameCopper
            "#,
        )
        .expect("TradePlayerInputMoneyFrame should instantiate from MoneyInputFrameTemplate");

    assert_eq!(
        result,
        (
            "table".to_string(),
            "TradePlayerInputMoneyFrameCopper".to_string(),
            true
        ),
        "TradePlayerInputMoneyFrame must expose its copper edit box through both parentKey and named global wiring"
    );
}

#[test]
fn mists_world_map_set_opacity_reproduces_nil_opacity_arithmetic() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let source = mists_lua_source("Blizzard_WorldMap/Cata/Blizzard_WorldMap.lua");
    env.exec(&source)
        .expect("Cata/Mists WorldMap Lua should define opacity helpers");

    let (ok, err): (bool, String) = env
        .eval(
            r#"
            WorldMapFrame = {
                SetAlpha = function() end,
                ScrollContainer = { SetAlpha = function() end },
            }
            QuestMapFrame = { SetAlpha = function() end }

            local ok, err = pcall(WorldMapFrame_SetOpacity, nil)
            return ok, tostring(err)
            "#,
        )
        .expect("WorldMapFrame_SetOpacity pcall should return a status");

    assert!(!ok, "nil world map opacity should fail during arithmetic");
    assert!(
        err.contains("opacity") || err.contains("arithmetic") || err.contains("nil"),
        "expected nil opacity arithmetic failure, got: {err}"
    );
}

#[test]
fn mists_bootstrap_supplies_legacy_startup_api_shapes() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let result = query_mists_startup_api_shape(&env);

    assert_eq!(
        result,
        expected_mists_startup_api_shape(),
        "Mists startup helpers should return non-nil legacy API shapes"
    );
}

fn query_mists_startup_api_shape(env: &WowLuaEnv) -> MistsStartupApiShape {
    env.eval(
        r#"
            local skillName, _header, _isExpanded, skillRank, tempPoints, _modifier, skillMaxRank = GetSkillLineInfo(GetSelectedSkill())
            SetGuildRosterSelection(7)
            local hk, contribution = GetPVPThisWeekStats()
            return GetNumClasses(),
                GetSelectedSkill(),
                skillName,
                skillRank,
                tempPoints,
                HonorSystemEnabled(),
                hk,
                contribution,
                GetCurrencyListSize(),
                GetGuildRosterSelection(),
                skillMaxRank,
                type(C_ProductChoice.GetChoices())
            "#,
    )
    .expect("Mists legacy startup APIs should be callable")
}

fn expected_mists_startup_api_shape() -> MistsStartupApiShape {
    (
        11,
        1,
        "Weapon Skills".to_string(),
        1,
        0,
        false,
        0,
        0,
        19,
        7,
        1,
        "table".to_string(),
    )
}

#[test]
fn mists_honor_frame_shared_reproduces_missing_honor_system_enabled() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        rawset(_G, "HonorSystemEnabled", nil)
        HonorFrame_GetCurrencyFrame = function()
            return {
                Hide = function() end,
                Show = function() end,
            }
        end
        "#,
    )
    .expect("install HonorFrame reproduction fixtures");
    let source = mists_lua_source("Blizzard_UIPanels_Game/Classic/HonorFrame_Shared.lua");
    env.exec(&source)
        .expect("HonorFrame_Shared.lua should define functions before OnLoad runs");

    let (ok, err): (bool, String) = env
        .eval(
            r#"
            local frame = { RegisterEvent = function() end }
            local ok, err = pcall(HonorFrame_OnLoad, frame)
            return ok, tostring(err)
            "#,
        )
        .expect("HonorFrame_OnLoad pcall should return a status");

    assert!(!ok, "HonorFrame_OnLoad should reproduce the nil global");
    assert!(
        err.contains("nil"),
        "expected a nil-call failure from missing HonorSystemEnabled, got: {err}"
    );
}

#[test]
fn mists_honor_system_enabled_matches_disabled_legacy_honor_surface() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let (kind, enabled): (String, bool) = env
        .eval(
            r#"
            return type(HonorSystemEnabled()), HonorSystemEnabled()
            "#,
        )
        .expect("HonorSystemEnabled should be callable in Mists");

    assert_eq!(
        (kind, enabled),
        ("boolean".to_string(), false),
        "MoP Classic keeps HonorSystemEnabled as a global boolean gate; false hides the legacy HonorFrame honor currency surface"
    );
}

#[test]
fn mists_honor_pvp_api_contract_matches_classic_shapes() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    {
        let mut state = env.state().borrow_mut();
        state.player.honor_level = 5;
        state.pvp_honor.classic_honor_system_enabled = true;
        state.pvp_honor.yesterday_honorable_kills = 11;
        state.pvp_honor.yesterday_dishonorable_kills = 1;
        state.pvp_honor.this_week_honorable_kills = 22;
        state.pvp_honor.this_week_contribution = 330;
        state.pvp_honor.last_week_honorable_kills = 44;
        state.pvp_honor.last_week_dishonorable_kills = 2;
        state.pvp_honor.last_week_contribution = 550;
        state.pvp_honor.last_week_rank = 6;
        state.pvp_honor.lifetime_honorable_kills = 88;
        state.pvp_honor.lifetime_highest_rank = 9;
        state.pvp_honor.rank_progress = 0.25;
    }

    let shape: (String, bool, i32, i32, i32, i32) = env
        .eval(
            r##"
            return type(HonorSystemEnabled()),
                HonorSystemEnabled(),
                select("#", GetPVPYesterdayStats()),
                select("#", GetPVPThisWeekStats()),
                select("#", GetPVPLastWeekStats()),
                select("#", GetPVPLifetimeStats())
            "##,
        )
        .expect("Mists honor/PvP APIs should expose Classic call shapes");
    let values: (i32, i32, i32, i32, i32, i32, i32, i32, String, i32, f64) = env
        .eval(
            r##"
            local yesterdayHK, yesterdayDK = GetPVPYesterdayStats()
            local weekHK, weekContribution = GetPVPThisWeekStats()
            local lastWeekHK, lastWeekDK = GetPVPLastWeekStats()
            local lifetimeHK, _, highestRank = GetPVPLifetimeStats()
            local rankName, rankNumber = GetPVPRankInfo(UnitPVPRank("player"))
            return
                yesterdayHK,
                yesterdayDK,
                weekHK,
                weekContribution,
                lastWeekHK,
                lastWeekDK,
                lifetimeHK,
                highestRank,
                rankName,
                rankNumber,
                GetPVPRankProgress()
            "##,
        )
        .expect("Mists honor/PvP APIs should read simulator state");

    assert_eq!(
        shape,
        ("boolean".to_string(), true, 2, 2, 4, 3),
        "Mists honor/PvP startup APIs should keep the Classic return shapes HonorFrame_Shared.lua consumes"
    );
    assert_eq!(
        values,
        (11, 1, 22, 330, 44, 2, 88, 9, "Rank".to_string(), 5, 0.25),
        "Mists honor/PvP startup APIs should read Classic return shapes from simulator state"
    );
}

#[test]
fn mists_trade_money_frame_onload_reproduces_missing_copper_child() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let (ok, err): (bool, String) = env
        .eval(
            r#"
            MoneyInputFrame_SetCompact = function() end
            MoneyInputFrame_SetOnValueChangedFunc = function() end
            TradeFrame_UpdateMoney = function() end

            local frame = {
                silver = { SetPoint = function() end },
                RegisterEvent = function() end,
            }

            local ok, err = pcall(function()
                MoneyInputFrame_SetCompact(frame, 56, 7)
                MoneyInputFrame_SetOnValueChangedFunc(frame, TradeFrame_UpdateMoney)
                frame:RegisterEvent("PLAYER_TRADE_MONEY")
                frame.copper:SetPoint("LEFT", "TradePlayerInputMoneyFrameSilver", "RIGHT", 11, 0)
                frame.silver:SetPoint("LEFT", "TradePlayerInputMoneyFrameGold", "RIGHT", 22, 0)
            end)

            return ok, tostring(err)
            "#,
        )
        .expect("TradePlayerInputMoneyFrame OnLoad reproduction should run under pcall");

    assert!(!ok, "missing TradePlayerInputMoneyFrame.copper should fail");
    assert!(
        err.contains("copper"),
        "expected missing copper child failure, got: {err}"
    );
}

#[test]
fn mists_bootstrap_registers_startup_cvar_defaults() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String, String, bool, bool) = env
        .eval(
            r#"
            return GetCVar("worldMapOpacity"),
                GetCVar("NamePlateHorizontalScale"),
                GetCVar("NamePlateVerticalScale"),
                GetCVarBool("ShowClassColorInFriendlyNameplate"),
                GetCVarBool("ColorNameplateNameBySelection")
            "#,
        )
        .expect("Mists startup CVars should be readable");

    assert_eq!(
        result,
        (
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            true,
            false
        ),
        "Mists startup CVars should have concrete defaults"
    );
}

#[test]
fn mists_skill_line_zero_selection_still_returns_numeric_rank() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (i32, String, i32, i32, i32) = env
        .eval(
            r#"
            SetSelectedSkill(0)
            local skillName, _header, _isExpanded, skillRank, tempPoints, _modifier, skillMaxRank = GetSkillLineInfo(GetSelectedSkill())
            return GetSelectedSkill(), skillName, skillRank, tempPoints, skillMaxRank
            "#,
        )
        .expect("Mists skill selection zero should still have a startup row");

    assert_eq!(
        result,
        (0, "Weapon Skills".to_string(), 1, 0, 1),
        "Mists SkillFrame startup should not receive nil rank fields"
    );
}

#[test]
fn mists_selected_skill_api_matches_skill_frame_tuple_shape() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (i32, String, bool, bool, i32, i32, i32, i32, bool, bool) = env
        .eval(
            r#"
            SetSelectedSkill(1)
            local selected = GetSelectedSkill()
            local skillName, header, isExpanded, skillRank, tempPoints,
                skillModifier, skillMaxRank, isAbandonable = GetSkillLineInfo(selected)
            local missingSkill = GetSkillLineInfo(2) == nil
            return selected,
                skillName,
                header,
                isExpanded,
                skillRank,
                tempPoints,
                skillModifier,
                skillMaxRank,
                isAbandonable,
                missingSkill
            "#,
        )
        .expect("Mists selected skill API should return the tuple SkillFrame destructures");

    assert_eq!(
        result,
        (
            1,
            "Weapon Skills".to_string(),
            false,
            false,
            1,
            0,
            0,
            1,
            false,
            true
        ),
        "SkillFrame expects concrete rank numbers for selected rows and nil for out-of-range rows"
    );
}

#[test]
fn mists_bootstrap_supplies_pvp_currency_and_debugbar_shapes() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            local rankName, rankNumber = GetPVPRankInfo(0)
            local honor = C_CurrencyInfo.GetCurrencyInfo(Constants.CurrencyConsts.CLASSIC_HONOR_CURRENCY_ID)
            local lifetimeKills, lifetimeDishonorable, highestRank = GetPVPLifetimeStats()
            return rankName, rankNumber, honor.quantity, DebugBarManager:GetScaledInternalBarsHeight(),
                UnitPVPRank("player"), highestRank
            "#,
        )
        .expect("Mists PVP, currency, and debug bar startup APIs should be callable");

    assert_eq!(
        result,
        ("None".to_string(), 0, 0, 0, 0, 0),
        "Mists startup helpers should return concrete numeric fields"
    );
}

#[test]
fn mists_bootstrap_supplies_legacy_bank_and_aura_shapes() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (i32, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            local _name, _icon, _count, _debuffType, duration, expirationTime,
                _source, _stealable, _personal, _spellID, _unused1, _unused2, _unused3,
                _unused4, timeMod = UnitBuff("player", 1)
            return NUM_BANKGENERIC_SLOTS,
                NUM_BANKBAGSLOTS,
                Constants.InventoryConstants.NumGenericBankSlots,
                Constants.InventoryConstants.NumBankBagSlots,
                duration,
                timeMod
            "#,
        )
        .expect("Mists bank constants and legacy aura tuple should be callable");

    assert_eq!(
        result,
        (28, 7, 28, 7, 3600, 1),
        "Mists startup should see bank constants and the legacy UnitBuff tuple shape"
    );
}

#[test]
fn mists_bootstrap_supplies_settings_label_globals() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String) = env
        .eval(
            r#"
            return SHOW_AGGRO_PERCENTAGES,
                SHOW_COMBAT_HEALING_TEXT
            "#,
        )
        .expect("Mists settings labels should be strings");

    assert_eq!(
        result,
        ("Show aggro percentages".to_string(), "Healing".to_string()),
        "Mists settings variables should not register nil display names"
    );
}

#[test]
fn mists_bootstrap_supplies_hidden_non_priest_priest_bar_mixins() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "MistsPriestBarMixinProbe", UIParent)
            PriestBarMixin.OnLoad(frame)
            return type(PriestBarMixin.OnLoad),
                type(PriestBarOrbMixin),
                frame:IsShown()
            "#,
        )
        .expect("Mists priest bar mixin probe should run");

    assert_eq!(
        result,
        ("function".to_string(), "table".to_string(), false),
        "Mists should define priest bar mixins and hide the frame for non-priest startup"
    );
}

#[test]
fn mists_intrinsic_templates_supply_legacy_item_button_template_children() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String, String, String, String) = env
        .eval(
            r#"
            local button = CreateFrame("CheckButton", "MistsLegacyBagSlotTemplateProbe", UIParent, "ItemButtonTemplate")
            return type(button),
                type(MistsLegacyBagSlotTemplateProbeCount),
                type(MistsLegacyBagSlotTemplateProbeNormalTexture),
                type(button.Count),
                type(button.NormalTexture)
            "#,
        )
        .expect("Mists legacy ItemButtonTemplate probe should run");

    assert_eq!(
        result,
        (
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
        ),
        "Mists bag slot XML expects ItemButtonTemplate to create Count and NormalTexture children"
    );
}

#[test]
fn mists_create_forbidden_frame_forwards_to_create_frame() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String, bool) = env
        .eval(
            r#"
            local frame = CreateForbiddenFrame("Button", "MistsForbiddenProbe", UIParent, "UIPanelButtonTemplate")
            return frame:GetObjectType(), frame:GetName(), frame:IsForbidden()
            "#,
        )
        .expect("CreateForbiddenFrame should create a real forbidden frame");

    assert_eq!(
        result,
        (
            "Button".to_string(),
            "MistsForbiddenProbe".to_string(),
            true
        ),
        "CreateForbiddenFrame should preserve CreateFrame semantics and mark the frame forbidden"
    );
}
