# Wiki Index

LLM-maintained knowledge base for the wow-ui-sim project.

## design/

| Page | Summary |
|------|---------|
| [[architecture-overview]] | Project goals, Lua+Rust dual system, module layout, design decisions, phase summary |
| [[scaling-coordinates]] | Versioned physical, WoW-screen, renderer, local-frame, and anchor coordinate contracts for native capture |
| [[debug-tools]] | Inspector panel (middle-click), dump-tree CLI (standalone + connected), debug overlay flags |

## reference/

| Page | Summary |
|------|---------|
| [[api-coverage]] | ~97% C_* coverage, missing APIs by category, three-layer stub methodology |
| [[cli-commands]] | wow-sim and wow-cli subcommands: lua-errors, run-tests, screenshot, dump-tree, audit-api |
| [[addon-compatibility]] | 127+ tested addons, Wowless integration, SavedVariables loading, Docker CI |
| [[blizzard-ui-test-lanes]] | Explicit split between Blizzard UI unit tests and addon-bootstrap coverage |
| [[layout-lock-inventory]] | Canonical list of UI elements with explicit layout lock coverage, mapped by subsystem/test |
| [[development-phases]] | Active phases 31–33: widget stubs, audit tool, performance regression tests |

## systems/

| Page | Summary |
|------|---------|
| [[layout-system]] | AnchorPoint enum (9 positions), single vs multi-anchor resolution, coordinate system (top-left screen / bottom-left Lua), SetPoint API, cycle detection |
| [[rendering-pipeline]] | QuadBatch (36-byte QuadVertex), four-tier GPU texture atlas, WGSL shaders, strata/level sorting, alpha propagation, hit testing |
| [[widget-system]] | Frame struct (~140 fields), WidgetType enum (18 types), WidgetRegistry, default children, button text rendering, three-slice pattern |
| [[lua-api]] | WowLuaEnv, FrameHandle userdata, 300+ frame methods, 200+ globals, C_* namespaces, spell-description token resolution, timer system, animation system |
| [[event-system]] | EventQueue, 36+ script handler types, dispatch flow, OnUpdate tick, startup event sequence, XML script setup |
| [[xml-template-system]] | XML parsing (30+ element types), template registry, inheritance chain resolution, XML-to-widget Lua code generation, inline scripts |
| [[addon-loading]] | TOC parsing, discovered Blizzard load order, inline `[Bootstrap]` semantics, per-file Lua/XML loading, SavedVariables, startup sequence |
| [[server-snapshot-action-bars]] | Imports action-bar spell slots captured by the ServerSnapshot addon from real WoW SavedVariables before Blizzard action-bar UI loads |
| [[client-profiles]] | Six client profiles (retail/PTR/wrath/mists/era/anniversary) selected by mutually-exclusive cargo features; profile-aware loader, per-profile manifests, compat bootstraps, vendor pinning, CI matrix |
| [[texture-atlas]] | TextureManager (BLP/PNG/WebP), ~50K-entry compiled atlas database, nine-slice kit detection, UV remapping |
| [[frame-data-flow]] | Parallel Lua/Rust systems, global tables (__frame_fields/__scripts), method lookup order, Mixin() application, event dispatch flow |
| [[taint-system]] | Protected-frame gating, dual Lua environment (genv/secureenv), Elune-backed issecure/securecall, Blizzard `issecure()` call-site matrix, SecureHandler fallback, state/attribute drivers |
| [[casc-asset-cache]] | CASC cache layers (FDID resolution sqlite, BLP byte cache, Blizzard UI source cache, in-memory texture cache), measured timings, failure modes |

## investigations/

| Page | Summary |
|------|---------|
| [[fontstring-ancestor-movement-cache]] | FontString state and geometry were correct but cached absolute glyph quads stayed at the old ancestor position; changed descendant layout rects now invalidate their render snapshots |
| [[retail-ptr-full-startup-lua-errors]] | Full GUI startup logs caught PVPUI, PTR cursor, and Store micro-button errors missed by `lua-errors`; fixes added backed PVP/cursor surfaces and guarded the Store inbound fallback |
| [[lib-test-failure-sweep-2026-06]] | Nine accumulated lib-test failures: ScriptErrors missing from runtime foundations inverted the SharedXMLBase load order; `hooksecurefunc(C_AddOns, "LoadAddOn")` is silently refused so post-load repairs must use `apply_blizzard_post_load_patches`; two more pieces of code lost in the classic rebase; duplicated `== nil`-guarded Lua installers drifting; tests stale against deliberate semantic changes |
| [[store-secure-pool-constructors]] | Retail Store blank/red cards came from `__secureenv.CreateFramePoolCollection` retaining the simulator fallback after `Blizzard_SharedXMLBase` replaced `_G` with Blizzard's proxy-backed constructor; secure replay of SharedXMLBase Lua now populates secureenv directly |
| [[post-load-workaround-audit]] | Retail post-load workaround audit: duplicate loader hooks retired, remaining hooks classified with temporary rationale and retirement paths |
| [[action-button-icon-mask]] | Main action-button icons vanished because `UI-HUD-ActionBar-IconFrame-Mask` stores coverage in alpha while the minimap mask fix sampled RGB intensity; renderer now marks alpha-backed masks with a shader flag |
| [[action-bar-spell-icons]] | 4 bugs: SetDrawLayer no-op, draw order, sublevel ignored, textureSubLevel not parsed |
| [[journeys-renown-card-text-anchor]] | Journeys renown card text invisible because a Layers region's `relativeKey="$parent.IconFrame"` anchor fell back to the parent (child Frames created after Layers); SetPoint now stores unresolved $parent keys for the finalize pass to resolve |
| [[mouse-dead-probe-blockers-idle-ticks]] | Mouse input dead at "50 FPS": CoreBehaviorProbe left full-screen DIALOG mouse blockers; root cause was tick-subscription churn (raw shrinking timer delay changed the `time::every` identity every update, starving ProcessTimers under continuous input — fixed by quantized intervals), plus loader now refuses TOCs that don't name their folder |
| [[casc-root-v2-parsing-missing-textures]] | Dispel debuff borders (and ~89% of all fdids) unextractable because cascette-rs misparsed 12.0.5 TSFM v2 root blocks (split content flags / NoNameHash bit); fixed via pin bumps + `casc_refresh` + clearing `.missing` markers |
| [[addon-load-order]] | Bag buttons partially initialized at load; workaround mirrors real WoW event recovery |
| [[paladin-aura-stance-bar]] | Paladin aura/stance bar vanished because default `SimState.shapeshift_forms` was empty even though the player is seeded as Paladin; state now seeds Devotion, Crusader, and Retribution Aura forms |
| [[mists-panel-stack-overflow-layout-cycle]] | Achievements/Talents panel clicks aborted from layout cache recursion on cyclic parent/anchor dependencies; `headless-click-probe` now exercises the real GUI click path |
| [[mount-journal-click-selection]] | Mount Journal mouse clicks never switched selection: the startup-XML fast path fused multi-arg `RegisterForClicks("LeftButtonUp", "RightButtonUp")` into one garbage registration string; `Button:Click()` bypasses registration so only the real GUI dispatch path (`headless-click-probe mounts`) could see it |
| [[micro-menu-click-offset]] | Micro menu (LFD etc.) clicks dead: MicroMenu's quadrant-dependent anchor was computed before EditMode anchors and the real window size were applied, so the first press snapped the bar one QueueStatusButton slot and the down/up frames mismatched; fixed by replaying `InvokeOnAnyEditModeSystemAnchorChanged` after anchor apply and on `set_screen_size` |
| [[mists-elvui-startup-compat]] | Mists ElvUI startup errors came from separate simulator surface gaps: trim aliases, MessageFrame-only methods visible on plain frames, AuraUtil tuple shape, unanchored Slider label fontstrings, physical-vs-UI screen-size globals, and chat hook globals |
| [[mists-addon-panel-resume-error]] | Mists addon-panel matrix must resume from first unproven addon; rerunning passed rows from `AllTheThings` was the mistake, and `--start-at` / `.passed` markers guard against repeating it |
| [[mists-heirloom-tooltip]] | Mists Collections heirloom buttons need `GameTooltip:SetHeirloomByItemID`, routed through `C_TooltipInfo.GetHeirloomByItemID`, so heirloom tooltip initialization does not throw |
| [[mists-syndicator-baganator-startup]] | Mists full-profile startup errors in Syndicator and Baganator came from Classic/Mists item taxonomy labels plus TokenUI loading before a minimal CharacterFrame parent existed |
| [[playerspells-runtime-load]] | Retail PlayerSpells keybind loading needs stable `C_AddOns.LoadAddOn` call-frame restoration plus temporary PlayerSpells/PvP talent child-frame backfills |
| [[retail-core-behavior-probes]] | Live retail `12.0.5.67823` probes pinned `SetForbidden`, `CreateForbiddenFrame`, invalid `RegisterUnitEvent`, wildcard false attributes, and the improved Raise/Lower capture path |
| [[minimap-map-ring-alignment]] | Active minimap bug is map texture/mask/ring alignment, not the SimCommands minimap button; debug the minimap render mask/clip and ring aperture directly |
| [[casc-fdid-1579624-root-debug]] | FDID 1579624 root/CASC resolution data, CRLF hash proof, and parser debugging checklist |
| [[windows-casc-blizzard-taint]] | Windows CASC-synced Blizzard UI cache needed TOC/folder-name taint semantics plus runtime LoadAddOn stack-taint clearing |
| [[achievement-panel-hide]] | Achievement panel hide now uses Blizzard's managed panel path; animation completion also fires child animation `OnFinished` scripts for alert hide XML |
| [[adventure-guide-layout]] | Suggested Content card overlap came from synchronous geometry queries resolving resized anchor targets without updating dependent sibling frames |
| [[adventure-guide-disabled-tabs]] | Disabled Adventure Guide boss/model tabs looked active because model no-op stubs overwrote generic desaturation setters on the shared frame metatable |
| [[adventure-guide-boss-icons]] | Encounter Journal creature icon fileDataID `0` must be returned as nil so Blizzard boss buttons use their default icon instead of clearing the texture |
| [[adventure-guide-simplehtml-markup]] | Encounter Journal overview text uses SimpleHTML; HTML stripping now also removes WoW color/link escapes and converts `|n` line breaks |
| [[appearances-wardrobe-api]] | Collections Journal Appearances opens, but browsing/filtering/search/favorites need stateful `C_TransmogCollection` source, visual, filter, and search backing instead of fixed stubs |
| [[backpack-background-texture]] | Reported missing tan/brown body on Backpack — investigation showed sim renders exactly what `FlatPanelBackgroundTemplate` authors (solid `PANEL_BACKGROUND_COLOR`); retail texture comes from outside the Gethe public source (addon overlay or unmirrored patch path), no sim-side fix |
| [[talent-performance]] | Lazy `_G` lookup (431ms→263ms), rect-dirty stale cache causing infinite OnUpdate loop, shallow `issecretvalue` for pool releases (2159ms→2.6ms) |
| [[character-select-performance]] | Lazy atlas crop stalls (fixed), first-resize relayout deduplication (partial) |
| [[class-talents-artifact]] | Gold blob ruled out as lossy WebP encoding artifact, not a live render bug |
| [[class-talents-edge-lines]] | Class-talent connector lines disappeared because `IsRectValid()` did not resolve dirty anchored buttons and endpoint-positioned Line widgets under anchorless edge frames were filtered before quad emission |
| [[class-talents-edge-frame-levels]] | Class-talent connector edges were rendering above node icons; edge-frame-level workaround now patches both mixin + live frame and re-levels active edges |
| [[class-talents-trait-loadout-state]] | `PlayerSpells` trait queries now read live loadout state; hero subtree visibility uses correct spec-condition OR semantics; config-scoped `GetNodeInfo` ignores stale view spec |
| [[editmode-layout]] | 3 frame regressions from EditMode overrides after `__index` ordering fix; fenv workaround |
| [[editbox-render-text-cache]] | SimCommands search text can disappear from stale `text_stripped` cache or because opaque EditBox child regions render above the internal text/caret emitter |
| [[explicit-xml-parent-anchors]] | Nested XML frames with `parent="..."` must use that explicit parent for implicit anchors; fixed PaperDoll sidebar tabs anchoring to `PaperDollFrame` instead of `CharacterFrameInsetRight` |
| [[fontstring-default-anchors]] | Unanchored XML `FontString` layer children and `ButtonText` pick their implicit anchor from `justifyH`; explicit anchors suppress the default, while EditBox backing FontStrings stay unanchored |
| [[frame-surrogate-identity-slot]] | Frame method dispatch now seeds `frame[0]` with a backed identity token so Restricted Environment-style surrogates resolve through `[0]`; `[1]`-only surrogates no longer dispatch |
| [[generated-stubs-audit]] | 6 priority findings in generated_stubs.rs affecting startup/panel-load paths |
| [[chatframe-scrollbar-anchor-reapply]] | Inherited child anchor reapply used the child name for `$parent...` substitution, pushing `ChatFrame1` scrollbar descendants off-screen |
| [[crafting-cast-bar]] | `C_TradeSkillUI.CraftRecipe` updated inventory but did not start player casting or fire `UNIT_SPELLCAST_START`, so Blizzard's professions overlay cast bar had no backing spellbar state |
| [[display-size-ui-scale-events]] | Live probe proved retail fires `DISPLAY_SIZE_CHANGED` → `UI_SCALE_CHANGED` as an ordered pair on every display/scale change (resize, slider, maximize, resolution) — never one alone; sim resize path and inverted test fixed |
| [[hero-spec-dialog-anchors]] | LIGHTSMITH/TEMPLAR selection dialog: layer-children batched out of XML order + runtime templates skipped named-anchor re-resolution, dropping panel content to spec-frame edge |
| [[hero-spec-icon-bug]] | Retired — 5 layers of evidence confirm icon renders correctly |
| [[xml-scale-attribute]] | XML `scale` attribute was silently dropped (no `FrameXml` field); hero talents `scale="0.85"` never applied, node buttons overflowed the fixed 284×362 backplate |
| [[hit-testing]] | Two-phase algorithm: HitGrid spatial index + depth-first child drill-down |
| [[journeys-midnight-empty]] | Journeys tab was empty because current expansion was Midnight but default major-faction data only seeded War Within rows |
| [[keybinding-system]] | Two Lua tables, key press pipeline, default bindings, Lua API |
| [[lfd-dungeon-list-empty]] | Dungeons & Raids panel showed empty Specific list: missing `GetLFGLockList` etc., never-fired `LFG_UPDATE_RANDOM_INFO`, header marked `is_random` |
| [[lfd-role-icon-slowness]] | LFD role icons use small button atlas crops from a 2048x2048 shared LFG prompt BLP; persistent crop caching avoids re-decoding the full source texture for repeated crop requests |
| [[mask-texture]] | UV computation, useAtlasSize default, SmallActionButtonMixin override |
| [[method-dispatch-refactor]] | Runtime pollution fixed; target: direct Rust dispatch |
| [[modelscene-player-actor-stub]] | ModelScene keeps 3D rendering stubbed, but `GetPlayerActor():SetModelByUnit("player")` must return a reusable actor for addon probes |
| [[micro-menu-atlas-revert]] | Micro menu normal icons could disappear after hover because button atlas setters skipped child `atlas_tex_coords`, preventing restored normal textures from using isolated atlas crop requests |
| [[minimap]] | Basic circular placeholder; missing real content/mask/blips/POIs |
| [[on-update-dirty]] | Blanket dirty discard suppresses cast bar; now tracks compact-raid cleanup, the `GameTimeFrame_SetDate()` calendar-atlas no-op fix, and the AuraButton OnUpdate lock-down (`~0.86ms` → `31.44us`, budgeted at `<=0.5ms`) |
| [[startup-createframe-profile]] | Runtime `CreateFrame` profiling started with action-bar button template expansion, then widened into the XML loader fast path; current safe loader state lands around 4.8s-5.8s on debug no-addons/no-saved-vars runs, with remaining misses dominated by XML script bodies |
| [[table-rehashing]] | 97K rehashes on startup; 98% from non-frame Lua tables, 81% land at hash size ≤16; root cause is `OP_NEWTABLE(0,0)` for addon `local t = {}` patterns |
| [[layout-profile]] | Layout was 7.5% of release startup; `LayoutCache` siphash dominated. `FxHashMap` switch drops to 5.0%, −170M layout samples, −219M total siphash samples |
| [[intern-string-ranking]] | 1.25M intern_string calls per startup; rilua bug found (mid-cycle inserts swept) + fixed. Migration landed for registry/metatable helpers: 1.25M → 1.10M (−12%), 1.18s → 1.15s, and follow-up perf shows `lua_hash` itself is down to 0.08% of startup. `frame_ref_cache` still deferred |
| [[partyframe-portrait-composition]] | Party member class icon is a `37x37` `Portrait`; the visible ring is not a separate widget but part of the larger `UI-HUD-UnitFrame-Party-PortraitOn` frame-art texture (`120x49` live on master) |
| [[partyframe-tree]] | `rilua-migration` regresses `PartyFrame` from master's `(120x244)` 4-member layout down to `(4x2)` with zero member frames; regression test pinned against the master dump |
| [[partyframe-statusbar-textures]] | XML loader passes `StatusBar` bar textures as userdata into `SetStatusBarTexture()`, so the party health/mana bar source gets cleared while masks still render via `SetAtlas()` |
| [[pve-tabs-direct-offset]] | Dungeons & Raids bottom tabs lost their base offset because direct `<Offset x="..." y="..."/>` attributes were ignored unless nested in `<AbsDimension>` |
| [[quest-scrollbar-partial-size]] | Quest scrollbar track/thumb drifted right because partial XML sizes like `<Size x="8"/>` were ignored unless both dimensions were present |
| [[rilua-mlua-gap-audit]] | Audit of mlua-era Lua API handling still missing on rilua: sandbox cleanup parity, dropped MessageFrame methods, secure/event follow-ups, and an unwired namespace patch hook |
| [[world-map-onupdate-hover-polling]] | Chat-frame hover polling was forcing mutable `IsMouseOver()` work on every idle tick; clean-layout hover checks are now read-only, empty `UIParent` worklists short-circuit, but the fresh 90s world-map profile still sits at 31 steady-state handlers |
| [[world-map-voice-chat-alerts]] | Reduced world-map stacks can show voice prompt frames above the map when `Blizzard_Channels` is loaded without `Blizzard_SocialToast` / chat-alert prerequisites |
| [[protected-frames]] | 3-condition enforcement, covered methods, remaining gaps |
| [[transparent-wrapper-render-order]] | Renderless `Frame`/`ScrollFrame` wrappers were creating fake z-order boundaries; descendant regions now hoist through them |
| [[talent-sheen]] | 22s synchronized sweep; white rectangle bug when masking broken |
| [[tooltip-alignment]] | NineSlice inner box vs outer bounds; 15px effective inset |
| [[tooltip-layout-timing]] | Tooltip sizing ran after layout resolution, so one frame could use stale bounds |
| [[tooltip-double-shell]] | Fake bootstrap `NineSlice` plus Rust fallback shell caused duplicate tooltip chrome; Lua-owned shell still needs center fill |
| [[unanchored-frame-render-leak]] | Unanchored frames have no valid WoW rect but were rendered at parent origin; render-list filtering now skips them and their descendants |
| [[glow-effects]] | Additive blending end-to-end; one gap: SetBorderBlendMode missing |
| [[global-frame-index]] | Lazy `_G` lookup design; Phase 1 done, Phases 2-3 planned |
| [[hybrid-scrollbar-thumb-texture]] | Runtime templates apply Blizzard `<ThumbTexture>` XML to intrinsic slider thumb children instead of creating HybridScrollBar placeholders |
| [[world-map-frame-level-rebuilds]] | World map pins were forcing no-op `SetFrameLevel()` invalidations; steady-state bucket rebuilds are now gone |
| [[root-region-render-order]] | Root-level regions in the same draw layer must sort by ascending creation id; the old reverse-id tie breaker made newer root regions draw underneath older ones |
| [[world-map-create-texture-sublevel]] | World-map textures were created at sublevel 0 because `CreateTexture(..., subLevel)` ignored its fourth argument; immediate `SetDrawLayer()` repair churn is now gone |
| [[world-map-fog-of-war-first-open-size]] | First-open world-map fog pins could keep a stale size because `FogOfWarPinMixin` only resized on canvas scale changes; simulator now patches size-change handling for existing and future pins |
| [[world-map-fog-of-war-overlay-model]] | Current world map has no `UiMapFogOfWar` entry; simulator hides fake fog, seeds one real unexplored chunk, and now serves `C_MapExplorationInfo` from DB2-backed Rust handlers for non-current maps too |
| [[mists-world-map-startup]] | Mists startup failures came from missing profile-cache files, escaped `Interface/AddOns` XML paths, QuestUtil reset timing, missing GameTooltipTemplate Lua, and narrow Mists-only proving-grounds/EditMode gaps |
| [[world-map-texture-loading-budget]] | World map tile uploads were hidden in BC texture work; retained-path follow-up fixed cached-request selection, dirty-sentinel clobbering, inverted request priority, and the staged-vs-ready texture state bug; later Lua pin retries were simplified to single deferred refreshes; current live startup trace shows the first world-map `present` already textured while atlas-ready still climbs `6 -> 24 -> 33 -> 282 -> 335`, and later redraws continue because `strata_dirty=0x1c` stays set after `textures_pending=false`; a fresh no-world-map retained trace after the BC-negative cache change dropped peak `draw textures` from `482.2ms` to `283.2ms` and `bc_parse` from `240.6ms` to `139.2ms` |
| [[dropdown-intrinsic-script-chain]] | Reputation dropdowns did not open because style-template scripts replaced intrinsic `DropdownButton` scripts; simulator now chains intrinsic handlers, stores `RegisterForMouse`, and propagates child mouse clicks to parents |
| [[menu-pool-set-to-defaults]] | Guild roster Mythic+ Rating dropdown rendered as a screen-spanning stripe because pooled menu element frames retained stale sizes; `SetToDefaults` now resets size to 0,0 and clears anchors |
| [[windows-port-build]] | Windows default builds failed at `iced_dynamic.dll` with MSVC `LNK1189`; `fast-build` is opt-in so default GUI builds avoid the forced DLL link, while headless CI gates GUI-only tests |
| [[three-slice-button-tiling]] | Escape menu button stripes came from inactive `HighlightTexture` children rendering every frame; standard button highlight children now render only on hover or locked highlight |
| [[dialog-background-dxt3-stripes]] | Escape-menu dialog background stripes came from treating DXT3 BLPs as BC3 on the raw compressed upload path; DXT3 now falls back to RGBA until BC2 atlas support exists |
| [[addon-startup-settings-and-item-load]] | Addon startup failures can share roots in Settings canvas visibility, secure attribute delegate taint boundaries, and synthetic C_Item item-load behavior |
