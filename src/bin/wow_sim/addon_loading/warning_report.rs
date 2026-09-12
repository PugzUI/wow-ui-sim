const VERBOSE_WARNING_ADDONS: &[&str] = &[
    "BetterWardrobe",
    "Plumber",
    "BetterBlizzFrames",
    "Baganator",
    "Angleur",
    "ExtraQuestButton",
    "WaypointUI",
    "TomTom",
    "WorldQuestTracker",
    "SavedInstances",
    "Rarity",
    "SimpleItemLevel",
    "TalentLoadoutManager",
    "Simulationcraft",
    "TomCats",
    "RaiderIO",
    "!BugGrabber",
    "CraftSim",
    "AdvancedInterfaceOptions",
    "BlizzMove_Debug",
    "ClickableRaidBuffs",
    "Dejunk",
    "Cell",
    "AngryKeystones",
    "AutoPotion",
    "BigWigs_Plugins",
    "BugSack",
    "Clicked",
    "DeathNote",
    "DeModal",
    "ElvUI_OptionsUI",
    "DragonRaceTimes",
    "DynamicCam",
    "DialogueUI",
    "Chattynator",
    "AstralKeys",
    "Leatrix_Plus",
    "CooldownToGo_Options",
    "HousingItemTracker",
    "idTip",
    "Macroriffic",
    "NameplateSCT",
    "Krowi_ExtendedVendorUI",
    "OmniCD",
    "Auctionator",
    "EditModeExpanded",
    "GlobalIgnoreList",
    "AllTheThings",
    "BigWigs_KhazAlgar",
    "LegionRemixHelper",
    "Collectionator",
    "Syndicator",
    "BigWigs",
    "!KalielsTracker",
    "KRaidSkipTracker",
    "MacroToolkit",
    "MinimapButtonButton",
    "OribosExchange",
    "ElvUI",
];

pub(super) fn print_addon_warnings(name: &str, warnings: &[String]) {
    if std::env::var("WOW_SIM_DEBUG_NIL_GLOBALS").is_err() {
        return;
    }
    if warnings.is_empty() || !VERBOSE_WARNING_ADDONS.contains(&name) {
        return;
    }
    for (i, w) in warnings.iter().take(300).enumerate() {
        println!("  [{}] {}", i + 1, w);
    }
    if warnings.len() > 300 {
        println!("  ... and {} more", warnings.len() - 300);
    }
}
