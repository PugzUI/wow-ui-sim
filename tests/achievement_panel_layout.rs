//! Layout lock for the Achievements panel.

use crate::common;

const ACHIEVEMENT_LAYOUT_ASSERTIONS_LUA: &str = r#"
    local EPS = 0.75

    local function approx(actual, expected, eps)
        if type(actual) ~= "number" or type(expected) ~= "number" then
            return false
        end
        return math.abs(actual - expected) <= (eps or EPS)
    end

    local function rect(frame, name)
        if type(frame) ~= "table" then
            return nil, name .. "_missing"
        end
        local l, b, w, h = frame:GetRect()
        if not (l and b and w and h) then
            return nil, name .. "_missing_rect"
        end
        return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }, nil
    end

    local function has_point(frame, point, rel, rel_point, x, y, eps)
        for i = 1, frame:GetNumPoints() do
            local p, r, rp, ox, oy = frame:GetPoint(i)
            local rel_matches = (r == rel) or (r == nil and rel ~= nil and frame.GetParent and frame:GetParent() == rel)
            if p == point and rel_matches and rp == rel_point and approx(ox or 0, x, eps) and approx(oy or 0, y, eps) then
                return true
            end
        end
        return false
    end

    local function expect_rect(frame, name, left, bottom, width, height)
        local r, e = rect(frame, name)
        if not r then
            return nil, e
        end
        if not approx(r.l, left) then
            return nil, name .. "_left=" .. tostring(r.l)
        end
        if not approx(r.b, bottom) then
            return nil, name .. "_bottom=" .. tostring(r.b)
        end
        if not approx(r.w, width, 0.1) then
            return nil, name .. "_width=" .. tostring(r.w)
        end
        if not approx(r.h, height, 0.1) then
            return nil, name .. "_height=" .. tostring(r.h)
        end
        return r, nil
    end

    if not ToggleAchievementFrame then
        return "missing_toggle_achievement_frame"
    end

    ToggleAchievementFrame()
    if not AchievementFrame or not AchievementFrame:IsShown() then
        return "achievement_not_shown"
    end

    local frame_rect, frame_err = expect_rect(AchievementFrame, "achievement_frame", 96, 152, 768, 500)
    if not frame_rect then return frame_err end
    if not has_point(AchievementFrame, "TOPLEFT", UIParent, "TOPLEFT", 96, -116, 0.1) then
        return "achievement_frame_anchor_mismatch"
    end

    -- Cata/Blizzard_AchievementUI.xml uses Classic UIPanelCloseButton (32px)
    -- at (3, 4); Mainline uses a 24px button at (0, 0).
    local close_rect, close_err = expect_rect(AchievementFrameCloseButton, "achievement_close_button", isMists and 835 or 840, isMists and 624 or 628, isMists and 32 or 24, isMists and 32 or 24)
    if not close_rect then return close_err end
    if not has_point(AchievementFrameCloseButton, "TOPRIGHT", AchievementFrame, "TOPRIGHT", isMists and 3 or 0, isMists and 4 or 0, 0.1) then
        return "achievement_close_button_anchor_mismatch"
    end

    if not AchievementFrameCategories or not AchievementFrameCategories:IsShown() then
        return "achievement_categories_missing_or_hidden"
    end
    local categories_rect, categories_err = expect_rect(
        AchievementFrameCategories,
        "achievement_categories",
        117,
        172,
        isMists and 197 or 175,
        461
    )
    if not categories_rect then return categories_err end
    if not has_point(AchievementFrameCategories, "TOPLEFT", AchievementFrame, "TOPLEFT", 21, -19, 0.1) then
        return "achievement_categories_top_anchor_mismatch"
    end
    if not has_point(AchievementFrameCategories, "BOTTOMLEFT", AchievementFrame, "BOTTOMLEFT", 21, 20, 0.1) then
        return "achievement_categories_bottom_anchor_mismatch"
    end

    local categories_bg_rect, categories_bg_err = expect_rect(
        AchievementFrameCategoriesBG,
        "achievement_categories_bg",
        121,
        175,
        195,
        454
    )
    if not categories_bg_rect then return categories_bg_err end
    if not has_point(AchievementFrameCategoriesBG, "TOPLEFT", AchievementFrame, "TOPLEFT", 25, -23, 0.1) then
        return "achievement_categories_bg_top_anchor_mismatch"
    end
    if not has_point(AchievementFrameCategoriesBG, "BOTTOMLEFT", AchievementFrame, "BOTTOMLEFT", 25, 23, 0.1) then
        return "achievement_categories_bg_bottom_anchor_mismatch"
    end

    local background = isMists and AchievementFrameBackground or AchievementFrame.Background
    local bg_rect, bg_err = expect_rect(background, "achievement_background", 112, 168, 736, 468)
    if not bg_rect then return bg_err end
    if not has_point(background, "TOPLEFT", AchievementFrame, "TOPLEFT", 16, -16, 0.1) then
        return "achievement_background_top_anchor_mismatch"
    end
    if not has_point(background, "BOTTOMRIGHT", AchievementFrame, "BOTTOMRIGHT", -16, 16, 0.1) then
        return "achievement_background_bottom_anchor_mismatch"
    end

    local header = isMists and AchievementFrameHeader or AchievementFrame.Header
    local header_rect, header_err = expect_rect(header, "achievement_header", 122, isMists and 613 or 614, 726, 106)
    if not header_rect then return header_err end
    if not has_point(header, "BOTTOMLEFT", AchievementFrame, "TOPLEFT", 26, isMists and -39 or -38, 0.1) then
        return "achievement_header_anchor_mismatch"
    end

    local scrollbox = isMists and AchievementFrameCategoriesContainer or AchievementFrameCategories.ScrollBox
    local scrollbox_rect, scrollbox_err = expect_rect(
        scrollbox,
        "achievement_categories_scrollbox",
        117,
        177,
        isMists and 197 or 175,
        451
    )
    if not scrollbox_rect then return scrollbox_err end
    if not scrollbox:IsShown() then
        return "achievement_categories_scrollbox_hidden"
    end

    local summary_rect, summary_err = expect_rect(AchievementFrameSummary, "achievement_summary", 314, 172, 530, 461)
    if not summary_rect then return summary_err end
    if not AchievementFrameSummary:IsShown() then
        return "achievement_summary_hidden"
    end
    if AchievementFrameAchievements and AchievementFrameAchievements:IsShown() then
        return "achievement_list_should_start_hidden"
    end
    if AchievementFrameStats and AchievementFrameStats:IsShown() then
        return "achievement_stats_should_start_hidden"
    end
    if AchievementFrameComparison and AchievementFrameComparison:IsShown() then
        return "achievement_comparison_should_start_hidden"
    end

    local status_rect, status_err = expect_rect(
        AchievementFrameSummaryCategoriesStatusBar,
        "achievement_summary_categories_status_bar",
        335,
        isMists and 348 or 372,
        488,
        21
    )
    if not status_rect then return status_err end
    if not AchievementFrameSummaryCategoriesStatusBar:IsShown() then
        return "achievement_summary_categories_status_bar_hidden"
    end

    if PanelTemplates_GetSelectedTab and PanelTemplates_GetSelectedTab(AchievementFrame) ~= 1 then
        return "achievement_selected_tab=" .. tostring(PanelTemplates_GetSelectedTab(AchievementFrame))
    end

    return "ok"
"#;

#[test]
#[cfg(feature = "gui")]
fn achievement_frame_layout_stays_locked() {
    test_timeout! {
        let env = common::panel_fixtures::setup_env();
        let assertions = format!(
            "local isMists = {}\n{}",
            cfg!(feature = "client-mists"),
            ACHIEVEMENT_LAYOUT_ASSERTIONS_LUA
        );
        let result: String = env.eval(&assertions).unwrap();
        assert_eq!(
            result, "ok",
            "AchievementFrame layout should remain locked after ToggleAchievementFrame(): {result}"
        );
    }
}

#[test]
#[cfg(feature = "gui")]
fn achievement_summary_empty_text_does_not_overlap_summary_entries() {
    test_timeout! {
        let env = common::panel_fixtures::setup_env();
        let result: String = env
            .eval(
                r#"
                if not ToggleAchievementFrame then
                    return "missing_toggle_achievement_frame"
                end

                ToggleAchievementFrame()
                if not AchievementFrameSummary or not AchievementFrameSummary:IsShown() then
                    return "achievement_summary_hidden"
                end

                local emptyText = AchievementFrameSummaryAchievementsEmptyText
                if not emptyText then
                    return "summary_empty_text_missing"
                end

                local first = AchievementFrameSummaryAchievement1
                local emptyShown = emptyText:IsShown()
                local firstShown = first and first:IsShown() or false
                if emptyShown and firstShown then
                    return "summary_empty_text_overlap"
                end

                return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "achievement summary empty text should not overlap summary rows: {result}"
        );
    }
}

#[test]
#[cfg(feature = "gui")]
fn achievement_frame_toggle_hides_visible_panel_tree() {
    test_timeout! {
        let env = common::panel_fixtures::setup_env();
        let result: String = env
            .eval(
                r#"
                if not ToggleAchievementFrame then
                    return "missing_toggle_achievement_frame"
                end

                ToggleAchievementFrame()
                if not AchievementFrame or not AchievementFrame:IsShown() then
                    return "achievement_not_shown"
                end

                local hideCalls = 0
                local originalHideUIPanel = HideUIPanel
                HideUIPanel = function(frame, ...)
                    if frame == AchievementFrame then
                        hideCalls = hideCalls + 1
                    end
                    return originalHideUIPanel(frame, ...)
                end

                ToggleAchievementFrame()
                HideUIPanel = originalHideUIPanel
                if AchievementFrame:IsShown() then
                    return "achievement_still_shown"
                end
                if hideCalls ~= 1 then
                    return "hide_uipanel_calls=" .. tostring(hideCalls)
                end

                local leaked = {}
                local function visit(frame, path)
                    if (type(frame) ~= "table" and type(frame) ~= "userdata") or type(frame.GetChildren) ~= "function" then
                        return
                    end
                    for index, child in ipairs({ frame:GetChildren() }) do
                        local childPath = path .. "." .. tostring(child:GetName() or index)
                        if type(child.IsVisible) == "function" and child:IsVisible() then
                            table.insert(leaked, childPath)
                        end
                        visit(child, childPath)
                    end
                end

                visit(AchievementFrame, "AchievementFrame")
                if #leaked > 0 then
                    return "visible_child_after_hide=" .. table.concat(leaked, ",")
                end

                return "ok"
                "#,
            )
            .unwrap();
        assert_eq!(
            result, "ok",
            "AchievementFrame toggle should hide the full visible panel tree: {result}"
        );
    }
}
