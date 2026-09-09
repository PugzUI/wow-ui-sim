//! Temporary UI color defaults.
//!
//! These values keep Blizzard startup code and addon probes working until the
//! simulator has a real color registry surface.

const COLOR_DEFAULTS_LUA: &str = r#"
local function __wow_make_color(r, g, b, a)
  local color = {
    r = r or 1,
    g = g or 1,
    b = b or 1,
    a = a or 1,
  }

  function color:GetRGB()
    return self.r, self.g, self.b
  end

  function color:GetRGBA()
    return self.r, self.g, self.b, self.a
  end

  local function channel_byte(value)
    return math.floor((value or 0) * 255 + 0.5)
  end

  function color:GetRGBAsBytes()
    return channel_byte(self.r), channel_byte(self.g), channel_byte(self.b)
  end

  function color:GetRGBAAsBytes()
    return channel_byte(self.r), channel_byte(self.g), channel_byte(self.b), channel_byte(self.a or 1)
  end

  function color:GenerateHexColor()
    return string.format("FF%02X%02X%02X", math.floor(self.r * 255), math.floor(self.g * 255), math.floor(self.b * 255))
  end

  function color:GenerateHexColorNoAlpha()
    return string.format("%02X%02X%02X", self:GetRGBAsBytes())
  end

  function color:GenerateHexColorMarkup()
    return "|c" .. self:GenerateHexColor()
  end

  function color:WrapTextInColorCode(text)
    return self:GenerateHexColorMarkup() .. tostring(text or "") .. "|r"
  end

  return color
end

if CreateColor == nil then
  function CreateColor(r, g, b, a)
    return __wow_make_color(r, g, b, a)
  end
end
if CreateColorFromHexString == nil then
  function CreateColorFromHexString(hex)
    local value = tostring(hex or ""):gsub("^|c", ""):gsub("^#", "")
    if #value == 6 then value = "ff" .. value end
    local a = tonumber(value:sub(1, 2), 16) or 255
    local r = tonumber(value:sub(3, 4), 16) or 255
    local g = tonumber(value:sub(5, 6), 16) or 255
    local b = tonumber(value:sub(7, 8), 16) or 255
    return __wow_make_color(r / 255, g / 255, b / 255, a / 255)
  end
end
if WrapTextInColorCode == nil then
  function WrapTextInColorCode(text, colorCode)
    local code = tostring(colorCode or "ffffffff"):gsub("^|c", "")
    return "|c" .. code .. tostring(text or "") .. "|r"
  end
end


local function __wow_color_merge_namespace(existing, defaults)
  local namespace = type(existing) == "table" and existing or {}
  for key, value in pairs(defaults or {}) do
    if rawget(namespace, key) == nil then
      rawset(namespace, key, value)
    end
  end
  return namespace
end

C_UIColor = __wow_color_merge_namespace(C_UIColor, {
  GetColors = function()
    return {
      { baseTag = "HIGHLIGHT_FONT_COLOR", color = { r = 1, g = 1, b = 1, a = 1 } },
      { baseTag = "PLAYER_FACTION_COLOR_HORDE", color = { r = 1, g = 0.1, b = 0.1, a = 1 } },
      { baseTag = "PLAYER_FACTION_COLOR_ALLIANCE", color = { r = 0.2, g = 0.4, b = 1, a = 1 } },
      { baseTag = "NORMAL_FONT_COLOR", color = { r = 1, g = 0.82, b = 0, a = 1 } },
      -- Blizzard_Professions panels look up the tradeskill experience bar
      -- fill color by baseTag in the C_UIColor.GetColors() return value.
      { baseTag = "TRADESKILL_EXPERIENCE_COLOR", color = { r = 0.25, g = 0.25, b = 0.75, a = 1 } },
    }
  end,
})

QuestDifficultyColors = QuestDifficultyColors or {}
QuestDifficultyColors.trivial = QuestDifficultyColors.trivial or { r = 0.50, g = 0.50, b = 0.50 }
QuestDifficultyColors.standard = QuestDifficultyColors.standard or { r = 0.25, g = 0.75, b = 0.25 }
QuestDifficultyColors.difficult = QuestDifficultyColors.difficult or { r = 1.00, g = 1.00, b = 0.00 }
QuestDifficultyColors.verydifficult = QuestDifficultyColors.verydifficult or { r = 1.00, g = 0.50, b = 0.25 }
QuestDifficultyColors.impossible = QuestDifficultyColors.impossible or { r = 1.00, g = 0.10, b = 0.10 }

QuestDifficultyHighlightColors = QuestDifficultyHighlightColors or {}
QuestDifficultyHighlightColors.trivial = QuestDifficultyHighlightColors.trivial or { r = 0.70, g = 0.70, b = 0.70 }
QuestDifficultyHighlightColors.standard = QuestDifficultyHighlightColors.standard or { r = 0.50, g = 1.00, b = 0.50 }
QuestDifficultyHighlightColors.difficult = QuestDifficultyHighlightColors.difficult or { r = 1.00, g = 1.00, b = 0.50 }
QuestDifficultyHighlightColors.verydifficult = QuestDifficultyHighlightColors.verydifficult or { r = 1.00, g = 0.75, b = 0.50 }
QuestDifficultyHighlightColors.impossible = QuestDifficultyHighlightColors.impossible or { r = 1.00, g = 0.40, b = 0.40 }

C_ColorUtil = __wow_color_merge_namespace(C_ColorUtil, {
  ConvertRGBToHSV = function(r, g, b)
    return 0, 0, math.max(r or 0, g or 0, b or 0)
  end,
  ConvertHSVToHSL = function(h, s, v)
    return h or 0, s or 0, v or 0
  end,
  GenerateTextColorCode = function(color)
    local r = math.floor((color.r or 1) * 255)
    local g = math.floor((color.g or 1) * 255)
    local b = math.floor((color.b or 1) * 255)
    return string.format("ff%02x%02x%02x", r, g, b)
  end,
  WrapTextInColor = function(text, color)
    return "|c" .. C_ColorUtil.GenerateTextColorCode(color) .. tostring(text or "") .. "|r"
  end,
  WrapTextInColorCode = function(text, colorCode)
    local code = tostring(colorCode or "ffffffff"):gsub("^|c", "")
    return "|c" .. code .. tostring(text or "") .. "|r"
  end,
})

C_ColorOverrides = __wow_color_merge_namespace(C_ColorOverrides, {
  GetColorForQuality = function(_quality)
    return CreateColor(1, 1, 1, 1)
  end,
})

C_PvP = __wow_color_merge_namespace(C_PvP, {
  IsInBrawl = function()
    return false
  end,
  IsSoloShuffle = function()
    return false
  end,
  GetArenaCrowdControlInfo = function(_unit)
    return nil, 0, 0
  end,
})
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(COLOR_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_color_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(C_UIColor.GetColors) ~= "function" then return "ui_colors" end
                local foundTradeskillColor = false
                for _, entry in ipairs(C_UIColor.GetColors()) do
                  if entry.baseTag == "TRADESKILL_EXPERIENCE_COLOR" then
                    foundTradeskillColor = entry.color.b == 0.75
                  end
                end
                if not foundTradeskillColor then return "tradeskill" end
                if C_ColorUtil.GenerateTextColorCode({ r = 1, g = 0.5, b = 0 }) ~= "ffff7f00" then return "text_code" end
                if C_ColorUtil.WrapTextInColorCode("Ready", "ff112233") ~= "|cff112233Ready|r" then return "wrap" end
                local color = CreateColor(0.25, 0.5, 0.75, 0.8)
                local r, g, b, a = color:GetRGBA()
                if r ~= 0.25 or g ~= 0.5 or b ~= 0.75 or a ~= 0.8 then return "rgba" end
                local rb, gb, bb, ab = color:GetRGBAAsBytes()
                if rb ~= 64 or gb ~= 128 or bb ~= 191 or ab ~= 204 then return "bytes" end
                if color:GenerateHexColor() ~= "FF3F7FBF" then return "hex" end
                if color:GenerateHexColorNoAlpha() ~= "4080BF" then return "hex_no_alpha" end
                if color:WrapTextInColorCode("Ready") ~= "|cFF3F7FBFReady|r" then return "color_wrap" end
                local hexColor = CreateColorFromHexString("ff1f1e21")
                local hr, hg, hb, ha = hexColor:GetRGBA()
                if math.abs(hr - (0x1f / 255)) > 0.0001 or math.abs(hg - (0x1e / 255)) > 0.0001
                    or math.abs(hb - (0x21 / 255)) > 0.0001 or ha ~= 1 then return "hex_color" end
                if QuestDifficultyColors.impossible.g ~= 0.10 then return "quest" end
                if QuestDifficultyHighlightColors.standard.g ~= 1.00 then return "highlight" end
                return "ok"
                "#,
            )
            .expect("color defaults probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn installs_color_override_and_pvp_defaults_without_replacing_existing_methods() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local color = C_ColorOverrides.GetColorForQuality(1)
                if color.r ~= 1 or color.g ~= 1 or color.b ~= 1 or color.a ~= 1 then return "color" end
                if C_PvP.IsInBrawl() ~= false then return "brawl" end
                if C_PvP.IsSoloShuffle() ~= false then return "shuffle" end
                local spellID, startTime, duration = C_PvP.GetArenaCrowdControlInfo("player")
                if spellID ~= nil or startTime ~= 0 or duration ~= 0 then return "cc" end
                C_PvP.SetLocklistMap(566)
                if C_PvP.GetLocklistMap(1) ~= 566 then return "state-backed" end
                return "ok"
                "#,
            )
            .expect("color/PvP defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_color_override_and_pvp_functions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_ColorOverrides.GetColorForQuality = function()
                return { r = 0.1, g = 0.2, b = 0.3, a = 0.4 }
            end
            C_PvP.IsInBrawl = function()
                return true
            end
            "#,
        )
        .expect("fixture should install existing functions");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                local color = C_ColorOverrides.GetColorForQuality(1)
                if color.r ~= 0.1 or color.g ~= 0.2 or color.b ~= 0.3 or color.a ~= 0.4 then return "color" end
                if C_PvP.IsInBrawl() ~= true then return "brawl" end
                return "ok"
                "#,
            )
            .expect("existing color/PvP functions should remain callable");

        assert_eq!(result, "ok");
    }
}
