#![cfg(feature = "client-mists")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn mists_lua_source(relative_path: &str) -> String {
    std::fs::read_to_string(
        wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
            "CARGO_MANIFEST_DIR"
        )))
        .join(relative_path),
    )
    .unwrap_or_else(|error| panic!("Mists Lua source {relative_path} should be readable: {error}"))
}

fn load_club_finder_lua(env: &WowLuaEnv) {
    let loader = format!(
        r#"
        CLUB_FINDER_TANK = "Tank"
        CLUB_FINDER_HEALER = "Healer"
        CLUB_FINDER_DAMAGE = "Damage"
        CLUB_FINDER_ANY_FLAG = "Any"
        CLUB_FINDER_MULTIPLE_ROLES = "Multiple"
        CHECK_ALL = "Check All"
        UNCHECK_ALL = "Uncheck All"
        TALENT_SPEC_AND_CLASS = "%s %s"
        MenuResponse = {{ Refresh = "refresh" }}

        local source = [==[
{}
        ]==]
        local chunk = assert(loadstring(source))
        chunk("Blizzard_Communities", {{}})
        "#,
        mists_lua_source("Blizzard_Communities/ClubFinder.lua")
    );

    env.exec(&loader)
        .expect("Mists ClubFinder Lua should define dropdown mixins");
}

fn load_color_util_lua(env: &WowLuaEnv) {
    let source = mists_lua_source("Blizzard_SharedXML/ColorUtil.lua");
    env.exec(&source)
        .expect("Mists ColorUtil Lua should define class color helpers");
}

#[test]
fn club_finder_setup_menu_reproduces_missing_class_color() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    load_club_finder_lua(&env);

    let (ok, err): (bool, String) = env
        .eval(
            r#"
            DropdownButtonMixin = {
                SetupMenu = function(_self, initializer)
                    local submenu = {
                        CreateButton = function() end,
                        CreateCheckbox = function() end,
                    }
                    local rootDescription = {
                        SetTag = function() end,
                        CreateCheckbox = function()
                            return submenu
                        end,
                    }
                    initializer({}, rootDescription)
                end,
            }
            C_SpecializationInfo = {
                GetNumSpecializationsForClassID = function()
                    return 1
                end,
            }
            GetNumClasses = function()
                return 1
            end
            GetClassInfo = function()
                return "Broken Class", "BROKENCLASS", 99
            end
            GetClassColorObj = function()
                return nil
            end
            GetSpecializationInfoForClassID = function()
                return 9901, "Broken Spec", nil, nil, "TANK"
            end
            UnitSex = function()
                return 2
            end

            local dropdown = {
                SetSelectionText = function() end,
            }
            setmetatable(dropdown, { __index = ClubLookingForDropdownMixin })

            local ok, err = pcall(dropdown.SetupMenu, dropdown)
            return ok, tostring(err)
            "#,
        )
        .expect("ClubLookingForDropdownMixin:SetupMenu pcall should return a status");

    assert!(!ok, "ClubFinder setup should reproduce the nil class color");
    assert!(
        err.contains("classColor") || err.contains("WrapTextInColorCode"),
        "expected nil classColor failure, got: {err}"
    );
}

#[test]
fn mists_visible_classes_have_color_data() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    load_color_util_lua(&env);

    let result: (i32, String, String, String, String) = env
        .eval(
            r#"
            local missing = {}
            local seen = {}

            for classIndex = 1, GetNumClasses() do
                local _className, classFile = GetClassInfo(classIndex)
                seen[classFile] = true
                if not GetClassColorObj(classFile) then
                    table.insert(missing, classFile)
                end
            end

            return GetNumClasses(),
                table.concat(missing, ","),
                tostring(seen.MONK),
                tostring(seen.DEMONHUNTER),
                tostring(seen.EVOKER)
            "#,
        )
        .expect("Mists visible classes should be color-backed");

    assert_eq!(
        result,
        (
            11,
            String::new(),
            "true".to_string(),
            "nil".to_string(),
            "nil".to_string()
        ),
        "Mists should expose only classes with RAID_CLASS_COLORS entries"
    );
}

#[test]
fn visible_classes_have_localized_names_for_chat_config() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        LOCALIZED_CLASS_NAMES_MALE = {}
        LOCALIZED_CLASS_NAMES_FEMALE = {}
        FillLocalizedClassList(LOCALIZED_CLASS_NAMES_MALE, false)
        FillLocalizedClassList(LOCALIZED_CLASS_NAMES_FEMALE, true)
        for classIndex = 1, GetNumClasses() do
            local name, classFile = GetClassInfo(classIndex)
            assert(LOCALIZED_CLASS_NAMES_MALE[classFile] == name, classFile .. " male name")
            assert(LOCALIZED_CLASS_NAMES_FEMALE[classFile] == name, classFile .. " female name")
        end
        "#,
    )
    .expect("ChatConfig class legends require localized names for every visible class");
}
