//! Tests for the 16 texture methods ported from master onto the rilua registrar.
//!
//! Each test creates a Texture (or Frame) and exercises setter/getter round-trips,
//! clear behaviour, or confirms that no-op stubs accept arguments and return nothing.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn set_texture_empty_string_clears_texture_path() {
    let env = env();
    let is_cleared: bool = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", nil, UIParent)
            parent:Show()
            local tex = parent:CreateTexture()
            tex:Show()
            tex:SetTexture("Interface\\Icons\\INV_Misc_QuestionMark")
            tex:SetTexture("")
            return tex:GetTexture() == nil
            "#,
        )
        .unwrap();

    assert!(is_cleared, "SetTexture(\"\") should clear the texture");

    let visible_paths = env.state().borrow().widgets.visible_texture_paths();
    assert!(
        !visible_paths.iter().any(|path| path.is_empty()),
        "empty texture paths should not reach the renderer: {visible_paths:?}"
    );
}

// ---------------------------------------------------------------------------
// 1. SetTextureSliceMargins / GetTextureSliceMargins
// ---------------------------------------------------------------------------

#[test]
fn test_set_texture_slice_margins_roundtrip() {
    let env = env();
    let (l, r, t, b): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetTextureSliceMargins(4, 8, 12, 16)
            return tex:GetTextureSliceMargins()
            "#,
        )
        .unwrap();
    assert_eq!(l, 4.0);
    assert_eq!(r, 8.0);
    assert_eq!(t, 12.0);
    assert_eq!(b, 16.0);
}

#[test]
fn test_get_texture_slice_margins_default() {
    let env = env();
    let (l, r, t, b): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            return tex:GetTextureSliceMargins()
            "#,
        )
        .unwrap();
    assert_eq!((l, r, t, b), (0.0, 0.0, 0.0, 0.0));
}

// ---------------------------------------------------------------------------
// 2. SetTextureSliceMode / GetTextureSliceMode
// ---------------------------------------------------------------------------

#[test]
fn test_set_texture_slice_mode_roundtrip() {
    let env = env();
    let mode: i64 = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetTextureSliceMode(3)
            return tex:GetTextureSliceMode()
            "#,
        )
        .unwrap();
    assert_eq!(mode, 3);
}

#[test]
fn test_get_texture_slice_mode_default() {
    let env = env();
    let mode: i64 = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            return tex:GetTextureSliceMode()
            "#,
        )
        .unwrap();
    assert_eq!(mode, 0);
}

// ---------------------------------------------------------------------------
// 3. ClearTextureSlice — resets margins + mode
// ---------------------------------------------------------------------------

#[test]
fn test_clear_texture_slice_resets_margins_and_mode() {
    let env = env();
    let (l, r, t, b, mode): (f64, f64, f64, f64, i64) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetTextureSliceMargins(4, 8, 12, 16)
            tex:SetTextureSliceMode(2)
            tex:ClearTextureSlice()
            local l, r, t, b = tex:GetTextureSliceMargins()
            local m = tex:GetTextureSliceMode()
            return l, r, t, b, m
            "#,
        )
        .unwrap();
    assert_eq!((l, r, t, b), (0.0, 0.0, 0.0, 0.0));
    assert_eq!(mode, 0);
}

// ---------------------------------------------------------------------------
// 4. SetRotation / GetRotation
// ---------------------------------------------------------------------------

#[test]
fn test_set_rotation_roundtrip() {
    let env = env();
    let quarter_turn = std::f64::consts::FRAC_PI_2;
    let radians: f64 = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetRotation(math.pi / 2)
            return tex:GetRotation()
            "#,
        )
        .unwrap();
    assert!((radians - quarter_turn).abs() < 0.0001, "got {radians}");
}

#[test]
fn test_set_radians_alias_roundtrip() {
    let env = env();
    let quarter_turn = std::f64::consts::FRAC_PI_2;
    let radians: f64 = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetRadians(math.pi / 2)
            return tex:GetRotation()
            "#,
        )
        .unwrap();
    assert!((radians - quarter_turn).abs() < 0.0001, "got {radians}");
}

#[test]
fn test_get_rotation_default() {
    let env = env();
    let radians: f64 = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            return tex:GetRotation()
            "#,
        )
        .unwrap();
    assert_eq!(radians, 0.0);
}

// ---------------------------------------------------------------------------
// 5. SetMask — no-op stub: accepts any arg, returns nothing
// ---------------------------------------------------------------------------

#[test]
fn test_set_mask_no_op() {
    let env = env();
    // Should not error and should return nil (0 return values → nil in eval context)
    env.exec(
        r#"
        local tex = CreateFrame("Frame"):CreateTexture()
        tex:SetMask("Interface\\Masks\\CircleMaskScalable")
        "#,
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// 6. SetGradient — persists gradient record
// ---------------------------------------------------------------------------

#[test]
fn test_set_gradient_horizontal_no_error() {
    let env = env();
    // Master stores the gradient; we verify the call completes without error.
    env.exec(
        r#"
        local tex = CreateFrame("Frame"):CreateTexture()
        local minC = {r=0, g=0, b=0, a=1}
        local maxC = {r=1, g=1, b=1, a=1}
        tex:SetGradient("HORIZONTAL", minC, maxC)
        "#,
    )
    .unwrap();
}

#[test]
fn test_set_gradient_vertical_no_error() {
    let env = env();
    env.exec(
        r#"
        local tex = CreateFrame("Frame"):CreateTexture()
        local minC = {r=0.2, g=0.4, b=0.6, a=0.8}
        local maxC = {r=0.8, g=0.6, b=0.4, a=1.0}
        tex:SetGradient("VERTICAL", minC, maxC)
        "#,
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// 7. SetCenterColor — no-op
// ---------------------------------------------------------------------------

#[test]
fn test_set_center_color_no_op() {
    let env = env();
    env.exec(
        r#"
        local tex = CreateFrame("Frame"):CreateTexture()
        tex:SetCenterColor(1, 0, 0, 1)
        "#,
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// 8. SetVisuals — no-op
// ---------------------------------------------------------------------------

#[test]
fn test_set_visuals_no_op() {
    let env = env();
    env.exec(
        r#"
        local tex = CreateFrame("Frame"):CreateTexture()
        tex:SetVisuals({})
        "#,
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// 9. SetSpriteSheetCell — no-op stub
// ---------------------------------------------------------------------------

#[test]
fn test_set_sprite_sheet_cell_no_op() {
    let env = env();
    env.exec(
        r#"
        local tex = CreateFrame("Frame"):CreateTexture()
        tex:SetSpriteSheetCell(0, 0, 64, 64)
        "#,
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// 10 + 11 + 12. SetVertexOffset / GetVertexOffset / ClearVertexOffsets
// ---------------------------------------------------------------------------

#[test]
fn test_set_get_vertex_offset_roundtrip() {
    let env = env();
    let (x, y): (f64, f64) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetVertexOffset(1, 3.5, -2.0)
            return tex:GetVertexOffset(1)
            "#,
        )
        .unwrap();
    assert!((x - 3.5).abs() < 0.001, "x={x}");
    assert!((y + 2.0).abs() < 0.001, "y={y}");
}

#[test]
fn test_get_vertex_offset_default_zero() {
    let env = env();
    let (x, y): (f64, f64) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            return tex:GetVertexOffset(2)
            "#,
        )
        .unwrap();
    assert_eq!((x, y), (0.0, 0.0));
}

#[test]
fn test_clear_vertex_offsets_resets_to_zero() {
    let env = env();
    let (x, y): (f64, f64) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetVertexOffset(3, 10.0, 20.0)
            tex:ClearVertexOffsets()
            return tex:GetVertexOffset(3)
            "#,
        )
        .unwrap();
    assert_eq!((x, y), (0.0, 0.0));
}

#[test]
fn test_vertex_offset_all_four_corners() {
    let env = env();
    let (x1, y1, x2, y2, x3, y3, x4, y4): (f64, f64, f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetVertexOffset(1,  1,  2)
            tex:SetVertexOffset(2,  3,  4)
            tex:SetVertexOffset(3,  5,  6)
            tex:SetVertexOffset(4,  7,  8)
            local x1,y1 = tex:GetVertexOffset(1)
            local x2,y2 = tex:GetVertexOffset(2)
            local x3,y3 = tex:GetVertexOffset(3)
            local x4,y4 = tex:GetVertexOffset(4)
            return x1,y1,x2,y2,x3,y3,x4,y4
            "#,
        )
        .unwrap();
    assert_eq!((x1, y1), (1.0, 2.0));
    assert_eq!((x2, y2), (3.0, 4.0));
    assert_eq!((x3, y3), (5.0, 6.0));
    assert_eq!((x4, y4), (7.0, 8.0));
}

#[test]
fn test_vertex_offset_constants_are_numeric_and_apply_in_native_order() {
    let env = env();
    let (
        constants_match,
        upper_left_type,
        lower_left_type,
        upper_right_type,
        lower_right_type,
        ulx,
        uly,
        llx,
        lly,
        urx,
        ury,
        lrx,
        lry,
    ): (bool, String, String, String, String, f64, f64, f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetVertexOffset(UPPER_LEFT_VERTEX,  1,  2)
            tex:SetVertexOffset(LOWER_LEFT_VERTEX,  3,  4)
            tex:SetVertexOffset(UPPER_RIGHT_VERTEX, 5,  6)
            tex:SetVertexOffset(LOWER_RIGHT_VERTEX, 7,  8)
            local ulx, uly = tex:GetVertexOffset(UPPER_LEFT_VERTEX)
            local llx, lly = tex:GetVertexOffset(LOWER_LEFT_VERTEX)
            local urx, ury = tex:GetVertexOffset(UPPER_RIGHT_VERTEX)
            local lrx, lry = tex:GetVertexOffset(LOWER_RIGHT_VERTEX)
            return UPPER_LEFT_VERTEX == 1 and LOWER_LEFT_VERTEX == 2
                   and UPPER_RIGHT_VERTEX == 3 and LOWER_RIGHT_VERTEX == 4,
                   type(UPPER_LEFT_VERTEX), type(LOWER_LEFT_VERTEX),
                   type(UPPER_RIGHT_VERTEX), type(LOWER_RIGHT_VERTEX),
                   ulx, uly, llx, lly, urx, ury, lrx, lry
            "#,
        )
        .unwrap();

    assert!(constants_match);
    assert_eq!(
        (
            upper_left_type,
            lower_left_type,
            upper_right_type,
            lower_right_type
        ),
        ("number".into(), "number".into(), "number".into(), "number".into())
    );
    assert_eq!((ulx, uly), (1.0, 2.0));
    assert_eq!((llx, lly), (3.0, 4.0));
    assert_eq!((urx, ury), (5.0, 6.0));
    assert_eq!((lrx, lry), (7.0, 8.0));
}

// ---------------------------------------------------------------------------
// 13. ResetTexCoord
// ---------------------------------------------------------------------------

#[test]
fn test_reset_tex_coord_restores_defaults() {
    let env = env();
    // After SetTexCoord + ResetTexCoord, GetTexCoord should return defaults (0,1,0,1)
    let (l, r, t, b): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetTexCoord(0.1, 0.9, 0.1, 0.9)
            tex:ResetTexCoord()
            return tex:GetTexCoord()
            "#,
        )
        .unwrap();
    assert_eq!((l, r, t, b), (0.0, 1.0, 0.0, 1.0));
}

// ---------------------------------------------------------------------------
// 14 + 15. SetBlockingLoadsRequested / IsBlockingLoadRequested
// ---------------------------------------------------------------------------

#[test]
fn test_set_blocking_loads_requested_roundtrip() {
    let env = env();
    let v: bool = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetBlockingLoadsRequested(true)
            return tex:IsBlockingLoadRequested()
            "#,
        )
        .unwrap();
    assert!(v);
}

#[test]
fn test_is_blocking_load_requested_default_false() {
    let env = env();
    let v: bool = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            return tex:IsBlockingLoadRequested()
            "#,
        )
        .unwrap();
    assert!(!v);
}

#[test]
fn test_set_blocking_loads_requested_toggle() {
    let env = env();
    let (a, b): (bool, bool) = env
        .eval(
            r#"
            local tex = CreateFrame("Frame"):CreateTexture()
            tex:SetBlockingLoadsRequested(true)
            local a = tex:IsBlockingLoadRequested()
            tex:SetBlockingLoadsRequested(false)
            local b = tex:IsBlockingLoadRequested()
            return a, b
            "#,
        )
        .unwrap();
    assert!(a);
    assert!(!b);
}
