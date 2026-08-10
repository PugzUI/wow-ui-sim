# Wiki Log

Chronological record of wiki operations.

## [2026-08-10] update | Native Visualizer interactive streaming

Added `design/scalpel-visualizer-streaming.md` after separating exact native
manifest capture from reduced interactive delivery. Documented the `StreamFrame`
IPC schema, persistent WGPU pipeline/target/readback allocations, live strata
`Arc` cache reuse, texture and glyph revision reuse, atomic JPEG/WebP output,
and the 512 × 288 / 8 ms supported capacity profile. Exact 2560 × 1440 PNG
and manifest evidence remains an independent non-substitutable path. Updated
`index.md`.

## [2026-08-09] investigation | FontString ancestor movement cache

Created `investigations/fontstring-ancestor-movement-cache.md` after a
Lua-created FontString reported correct API/manifest state but rendered no
screenshot pixels following unnamed-parent movement. Recorded that layout
recomputed descendant rectangles without invalidating their cached absolute
glyph vertices, the shared rect-change invalidation fix, and the GPU raster
regression using the bundled Friz Quadrata font. Updated `index.md`.

## [2026-07-02] update | XML method binding timing

Updated `systems/xml-template-system.md` after live PTR probing and simulator
regressions clarified XML `method="..."` behavior: XML binding installs the
currently composed method function as the script handler, object fields and
`GetScript` storage diverge after `frame.X = ...` or `SetScript`, and private
methods under `useForbiddenObjectTable` resolve from the forbidden object table
for AuraContainer-style XML.

## [2026-07-02] update | 12.1 XML partitioned mixins

Updated `systems/xml-template-system.md` after implementing PTR 12.1
`ScopedModifier useForbiddenObjectTable` and partitioned mixin semantics for
AuraContainer-style XML. Recorded public vs forbidden frame partitions,
partition-aware KeyValues, `targetPartition`, `inboundPartition` self
substitution, and `secureDelegates` public delegate behavior.

## [2026-07-02] update | Blizzard UI CDN missing chunks

Updated `systems/casc-asset-cache.md`, `systems/addon-loading.md`, and `PLAN.md`
after wiring Blizzard UI sync to fetch missing authoritative CASC chunks from
Blizzard CDN by encoding key via public `Osso/casc-extract` when local streaming
install archives are incomplete. Recorded that repo/source mirrors remain
disabled and CDN archive indexes persist under `~/.cache/casc-extract/`.

## [2026-07-01] update | Per-profile Blizzard UI manifests

Updated `systems/addon-loading.md`, `systems/casc-asset-cache.md`, and `index.md`
after splitting the Blizzard UI cache manifest into profile-specific files under
`data/blizzard-ui-files/`. Recorded that the active `client-*` profile selects
both the manifest and CASC product (`wow` vs `wowt`) so PTR-only addon files no
longer have to be shoehorned into the retail manifest. Documented that Blizzard
UI cache population has no repo-source fallback; tracked listfile overrides are
used only to teach CASC path→FDID mappings until upstream listfiles catch up.

## [2026-07-01] update | Bootstrap TOC semantics

Updated `systems/addon-loading.md` after correcting `[Bootstrap]` semantics:
annotated entries stay in normal TOC order, no standalone bootstrap pass runs,
and LoadOnDemand addons execute those files only during normal `C_AddOns.LoadAddOn`.
Updated `index.md` summary wording.

## [2026-06-29] investigation | Retail/PTR full startup Lua errors

Created `investigations/retail-ptr-full-startup-lua-errors.md` after full GUI
startup logs exposed handler-time errors missed by `lua-errors`. Recorded the
PVPUI API gaps (`C_WeeklyRewards`, `C_PvP`, legacy PVP role globals,
`ClearBattlemaster`), PTR cursor gap, Store inbound nil `StoreFrame` fallback
issue, and verification with clean retail/PTR startup logs. Updated `index.md`.

## [2026-06-29] update | Mists 5.5.4 lua-errors cleanup

Updated `investigations/mists-world-map-startup.md` after reducing Mists
5.5.4 startup `lua-errors` to `[]`. Recorded the root causes: escaped
`Interface/AddOns` XML paths, missing profile-cache manifest files
(`Blizzard_UIParent/Classic/*`, `Blizzard_SharedXML/Classic/GameTooltipTemplate.lua`),
`QuestUtil` being reset by `Blizzard_FrameXMLUtil/Classic/QuestUtils.lua`,
Mists XML referencing unshipped `WorldStateProvingGrounds_*` helpers, missing
native EditMode frame methods, and missing `UNIT_LEVEL_NON_ATTACKABLE` color.
Updated `index.md`.

## [2026-06-19] update | Bag button OnLoad load-order resolved

Marked `investigations/addon-load-order.md` RESOLVED. The historical
`PaperDollItemSlotButton_OnLoad`-before-definition failure no longer reproduces:
OnLoad completes (verified via `CharacterBag0Slot` event registration and clean
`lua-errors`), and the replay workaround was removed (`70fca4e25`, `d4f1287f9`).
Fixed dead `workarounds_bags.rs` / `l.rs` path references in both the wiki page
and the legacy `docs/addon-load-order-investigation.md`. Documented the root
cause: transitive `LoadFirst` — `Blizzard_EnvironmentCleanup` (`LoadFirst: 1`)
depends on `Blizzard_UIPanels_Game`, and the eager two-pass loader emits a
LoadFirst addon's deps first, so the definer loads before the bag buttons.

## [2026-06-19] update | PTR client profile

Updated `systems/client-profiles.md` after adding the `client-ptr` profile for
12.1 PTR. Documented the new `ptr` Blizzard UI cache scope, `120100` interface
version, mainline TOC/game-type behavior, and fallback preference for the
cache-managed `Gethe/wow-ui-source@ptr` archive before live/beta.
Updated again after the first PTR cache sync investigation: PTR sync now selects
the `wowt` CASC product automatically and filters legacy-profile manifest entries
before cache-completeness checks.

## [2026-06-19] update | Blizzard UI profile cache migration

Updated `systems/addon-loading.md` after moving runtime Blizzard UI loading to
profile-scoped user-cache roots under
`~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns`. Documented
`wow-cli casc sync-blizzard-ui` as the canonical cache population path, the
completion/provenance marker pair, and the setup-script compatibility wrappers.

## [2026-06-12] investigation | Lib test failure sweep

Created `investigations/lib-test-failure-sweep-2026-06.md` after root-causing
nine accumulated `cargo test --lib` failures: runtime foundation order
(Blizzard_ScriptErrors), the silently-refused
`hooksecurefunc(C_AddOns, "LoadAddOn")` target (two workarounds rewired through
`apply_blizzard_post_load_patches`; one dead hook remains in
`runtime_surface_bootstrap.lua`), two more classic-rebase code losses, drifting
duplicated Lua installers, and tests stale against deliberate semantic changes.
Updated `index.md`.

## [2026-06-10] investigation | Retail core behavior probes

Created `investigations/retail-core-behavior-probes.md` after adding and
installing `CoreBehaviorProbe`. Recorded the retail `12.0.5.67823`
observations for `SetForbidden`, `CreateForbiddenFrame`, invalid
`RegisterUnitEvent`, wildcard false attributes, and the improved Raise/Lower
probe that needs a fresh live-client capture before simulator behavior changes.
Updated it after the fresh reload capture: `Raise()` and `Lower()` succeeded,
but simple shown sibling frames kept `GetFrameLevel()` at `1`/`10` and
`GetRaisedFrameLevel()` at `0` before and after under both a private parent and
`UIParent`.
Updated it again after fixing wow-ui-sim so `GetRaisedFrameLevel()` returns the
retail-observed `0` for the simple sibling probe while keeping internal
`raise_order` as render/hit-order bookkeeping.
Updated it again after adding explicit regression coverage for wildcard
`GetAttribute(prefix, name, suffix)` preserving a stored false value.
Updated it again after the hit-order capture showed the higher raw frame level
kept mouse focus before and after `Raise()`/`Lower()`; wow-ui-sim now treats
`raise_order` as a same-level tie-breaker instead of adding it to
`frame_level`.

## [2026-06-10] update | Frame identity token userdata

Updated `investigations/frame-surrogate-identity-slot.md` after switching `frame[0]` from a tiny backed table to a `FrameIdentity` userdata token. Recorded that DevTools dumping also needs raw frame iteration plus `dumpobject` returning nil so `[0]` renders as opaque userdata.
Updated it again after making `extract_frame_id` dispatch-aware and adding `native_frame_id_from_val` for the rare cases that need the original table backing.
Updated it again after real-client and wowless duplicate-name probes showed replacement frames get fresh identity and do not migrate custom Lua fields; recorded the simulator fix that removed old-global field copying during `CreateFrame` registration.
Updated `systems/addon-loading.md` after fixing third-party addon enable-state merging and startup keybinding import. Recorded that local `AddOns.txt` overlays real WTF state, required-dependency disables apply to default-enabled dependents, and `bindings-cache.wtf` imports before addon loading.
Updated it again after extending the existing `ServerSnapshot` addon to capture AddOn List enable state and keybindings. Recorded that `ServerSnapshotDB` is the preferred live-client overlay for addon state because `AddOns.txt` alone is not reliable for the per-character UI state.

## [2026-06-08] update | Mists 5.5.4 EditMode

Updated `investigations/editmode-layout.md` after bumping the Mists Blizzard UI
source pin to 5.5.4. Startup was clean with the existing `C_EditMode` fallback,
but a real frame tick exposed missing `MIRRORTIMER_NUMTIMERS` in
`WorldFrame_OnUpdate`; the mirror-timer Rust surface now publishes the constant.
The update also records that Mists now loads `Blizzard_UIParentPanelManager`
through its `_Classic.toc`, so the simulator no longer excludes that addon and
the cache manifest now carries the Classic panel-manager files.

## [2026-06-08] investigation | PlayerSpells runtime load

Created `investigations/playerspells-runtime-load.md` after fixing the retail `TOGGLETALENTS` keybind path. The note records the `C_AddOns.LoadAddOn` call-frame preservation issue plus the temporary PlayerSpells ModelScene/PvP talent backfills needed for `Blizzard_PlayerSpells` demand-load.

## [2026-06-08] investigation | ModelScene player actor stub

Created `investigations/modelscene-player-actor-stub.md` after Collectionator's transmog recovery helper crashed while calling `GetPlayerActor():SetModelByUnit("player")`. Documented the compatibility boundary: 3D rendering remains intentionally stubbed, but ModelScene actor object methods must exist for addon probes.

## [2026-06-08] update | FontString default anchors

Updated `investigations/fontstring-default-anchors.md` after follow-up retail JustifyProbe data showed XML `ButtonText` uses the same `justifyH` implicit anchor as layer `FontString`, explicit vertical-only anchors suppress the default, and EditBox backing FontStrings remain unanchored even with XML `TextInsets`.

## [2026-06-08] investigation | FontString default anchors

Created `investigations/fontstring-default-anchors.md` after real-client testing showed unanchored XML `FontString` layer children use `justifyH` for their implicit anchor point. Updated `index.md` with the new investigation page.

## [2026-05-28] investigation | action button icon mask coverage

Created `investigations/action-button-icon-mask.md` after tracing vanished main
action-bar icons to mask sampling, not action state. The prior minimap mask fix
made the shader sample RGB mask intensity; action-bar icon masks store coverage
in alpha, so their black visible regions went transparent. The renderer now
marks alpha-backed masks with a shader flag while retaining RGB coverage for
opaque black/white masks.

## [2026-05-26] update | EditMode cache with no saved vars

Updated `investigations/editmode-layout.md` with the `--no-saved-vars` status
tracking regression. EditMode profile cache files are not Lua SavedVariables, so
startup now loads them through a separate WTF cache path when Lua SavedVariables
are disabled.

## [2026-05-18] investigation | ElvUI tooltip skin ordering

Updated `investigations/tooltip-double-shell.md` after tracing Character panel item tooltips under ElvUI to direct `GameTooltip` skin textures rendering after the tooltip frame's internal text emitter. `GameTooltip` now renders direct texture regions before the tooltip frame/text while still deferring direct FontStrings above it.

## [2026-05-18] investigation | Mists full-addon login profile

Updated `investigations/talent-performance.md` with a fresh full-addon Mists login profile. Startup is currently dominated by third-party Lua compilation (`16.73s` compile out of `25.64s` third-party addon load, bytecode cache `1/1395` hits), with AllTheThings and RaiderIO DB addons as the largest contributors.

## [2026-05-17] investigation | Mists ElvUI startup compatibility

Created `investigations/mists-elvui-startup-compat.md` after the full-addon Mists probe isolated ElvUI startup failures to trim aliases, overexposed MessageFrame methods on plain frames, Mists AuraUtil tuple shape, and unanchored Slider label fontstrings.
Updated it after tracing ElvUI install text displacement and raid-control centering to fixed physical screen-size globals under `UIParent` scale; runtime screen-size changes now also dispatch display/scale events.
Updated it again after fixing ElvUI Chat initialization: Mists now provides `RedockChatWindows`, and the shared runtime surface provides `GetPlayerInfoByGUID`.
Updated it again after fixing ElvUI Tooltip font initialization: Font objects now share an object-type metatable, so ElvUI `FontTemplate` additions made through `GameFontNormal` are visible on `GameTooltipText`.
Updated it again after adding `GetInventoryItemDurability`; ElvUI DataTexts now sees the expected inventory durability global and the full-addon probe no longer reports that startup error.
Updated it again after tracing the ElvUI static-popup `OnUpdate` error to simulator-driven layout dirtying. The size/layout dirty path now snapshots and restores existing custom `OnUpdate` handlers across recursive layout-parent `MarkDirty()` calls, so `ElvUI_StaticPopup1` keeps ElvUI's handler after `E:StaticPopup_Show`.
Updated it again after tracing the ElvUI Installation close-button click failure to unscaled `SetHitRectInsets` in hit-grid construction. Hit rectangles now scale insets by the frame's effective scale before subtracting them from scaled layout rects.
Updated it again after finding the remaining ElvUI installer and raid-control placement issue: startup screen globals reported 1024x768 while `SimState`/`UIParent` still defaulted to 1600x1200, so ElvUI anchored a correctly sized parent inside the wrong root canvas.

## [2026-05-17] update | SetAtlas empty clear semantics

Updated `systems/texture-atlas.md` after tracing missing Character panel paper-doll elements to `SetAtlas("")` resolving the generated empty-name atlas entry (`Interface\castingbar\uicastingbarstandardflipbook`). Empty atlas names now clear texture/atlas state and clear propagated parent button slots, which restores ElvUI-stripped equipment slot rendering.

## [2026-05-17] update | frame field environment numeric slot

Updated `systems/frame-data-flow.md` after tracing the ElvUI/oUF aura `button:SetSize` startup error to frame field storage occupying raw numeric key `1`. Frame refs now keep addon array slots free while `debug.getfenv(frame)[1]` remains a compatibility view onto normal frame fields.

## [2026-05-17] update | Mists talent first-open latency

Updated `investigations/talent-performance.md` after tracing full-addon Mists talent first-open latency to a deferred AceAddon enable queue. BlizzMove's skipped `PLAYER_LOGIN` left 27 queued Ace addons until `ADDON_LOADED("Blizzard_TalentUI")`; allowing BlizzMove to receive login again makes the talent load itself sub-second, with remaining ElvUI login cost tracked separately.

## [2026-05-17] investigation | Mists panel stack overflow layout cycle

Created `investigations/mists-panel-stack-overflow-layout-cycle.md` after reproducing the Achievements/Talents abort through the real GUI click path. Documented that the root cause was active layout resolution re-entering through parent/anchor cycles, not the Lua open-panel path, and recorded the new `headless-click-probe` regression check.

## [2026-05-17] investigation | unanchored frame render leak

Created `investigations/unanchored-frame-render-leak.md` after tracing the startup stray editbox/dropdown to render-list fallback geometry for unanchored frames. The render path now matches Lua rect validity and skips descendants of unanchored frames too.

## [2026-05-17] update | minimap mask clipping

Updated `investigations/minimap-map-ring-alignment.md` after fixing minimap rendering to use the stored/default minimap mask texture instead of synthetic circle clipping. The shader now treats RGB mask intensity as coverage for opaque black/white masks.

## [2026-05-17] investigation | minimap map/ring correction

Created `investigations/minimap-map-ring-alignment.md` to record the correction that the active minimap bug is the map texture/mask/ring alignment, not the SimCommands minimap button. The note documents the reasoning error and directs future debugging at minimap mask/clip/ring geometry.

## [2026-05-15] add | Mists heirloom tooltip

Created `investigations/mists-heirloom-tooltip.md` after the Mists Collections
heirloom button path threw on missing `GameTooltip:SetHeirloomByItemID`.
Documented that the fix belongs on the tooltip widget/data surface and routes
through `C_TooltipInfo.GetHeirloomByItemID`, reusing item tooltip data.

## [2026-05-15] add | Mists addon-panel resume mistake

Created `investigations/mists-addon-panel-resume-error.md` after incorrectly
rerunning already-proven Mists addon panel rows from `AllTheThings`. The page
records the direct rule: resume from the first unproven addon, which was
`Plater` for this run, and use `--start-at` / stable artifact roots instead of
discarding retained evidence. The resumed `Plater` and `SimpleItemLevel` rows
now have retained pass artifacts under the shared cache audit root.

## [2026-05-15] update | Mists backpack slot chrome

Updated `investigations/backpack-background-texture.md` with the Mists-specific
container path: bag ID 0 uses `UI-BackpackBackground` and its item buttons still
keep the authored `UI-Quickslot2` normal texture. Documented that clearing those
normal textures in post-load code is the wrong fix for Mists slot chrome.

## [2026-05-14] add | Hybrid scrollbar thumb texture

Created `investigations/hybrid-scrollbar-thumb-texture.md` after replacing the
HybridScrollBar-specific placeholder fallback with XML-backed slider
`<ThumbTexture>` application. Also documented the SharedXML test helper bug:
tests were pointed at removed `Interface/BlizzardUI` instead of the simulator's
Blizzard UI cache.

## [2026-05-13] update | EditMode active profile fallback

Updated `investigations/editmode-layout.md` with the C_EditMode active profile
state bug: the bootstrap fallback always returned `activeLayout = 1` and had a
no-op `SetActiveLayout()`, so Blizzard selected the first preset layout instead
of a saved profile. Documented the in-memory fallback state model and regression
coverage.

Follow-up: documented the real WTF import path. EditMode layouts come from
`edit-mode-cache-account.txt`; active per-spec selection comes from
`edit-mode-cache-character.txt`. Startup now imports those cache files before
Blizzard addons load, and Blizzard addons use the saved-variable-aware loader.

Follow-up: documented the second-stage layout mapping bug. `C_EditMode` selected
the saved layout, but `EditModeManagerFrame.layoutInfo` prepended presets and
kept the old index, activating `Classic`; startup now remaps the active saved
layout after prepending presets.

Follow-up: documented the visible action-bar anchoring bug. The Widescreen
layout was active and systemInfo was seeded, but action bars stayed at the
temporary `TOPLEFT, 0, 0` anchor because the startup fast path skipped
`ApplySystemAnchor`.

Follow-up: documented main action-bar side art as the `HideBarArt` EditMode
setting. Sparse saved layouts now merge missing defaults from Blizzard's modern
preset map, and the action-bar bootstrap path applies those values without
calling handlers that require an initialized `actionButtons` Lua array.

Follow-up: documented broader action-bar saved setting replay. The startup
fast path now applies safe runtime effects for saved visibility, icon
count/scale/padding, page number visibility, show-grid state, and button art
without repacking saved anchors.

Follow-up: documented raw/display conversion for saved action-bar profile
settings. The cache stores compact raw slider values, so the action-bar fast
path now reads through Blizzard's `GetSettingValue()` before applying display
values such as icon-size percentages.

Follow-up: documented account-level EditMode setting application. Startup now
invokes Blizzard's `InitializeAccountSettings()` after rebuilding layout info,
so saved account toggles are applied rather than merely copied into
`accountSettings`.

Follow-up: documented sparse account cache reconciliation. Imported account
settings now merge over defaults so older profiles still receive default values
for newer account-level EditMode toggles before Blizzard initializes account
settings.

Follow-up: documented cast-bar lock fidelity. Startup no longer overwrites the
active saved profile's `LockToPlayerFrame` / `CastBarUnderneath` settings.

Follow-up: documented saved setting replay for seeded EditMode systems. Startup
now applies saved settings for systems that skip full `UpdateSystem()`, while
deferring unit-frame `BuffsOnTop` if the seeded frame has no `UpdateAuras()`
method and would otherwise add a ScriptErrors entry.

## [2026-05-13] update | CASC font path fallback

Updated `investigations/casc-fdid-1579624-root-debug.md` after the standard
font FDIDs (`615960`, `615958`, `615971`) failed through asset-resolver's
path fallback with lowercase `fonts/...` paths. Documented the fix: preserve
canonical `Fonts/...` casing in the bundled listfile, normalize lookup keys
without losing entry path casing, and skip noisy `resolve_bytes(fdid)` calls
when the CASC resolution cache has no font FDID entry.

Follow-up: documented the real rendering failure after the noise fix. Skipping
missing font FDIDs suppressed the error but selected a system fallback font.
Font loading now reads standard font bytes from local CASC archives by
path-to-encoding resolution or known encoding-key fallback.

## [2026-05-09] update | Wrath vendor source switched to Gethe

Updated `systems/addon-loading.md` and `systems/client-profiles.md` after
standardizing the Wrath 3.3.5 source on `Gethe/wow-ui-source` tag `3.3.5`
(`c4e0255f`). Documented that Wrath's symlink points at the checkout root
because Gethe's 3.3.5 tag stores `AddOns/` and `FrameXML/` at repo root,
unlike newer profiles that keep sources under `Interface/`.
Recaptured the Wrath startup snapshot against the Gethe source (128 distinct
startup messages). The snapshot file was later removed from the Mists-only
merge scope.

## [2026-05-08] update | Windows default GUI build and headless CI compile

Updated `investigations/windows-port-build.md` after reproducing MSVC `LNK1189`
in the default `cargo build --bin wow-sim` path. The root cause remains the
local `iced-dynamic` DLL, but the current fix is to make `fast-build` opt-in
rather than part of default features. Also documented the no-default test
compile contract: GUI/render tests need `cfg(feature = "gui")`, GUI benchmark
binaries need `required-features = ["gui"]`, and CASC examples need
`required-features = ["casc"]`.

## [2026-05-08] update | CASC Friz Quadrata root probe

Updated `investigations/casc-fdid-1579624-root-debug.md` with known-good
FDID `615960` / `fonts/frizqt__.ttf` resolution data, hashes, and extraction
proof for Windows root parser debugging.

## [2026-05-08] add | Windows CASC Blizzard taint

Created `investigations/windows-casc-blizzard-taint.md` after fixing Windows
startup against the CASC-synced Blizzard UI cache. Documented the TOC and
`Blizzard_` folder-name taint semantics, plus the runtime `C_AddOns.LoadAddOn`
stack-taint clearing needed for Blizzard/secure TOCs loaded under a tainted
caller.

## [2026-05-08] ingest | CASC FDID 1579624 root debug

Created `investigations/casc-fdid-1579624-root-debug.md` with the verified
FDID-to-path mapping, content key, encoding key, CRLF hash proof against Gethe
`12.0.5`, extraction proof, local build caveat, and root parser debugging
checklist.

## [2026-05-08] update | CASC resolution cache location

Updated `systems/casc-asset-cache.md` after moving generated CASC resolution
metadata out of repo `data/casc` and into the asset-resolver user cache. The
page now documents product/build-key scoped cache paths and automatic rebuild
behavior for missing or stale `resolution.sqlite`.

## [2026-05-03] add | PVE tabs direct offset

Added `investigations/pve-tabs-direct-offset.md` after tracing the Dungeons &
Raids bottom tab placement to XML direct `<Offset x="..." y="..."/>`
attributes being ignored unless they used nested `<AbsDimension>`.

## [2026-05-02] add | Root-region render order

Added `investigations/root-region-render-order.md` after tracing an inverted
root-region tie breaker. Documented that root-level regions should use ascending
creation order inside the same draw layer, matching child regions, and that
`Reverse(id)` made newer root regions draw underneath older ones.

## [2026-05-02] update | Dropdown base mouse handling

Updated `investigations/dropdown-intrinsic-script-chain.md` after shared input
dispatch gained `RegisterForMouse` state and `SetPropagateMouseClicks` parent
dispatch. Documented why dropdown-like widgets should not need per-dropdown
click shims when the issue is physical mouse registration or child hit targets.

## [2026-05-02] update | LFD queue verbs

Updated `investigations/lfd-dungeon-list-empty.md` after the LFD join path got
past the group-size gate and hit missing `ClearAllLFGDungeons`. Documented the
new `ClearAllLFGDungeons` / `SetLFGDungeon` / `JoinLFG` / `GetLFGInfoServer`
surface and queued-mode state through Blizzard's Lua `GetLFGMode`.

## [2026-05-02] update | LFD normal dungeon group-size gate

Updated `investigations/lfd-dungeon-list-empty.md` after a five-player party saw
`You need a group of 1 players` when joining LFD. Documented that normal dungeon
entries must return nil for `GetLFGDungeonInfo` slot 17 (`minPlayers`) because
Blizzard treats a non-nil value as an exact group-size requirement.

## [2026-05-02] update | LFD reward cap info missing

Updated `investigations/lfd-dungeon-list-empty.md` after the Join as Party path
advanced into `LFGRewardsFrame_EstimateRemainingCompletions()` and hit missing
`GetLFGDungeonRewardCapInfo`. Documented the inert 11-nil return shape Blizzard
uses as the no-cap path.

## [2026-05-02] update | LFD Join as Party format error

Updated `investigations/lfd-dungeon-list-empty.md` after clicking `Join as Party`
raised `bad argument #2 to 'string.format' (number expected)`. Documented that
`GetLFGDungeonInfo` returned `mapName` in Blizzard's `minPlayers` slot, so
`ERR_LFG_MEMBERS_REQUIRED` received a dungeon name where `%d` expected a number.

## [2026-05-02] add | Adventure Guide disabled tabs

Added `investigations/adventure-guide-disabled-tabs.md` after tracing the
apparently unclickable Adventure Guide abilities tab. Blizzard intentionally
shows the tab disabled until a boss is selected, but simulator model stubs had
overwritten the shared `SetDesaturated` / `SetDesaturation` implementation, so
the disabled tab art stayed saturated and looked active.

## [2026-05-01] update | Journeys breadcrumb overlap

Updated `investigations/journeys-midnight-empty.md` with the later Midnight
renown-card overlap root cause. The issue was XML property order:
`setAllPoints=true` was applied after explicit `$parentInset` anchors, clearing
them and stretching `EncounterJournalJourneysFrame` to the full parent.

## [2026-05-01] update | EditBox child background render ordering

Updated `investigations/editbox-render-text-cache.md` after tracing the SimCommands search box typed text to render ordering: the search box's opaque child `BACKGROUND` texture rendered after the EditBox frame, while the EditBox frame emitter owns the internal input text and caret. Documented the new EditBox-specific strata DFS rule that child regions render before the EditBox frame emitter.

## [2026-05-01] update | Tooltip Lua NineSlice center fill

Updated `investigations/tooltip-double-shell.md` after the Journeys renown-card tooltip showed underlying card text through the tooltip body. Documented that the Lua-owned tooltip `NineSlice` should suppress the Rust fallback border/shell, but not the solid center fill needed while the simulator does not have a renderable opaque Lua center.

## [2026-05-01] ingest | Appearances Wardrobe API baseline

Created `investigations/appearances-wardrobe-api.md` after opening Collections Journal > Appearances in the simulator and auditing Blizzard Wardrobe/Transmog call sites against the current `C_TransmogCollection`, `C_Transmog`, and `C_TransmogSets` surfaces. Documented that the panel opens with no Lua errors, but real browsing/filtering/search/favorite behavior needs stateful source, visual, filter, and search backing rather than no-op filter setters and empty Lua bootstrap fallbacks.

## [2026-05-01] update | Wardrobe weapon slot switch crash

Updated `investigations/appearances-wardrobe-api.md` with the root cause for the `Blizzard_Wardrobe.lua:687` nil-index crash. The simulator was reporting main/offhand appearance slot location metadata as weapon collection categories, which made Blizzard treat weapons as armor setup slots; weapon slot metadata now uses `Enum.TransmogCollectionType.None` so the weapon-category path handles them.

## [2026-05-01] update | Wardrobe invalid appearance overlays

Updated `investigations/appearances-wardrobe-api.md` after Wardrobe rendered every head appearance card with the red invalid overlay. Root cause was missing displayability/usability fields on simulator appearance rows; `canDisplayOnPlayer`, usability, source validity, hidden/favorite defaults, name, and quality now come from the `C_TransmogCollection` row backing instead of forcing Blizzard's invalid-card path.

## [2026-05-01] ingest | Adventure Guide boss icon fallback

Created `investigations/adventure-guide-boss-icons.md` after tracing blank Encounter Journal boss icons to `EJ_GetCreatureInfo` returning `0` for missing creature icon fileDataIDs. Documented that Blizzard's boss button Lua relies on nil to select `UI-EJ-BOSS-Default`; `0` is truthy and makes `SetTexture(0)` clear the texture.

## [2026-05-13] update | asset-resolver cache root decoupled from game-engine

Updated `systems/casc-asset-cache.md` after moving `asset-resolver` path selection behind an explicit resolver config. wow-ui-sim now constructs the resolver with `$ASSET_RESOLVER_CACHE_DIR` or the normal user cache root and no longer sets `GAME_ENGINE_SHARED_ROOT` or assumes a local game-engine checkout for CASC/listfile cache data.

## [2026-05-01] update | Adventure Journal dungeon click stack overflow

Updated `investigations/lfd-dungeon-list-empty.md` after reproducing the stack overflow from `EncounterJournal_DisplayInstance(1271)`. Documented the split between modern `C_EncounterJournal.GetInstanceInfo` slot 9 (`linkDungeonID`) and legacy `EJ_GetInstanceInfo` slot 9 (`shouldDisplayDifficulty`) plus the same-button `Button:Click()` reentry guard that prevents the programmatic overview-tab click loop from aborting the process.

## [2026-05-01] update | LFD Join as Party leadership gating

Updated `investigations/lfd-dungeon-list-empty.md` after `JOIN_AS_PARTY` was still greyed out with valid role and dungeon selection state. Documented that party-size fixtures had made `party1` the leader, causing Blizzard's `LFD_IsEmpowered()` to reject the local player; `A_Admin.SetPartySize` and GUI party-size changes now default to local-player leadership.

## [2026-05-01] update | Adventure Journal LFD dungeon handoff

Updated `investigations/lfd-dungeon-list-empty.md` after the Adventure Journal dungeon click path exposed another LFD id/state gap. Documented that `AJ_DUNGEON_ACTION` depends on `DungeonAppearsInRandomLFD` and on Encounter Journal `linkDungeonID` values using the LFD id family, not Encounter Journal instance ids.

## [2026-05-01] update | Crafting cast duration

Updated `investigations/crafting-cast-bar.md` after the crafting bar was found to finish too quickly. The simulator had reused a 1.5 second GCD-style duration for `C_TradeSkillUI.CraftRecipe`; normal profession crafts now use a 2.0 second default, with a regression in `tests/test_crafting.rs`.

## [2026-04-30] ingest | CASC asset cache layers and measured costs

Created `systems/casc-asset-cache.md` after measuring the three stacked caches end-to-end with `examples/casc_bench.rs`. The doc covers (a) the resolution sqlite shared with the game-engine repo via `GAME_ENGINE_SHARED_ROOT`, (b) the per-listfile-path BLP byte cache at `~/.cache/wow-ui-sim/casc-extract/`, and (c) the per-process `TextureManager` in-memory cache, with concrete timings (~300 ms one-time CASC init, ~10 ms steady-state extract, ~1 ms disk hit, ~2 µs mem hit). Records that `Installation::initialize` no longer parses `root.bin`/`encoding.bin` — that work is permanently delegated to the resolution sqlite — so the per-extract cost stays in the millisecond range.

## [2026-04-29] ingest | Backpack body renders gray, not textured

Created `investigations/backpack-background-texture.md`. User showed a retail
screenshot of an open `Backpack` (combined-bags) window with a tan/brown
textured body and reported the simulator was missing it. Render-time tracing
confirmed the sim emits a solid `PANEL_BACKGROUND_COLOR` quad on the
`Bg.TopSection`/`Bg.BottomEdge` textures — i.e. exactly what
`FlatPanelBackgroundTemplate` authors. Both the pinned `12.0.5` vendor and the
`Gethe/wow-ui-source` `live` HEAD (verified via WebFetch + Codex
gpt-5.5/high) define `ContainerFrameCombinedBags` with no body atlas/file and
a no-op `UpdateBackground`. Bank's tan body comes from a separate
`bank-frame-background` atlas declared on `BankFrame` itself, not shared with
the bag panel. Conclusion: the textured retail look is applied outside the
public Blizzard source we have (addon overlay, unmirrored patch, or an
unknown runtime path); closed without a sim-side change.

## [2026-04-30] ingest | client-profiles system page

Created `systems/client-profiles.md` documenting the five-profile cargo-feature layout (retail/wrath/mists/era/anniversary), vendor pinning, profile-aware TOC suffix and gametype tables, per-profile compat bootstraps (`src/wrath/`, `src/mists/`, `src/era/` shared by era + anniversary), wrath's synthetic FrameXML addon, and the CI matrix. Updated `systems/addon-loading.md` to fix the now-stale "Mainline-only TOC suffix" claims and link to the new page; updated `index.md` with a row in `systems/`. Source: PLAN.classic.md Phase 7.x landings (cargo features, profile-aware loader/toc, src/era/ bootstrap), commits 2915f2b9..6b320417.

## [2026-04-30] update | addon-loading per-profile vendor structure

Expanded `systems/addon-loading.md` with a new "Vendor sources & per-profile setup" section: documents the gitignored `Interface/BlizzardUI/<Profile>/` symlink layout, the `setup-blizzard-ui.sh <profile> [ref]` and `init-worktree.sh` scripts, and the canonical vendor-repo + pinned-SHA table. Added per-profile addon-set notes to "Blizzard Addon Load Order" (24 wrath + synthetic FrameXML, ~112 mists, ~35 era/anniversary discovered after `_Vanilla.toc` filtering), and listed `[[client-profiles]]` under See Also. Source: `scripts/setup-blizzard-ui.sh`, `scripts/init-worktree.sh`, vendor-pin commits 73ba3465..afc7189b.

## [2026-04-28] ingest | Windows port build unblock

Created `investigations/windows-port-build.md` after the Windows smoke pass. The root cause was the local `iced-dynamic` re-export crate forcing a huge `iced_dynamic.dll` link, which hit MSVC `LNK1189`; the build now depends on upstream `iced` directly. Verified `wow-sim`, `wow-cli`, GUI startup, and screenshot output on Windows. Updated the note after adding shared WoW resource discovery for install root, CASC `Data`, extracted Interface art, AddOns, and WTF. Live WTF is documented and tested as read-only import; simulator-local SavedVariables take precedence once present.

## [2026-04-28] update | EditMode group-frame startup skip

Updated `investigations/startup-createframe-profile.md` with the resolved
remaining `apply_system_anchors` hotspot. File-based Lua profiling showed full
`UpdateSystem()` on `CompactRaidFrameContainer`, `PartyFrame`, and
`CompactArenaFrame` dominated the first pass; the workaround now seeds those
frames and applies anchors without running full roster/unit layout work.

## [2026-04-28] update | remaining EditMode startup hotspot narrowed

Updated `investigations/startup-createframe-profile.md` with follow-up probes
on the remaining post-load `apply_system_anchors` cost. `InitSystemAnchors`
and the registered-system loop were effectively free; the cost stayed in
`UpdateBottomActionBarPositions()` / managed-frame layout. Recorded the
discarded forbidden-attribute no-op idea because it broke
`tests/frame_positions.rs` by shifting `ObjectiveTrackerFrame`.

## [2026-04-28] update | duplicate post-event EditMode pass

Updated `investigations/startup-createframe-profile.md` with the startup
workaround duplication found during dump-tree profiling. `PLAYER_ENTERING_WORLD`
already ran post-event workarounds through `env_events.rs`, then
`fire_startup_events()` and `settle_headless_startup()` ran the same pass again.
Added one-shot state for `WowLuaEnv::apply_post_event_workarounds`; local
dump-tree logs now show two `apply_system_anchors` passes instead of four.

## [2026-04-28] ingest | three-slice button tiling

Created `investigations/three-slice-button-tiling.md`. Escape menu red button
stripes came from standard `HighlightTexture` children rendering while the
buttons were not hovered. The red-button center atlas special case was removed;
atlas tiling uses source size, and highlight children now render only when the
parent button is hovered or highlight-locked. Additive overlays also skip the
shader brightness boost so active highlights do not amplify low-alpha atlas edge
pixels into visible stripes.

## [2026-04-28] update | eager animation_frame_ids_for_group

Updated `investigations/talent-performance.md` with the panel-open 1.3 FPS root
cause. `advance_animation_group()` called `animation_frame_ids_for_group` on
every group every tick (full linear scan of `anim_frame_to_anim`), but the
result was only consumed inside the `if group_finished` branch. Moved the call
into the conditional. Live-process flamegraph went from 63.9% CPU in that
function to 0.00%; total animation-tick cost dropped from dominant to 3.5%.

## [2026-04-28] update | talent strata repair skip

Updated `investigations/talent-performance.md` with the discarded strata repair
root cause. `set_frame_visible()` was building same-strata repair plans during
talent frame show even when `strata_buckets` was `None`, so the work was thrown
away. Added the guard in `try_repair_strata_buckets_after_show`; release
`bench_talents` subsequent opens dropped from roughly 211-282ms to 112-150ms in
this worktree.

## [2026-04-28] ingest | menu pool SetToDefaults size/anchor reset

Created `investigations/menu-pool-set-to-defaults.md`. Guild roster Mythic+ Rating dropdown rendered as a screen-spanning stripe because `Frame:SetToDefaults` did not reset size or clear anchors. `Menu.lua` `MeasureFrameExtents` reads `frame:GetSize()` from pooled element frames, so previous-user widths inflated each menu measurement. Two `SetToDefaults` registrations existed on the shared frame metatable; `map_frames::register_all` runs after `misc::register_all` so the map_frames version is the active one (the misc registration is dead code). Extended `map_frames::set_to_defaults` to call `frame.clear_all_points()`, `frame.set_size(0.0, 0.0)`, clear `width_is_text_auto`, clear `layout_rect`, and `remove_all_anchor_dependents_for(id)` — matching real WoW semantics documented in `Compositor.lua`. Verified: Guild dropdown now stays at 180×103 even after a 900px synthetic dropdown is opened first; previously it grew to 1036→1172. All 8 minimap_specialized tests still pass.

## [2026-04-27] update | shallow `issecretvalue` for pool releases

Updated `investigations/talent-performance.md` with a new "Spec→Talents Tab Switch (~3.5s)" section. The Spec→Talents tab switch was multiple seconds because `LoadTalentTreeInternal` rebuilds the tree on every Show (`refreshOnShow=true`), and `talentButtonCollection:ReleaseAll()` calls `issecretvalue(frame)` 3× per button. The Rust fallback recursed into the entire frame's table tree (~7.4ms/call). Added `value_is_secret_shallow` in `src/lua_api/globals/security/secret_values.rs` that only inspects direct slot taints on tables, used by `issecretvalue`/`canaccessvalue`/`canaccessallvalues`. `canaccesstable` keeps the deep walk so its accessibility semantics still detect nested secret strings. Result: ReleaseAll 2159ms → 2.6ms; tab switch 3500ms → ~90ms; all 45 security_api tests pass.

## [2026-04-27] ingest | hero spec dialog anchor investigation

Created `investigations/hero-spec-dialog-anchors.md` documenting two anchor resolution bugs in `HeroTalentsSelectionDialog`. (1) `xml_layer_batch.rs` emitted all textures before all fontstrings into the same Lua chunk, breaking XML document order so SpecImage's `relativeKey="$parent.SpecName"` ran before SpecName existed and fell back to the spec frame. (2) Runtime template path lacked the loader's `resolve_named_anchor_targets_for_frame` re-pass, leaving NodesContainer's `$parent.Description` anchor as an unresolved string. Both fixed; talent node icons now render inside their LIGHTSMITH/TEMPLAR panels instead of at the screen bottom.

## [2026-04-27] update | CASC migration: textures and fonts

Migrated texture and font loading off bundled extracts onto direct CASC reads from the live WoW install at `/syncthing/World of Warcraft/Data` (via the `asset-resolver` crate). Removed `./textures` (~1740 WebPs), the `~/Projects/wow/Interface` BLP fallback path, the `disk_cache_dir` field, and `src/texture_cache.rs`. `WowFontSystem::new()` became no-arg and pulled FRIZQT__/ARIALN/frizqt___cyr from CASC; a short-lived embedded FRIZ fallback was later replaced by CASC encoding-key fallback so the repo does not ship font bytes. Updated `docs/wiki/systems/texture-atlas.md`, `docs/texture-atlas-system.md`, `docs/rendering-pipeline.md`, `docs/wiki/reference/cli-commands.md`, and `docs/wiki/index.md` to drop references to the old curated paths and the now-obsolete `convert-texture` / `extract-textures` add-a-missing-texture flow. CASC is gated by the `casc` feature (default-on); set `WOW_SIM_CASC=0` to disable.

## [2026-04-26] ingest | dropdown intrinsic script chain investigation

Created `investigations/dropdown-intrinsic-script-chain.md` to document the ReputationFrame dropdown root cause: style dropdown templates replaced intrinsic `DropdownButton` scripts, so Blizzard's `OnMouseDown_Intrinsic` was not in the click path. Recorded the simulator-side fix, the fake menu fallback removal, and the two regression tests.

## [2026-04-26] update | api-coverage refresh, FUTURE.md retired

Folded `FUTURE.md` into `docs/wiki/reference/api-coverage.md` and deleted the root file. The old "three-layer" stub architecture (Hand-written / `c_stubs_api*.rs` / `generated_stubs.rs (~19K lines)`) has been replaced by per-namespace modules under `src/c_api/` with explicit `permanent_shims/` and `temporary_shims/` subtrees, matching the C-API boundary policy in CLAUDE.md. The wiki page now describes the actual module layout, calls out the shim sub-trees, and points to `wow-cli audit-api --gaps --format plan` as the live source of remaining work instead of a hand-curated task list. Removed the `FUTURE.md` source link.

## [2026-04-26] update | architecture-overview refresh, DESIGN.md retired

Folded `DESIGN.md` into `docs/wiki/design/architecture-overview.md` and deleted the root file. Corrected several stale claims: rilua provides native taint tracking (the previous "stubbed as always-secure" line was wrong — `issecure`/`issecurevariable` work; what's missing is `SetAttribute`/`SetForbidden` enforcement, now spelled out as a non-goal). Removed the dropped `generated_stubs.rs (~19K lines)` reference. Expanded the module diagram beyond `lua_api`/`widget`/`render` to also cover `c_api`, `iced_app`, `loader`, `event`, `xml`, `lua_bridge`, `texture`, and `sound`, with `c_api` called out as a peer of `lua_api` per CLAUDE.md. Updated `addon-compatibility.md` and `development-phases.md` to drop the fixed "127+ addons" number and reflect that the Blizzard UI tree under `Interface/BlizzardUI/` (`SharedXMLBase`/`SharedXML`/`SharedXMLGame`/`FrameXMLBase`/`FrameXMLUtil`/`FrameXML` + per-feature addons) loads, not just `SharedXML`. Removed source links to the deleted `DESIGN.md`.

## [2026-04-26] update | scaling-coordinates refresh

Verified `docs/wiki/design/scaling-coordinates.md` against current source and folded the standalone `SCALING.md` into the wiki page (root `SCALING.md` deleted). Most open items from the original note are done: `GetScreenWidth`/`GetScreenHeight`/`GetPhysicalScreenSize` are now installed dynamically by `install_screen_size_globals()` in `src/lua_api/env_runtime.rs` and re-run from `set_screen_size()`; the hardcoded `TOPLEFT (10, -10)` override in `main.rs` and the debug purple border are gone; layout `size` flows through `src/iced_app/render/rebuild.rs`. Updated file paths (`src/iced_app/` is a directory; `src/lua_api/globals.rs` no longer exists), clarified that the renderer runs in iced top-left Y-down with Y flipped in `Uniforms::new`, and trimmed open items to anchor Y-axis end-to-end docs and a `CENTER`-anchor resize regression.

## [2026-04-24] add | achievement panel hide investigation

Added `investigations/achievement-panel-hide.md` to document the achievement
panel hide fix. The simulator workaround now delegates to Blizzard's real
`AchievementFrame_ToggleAchievementFrame()` / managed `HideUIPanel` path, and
animation advancement now fires child animation `OnFinished` handlers so XML
outro hide scripts can run. Updated `index.md` with the new page.

## [2026-04-24] update | taint-system doc refresh

Updated `systems/taint-system.md` to match the current rilua implementation and added a Blizzard `issecure()` call-site matrix with test coverage references.
SecureHandler APIs now document fallback frame-ref/snippet/wrap/unwrap behavior,
state and attribute drivers now document shallow driver application,
secret-value accessors now document marked/tainted value tracking, and source
links now point at `security.rs` plus current frame-method helpers instead of
removed `security_api.rs`, `secure_env.rs`, and `combat_lockdown.rs` paths.
Refreshed the `index.md` summary.

## [2026-04-24] update | spell description token resolver

Updated `systems/lua-api.md` to record the shared spell-description token
resolver used by both `C_Spell.GetSpellDescription()` and
`C_TooltipInfo.GetSpellByID()`. The note covers the supported DB2 token
families (`$s1`, `$<damage>`, `$<shield>`, `${...}`, `$STR`, `$INT`, `$AP`)
and the reason for the shared path: keep spellbook/tooltips from exposing raw
placeholders or drifting from the C API result. Refreshed the `index.md`
`[[lua-api]]` summary.

## [2026-04-24] update | SimulationCraft spell coefficients

Updated `systems/lua-api.md` with the local SimulationCraft source used for
spell-description coefficients and variables. Recorded the concrete formulas
now mirrored by `src/spell_description_resolver.rs`: Avenger's Shield
`1.55 * AP`, Crusader Strike `1.4 * AP`, Shield of the Righteous `0.95 * AP`,
Eye Beam `$<dmg>` as `10 * 0.4026 * AP`, and Shield of Vengeance `$<shield>`
as `30% max health * (1 + versatility damage)`.

## [2026-04-23] add | quest scrollbar partial XML size investigation

Added `investigations/quest-scrollbar-partial-size.md` to document the
QuestScrollFrame scrollbar alignment bug. Root cause was the direct XML
property path ignoring partial `<Size x="..."/>` / `<Size y="..."/>` values
unless both dimensions were present. Updated `systems/xml-template-system.md`
to record per-dimension size resolution and added the new investigation to
`index.md`.

## [2026-04-22] add | layout lock inventory reference page

Added `reference/layout-lock-inventory.md` as the consolidated source of truth
for UI layout lock coverage. The page inventories baseline frame rect locks
(`tests/frame_positions.rs`) plus subsystem-specific lock tests for objective
tracker, main action bar, status tracking bars, bag bar, micro menu, chat
frame, compact raid manager, character/reputation panels, and buff
icons/durations. Updated `index.md` with a new `[[layout-lock-inventory]]`
entry under `reference/` for discoverability.

## [2026-04-22] update | buff aura onupdate perf lock-down to 0.5ms

Updated `investigations/on-update-dirty.md` with the AuraButton
`OnUpdate <=0.5ms` lock-down pass. Recorded the focused perf harness
(`tests/buff_aura_onupdate_perf.rs`), the pre-fix max (`~0.86ms`),
the dominant hotspot (`SetFormattedText` inside `UpdateDuration`), and the
post-fix max (`31.44us`). Documented the engine-side fixes:
`SetFormattedText` width-hint no-op fast path,
`securecall` direct multi-return fast path with protected fallback, and
non-visual `FrameRef.__newindex` bookkeeping updates.

## [2026-04-22] update | setpoint no-op fast-path optimization

Updated `investigations/on-update-dirty.md` with the post-optimization
`SetPoint` rerun after moving the same-anchor no-op bail-out to
`set_point()` before `ensure_no_anchor_cycle(...)`. Recorded the measured
before/after no-op totals (`3.626955us -> 1.392966us`, about `61.6%` lower)
and updated the control-flow notes to reflect that cycle detection now runs
only on real anchor changes.

## [2026-04-22] update | formattedtext and fontobject no-op fast paths

Updated `investigations/on-update-dirty.md` with the post-optimization
measurements for `SetFormattedText` and `SetFontObject` after
`text/formatting.rs` changes. Documented the new same-signature
`SetFormattedText` cache guard (enabled only while global `format` matches the
captured default), confirmed behavior parity for overridden `format`
(`format_call_probe_same_text_calls=100`), and recorded the large
`SetFontObject` no-op drop (`steady_same_total_us` from `6.104us` baseline to
`1.201us` in the rerun).

## [2026-04-22] update | setalpha no-op fast-path optimization

Updated `investigations/on-update-dirty.md` with the `SetAlpha` fast-path
optimization follow-up in `core_state/alpha.rs`. Recorded a post-change
rerun of the existing `setalpha_bench.lua` microbenchmark and the key no-op
delta metric (`same - get`) showing lower measured no-op overhead than the
earlier baseline, plus the reminder that `noop_hot_setters` behavioral
contracts still pass.

## [2026-04-22] update | no-world-map retained trace after bc-negative cache

Updated `investigations/world-map-texture-loading-budget.md` with a fresh
no-world-map retained GUI trace captured after the BC-negative cache change.
Recorded the exact command and before/after peak timings from the trace logs:
`draw textures` dropped `482.2ms -> 283.2ms` and `bc_parse` dropped
`240.6ms -> 139.2ms`. Refreshed the `index.md` summary row for
`world-map-texture-loading-budget` with the new no-world-map baseline deltas.

## [2026-04-22] update | setalpha no-op microbenchmark baseline

Updated `investigations/on-update-dirty.md` with a pre-optimization
`SetAlpha` microbenchmark baseline from a headless `--exec-lua` run. Recorded
batch timings (`empty`, `GetAlpha`, same-value `SetAlpha(1)`, and alternating
state-change `SetAlpha`) and the required split:
Lua->Rust call overhead, same-value fast-path overhead, and real
state-change overhead.

## [2026-04-22] update | setformattedtext no-op microbenchmark baseline

Updated `investigations/on-update-dirty.md` with a pre-optimization
`SetFormattedText` microbenchmark baseline from a headless `--exec-lua` run.
Recorded the three required splits (argument formatting/parsing cost,
text-equality fast-path cost, and real text-change cost) and added a runtime
probe proving global `format()` is still called on same-text no-op updates.

## [2026-04-22] update | setpoint no-op microbenchmark baseline

Updated `investigations/on-update-dirty.md` with a pre-optimization
`SetPoint` differential microbenchmark baseline. Recorded split timings for
implicit-noop parse baseline, explicit target normalization/lookup overhead,
string target lookup overhead, a no-op equivalence-check proxy, and full
relayout/dirty extra cost. Also documented the static control-flow ordering in
`anchors.rs` proving anchor resolution and `ensure_no_anchor_cycle` run before
the no-op `unchanged` bail-out in `apply_set_point`.

## [2026-04-22] update | fontobject shown vertexcolor no-op baselines

Updated `investigations/on-update-dirty.md` with measured no-op and change-path
splits for `SetFontObject`, `SetShown`, and `SetVertexColor`
(`dispatch`, `pre-bail`, `true state-change`). Recorded the steady-state
comparison against the earlier `SetAlpha` baseline and pinned
`SetFontObject` as the highest remaining no-op cost in this primitive set.

## [2026-04-22] update | world-map Lua retry simplification

Updated `investigations/world-map-texture-loading-budget.md` with the
follow-up Lua pin retry cleanup after the request-handle refactor. The
`MapExplorationPinMixin` workaround now keeps only one deferred refresh at a
time instead of recursively re-arming the old timer loop, and the world-map
detail tests still pass. Refreshed the `index.md` summary row for
`world-map-texture-loading-budget` to mention the simplified Lua retry layer.

## [2026-04-22] update | world-map live retained trace recapture

Updated `investigations/world-map-texture-loading-budget.md` again with the
current HEAD live-GUI recapture. Recorded the retained startup sequence from
`ToggleWorldMap()` through `tick -> draw -> prepare -> present`, using the
exact `iced-debug` socket emitted by the process for the screenshot burst. The
first world-map tick was `dirty=0x1ff pending=true ready=6`; later draws
advanced atlas-ready `6 -> 24 -> 33 -> 282 -> 335`; the first post-present
world-map screenshot was already textured; and redraws continued after
`pending=false` because `strata_dirty` remained `0x1c`. Refreshed the
`index.md` summary row for `world-map-texture-loading-budget`.

## [2026-04-22] update | world-map RedrawAll verification

Updated `investigations/world-map-texture-loading-budget.md` with the
timer-path `RedrawAll` verification from the same live retained-GUI repro.
Recorded that the first post-warmup frame reached `draw -> prepare -> present`
without any extra presentation gate after texture warmup, with
`ready=38 -> 335` as `prepare()` drained the atlas backlog. Refreshed the
`index.md` summary row for `world-map-texture-loading-budget` to mention the
verified post-warmup frame path.

## [2026-04-22] update | world-map atlas tier pressure audit

Updated `investigations/world-map-texture-loading-budget.md` with the atlas
tier pressure audit from the same live GUI repro. Recorded that every observed
`prepare()` pass kept `retry=0` and `force_rgba_retry=0`, so there were zero
RGBA fallback failures and zero BC upload rejections while the world-map tile
paths drained queued work and reached atlas-ready completion. Refreshed the
`index.md` summary row for `world-map-texture-loading-budget` with the
no-rejection conclusion.

## [2026-04-22] update | world-map retained GPU buffer reupload audit

Updated `investigations/world-map-texture-loading-budget.md` with the retained
GPU buffer reupload audit from the same live GUI repro. Recorded that
`upload_strata` showed the dirty world-map strata being re-written with
`pending_tex_vertices=0` and resolved sample `tex_index` / UVs, proving the
first-open retained path was re-uploading the affected vertex buffers after the
atlas transition. Refreshed the `index.md` summary row for
`world-map-texture-loading-budget` with the buffer-reupload conclusion.

## [2026-04-22] update | world-map retained texture display follow-up

Updated `investigations/world-map-texture-loading-budget.md` with the latest
live-GUI follow-up. Documented that the screenshot path could render the world
map on first pass while the retained GUI still missed random tiles/overlays,
then recorded the four later bugs behind that gap: wrong warmup request source,
full-rebuild sentinel clobbering, inverted world-map request priority, and the
remaining `textures_pending` ownership conflict between queued preload and
draw-time retained recovery. Refreshed the `index.md` summary row for
`world-map-texture-loading-budget`.

Later that day, updated the same page again after the deeper state-model fix:
`gpu_uploaded_textures` was only a draw-staging set populated before
`prepare()`, not proof that a path was ready in the atlas. Recorded the new
atlas-ready tracker populated from `WowUiPrimitive::prepare()` and the switch
away from using the staging set for display-readiness checks.

## [2026-04-22] update | buffframe slow onupdate interpretation

Updated `investigations/on-update-dirty.md` with the current BuffFrame
slow-handler interpretation. Documented that logs like
`addon=Blizzard_BuffFrame handler=OnUpdate frame=#21946` map to anonymous
`AuraButtonTemplate` children, not the named `BuffFrame` root, using the
current `BuffFrame.lua` creation path, `handler_timing.rs` fallback formatting,
and a live `dump-tree --filter-key BuffFrame --visible-only` run that showed
five visible anonymous timed buff buttons. Also corrected the stale audit note
to match the current `onupdate_handler_audit.rs` regression coverage.

## [2026-04-21] update | world-map exploration non-current map surface

Updated `investigations/world-map-fog-of-war-overlay-model.md` with a
follow-up regression where `runtime_surface_bootstrap.lua` was still installing
synthetic `C_MapExplorationInfo` handlers and limiting explored overlays to
`currentMapID` / map `1`. Documented the new Rust-backed
`src/c_api/c_map_exploration_info.rs` implementation, fallback-only bootstrap
stubs, and focused non-current-map regressions in
`tests/c_map_exploration_info.rs`. Refreshed the `index.md` summary row for
`world-map-fog-of-war-overlay-model`.

## [2026-04-21] add | class talents edge frame levels

Added `investigations/class-talents-edge-frame-levels.md` documenting the
class-talents edge-over-icon regression and the fix in
`src/lua_api/workarounds.rs` that patches both the mixin and the live
`PlayerSpellsFrame.TalentsFrame` method, then re-levels active edges.
Recorded regression coverage in
`test_class_talent_edges_render_below_visible_talent_buttons`
(`tests/hero_talents.rs`) and updated `index.md`.

## [2026-04-21] update | hero talents visibility and edges

Updated `investigations/class-talents-trait-loadout-state.md` with the hero
subtree rendering regression where only one node appeared and connector edges
were missing. Documented root cause in
`check_spec_conditions_met()` (`src/lua_api/globals/missing_surface/traits.rs`):
spec-set conditions were treated as `AND` instead of `OR` across shared hero
node groups. Recorded the fix and new regression test
`test_active_hero_subtree_exposes_multiple_visible_nodes_and_edges` in
`tests/hero_talents.rs`. Updated `index.md` summary for the investigation page.

## [2026-04-20] investigate | tooltip layout timing

Added `investigations/tooltip-layout-timing.md` to capture the tooltip
one-frame mismatch caused by sizing after layout resolution. Documented that
`update_tooltip_sizes()` runs too late in the live render path, so the current
frame can render from stale `layout_rect` data even though tooltip line data is
fresh.

## [2026-04-20] ingest | tooltip double shell

Added `investigations/tooltip-double-shell.md` to capture the duplicate
tooltip chrome bug. Documented the two-layer root cause: a bootstrap-created
fake `NineSlice` surface on the Lua side plus an unconditional Rust fallback
tooltip background. Recorded the fix: remove the bootstrap injection, repair
tooltip `NineSlice` post-load with the real template-backed surface, and gate
the Rust fallback shell on whether the frame already owns `NineSlice`.

## [2026-04-20] update | blizzard ui test lanes

Added `reference/blizzard-ui-test-lanes.md` and updated
`reference/addon-compatibility.md` to document the explicit split between
Blizzard UI unit tests and addon-bootstrap coverage.

## [2026-04-20] update | blizzard ui addon closure resolver

Added `loader::discover_blizzard_addon_closure_for_screen()` and switched the
render-order test helpers to use it. The resolver walks TOC `Dependencies` and
`OptionalDeps` across the full screen-allowed Blizzard TOC set, so tests can
resolve explicit closures for load-on-demand roots instead of relying on a fake
monolithic Blizzard bundle.

## [2026-04-20] update | blizzard ui smoke targets

Updated `reference/blizzard-ui-test-lanes.md` with the first four explicit
addon-bootstrap smoke targets: combat log, macro UI, world map, and
settings panel. Added the shared smoke-target manifest and harness coverage in
`tests/common/blizzard_addon_manifest.rs`,
`tests/common/blizzard_addon_harness.rs`, and
`tests/blizzard_addon_smoke_targets.rs`.

## [2026-04-20] update | blizzard ui smoke target startup shape

Updated the smoke-target lane so it asserts target startup shape instead of
just "closure loads". The harness now preloads shared panel support, clears
that preload noise, then loads only the target closure; each target asserts no
recorded Lua errors, the expected global/frame pair, and one or two
representative behaviors.

## [2026-04-20] update | blizzard ui addon-bootstrap test home

Documented that addon-bootstrap regressions stay in `cargo test`, while
`wow-sim run-tests` remains the lane for addon-authored Lua suites. The key
tradeoff is that the smoke harness needs direct Rust access to loader state,
recorded Lua errors, and frame-tree assertions, which is easier to maintain in
the normal Rust test binaries than in a Lua-only wrapper.

## [2026-04-20] update | keybinding spellbook direct dispatch

Updated `investigations/keybinding-system.md` with the spellbook follow-up.
Documented that the simulator now routes `S` back to Blizzard-owned
`PlayerSpellsUtil.ToggleSpellBookFrame()`, removes the bootstrap spellbook
fallback wrapper, and dispatches simple zero-arg binding targets directly
instead of always compiling a Lua chunk first. Recorded the key finding that
raw Blizzard spellbook toggles were already correct and the remaining
first-open regression only happened through binding dispatch.

## [2026-04-20] add | class talents trait loadout state

Added `investigations/class-talents-trait-loadout-state.md` to capture the
remaining `C_Traits` restore that unblocked `PlayerSpells`. Documented the root
cause (live talent state existed, but Blizzard-facing trait queries still
returned placeholder config IDs / staged-change surfaces), the new
working-vs-committed diff model exposed through `TalentState`, and the focused
regression coverage for config mapping, purchase gating, and staged edits.
Updated `index.md` with the new page.

## [2026-04-20] ingest | partyframe portrait composition

Added `investigations/partyframe-portrait-composition.md` to capture the
party portrait sizing/composition result from live queries and Blizzard XML.
Documented that the class icon is the `37x37` `Portrait` texture, while the
visible ring/surround is not a separate widget and instead comes from the
larger `UI-HUD-UnitFrame-Party-PortraitOn` frame-art texture (`120x49` in the
live master GUI tree). Updated `index.md` with the new page.

## [2026-04-19] ingest | partyframe status-bar texture drop

Added `investigations/partyframe-statusbar-textures.md` to capture the root cause for the party health/mana bar `MISSING` render. The XML loader creates the bar child correctly, but `SetStatusBarTexture(bar)` passes a userdata frame into a setter that only accepts strings/numbers, so the status-bar source is cleared. Updated `index.md` with the new page.

## [2026-04-18] update | Track 2 metatable handle threading

Updated `investigations/track-2-intern-audit.md` with the current
handle-threading progress: a shared `hot_metatable_key(...)` accessor
now reuses the prewarmed registry handle for `__index` / `__newindex`
lookups in `methods.rs`, `globals/create_frame/helpers.rs`,
`globals/security.rs`, and `env_init/freeze_globals.rs`, with a static
fallback for bootstrap-skipping tests.

## [2026-04-17] ingest | rilua vs mlua gap audit

Added `investigations/rilua-mlua-gap-audit.md` after comparing the current
rilua registration path against `master`'s mlua-era Lua API surface. Recorded
the highest-signal missing handling buckets: no-op sandbox cleanup in
`env_init`, dropped `MessageFrame` method registration, unfinished
attribute/event/text widget parity, and the unwired `patch_namespace_stubs()`
runtime hook. Updated `index.md` with the new investigation page.

## [2026-04-17] update | startup XML loader fast-path follow-up

Updated `investigations/startup-createframe-profile.md` with the current XML
loader fast-path state after the runtime `CreateFrame` work. Recorded the
safe widening steps in `xml_frame/setup.rs` and the split
`template_chain/` machinery, the current clean `xml fast path` counters
(`hits=1868`, `slow=350`, `scripts=234`), the shared-worktree debug startup
range (~`4.8s`-`5.8s` on `--no-addons --no-saved-vars`), and the failed
generic global-method literal-arg experiment that still regresses real
startup. Refreshed the `index.md` summary for the page.

## [2026-04-16] update | intern_string perf re-profile correction

Corrected the earlier PLAN/wiki summary for the post-migration interning
profile. Fresh release `perf` on `wow-sim --no-saved-vars lua-errors` shows
`Gc::intern_string` at 179.5M cycles (2.98%), `StringTable::intern_hashed`
at 169.2M (2.81%), and inline `lua_hash` at only ~4.8M (0.08%). The original
"lua_hash essentially flat" note was wrong; the hash primitive is no longer
the bottleneck, and the remaining cost sits in bucket traversal / dedup work.

## [2026-04-16] update | intern_string_static mid-cycle fix + migration landed

Found the root cause of the earlier migration breakage. `intern_string_static`
inserts into `static_intern_cache`, but `mark_gc_roots` only scans that cache
at cycle start — a mid-Propagate insert is still pre-flip current-white, gets
swept at cycle end, and the cache ends up pointing at a freed slot. Fixed in
rilua by colouring the new ref Black during Propagate/Sweep/Finalize; added
`intern_string_static_mid_cycle_survives_sweep` regression test.

Applied the migration for `registry_get/set/table_or_create`, `registry_table`,
`attach_frame_metatable`, and fan-out callers — intern counter 1,250,287 →
1,096,266 (−12%), release startup 1.18s → 1.15s median (n=5). `frame_ref_cache`
(the biggest single call site, 286K/startup) remains deferred: even with the
GC fix, migrating that path cascades into "OnLoad (a nil value)" failures
for ~300 addons and the root cause isn't yet understood.

## [2026-04-16] ingest | intern_string call-site ranking

Added `investigations/intern-string-ranking.md` and the rilua
`intern-stats` feature. Startup runs 1.25M `intern_string` calls with
top 5 literals accounting for 40%. Attempted migrating the
registry/frame helpers to `intern_string_static` (counter dropped
−74.5%) but release triggered 22 new Lua errors — frames lose their
metatable methods when `registry_set` uses `intern_string_static`.
Reverted the migration; filed as a rilua follow-up.

## [2026-04-16] ingest | layout computation profile

Added `investigations/layout-profile.md`. Release `perf` on `lua-errors`
showed layout-attributed samples at 7.5% of total (470M of 6.3B), with
the biggest single contributor being `LayoutCache::get` via siphash —
the cache was a default `HashMap<u64, CachedFrameLayout>`. Swapped to
`FxHashMap`. Layout 7.5% → 5.0%, total siphash 295M → 76M, release
startup median 1.21s → 1.18s (n=10). Remaining layout cost is real
anchor arithmetic and `resolve_parent_rect` recursion — no further
easy wins.

## [2026-04-16] update | table rehashing fix #2 declined

Tried fix #2 (short-circuit `raw_set_impl` for sequential integer keys
when hash is empty). Two variants both net-negative:
- `array.push` (grow by 1): rehashes 69K → 86K (+25%), wall time tied.
- `array.resize(next_power_of_two)`: rehashes 69K → 57K (−18%) but
  wall time +30% from eager nil-fill on each boundary.

Conclusion documented in `investigations/table-rehashing.md`: the
existing rehash path's `compute_sizes` already does good amortization;
naive replacements either skip the over-allocation (more rehashes
later) or pay the over-allocation eagerly (worse wall time). Status
quo retained.

## [2026-04-16] update | table rehashing fix #1 applied

Applied fix #1 from `investigations/table-rehashing.md`: rilua
`OpCode::NewTable` now pre-allocates 4 hash slots when both size hints
are zero. Measured: 97,340 → 69,105 rehashes (−29%); release startup
median 1.31s → 1.21s (−8%, n=5 each). All 685 rilua tests pass (includes
new `op_newtable_empty_hint_preallocates_hash_to_avoid_first_rehash`).

## [2026-04-16] ingest | table rehashing investigation

Added `investigations/table-rehashing.md`. Profiled startup rehash counts via a
new `rehash-stats` feature in rilua. Found 97K rehashes — 98% from non-frame
tables (`OP_NEWTABLE(0,0)` from addon `local t = {}` pattern), 81% landing at
hash size ≤ 16. Frame-table pre-sizing (64 slots) is working: only 1.6K frame
rehashes. Documented three candidate fixes; none applied in this card.

## [2026-04-14] ingest | world-map exploration seed follow-up

Updated `investigations/world-map-fog-of-war-overlay-model.md` with the current
map exploration follow-up. After removing synthetic fog, Isle of Dorn still
showed fully explored because every default-visible overlay was treated as
discovered. Documented the temporary seed that leaves one real overlay chunk
(`WorldMapOverlay.ID = 4885`, The Three Shields / Skolzgal Mill) unexplored
until per-character exploration state exists, and refreshed the `index.md`
summary.

## [2026-04-14] ingest | world-map fog-of-war overlay model correction

Updated `investigations/world-map-fog-of-war-overlay-model.md` to correct the
previous diagnosis. The current world map does not have a `UiMapFogOfWar` DB
row, so the bug was not a wrong irregular fog shape; it was the simulator
inventing fog for any map art and rendering synthetic geometry from exploration
overlay gaps. Documented the DB-backed fog lookup, the removal of the synthetic
renderer, and the new API/render regressions. Updated `index.md` with the
corrected summary.

## [2026-04-14] ingest | world-map fog-of-war overlay model

Created `investigations/world-map-fog-of-war-overlay-model.md` for the third
world-map fog bug: exploration APIs were already using real irregular overlay
chunks, but the fog renderer still assumed a synthetic half-map model.
Documented the root cause, the `FogOfWarFrame` `uiMapID` plumbing, the new
overlay-complement fog geometry, and the focused API/render regressions.
Updated `index.md` with the new investigation page.

## [2026-04-14] ingest | world-map texture loading budget follow-up

Updated `investigations/world-map-texture-loading-budget.md` with a first-frame
world-map follow-up: BC-preloaded tiles were landing in `bc_cache`, but
`TextureManager::is_cached()` only consulted the RGBA cache. That caused
budgeted draw to pause early after the first BC upload and could make the
world map open with an apparent quarter-map fog/exploration artifact. Added
the BC-cache root cause, fix, and regression coverage to the investigation
page.

## [2026-04-14] ingest | world-map fog-of-war first-open size

Created `investigations/world-map-fog-of-war-first-open-size.md` for the fog
overlay bug where first-open world-map fog could keep a stale size even though
the map tiles were already correct. Documented the missing
`FogOfWarPinMixin:OnCanvasSizeChanged()` handling, the simulator-side
workaround that patches both the mixin and existing fog pins, and the focused
regression tests. Updated `index.md` with the new investigation page.

## [2026-04-14] investigations | world map 90s OnUpdate recapture

Updated `investigations/world-map-onupdate-hover-polling.md` with the fresh
90s world-map profile after the recent OnUpdate fixes. Recorded the new
`/tmp/worldmap-onupdate-20260414.log` numbers (`485` total `fire_on_update`
spikes, `31` steady-state handlers, `64.73ms` post-90s average), added the
new `world_map_onupdate_inventory` handler-ceiling regression test, and
refreshed the `index.md` summary for the page.

## [2026-04-14] investigations | startup XML lifecycle frame-id threading

Updated `investigations/startup-createframe-profile.md` with the loader
follow-up that removes repeated `name -> id -> frame_ref` lifecycle resolution
during XML finalize. Recorded the new `xml_frame.rs` / `xml_lifecycle.rs`
threaded-frame-id path, the focused regression test that fires lifecycle
handlers with a wrong display name but the correct frame id, and refreshed the
`index.md` summary for the page.

## [2026-04-14] investigations | world map UIParent empty worklist follow-up

Updated `investigations/world-map-onupdate-hover-polling.md` with the
`UIParent_OnUpdate` fan-out follow-up: `FCF_OnUpdate`, `ButtonPulse_OnUpdate`,
and `AnimatedShine_OnUpdate` were still doing empty-list Lua dispatch every
tick. Recorded the new post-load wrappers in `workarounds.rs`, the focused
`uiparent_onupdate_worklists` regression tests, and refreshed the `index.md`
summary for the page.

## [2026-04-14] investigations | on-update dirty GameTimeFrame calendar atlas follow-up

Updated `investigations/on-update-dirty.md` with the `GameTimeFrame_SetDate()`
follow-up: same-day calendar atlas updates were still dirtying render because
the plain button texture setter took visual mutable borrows before checking for
real changes. Recorded the new no-op fast path in
`apply_set_button_texture_path()`, the focused atlas-backed button regression
test, the full-UI `GameTimeFrame_SetDate()` regression test, and refreshed the
`index.md` summary for the page.

## [2026-04-14] investigations | on-update dirty handler audit follow-up

Updated `investigations/on-update-dirty.md` with focused handler-audit results:
`LeaveInstanceGroupButton` now shows pure query/dispatch cost once its mutators
settle, while the remaining BuffFrame button cost comes from
`AuraButtonMixin:OnUpdate` doing duration formatting and font-threshold work on
every tick before the no-op setters bail out. Refreshed the `index.md` summary
for the page.

## [2026-04-14] investigations | on-update dirty solo compact raid manager follow-up

Updated `investigations/on-update-dirty.md` with the compact-raid follow-up:
`A_Admin.SetPartySize(0)` now fires `GROUP_ROSTER_UPDATE`, so solo transitions
hide `CompactRaidFrameManager` and remove `LeaveInstanceGroupButton` from the
visible `OnUpdate` handler set. Refreshed the `index.md` summary for the page.

## [2026-04-14] investigations | startup CreateFrame profiling ActionButtonTemplate regions

Updated `investigations/startup-createframe-profile.md` with the direct
`ActionButtonTemplate` layer/fontstring/button-texture fast path: the new
Rust-side region creation in `template/elements*.rs`, the focused regression
test that proves the hot path avoids Lua region fallback, and isolated
`WOW_SIM_PROFILE_CREATE_FRAME` numbers showing another `-27.36%` drop in
explicit template time across the profiled action-bar button families.

## [2026-04-14] investigations | startup CreateFrame profiling nested SpellFX follow-up

Updated `investigations/startup-createframe-profile.md` with the nested `ActionButtonSpellFXTemplate` follow-up: the remaining `ActionButtonInterruptTemplate` / `ActionButtonCastingAnimFrameTemplate` child creation fallback, the widened direct-child selector in `template/children.rs`, and new `WOW_SIM_PROFILE_CREATE_FRAME` numbers showing another `-28.6%` drop in explicit template time across action-bar button families.

## [2026-04-14] investigations | startup CreateFrame profiling MinimalScrollBar recursive fast path

Updated `investigations/startup-createframe-profile.md` with the `MinimalScrollBar` follow-up: the missed `Track -> Thumb` Lua `CreateFrame` fallback inside `apply_inline_frame_content()`, the recursive direct-child propagation change, the new focused regression test, and the smaller but measurable no-addons startup improvement after the fix.

## [2026-04-13] ingest | startup CreateFrame profiling

Created `investigations/startup-createframe-profile.md` to record runtime `CreateFrame` profiling results for Blizzard startup. Documented the new `WOW_SIM_PROFILE_CREATE_FRAME` instrumentation, the measured dominance of action-bar button template expansion (~4.1s across 34 runtime-created buttons), and the link to the planned pure-Rust template child creation work. Updated `index.md` with the new investigation page.

## [2026-04-13] ingest | world map preload API follow-up

Updated `investigations/world-map-texture-loading-budget.md` with the remaining explored-overlay delay root cause: Blizzard's `MapTexturePreloader.lua` was calling `C_Map.RequestPreloadMap()`, but the simulator stubbed that API as a no-op. Recorded the new queued preload path for map art + exploration overlays, the focused `request_preload_map_warms_map_art_and_overlay_textures` regression test, and refreshed the `index.md` summary for that page.

## [2026-04-13] ingest | chat frame scrollbar anchor reapply

Created `investigations/chatframe-scrollbar-anchor-reapply.md` to document the `ChatFrame1` scrollbar/edit-box layout bug. Recorded the real root cause in `reapply_inline_anchors()`: inherited child-frame anchors were resolving `$parent...` against the child name instead of the actual parent frame name, which broke `relativeTo="$parentBackground"` lookups and pushed the resize/scrollbar chain to screen-relative layout. Updated `index.md` with the new investigation page.

## [2026-04-13] ingest | world map texture loading budget follow-up

Updated `investigations/world-map-texture-loading-budget.md` with the second root cause behind the remaining world-map stalls: preload cleared `textures_pending` after CPU cache warmup even while the GPU atlas still lacked most tiles. Recorded the new `gpu_uploaded_textures`-based pending check, the focused `budgeted_preload` regression tests, and refreshed the `index.md` summary for that page.

## [2026-05-01] investigation | crafting cast bar

Created `investigations/crafting-cast-bar.md` to document the missing professions spellbar root cause. `C_TradeSkillUI.CraftRecipe` performed inventory changes but never started player casting or fired `UNIT_SPELLCAST_START`; successful crafts now populate `SimState.casting`, notify spellcast listeners, and emit `UPDATE_TRADESKILL_CAST_STOPPED` on completion. Updated `index.md` with the new investigation page.

## [2026-04-13] ingest | world map CreateTexture sublevel investigation

Created `investigations/world-map-create-texture-sublevel.md` to document the follow-up world-map open ordering churn: `CreateTexture(..., subLevel)` ignored its fourth argument, pooled textures started at sublevel 0, and Blizzard immediately repaired them with `SetDrawLayer()`. Recorded the new regressions for `CreateTexture(..., subLevel)` and no-op `SetDrawLayer()`, plus the traced repro where post-open `SetDrawLayer()` invalidations dropped from 150 to 0. Updated `index.md` with the new investigation page.

## [2026-04-13] ingest | world map voice chat alert investigation

## [2026-05-01] investigation | Journeys Midnight empty

Created `investigations/journeys-midnight-empty.md` to document the empty Journeys tab on Midnight. Root cause: current expansion constants were 11, but default major-faction data only seeded War Within expansion 10 rows, so Blizzard's `JourneysFrameMixin:Refresh()` received an empty `C_MajorFactions.GetMajorFactionIDs(11)` result. Updated `index.md` with the new investigation page.

Created `investigations/world-map-voice-chat-alerts.md` to document the reduced-stack world-map overlay where voice prompt frames appeared above the panel. Recorded the two harness prerequisites behind it: `Blizzard_Channels` needs `Blizzard_SocialToast` for `SocialToastTemplate hidden="true"`, and alert positioning needs the real chat-alert addons instead of the `ChatAlertFrame` stub. Updated `index.md` with the new investigation page.

## [2026-04-27] investigation | explicit XML parent anchors

Created `investigations/explicit-xml-parent-anchors.md` to document the PaperDoll sidebar tab positioning bug. Root cause: nested XML frame creation preferred the containing frame over an explicit child `parent="..."`, so implicit anchors resolved to `PaperDollFrame` instead of `CharacterFrameInsetRight`. Added the page to `index.md`.

## [2026-04-13] ingest | world map OnUpdate hover polling investigation

Created `investigations/world-map-onupdate-hover-polling.md` to document the post-texture-fix `UIParent_OnUpdate` cost: `FCF_OnUpdate` hover polling, the unnecessary mutable borrow in `IsMouseOver()`, the new immutable-borrow regression test, and the runtime repro where verbose OnUpdate logs stayed quiet after startup. Updated `index.md` with the new investigation page.

## [2026-05-02] investigation | Adventure Guide layout

Created `investigations/adventure-guide-layout.md` for the Suggested Content overlap bug. Root cause: `resolve_rect_if_dirty` fast-path geometry queries recomputed only the queried resized frame, leaving sibling frames anchored to it with stale cached rects. The fix now recomputes anchor dependents for direct dirty roots and dirty ancestor roots, and queues those moved dependents for hit-grid updates; regression coverage is `querying_resized_anchor_target_updates_dependent_siblings`.

Updated `investigations/adventure-guide-layout.md` with the follow-up Suggested Content text overlap root cause: tooltip `GetNumLines` overwrote the shared FrameRef method slot, so regular FontStrings returned zero lines. FontStrings now compute wrapped line count from measured text height while GameTooltip keeps tooltip-backed line counts; regression coverage is `fontstring_get_num_lines_reports_wrapped_line_count`.

## [2026-05-02] investigation | Adventure Guide SimpleHTML markup

Created `investigations/adventure-guide-simplehtml-markup.md` for the boss
overview text rendering raw `|c...|Hspell...|h...|r` and `|n` escapes. Root
cause: Encounter Journal uses `SimpleHTML`, whose stripped-text path only
removed HTML tags and bypassed WoW markup cleanup. Updated `index.md` with the
new investigation page.

## [2026-04-13] ingest | world map texture loading budget investigation

Created `investigations/world-map-texture-loading-budget.md` to document the post-rebuild-fix world-map stalls: hidden BC tile uploads, preload/draw source-cache mismatch, the new BC cache in `TextureManager`, and the smaller draw/tick texture budgets. Updated `index.md` with the new investigation page.

## [2026-05-03] investigation | LFD role icon slowness

Created `investigations/lfd-role-icon-slowness.md` for the LFD load pause around role selection icons. Root cause: role icons/backgrounds are button atlas crops from `Interface\lfgframe\uilfgprompts`, and crop extraction decoded the full 2048x2048 BLP before producing small sub-regions. Documented the persistent crop cache added in `TextureManager::load_sub_region` and updated `index.md`.

## [2026-04-13] ingest | world map frame-level rebuild investigation

## [2026-04-13] investigations | startup CreateFrame profiling follow-up

Updated `investigations/startup-createframe-profile.md` with section-level template profiling, the method-only XML script fast path, widened direct-child creation for `ActionButtonSpellFXTemplate` / `MinimalScrollBar`, and current shared-worktree startup numbers showing `36.79s -> 28.89s` on `--no-addons --no-saved-vars`.

Created `investigations/world-map-frame-level-rebuilds.md` to document the world-map performance bug where map pins repeatedly called `SetFrameLevel()` with the same value, forcing unnecessary `strata_buckets` invalidation and bucket rebuilds. Updated `index.md` with the new investigation page.

## [2026-04-29] investigation | LFD dungeon list empty

Created `investigations/lfd-dungeon-list-empty.md` for the Dungeons & Raids panel populating empty when "Specific Dungeons" was selected. Root causes: missing `GetLFDChoiceCollapseState`/`GetLFDChoiceEnabledState`/`GetLFGLockList` globals breaking `LFGDungeonList_Setup`; `LFG_UPDATE_RANDOM_INFO` never fired at startup so `LFDQueueFrame.Specific` stayed hidden and `OnShow=LFDQueueFrame_Update` never ran; `is_random=true` on the negative-id header in `default_lfd_dungeons` routed `GetRandomDungeonBestChoice` to `-1`. Updated `index.md` with the new investigation page.

## [2026-04-25] ingest | micro-menu atlas revert investigation

Created `investigations/micro-menu-atlas-revert.md` to document the micro-menu hover/leave icon disappearance root cause: button atlas setters populated child `tex_coords` but not `atlas_tex_coords`, so restored normal textures could miss the atlas-crop render path. Updated `index.md` with the new investigation page.

## [2026-05-02] update | Adventure Guide portrait masks and icons

Updated `investigations/adventure-guide-layout.md` with the follow-up Adventure Guide portrait bug: `Texture:SetMask` was still a no-op, so card icons did not clip to the gold portrait rings, and two seeded Adventure Journal icon paths did not resolve. Documented the mask wiring and manifest-backed icon replacements.

## [2026-05-28] ingest | Paladin aura stance bar

Created `investigations/paladin-aura-stance-bar.md` to document the root cause behind the missing Paladin aura bar: default Paladin state had zero shapeshift forms, so Blizzard `StanceBarMixin:Update()` hid the bar before rendering. Added the state-backed fix and regression coverage notes for the raw shapeshift globals and Blizzard StanceBar layer.

## [2026-05-08] update | Blizzard UI cache-only runtime

Simplified the Blizzard UI runtime model: `data/blizzard-ui-files.txt` is the committed file list, CASC is the extraction source, and `~/.cache/wow-ui-sim/blizzard-ui` is the only runtime Blizzard UI addon root. Removed the old runtime discovery language for `Interface/BlizzardUI`, `vendor/wow-ui-source`, and local `BlizzardInterfaceCode` fallback.

## [2026-05-08] update | Blizzard UI CASC source cache

Updated `systems/casc-asset-cache.md` and `reference/cli-commands.md` for `wow-cli casc sync-blizzard-ui`, the `~/.cache/wow-ui-sim/blizzard-ui` source cache, and GUI startup fallback behavior when the GitHub checkout is missing.

## [2026-04-29] ingest | dialog background DXT3 stripes

Created `investigations/dialog-background-dxt3-stripes.md` to document the escape-menu background stripe root cause: DXT3 BLPs were incorrectly mapped to BC3 on the raw compressed upload path. Updated `index.md` with the new investigation page.

## [2026-06-09] update | Mists 5.5.4 EditMode action-bar and UnitIsCivilian

Updated `investigations/editmode-layout.md` with the Mists 5.5.4 action-bar root cause: default managed action bars can remain at Blizzard's temporary `TOPLEFT UIParent` anchor when the manual EditMode layout pass aborts before clearing `layoutApplyInProgress`. Documented the Rust-side finalizer that clears the guard and replays `UpdateActionBarPositions()`, the modeled `UnitIsCivilian` fallback for Classic TargetFrame, and the ObjectiveTracker phantom duplicate root cause: bundled-addon startup exposed legacy `WatchFrame` alongside modern `ObjectiveTrackerFrame`.

## [2026-06-09] investigation | frame surrogate identity slot

Created `investigations/frame-surrogate-identity-slot.md` after replacing the simulator-only frame surrogate `[1]` dispatch path with a `[0]` identity token model. Updated `index.md` with the new investigation page.

## [2026-05-01] ingest | editbox render text cache investigation

Created `investigations/editbox-render-text-cache.md` to document the SimCommands search-box input bug: keyboard input updated `Frame.text` but left `text_stripped` stale after `SetText("")`, causing glyph rendering to shape an empty string. Updated `index.md` with the new investigation page.

## [2026-05-01] ingest | LFD checkbox and role-state follow-up

Updated `investigations/lfd-dungeon-list-empty.md` with the follow-up Group Finder bug where Specific Dungeons populated but dungeon selections were empty and `GetLFGRoles` was still a false stub, leaving "Join as Party" disabled. Documented the state-backed role and LFD checkbox fix plus regression coverage.

## [2026-05-01] update | Wardrobe filter click dispatch

Updated `investigations/appearances-wardrobe-api.md` with the follow-up Wardrobe filter bug where menu descriptions and callbacks were valid, but clicks could be swallowed by decorative child regions during GUI hit testing. Documented the fix: final mouse targets must be mouse-enabled frames, while decorative children only guide hit-test descent.

## [2026-05-01] update | Wardrobe class dropdown and set fallback

## [2026-05-18] investigation | ElvUI tooltip scale clipping

## [2026-06-08] ingest | ServerSnapshot action bar import

Created `systems/server-snapshot-action-bars.md` after adding startup import for action-bar spell slots captured by the ServerSnapshot addon. Updated `index.md` with the new systems page.

Updated `investigations/tooltip-double-shell.md` with the follow-up ElvUI tooltip clipping root cause. Tooltip frame bounds were scaled by ElvUI effective scale, but internal `GameTooltip` glyph emission was not; tooltip text now scales font size, line spacing, and text insets with the frame effective scale.

Updated `investigations/appearances-wardrobe-api.md` with the Wardrobe class dropdown casing/color contract and the `C_TransmogSets.GetBaseSets()` nil fallback stack overflow. Documented that class display names come from localized `className`, colors from uppercase `classFile`, and empty set surfaces must return tables.

## [2026-06-11] create | Class talent edge lines

Created `investigations/class-talents-edge-lines.md`. Talent connector lines were missing because `IsRectValid()` reported dirty-but-resolvable talent buttons as invalid, causing Blizzard's edge positioning to skip `Line:SetStartPoint()` / `SetEndPoint()`. A second render-list gate also filtered endpoint-positioned `Line` widgets and arrowhead textures under anchorless edge-frame parents. Fixed by resolving dirty rects inside `IsRectValid()` and allowing parent-independent line/anchor geometry through render-list filtering while preserving the ordinary unanchored-parent guard.

## [2026-04-12] ingest | transparent wrapper render-order investigation

Created `investigations/transparent-wrapper-render-order.md` for the world map / quest log render-order fix. Updated it after a follow-up regression to document the depth-aware transparent-wrapper hoist in `state_render.rs`, including both world-map visibility coverage (`world_map_tiles_render_after_tiled_background`) and world-quest pin ordering coverage.

## [2026-04-09] ingest | systems/ pages created (10 pages)

Created all 10 systems/ pages from source docs in docs/:

- systems/layout-system.md — from layout-system.md + anchor-resolution.md
- systems/rendering-pipeline.md — from rendering-pipeline.md
- systems/widget-system.md — from widget-system.md + button-text-rendering.md
- systems/lua-api.md — from lua-api.md
- systems/event-system.md — from event-system.md
- systems/xml-template-system.md — from xml-template-system.md
- systems/addon-loading.md — from addon-loading-pipeline.md
- systems/texture-atlas.md — from texture-atlas-system.md
- systems/frame-data-flow.md — from frame-data-flow.md
- systems/taint-system.md — from protected-frame-enforcement.md + src/lua_api/frame/methods/methods_helpers.rs + src/lua_api/globals/security.rs + tests/protected_frame_enforcement.rs + tests/secure_handler_fallback.rs + tests/security_api.rs

Updated index.md systems/ table.

## [2026-05-28] ingest | Mists WorldMap startup failure cluster

Created `investigations/mists-world-map-startup.md` after fixing Mists startup errors around `WorldMapFrame`, `WorldMapTrackQuest`, `UpdateUIPanelPositions`, `FogOfWarFrameMixin`, and MapCanvas provider `OnAdded` defaults. Updated `index.md` with the new investigation page.

## [2026-04-10] ingest | Initial bulk ingest from 30+ existing docs

Bootstrapped wiki from root-level documentation files. Created pages across systems/, design/, investigations/, and reference/ categories.

## [2026-04-09] ingest | design/ and reference/ pages created

Created 7 pages from DESIGN.md, SCALING.md, docs/debug-tools.md, FUTURE.md, docs/c-api-signature-audit.md, docs/c-api-stub-audit.md, AGENTS.md, and PLAN.md.

Pages created:
- design/architecture-overview.md
- design/scaling-coordinates.md
- design/debug-tools.md
- reference/api-coverage.md
- reference/cli-commands.md
- reference/addon-compatibility.md
- reference/development-phases.md
## [2026-05-16] ingest | addon startup Settings and item-load investigation

Created `investigations/addon-startup-settings-and-item-load.md` to capture the root causes behind addon startup errors: registered Settings canvases must start hidden, forbidden attribute delegates need secure dispatch, item subclasses must return enUS keyword-compatible names or nil, and positive live item IDs need synthetic placeholder item info so item-load callbacks terminate.

## [2026-06-19] create | Retail Store secure pool constructor mismatch

Created `investigations/store-secure-pool-constructors.md` after fixing the retail Store blank/red card state. Root cause: Store code runs in `__secureenv`, whose `CreateFramePoolCollection` still pointed at the simulator fallback after `Blizzard_SharedXMLBase` installed Blizzard's proxy-backed constructor in `_G`. The fix syncs real pool/factory constructors into `__secureenv` after SharedXMLBase loads and pins the behavior with Store tree, pool surface, and text-cache corruption tests.

## [2026-05-26] update | C API temporary shim module retired

Updated `reference/api-coverage.md` after moving the last `src/c_api/temporary_shims` surface (`C_TransmogOutfitInfo` slot/outfit defaults) into `src/lua_api/workarounds/temporary/` and deleting the empty C API temporary-shim module. Temporary unmodeled `C_*` defaults now belong in Lua workaround modules; `src/c_api/permanent_shims/` remains only for intentional unsupported domains.

## [2026-05-23] update | transmog sets shim boundary

Updated `investigations/appearances-wardrobe-api.md` after moving `C_TransmogSets` empty/default set APIs out of `runtime_surface_bootstrap.lua` and into `src/c_api/temporary_shims/c_transmog_sets.rs`. The investigation still records the same wardrobe contract: `GetBaseSets()` and related set APIs must return empty tables rather than nil until real set inventory state exists.

## [2026-05-29] ingest | Mists Syndicator and Baganator startup cleanup

Created `investigations/mists-syndicator-baganator-startup.md` after fixing full-profile Mists startup errors. The investigation records the Mists-gated item taxonomy overrides needed by Syndicator and the minimal CharacterFrame/TokenUI bootstrap path needed by Baganator.

## [2026-06-10] update | class talent config-scoped visibility

Updated `investigations/class-talents-trait-loadout-state.md` after fixing a full-addon/SavedVariables talent panel regression where `C_Traits.GetNodeInfo(configID, nodeID)` discarded `configID` and evaluated Protection nodes against a stale view spec.

## [2026-06-11] ingest | CASC root v2 misparse dropped 89% of fdids

Created `investigations/casc-root-v2-parsing-missing-textures.md` after the magic-dispel debuff border atlas resolved correctly but its texture (`interface/hud/uidebuffframes.blp`) could not be extracted. cascette-rs 0d0e79a misparsed 12.0.5 TSFM v2 root blocks (split content-flags fields, wrong NoNameHash bit), silently dropping 2.8M of 3.19M root records. Fixed by pinning cascette-rs c5de2b9 / asset-resolver 3ab8a14 (wow-ui-sim 598a29909), rebuilding the resolution cache (346K → 1.88M entries), and clearing 904 stale `.missing` markers.

## [2026-06-11] update | EditMode first-apply dirtiness + player debuff modeling

Updated `investigations/casc-root-v2-parsing-missing-textures.md` with parts 2-3: the dispel swirly was also blocked by the EditMode startup workaround seeding frames before UpdateSystem (destroying first-apply setting dirtiness, so ShowDispelType never applied), and by the absence of any player-debuff modeling. Fixed in c527f05fa (lookup-then-UpdateSystem flow mirroring EditModeManagerFrameMixin) and 2ac3057b6 (A_Admin.AddDebuff + UNIT_AURA isFullUpdate payloads + nilable dispelName).

## [2026-06-11] create | XML scale attribute ignored

Created `investigations/xml-scale-attribute.md`. The hero talents box didn't encompass its node buttons: `FrameXml` had no `@scale` field, so all 127 `scale="..."` attributes in Blizzard XML were silently dropped — including `HeroTalentsTreeNodesContainerTemplate`'s `scale="0.85"`, whose absence left only 212 of the needed 272 local units inside the fixed 284×362 backplate. Fixed in 91835d898 by parsing `@scale` and applying it through the template chain in both the XML loader and runtime CreateFrame paths, mirroring alpha.

## [2026-06-11] ingest | Journeys renown card text anchor fallback

Created `investigations/journeys-renown-card-text-anchor.md`. Reported as text z-ordering, but the Journeys "Renowns" card name/level FontStrings were anchored to the wrong target: their `relativeKey="$parent.IconFrame"` failed eager resolution (loader creates Layers before child Frames) and SetPoint silently fell back to the parent card, pushing the text under the adjacent column. Fixed in 7c0c0f987 by storing unresolved $parent key expressions on the anchor for the existing post-children lazy resolution pass.

## [2026-06-11] ingest | Mouse dead at frozen 50 FPS (probe blockers + idle tick stall)

Created `investigations/mouse-dead-probe-blockers-idle-ticks.md`. CoreBehaviorProbe (loaded from a renamed `.disabled` folder) left two full-screen mouse-enabled DIALOG blockers over UIParent because its 3-deep C_Timer.After cleanup chain stalls: pending C_Timers do not wake the tick loop once the app idles (open bug), and the frozen tick loop also freezes the FPS display at its last value. Loader fixed in b580ea005 to only accept TOCs naming their folder.

## [2026-06-11] update | tick-subscription churn root cause fixed

Updated `investigations/mouse-dead-probe-blockers-idle-ticks.md`: the idle timer stall was subscription churn — compute_tick_interval returned the raw shrinking remaining-time of the next C_Timer, changing the iced time::every identity on every update, so continuous input (mouse moves) recreated the tick stream before it could fire. Fixed in 397742569 with quantized interval buckets.

## [2026-06-12] create | DISPLAY_SIZE_CHANGED / UI_SCALE_CHANGED firing conditions

Created `investigations/display-size-ui-scale-events.md`. A live retail probe (`docs/addons/ScaleEventProbe`, 12.0.5.67823) disproved the sim's "resize never fires UI_SCALE_CHANGED" assumption: retail fires the two events as an ordered pair (display-first) on every display/scale recalculation — drag resize emits repeated ordered pairs during the drag, maximize/restore and resolution/fullscreen transitions can emit double-pairs even when dimensions are unchanged, scale slider and useUiScale changes also emit the pair, and startup fires the pair twice pre-PLAYER_LOGIN. Events fire before CVAR_UPDATE with GetEffectiveScale() already updated. Fixed `set_screen_size` to fire the pair and inverted the resize regression test to assert order. Startup ordering (sim fires post-login) and no-op dedupe remain divergent.

## [2026-06-12] update | Startup display/scale event ordering matched to retail

Updated `investigations/display-size-ui-scale-events.md`. Moved the `DISPLAY_SIZE_CHANGED`/`UI_SCALE_CHANGED` pair from `fire_post_login_events` into `fire_login_sequence` (after `VARIABLES_LOADED`, before `PLAYER_LOGIN`) — retail fires both pairs pre-login and none post-login. With the `set_screen_size` pair during GUI canvas startup this yields two pre-login pairs, matching the probe capture. Same-size window transitions reclassified as an observability limit (iced exposes no OS window-state signal). Verified: lib failure set unchanged, `lua-errors` clean with and without addons; new regression test pins pair-before-login ordering.

## [2026-06-12] update | hit-grid ordered insertion (the core mouse-freeze fix)

Updated `investigations/mouse-dead-probe-blockers-idle-ticks.md` with cause 4: full hit-grid rebuilds per hover transition (180-470ms each) saturated the main thread; the rebuilds were themselves a workaround for append-only HitGrid::insert breaking render order. Fixed with per-frame render-order keys + binary insertion, producer coverage for level/strata/raise changes, hover-time coalescing, and dev opt-level 1 (a7b131f65, telemetry 0c2668a3a).

## [2026-06-12] create | Mount Journal clicks never switched selection

Created `investigations/mount-journal-click-selection.md`. Real mouse clicks on mount rows fired OnMouseDown/OnMouseUp but never OnClick while `row:Click()` worked, because the startup-XML fast path's `parse_single_string_literal` stripped only the outer quotes — the generic MethodWithStringArg parser fused `RegisterForClicks("LeftButtonUp", "RightButtonUp")` into one garbage registration entry that no click edge can match. Fixed by rejecting interior quotes so multi-arg calls fall through to the dedicated parser. Added a `headless-click-probe mounts` panel (dotted-path frame resolution for anonymous ScrollBox rows + post-click `verify_lua`) and a `WOW_SIM_DEBUG_CLICK_DISPATCH=1` dispatch trace.

## [2026-06-12] create | Micro menu clicks missed from stale quadrant anchor

Created `investigations/micro-menu-click-offset.md`. GUI clicks on micro-menu buttons (LFD/Group Finder) did nothing: MicroMenuMixin:Layout anchors the menu inside MicroMenuContainer by screen quadrant, but the sim ran it before saved anchors and the real window size were applied, so the first press snapped the whole bar one QueueStatusButton slot (~46.5px) between mouse-down and mouse-up and the same-frame guard skipped OnClick. Fixed by replaying Blizzard's `InvokeOnAnyEditModeSystemAnchorChanged(force)` at the end of init_edit_mode_layout and from set_screen_size after the DISPLAY_SIZE_CHANGED/UI_SCALE_CHANGED pair. Added `headless-click-probe micromenu` regression panel.

## [2026-06-19] create | Post-load workaround audit

Created `investigations/post-load-workaround-audit.md` while auditing retail post-load hooks. The page records duplicate loader-side hooks retired for AccountStore, MapCanvas, and FrameXMLUtil, and classifies the remaining sampled hooks with current temporary rationale plus retirement paths.

## [2026-06-20] update | Third-party addon metadata pre-registration

Updated `systems/addon-loading.md` after fixing `!BugGrabber`'s false no-display warning when `BugSack` is present. The system page now records the invariant that discovered third-party addon metadata must be registered in `C_AddOns` before eager third-party Lua executes, while actual file loading still follows enabled, non-`LoadOnDemand`, dependency-sorted order.

## [2026-06-21] update | secureenv no-fallback retail probe

Updated `systems/taint-system.md` and `investigations/store-secure-pool-constructors.md` after a retail PrivateAurasUI cooldown-wrapper probe showed secure code does not hit late `_G` overrides. The simulator now models secureenv as a separate shallow copy without `__index = _G`; `Blizzard_SharedXMLBase` Lua is replayed into secureenv to populate shared secure symbols directly instead of copying constructors from `_G` after load.

## [2026-06-30] update | Initial TOC `[Bootstrap]` support

Added parser and loader support for TOC entries annotated with `[Bootstrap]`, motivated by LoD bootstrap glue such as `Blizzard_CooldownBroadcaster_Bootstrap.lua`. This was later corrected on 2026-07-01 after a live-client third-party probe showed `[Bootstrap]` does not reorder TOC files.

## [2026-07-01] correction | `[Bootstrap]` preserves TOC order

Updated `systems/addon-loading.md` after live-client probes showed `[Bootstrap]` is not a separate pass and must not move files out of TOC order. `TocFile` now keeps annotated files in `files` and records a per-file bootstrap flag. Startup loads full TOCs for non-LoD addons and only annotated bootstrap files for LoD addons, preserving addon order; runtime `LoadAddOn` skips already-executed bootstrap files and a self `LoadAddOn(thisAddon)` call from bootstrap remains a benign reentrancy no-op.

## [2026-08-09] update | Native Visualizer coordinate contract v2

Updated `design/scaling-coordinates.md` after proving that native Visualizer manifest rectangles were pre-raster renderer units mislabeled as physical pixels. Added explicit physical, WoW-screen, renderer, local-frame, and parent-anchor spaces; pinned the 2560 × 1440 / 0.53 API values; documented the single renderer-to-physical transform; and defined dual manifest geometry with flat compatibility fields now carrying real PNG pixels. The renderer and manifest share `src/render/coordinates.rs`, preventing caller-specific conversion rules.

## [2026-08-09] update | Deterministic native frame-time IPC

Added bounded `AdvanceFrameTime` IPC for native Visualizer behavior evidence. The first request switches the running GUI to manual frame-time ownership, freezes the WoW game clock, pauses wall-clock OnUpdate/timer/party/casting progression, advances both `GetTime()` and OnUpdate by an exact duration over explicit equal steps, and preserves zero-elapsed Lua mutation flushes. Gameplay rendering and time consumers now share that clock, while process logging/profiling retain separate monotonic wall time. A zero-second request freezes time before fixture activation, making Start/Main/Finish and custom WeakAuras animation captures reproducible without sleeps.
