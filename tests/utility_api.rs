//! Tests for utility API functions (utility_api.rs).

use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "utility_api/error_handler.rs"]
mod error_handler;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// strsplit
// ============================================================================

#[test]
fn test_strsplit_basic() {
    let env = env();
    let result: (String, String, String) = env.eval("return strsplit(',', 'a,b,c')").unwrap();
    assert_eq!(result, ("a".into(), "b".into(), "c".into()));
}

#[test]
fn test_strsplit_with_limit() {
    let env = env();
    let result: (String, String) = env.eval("return strsplit(',', 'a,b,c', 2)").unwrap();
    assert_eq!(result, ("a".into(), "b,c".into()));
}

#[test]
fn test_strsplit_no_delimiter_found() {
    let env = env();
    let result: String = env.eval("return strsplit(',', 'abc')").unwrap();
    assert_eq!(result, "abc");
}

#[test]
fn test_strsplit_empty_string() {
    let env = env();
    let result: String = env.eval("return strsplit(',', '')").unwrap();
    assert_eq!(result, "");
}

// ============================================================================
// getglobal / setglobal
// ============================================================================

#[test]
fn test_getglobal_setglobal() {
    let env = env();
    env.eval::<()>("setglobal('MY_TEST_VAR', 42)").unwrap();
    let val: i32 = env.eval("return getglobal('MY_TEST_VAR')").unwrap();
    assert_eq!(val, 42);
}

#[test]
fn test_getglobal_nil_for_missing() {
    let env = env();
    let is_nil: bool = env
        .eval("return getglobal('NONEXISTENT_VAR_XYZ') == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// loadstring
// ============================================================================

#[test]
fn test_loadstring_valid_code() {
    let env = env();
    let result: i32 = env
        .eval("local f = loadstring('return 1 + 2'); return f()")
        .unwrap();
    assert_eq!(result, 3);
}

#[test]
fn test_loadstring_syntax_error() {
    let env = env();
    let is_nil: bool = env
        .eval("local f, err = loadstring('invalid @@@ code'); return f == nil")
        .unwrap();
    assert!(is_nil);
    let has_err: bool = env
        .eval("local f, err = loadstring('invalid @@@ code'); return err ~= nil")
        .unwrap();
    assert!(has_err);
}

// ============================================================================
// wipe / table.wipe
// ============================================================================

#[test]
fn test_wipe_clears_table() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local t = {1, 2, 3, a = "b"}
            wipe(t)
            local n = 0
            for _ in pairs(t) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_wipe_returns_table() {
    let env = env();
    let same: bool = env
        .eval(
            r#"
            local t = {1, 2, 3}
            local r = wipe(t)
            return t == r
            "#,
        )
        .unwrap();
    assert!(same);
}

#[test]
fn test_table_wipe_alias() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local t = {1, 2, 3}
            table.wipe(t)
            local n = 0
            for _ in pairs(t) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

// ============================================================================
// tinsert / tremove
// ============================================================================

#[test]
fn test_tinsert_append() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local t = {1, 2}
            tinsert(t, 3)
            return t[3]
            "#,
        )
        .unwrap();
    assert_eq!(val, 3);
}

#[test]
fn test_tinsert_at_position() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local t = {1, 3}
            tinsert(t, 2, 99)
            return t[2]
            "#,
        )
        .unwrap();
    assert_eq!(val, 99);
}

#[test]
fn test_tremove() {
    let env = env();
    let (removed, len): (i32, i32) = env
        .eval(
            r#"
            local t = {10, 20, 30}
            local v = tremove(t, 1)
            return v, #t
            "#,
        )
        .unwrap();
    assert_eq!(removed, 10);
    assert_eq!(len, 2);
}

// ============================================================================
// tInvert
// ============================================================================

#[test]
fn test_tinvert() {
    let env = env();
    let result: (String, String) = env
        .eval(
            r#"
            local t = {a = "x", b = "y"}
            local inv = tInvert(t)
            return inv.x, inv.y
            "#,
        )
        .unwrap();
    assert_eq!(result, ("a".into(), "b".into()));
}

// ============================================================================
// tCompare
// ============================================================================

#[test]
fn test_tcompare_is_shallow_by_default() {
    let env = env();
    let result: (bool, bool, bool) = env
        .eval(
            r#"
            local nested = {value = 1}
            return tCompare({value = 1}, {value = 1}),
                   tCompare({child = nested}, {child = nested}),
                   tCompare({child = {value = 1}}, {child = {value = 1}})
            "#,
        )
        .unwrap();
    assert_eq!(result, (true, true, true));
}

#[test]
fn test_tcompare_uses_lua_defaults_indexing_and_equality() {
    let env = env();
    let result: (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local right = setmetatable({}, {__index = {value = 1}})
            local left = setmetatable({}, {__index = {value = 1}})
            local shallow_false = tCompare({child = {value = 1}}, {child = {value = 2}}, false)
            local default_false = tCompare({value = 1}, {value = 1}, false)
            local lhs_index = tCompare({value = 1}, right)
            local rhs_index = tCompare(left, {value = 1})
            local delayed_depth_error = not pcall(tCompare, {child = {value = 1}}, {child = {value = 1}}, "not a number")
            return shallow_false, default_false, lhs_index, rhs_index, delayed_depth_error
            "#,
        )
        .unwrap();
    assert_eq!(result, (true, true, true, true, true));
}

#[test]
fn test_tcompare_uses_lua_sub_for_custom_depth_and_errors_on_non_table_index() {
    let env = env();
    let result: (bool, bool, bool) = env
        .eval(
            r#"
            local sub_called = false
            local depth = setmetatable({}, {
              __lt = function() return true end,
              __sub = function(_, value)
                sub_called = value == 1
                return 2
              end,
            })
            local custom_depth = tCompare(
              {child = {value = 1}},
              {child = {value = 2}},
              depth
            )
            local bad_index = setmetatable({}, {__index = 42})
            local index_error = not pcall(tCompare, {value = 1}, bad_index)
            local chained_bad_index = setmetatable({}, {
              __index = setmetatable({}, {__index = false}),
            })
            local chain_error = not pcall(tCompare, {value = 1}, chained_bad_index)
            return custom_depth, sub_called, index_error and chain_error
            "#,
        )
        .unwrap();
    assert_eq!(result, (false, true, true));
}

#[test]
fn test_tcompare_recurses_to_requested_depth_and_rejects_extra_keys() {
    let env = env();
    let result: (bool, bool, bool) = env
        .eval(
            r#"
            return tCompare({child = {value = 1}}, {child = {value = 1}}, 2),
                   tCompare({child = {value = 1}}, {child = {value = 2}}, 2),
                   tCompare({value = 1}, {value = 1, extra = true})
            "#,
        )
        .unwrap();
    assert_eq!(result, (true, false, false));
}

#[test]
fn test_tcompare_preserves_argument_errors() {
    let env = env();
    let errored: bool = env
        .eval("local ok = pcall(tCompare, nil, {}); return not ok")
        .unwrap();
    assert!(errored);
}

// ============================================================================
// tContains
// ============================================================================

#[test]
fn test_tcontains_found() {
    let env = env();
    let found: bool = env.eval("return tContains({10, 20, 30}, 20)").unwrap();
    assert!(found);
}

#[test]
fn test_tcontains_not_found() {
    let env = env();
    let found: bool = env.eval("return tContains({10, 20, 30}, 99)").unwrap();
    assert!(!found);
}

#[test]
fn test_tcontains_checks_hash_entries_too() {
    let env = env();
    let found: bool = env
        .eval(
            r#"
            local t = {}
            t.onlyHash = 42
            return tContains(t, 42)
            "#,
        )
        .unwrap();
    assert!(found, "tContains should match Blizzard pairs() semantics");
}

// ============================================================================
// tIndexOf
// ============================================================================

#[test]
fn test_tindexof_found() {
    let env = env();
    let idx: i32 = env.eval("return tIndexOf({10, 20, 30}, 20)").unwrap();
    assert_eq!(idx, 2);
}

#[test]
fn test_tindexof_not_found() {
    let env = env();
    let is_nil: bool = env
        .eval("return tIndexOf({10, 20, 30}, 99) == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// tFilter
// ============================================================================

#[test]
fn test_tfilter_removes_matching() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local t = {1, 2, 3, 4, 5}
            tFilter(t, function(v) return v > 3 end)
            local n = 0
            for _ in pairs(t) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 2); // only 4 and 5 remain
}

// ============================================================================
// CopyTable
// ============================================================================

#[test]
fn test_copytable_shallow() {
    let env = env();
    let (a, b, independent): (i32, i32, bool) = env
        .eval(
            r#"
            local orig = {x = 1, y = 2}
            local copy = CopyTable(orig)
            copy.x = 99
            return orig.x, copy.x, orig.x ~= copy.x
            "#,
        )
        .unwrap();
    assert_eq!(a, 1);
    assert_eq!(b, 99);
    assert!(independent);
}

#[test]
fn test_copytable_deep() {
    let env = env();
    let independent: bool = env
        .eval(
            r#"
            local orig = {inner = {val = 10}}
            local copy = CopyTable(orig)
            copy.inner.val = 99
            return orig.inner.val == 10
            "#,
        )
        .unwrap();
    assert!(independent);
}

// ============================================================================
// MergeTable
// ============================================================================

#[test]
fn test_mergetable() {
    let env = env();
    let (a, b): (i32, i32) = env
        .eval(
            r#"
            local dest = {a = 1}
            local src = {b = 2}
            MergeTable(dest, src)
            return dest.a, dest.b
            "#,
        )
        .unwrap();
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}

#[test]
fn test_mergetable_overwrites() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local dest = {a = 1}
            local src = {a = 99}
            MergeTable(dest, src)
            return dest.a
            "#,
        )
        .unwrap();
    assert_eq!(val, 99);
}

// ============================================================================
// nop / sound stubs
// ============================================================================

#[test]
fn test_nop_exists() {
    let env = env();
    let is_func: bool = env.eval("return type(nop) == 'function'").unwrap();
    assert!(is_func);
}

#[test]
fn test_sound_stubs_exist() {
    let env = env();
    for func in &["PlaySound", "StopSound", "PlaySoundFile"] {
        let is_func: bool = env
            .eval(&format!("return type({}) == 'function'", func))
            .unwrap();
        assert!(is_func, "{} should be a function", func);
    }
}

#[test]
fn test_sound_shims_record_latest_requests() {
    let env = env();
    env.eval::<()>(
        r#"
        PlaySound(839)
        PlaySoundFile("sound/interface/test.ogg")
        StopSound(17)
        "#,
    )
    .unwrap();

    let sim = env.state().borrow();
    assert_eq!(
        sim.last_sound_kit_requested,
        Some(839),
        "PlaySound should record the latest requested sound kit"
    );
    assert_eq!(
        sim.last_sound_file_requested.as_deref(),
        Some("sound/interface/test.ogg"),
        "PlaySoundFile should record the latest requested file path"
    );
    assert_eq!(
        sim.last_stopped_sound_handle,
        Some(17),
        "StopSound should record the latest stopped handle"
    );
}

#[test]
fn launch_url_records_request_on_sim_state() {
    // StaticPopupDialogs["ACCOUNT_SAVE_SUCCESS"].OnAccept calls LaunchURL(data)
    // when the save success popup is accepted (Blizzard_AccountSaveUI.lua line 90).
    // The simulator never opens an external browser; tests assert the URL.
    let env = env();
    env.eval::<()>(r#"LaunchURL("https://example.com/account-save")"#)
        .unwrap();
    assert_eq!(
        env.state().borrow().last_launched_url.as_deref(),
        Some("https://example.com/account-save"),
        "LaunchURL should record the requested URL on SimState"
    );
}

#[test]
fn launch_url_overwrites_previous_request() {
    let env = env();
    env.eval::<()>(
        r#"
        LaunchURL("https://first.example/")
        LaunchURL("https://second.example/")
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().last_launched_url.as_deref(),
        Some("https://second.example/"),
        "LaunchURL should record only the latest URL"
    );
}

#[test]
fn test_map_scene_character_highlight_shims_track_guid() {
    let env = env();
    env.eval::<()>(r#"MapSceneCharacterHighlightStart("Player-1-0000BEEF")"#)
        .unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .highlighted_map_scene_character_guid
            .as_deref(),
        Some("Player-1-0000BEEF"),
        "highlight start should store the active character guid"
    );

    env.eval::<()>(r#"MapSceneCharacterHighlightEnd("Player-1-0000BEEF")"#)
        .unwrap();
    assert!(
        env.state()
            .borrow()
            .highlighted_map_scene_character_guid
            .is_none(),
        "highlight end should clear the active character guid"
    );
}

#[test]
fn test_can_hearth_and_resurrect_from_area_tracks_player_state() {
    let env = env();
    let unavailable: bool = env.eval("return CanHearthAndResurrectFromArea()").unwrap();
    assert!(
        !unavailable,
        "area resurrection should stay unavailable without a pending resurrect"
    );

    {
        let mut sim = env.state().borrow_mut();
        sim.pending_resurrect = Some("Tyrande".into());
    }
    let available: bool = env.eval("return CanHearthAndResurrectFromArea()").unwrap();
    assert!(
        available,
        "a pending resurrect plus the default teleport/hearth state should enable the probe"
    );

    {
        let mut sim = env.state().borrow_mut();
        sim.has_hearthstone = false;
    }
    let without_hearthstone: bool = env.eval("return CanHearthAndResurrectFromArea()").unwrap();
    assert!(
        !without_hearthstone,
        "the probe should turn off when the player cannot hearth from the area"
    );

    {
        let mut sim = env.state().borrow_mut();
        sim.has_hearthstone = true;
        sim.can_teleport = false;
    }
    let without_teleport: bool = env.eval("return CanHearthAndResurrectFromArea()").unwrap();
    assert!(
        !without_teleport,
        "the probe should turn off when teleport/hearth travel is disabled"
    );
}

// ============================================================================
// String library aliases
// ============================================================================

#[test]
fn test_strlen_alias() {
    let env = env();
    let len: i32 = env.eval("return strlen('hello')").unwrap();
    assert_eq!(len, 5);
}

#[test]
fn test_strtrim() {
    let env = env();
    let result: String = env.eval("return strtrim('  hello  ')").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_string_trim_alias() {
    let env = env();
    let result: (String, String) = env
        .eval(r#"return string.trim('  hello  '), ('xxhellox'):trim('x')"#)
        .unwrap();
    assert_eq!(result.0, "hello");
    assert_eq!(result.1, "hello");
}

#[test]
fn test_strtrim_custom_chars() {
    let env = env();
    let result: String = env.eval("return strtrim('\\nhello\\r', '\\n\\r')").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_strjoin() {
    let env = env();
    let result: String = env.eval("return strjoin(',', 'a', 'b', 'c')").unwrap();
    assert_eq!(result, "a,b,c");
}

#[test]
fn test_strsplittable() {
    let env = env();
    let (a, b): (String, String) = env
        .eval(
            r#"
            local t = strsplittable(',', 'x,y')
            return t[1], t[2]
            "#,
        )
        .unwrap();
    assert_eq!(a, "x");
    assert_eq!(b, "y");
}

#[test]
fn test_string_split_method() {
    let env = env();
    let (a, b, c): (String, String, String) = env
        .eval(r#"return ("a-b-c"):split("-")"#)
        .unwrap();
    assert_eq!((a.as_str(), b.as_str(), c.as_str()), ("a", "b", "c"));
}

#[test]
fn test_string_split_method_accepts_delimiter_receiver() {
    let env = env();
    let (major, minor, build): (String, String, String) =
        env.eval(r#"return ("."):split("12.0.5")"#).unwrap();
    assert_eq!(
        (major.as_str(), minor.as_str(), build.as_str()),
        ("12", "0", "5")
    );
}

#[test]
fn test_format_alias() {
    let env = env();
    let result: String = env.eval("return format('%d items', 5)").unwrap();
    assert_eq!(result, "5 items");
}

// ============================================================================
// Math library aliases
// ============================================================================

#[test]
fn test_math_aliases() {
    let env = env();
    let val: i32 = env.eval("return abs(-5)").unwrap();
    assert_eq!(val, 5);
    let val: i32 = env.eval("return ceil(1.2)").unwrap();
    assert_eq!(val, 2);
    let val: i32 = env.eval("return floor(1.9)").unwrap();
    assert_eq!(val, 1);
    let val: i32 = env.eval("return max(3, 7)").unwrap();
    assert_eq!(val, 7);
    let val: i32 = env.eval("return min(3, 7)").unwrap();
    assert_eq!(val, 3);
}

// ============================================================================
// Bitwise operations
// ============================================================================

#[test]
fn test_bit_band() {
    let env = env();
    let val: i32 = env.eval("return bit.band(12, 10)").unwrap();
    assert_eq!(val, 8); // 1100 & 1010 = 1000
}

#[test]
fn test_bit_bor() {
    let env = env();
    let val: i32 = env.eval("return bit.bor(12, 10)").unwrap();
    assert_eq!(val, 14); // 1100 | 1010 = 1110
}

#[test]
fn test_bit_bxor() {
    let env = env();
    let val: i32 = env.eval("return bit.bxor(12, 10)").unwrap();
    assert_eq!(val, 6); // 1100 ^ 1010 = 0110
}

#[test]
fn test_bit_bnot() {
    let env = env();
    let val: i64 = env.eval("return bit.bnot(0)").unwrap();
    assert_eq!(val, 4294967295); // 32-bit not of 0
}

#[test]
fn test_bit_shifts() {
    let env = env();
    let val: i32 = env.eval("return bit.lshift(1, 3)").unwrap();
    assert_eq!(val, 8);
    let val: i32 = env.eval("return bit.rshift(16, 2)").unwrap();
    assert_eq!(val, 4);
}

// ============================================================================
// Table aliases
// ============================================================================

#[test]
fn test_sort_alias() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local t = {3, 1, 2}
            sort(t)
            return t[1]
            "#,
        )
        .unwrap();
    assert_eq!(val, 1);
}

#[test]
fn test_getn_alias() {
    let env = env();
    let val: i32 = env.eval("return getn({10, 20, 30})").unwrap();
    assert_eq!(val, 3);
}

// ============================================================================
// Mixin system
// ============================================================================

#[test]
fn test_mixin() {
    let env = env();
    let (a, b): (i32, i32) = env
        .eval(
            r#"
            local obj = {a = 1}
            local mixin = {b = 2}
            Mixin(obj, mixin)
            return obj.a, obj.b
            "#,
        )
        .unwrap();
    assert_eq!((a, b), (1, 2));
}

#[test]
fn test_mixin_multiple() {
    let env = env();
    let (a, b, c): (i32, i32, i32) = env
        .eval(
            r#"
            local obj = {}
            Mixin(obj, {a = 1}, {b = 2}, {c = 3})
            return obj.a, obj.b, obj.c
            "#,
        )
        .unwrap();
    assert_eq!((a, b, c), (1, 2, 3));
}

#[test]
fn test_create_from_mixins() {
    let env = env();
    let (a, b): (i32, i32) = env
        .eval(
            r#"
            local m1 = {a = 1}
            local m2 = {b = 2}
            local obj = CreateFromMixins(m1, m2)
            return obj.a, obj.b
            "#,
        )
        .unwrap();
    assert_eq!((a, b), (1, 2));
}

#[test]
fn test_create_and_init_from_mixin() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
            local MyMixin = {}
            function MyMixin:Init(x)
                self.value = x
            end
            local obj = CreateAndInitFromMixin(MyMixin, 42)
            return obj.value
            "#,
        )
        .unwrap();
    assert_eq!(val, 42);
}
