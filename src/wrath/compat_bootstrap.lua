-- Wrath-era global aliases and stubs.
-- Loaded only when the client-wrath feature is active.
-- All definitions are guarded by nil-checks so later Blizzard Lua can override.

-- ─── Lua 5.0-era string/math aliases ────────────────────────────────────────
-- These lived as top-level globals in Wrath but retail removed them.

if gsub == nil and string ~= nil then gsub = string.gsub end
if strfind == nil and string ~= nil then strfind = string.find end
if strlower == nil and string ~= nil then strlower = string.lower end
if strupper == nil and string ~= nil then strupper = string.upper end
if strlen == nil and string ~= nil then strlen = string.len end
-- strsub and strconcat are registered from Rust (missing_surface.rs)
if strchar == nil and string ~= nil then strchar = string.char end
if strbyte == nil and string ~= nil then strbyte = string.byte end
if strrep == nil and string ~= nil then strrep = string.rep end
if strrev == nil and string ~= nil then strrev = string.reverse end
if mod == nil and math ~= nil then mod = math.fmod end

-- strjoin and strsplit are registered from Rust (utility_system_spell).
-- strconcat is registered from Rust (missing_surface.rs).

-- ─── Wrath-only API stubs ────────────────────────────────────────────────────

if BNGetMaxPlayersInConversation == nil then
  function BNGetMaxPlayersInConversation()
    return 10
  end
end

if CreateWorldMapArrowFrame == nil then
  function CreateWorldMapArrowFrame()
    return nil
  end
end

if FCF_DockUpdate == nil then
  function FCF_DockUpdate()
  end
end

if GetActionBarPage == nil then
  function GetActionBarPage()
    return 1
  end
end

if GetArenaTeam == nil then
  function GetArenaTeam()
    return nil
  end
end

if GetAvailableRoles == nil then
  function GetAvailableRoles()
    return false, false, false
  end
end

if GetCompanionInfo == nil then
  function GetCompanionInfo()
    return nil
  end
end

if GetCurrentMultisampleFormat == nil then
  function GetCurrentMultisampleFormat()
    return 0
  end
end

if GetCurrentResolution == nil then
  function GetCurrentResolution()
    return 1
  end
end

if GetExistingLocales == nil then
  function GetExistingLocales()
    return "enUS"
  end
end

if GetGMTicket == nil then
  function GetGMTicket()
    return nil
  end
end

if GetMasterLootCandidate == nil then
  function GetMasterLootCandidate()
    return nil
  end
end

if GetModifiedClick == nil then
  function GetModifiedClick()
    return nil
  end
end

if GetNumBattlegroundTypes == nil then
  function GetNumBattlegroundTypes()
    return 0
  end
end

if GetNumTrackingTypes == nil then
  function GetNumTrackingTypes()
    return 0
  end
end

if GetNumVoiceSessions == nil then
  function GetNumVoiceSessions()
    return 0
  end
end

if GetNumWorldStateUI == nil then
  function GetNumWorldStateUI()
    return 0
  end
end

if GetPVPYesterdayStats == nil then
  function GetPVPYesterdayStats()
    return 0, 0
  end
end

if GetQuestTimers == nil then
  function GetQuestTimers()
    return nil
  end
end

if GetRefreshRates == nil then
  -- Vararg of available refresh rate ints. Return one (60Hz).
  function GetRefreshRates()
    return 60
  end
end

if GetRuneType == nil then
  function GetRuneType()
    return 1
  end
end

if GetScreenResolutions == nil then
  -- Vararg of "WxH" strings.
  function GetScreenResolutions()
    return "1024x768"
  end
end

if GetSelectedDisplayChannel == nil then
  function GetSelectedDisplayChannel()
    return nil
  end
end

if GetTextHeight == nil then
  function GetTextHeight()
    return 12
  end
end

if GetTrackingTexture == nil then
  function GetTrackingTexture()
    return nil
  end
end

if GetZonePVPInfo == nil then
  function GetZonePVPInfo()
    return nil
  end
end

if HasPetUI == nil then
  function HasPetUI()
    return false, false
  end
end

if IsListedInLFR == nil then
  function IsListedInLFR()
    return false
  end
end

if IsPartyLeader == nil then
  function IsPartyLeader()
    if type(UnitIsGroupLeader) == "function" then
      return UnitIsGroupLeader("party1")
    end
    return false
  end
end

if IsPetAttackAction == nil then
  function IsPetAttackAction()
    return false
  end
end

if IsPossessBarVisible == nil then
  function IsPossessBarVisible()
    return false
  end
end

if IsStereoVideoAvailable == nil then
  function IsStereoVideoAvailable()
    return false
  end
end

if IsVoiceChatAllowedByServer == nil then
  function IsVoiceChatAllowedByServer()
    return false
  end
end

-- QuestDifficultyColors is a global table (not a function).
if QuestDifficultyColors == nil then
  QuestDifficultyColors = {
    Trivial    = { r = 0.5, g = 0.5, b = 0.5 },
    Easy       = { r = 0.5, g = 1.0, b = 0.5 },
    Standard   = { r = 1.0, g = 1.0, b = 1.0 },
    Difficult  = { r = 1.0, g = 0.5, b = 0.0 },
    Impossible = { r = 1.0, g = 0.1, b = 0.1 },
  }
end

if RegisterForSave == nil then
  function RegisterForSave()
  end
end

local selectedGuildRosterIndex = 0

if SetGuildRosterSelection == nil then
  function SetGuildRosterSelection(index)
    selectedGuildRosterIndex = tonumber(index) or 0
  end
end

if GetGuildRosterSelection == nil then
  function GetGuildRosterSelection()
    return selectedGuildRosterIndex
  end
end

if SetMapToCurrentZone == nil then
  function SetMapToCurrentZone()
  end
end

if SetMaxBytes == nil then
  function SetMaxBytes()
  end
end

if SetPlayerTextureWidth == nil then
  function SetPlayerTextureWidth()
  end
end

if SetSelectedSkill == nil then
  function SetSelectedSkill()
  end
end

if debuginfo == nil then
  function debuginfo()
    return ""
  end
end

-- ─── Additional Wrath-only stubs (batch 3.3b) ────────────────────────────────

if strmatch == nil and string ~= nil then strmatch = string.match end

if Blizzard_CombatLog_RefreshGlobalLinks == nil then
  function Blizzard_CombatLog_RefreshGlobalLinks()
  end
end

if GetActionBarToggles == nil then
  function GetActionBarToggles()
    return false, false, false, false, false, false
  end
end

if GetActiveVoiceChannel == nil then
  function GetActiveVoiceChannel()
    return nil
  end
end

if GetAdjustedSkillPoints == nil then
  function GetAdjustedSkillPoints(level)
    return level
  end
end

if GetCVarMin == nil then
  function GetCVarMin()
    return 0
  end
end

if GetMultiCastBarOffset == nil then
  function GetMultiCastBarOffset()
    return 0
  end
end

if GetMultisampleFormats == nil then
  -- Wrath caller iterates select("#", ...) in steps of 3 expecting
  -- (colorBits, depthBits, multiSample) triples. We have none — return nothing.
  function GetMultisampleFormats()
  end
end

if GetNumBankSlots == nil then
  function GetNumBankSlots()
    return 0, 0
  end
end

if GetPVPLifetimeStats == nil then
  function GetPVPLifetimeStats()
    return 0, 0, 0
  end
end

if GetPVPSessionStats == nil then
  function GetPVPSessionStats()
    return 0, 0
  end
end

if GetVoiceCurrentSessionID == nil then
  function GetVoiceCurrentSessionID()
    return nil
  end
end

if GetVoiceSessionInfo == nil then
  function GetVoiceSessionInfo()
    return nil
  end
end

if InitWorldMapPing == nil then
  function InitWorldMapPing()
  end
end

if IsAutoRepeatAction == nil then
  function IsAutoRepeatAction()
    return false
  end
end

if IsEquippedAction == nil then
  function IsEquippedAction()
    return false
  end
end

if IsRaidOfficer == nil then
  function IsRaidOfficer()
    return false
  end
end

if IsUsableAction == nil then
  function IsUsableAction()
    return true, false
  end
end

if IsVoiceChatEnabled == nil then
  function IsVoiceChatEnabled()
    return false
  end
end

if SelectQuestLogEntry == nil then
  function SelectQuestLogEntry()
  end
end

-- Frame proxy stubs moved to compat_frame_proxies.lua (wrath-only — these
-- conflict with real frames defined by Blizzard_SharedXML under mists).

-- ─── Additional Wrath-only stubs (batch 3.3c) ────────────────────────────────

if rawget(_G, "Blizzard_CombatLog_Update_QuickButtons") == nil then
  function Blizzard_CombatLog_Update_QuickButtons()
    return nil
  end
end

if rawget(_G, "GetCVarMax") == nil then
  function GetCVarMax(name)
    return 1
  end
end

if rawget(_G, "GetHonorCurrency") == nil then
  function GetHonorCurrency()
    return 0
  end
end

if rawget(_G, "GetMapInfo") == nil then
  function GetMapInfo()
    return nil, 0, 0
  end
end

if rawget(_G, "GetNumDisplayChannels") == nil then
  function GetNumDisplayChannels()
    return 0
  end
end

if rawget(_G, "GetNumVoiceSessionMembersBySessionID") == nil then
  function GetNumVoiceSessionMembersBySessionID(id)
    return 0
  end
end

if rawget(_G, "GetPVPRankInfo") == nil then
  function GetPVPRankInfo(rank, faction)
    return "", 0
  end
end

if rawget(_G, "GetQuestLogSelection") == nil then
  function GetQuestLogSelection()
    return 0
  end
end

if rawget(_G, "IsAttackAction") == nil then
  function IsAttackAction(slot)
    return false
  end
end

if rawget(_G, "IsConsumableAction") == nil then
  function IsConsumableAction(slot)
    return false
  end
end

if rawget(_G, "UnitCharacterPoints") == nil then
  function UnitCharacterPoints(unit)
    return 0, 0
  end
end

-- Phase 4.1: globals that surfaced after the previous batches landed.

if rawget(_G, "FCF_GetCurrentChatFrame") == nil then
  function FCF_GetCurrentChatFrame()
    return rawget(_G, "DEFAULT_CHAT_FRAME")
  end
end

if rawget(_G, "GetArenaCurrency") == nil then
  function GetArenaCurrency()
    return 0
  end
end

if rawget(_G, "GetChannelDisplayInfo") == nil then
  function GetChannelDisplayInfo(idx)
    return nil
  end
end

if rawget(_G, "GetCurrentMapContinent") == nil then
  function GetCurrentMapContinent()
    return 0, 0
  end
end

if rawget(_G, "GetGamma") == nil then
  function GetGamma()
    return 1.0
  end
end

if rawget(_G, "GetTabardCreationCost") == nil then
  function GetTabardCreationCost()
    return 0
  end
end

if rawget(_G, "GetTerrainMip") == nil then
  function GetTerrainMip()
    return 0
  end
end

if rawget(_G, "GetVideoCaps") == nil then
  -- Returns a table of capability flags in real wrath. Empty table is a safe
  -- fallback — callers do `caps.foo` lookups that all return nil.
  function GetVideoCaps()
    return {}
  end
end

if rawget(_G, "IsStackableAction") == nil then
  function IsStackableAction(slot)
    return false
  end
end

if rawget(_G, "UnitPVPRank") == nil then
  function UnitPVPRank(unit)
    return 0
  end
end

if rawget(_G, "GetActionText") == nil then
  function GetActionText(slot)
    return ""
  end
end

if rawget(_G, "GetCurrentArenaSeason") == nil then
  function GetCurrentArenaSeason()
    return 0
  end
end

if rawget(_G, "GetCurrentMapDungeonLevel") == nil then
  function GetCurrentMapDungeonLevel()
    return 0, 0
  end
end

if rawget(_G, "GetPVPRankProgress") == nil then
  function GetPVPRankProgress()
    return 0
  end
end

if rawget(_G, "DungeonUsesTerrainMap") == nil then
  function DungeonUsesTerrainMap()
    return false
  end
end

if rawget(_G, "GetPreviousArenaSeason") == nil then
  function GetPreviousArenaSeason()
    return 0
  end
end

if rawget(_G, "IsActionInRange") == nil then
  function IsActionInRange(slot)
    return nil
  end
end

-- ─── CVar defaults for wrath options panels ──────────────────────────────────
-- Wrath FrameXML/AudioOptionsPanels.lua reads `voiceChatMode` via
-- BlizzardOptionsPanel_GetCVarSafe and immediately does `voiceChatMode + 1`
-- for tooltip-key composition, with no nil guard. Real WoW ships a CVar
-- default of "0"; the simulator's CVar registry doesn't include wrath-era
-- audio CVars, so the local goes nil and arithmetic errors out. Seed the
-- value here so the panel loads cleanly.
if type(SetCVar) == "function" and (GetCVar == nil or GetCVar("voiceChatMode") == nil) then
  SetCVar("voiceChatMode", "0")
end

if rawget(_G, "IsZoomOutAvailable") == nil then
  function IsZoomOutAvailable() return false end
end
