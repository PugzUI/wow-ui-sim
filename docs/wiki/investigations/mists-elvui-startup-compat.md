# Mists ElvUI Startup Compatibility

Full-addon Mists startup exposed several unrelated simulator compatibility gaps that made ElvUI and Blizzard aura headers fail before the remaining addon errors could be isolated: missing trim aliases, plain frames exposing MessageFrame-only methods, Mists aura callbacks using the wrong tuple shape, scaled screen-size mismatches, missing chat hook globals, per-Font-object metatables, and simulator-driven layout dirtying that clobbered ElvUI popup scripts.

## Content

### Symptoms

The full Mists addon pass reported these startup failures:

- `SecureGroupHeaders.lua:742` called `value:trim()` and failed because simulator strings did not expose the Mists-era `trim` alias.
- ElvUI scrollbar skinning saw a plain frame field named `ScrollUp` as a callable method and treated that function as a button, later failing while indexing `btn`.
- `SecureGroupHeaders.lua:951` compared nil aura sort keys because Mists Blizzard code destructures `AuraUtil.ForEachAura` callbacks as legacy `UnitAura` tuples.
- ElvUI slider skinning hit `string.find` with nil anchor data because simulator-created Slider `Low`, `High`, and `Text` fontstrings had no default points.
- ElvUI Tooltip initialization failed at `GameTooltipText:FontTemplate(...)` because the method was added through one Font object's metatable but not visible on other Font objects.
- ElvUI DataTexts durability initialization failed because `GetInventoryItemDurability` was missing from the inventory probe globals.
- `ElvUI_StaticPopup1` ran Blizzard `GameDialogMixin:OnUpdate` instead of ElvUI's popup update handler, so `StaticPopup_OnUpdate` compared nil `dialog.timeleft` against zero.
- The `ElvUI Installation` close button rendered but was hard to click because ElvUI's scaled parent made the button resolve to a 20x20 screen rect while raw, unscaled `SetHitRectInsets(6, 6, 7, 7)` collapsed the clickable area to a tiny center strip.
- The `ElvUI Installation` panel and `RaidUtility_ShowButton` could still land in the wrong screen region after that hitbox fix because startup screen state was split: Lua reported a 1024x768 physical screen, while the backing `SimState` and root `UIParent` layout still defaulted to 1600x1200.

### Root Causes

The string error was a runtime surface gap: Mists vendor Lua expects both `strtrim(value, chars?)` and `string.trim(value, chars?)`.

The ElvUI scrollbar error was not a missing ElvUI button. The shared frame metatable exposed MessageFrame scroll-navigation methods such as `ScrollUp` on non-MessageFrame widgets, so ElvUI's child-field probing found a function where real WoW would return nil.

The aura sort error came from profile drift. Mists `SecureGroupHeaders` checks `AuraUtil.ForEachAura`, but its callback body still expects the legacy multi-return aura tuple rather than a mainline aura table. The simulator's generic AuraUtil behavior therefore supplied incompatible data after Blizzard created `AuraUtil`.

The slider error came from incomplete default child geometry. Real slider label regions have anchor points; simulator-generated default slider fontstrings were unanchored, so ElvUI's skinning logic received nil `anchorPoint`.

The Tooltip font error came from simulator Font objects each receiving a fresh metatable. Real WoW exposes a shared object-type method table for Font objects, so ElvUI's `AddAPI(GameFontNormal)` mutation of the Font metatable must also make `FontTemplate` visible on `GameTooltipText` and `GameTooltipHeaderText`.

The durability error was a missing inventory global. The simulator already tracked equipped item presence, but had no wear model; returning full current/max durability for equipped player slots and nil for empty slots matches the addon-facing shape without inventing persistent damage state.

### Fix Pattern

Keep the fixes at the compatibility surface that owns each behavior:

- `shared_bootstrap.lua` defines `strtrim` and `string.trim` because this alias is runtime-wide string surface.
- `frame_metatable.rs` filters MessageFrame-only method names from non-MessageFrame widget metatables while preserving ScrollingMessageFrame access.
- `mists/post_load.lua` patches `AuraUtil.ForEachAura` after Blizzard creates `AuraUtil`, adapting callbacks back to the Mists legacy tuple.
- `helpers_shared.rs` anchors generated Slider `Low`, `High`, and `Text` fontstrings at creation time.
- Font objects use a shared registry-backed metatable, because addon metatable mutations target the object type, not only one global Font instance.
- `inventory_probes.rs` exposes `GetInventoryItemDurability(slot)` from equipped item presence, returning full durability for modeled equipped items and nil for empty slots.

The full-addon probe after these fixes still reports separate ElvUI/Syndicator/StaticPopup issues, but no longer reports `SecureGroupHeaders`, `string.trim`, scrollbar `btn`, residual slider `string.find`, ElvUI Chat `SecureHook`, ElvUI Tooltip `FontTemplate`, or ElvUI DataTexts durability errors.

### Screen Size and ElvUIParent Placement

The ElvUI install panel text and raid-control position shared a later geometry root cause. ElvUI sets `UIParent:SetScale(0.64)` and expects `GetScreenWidth()` / `GetScreenHeight()` to return UI units, while `GetPhysicalScreenSize()` returns physical pixels. The simulator returned fixed physical values for all three during bootstrap, so ElvUI sized `ElvUIParent` to `1024x768` logical units under a `0.64` scale. That produced a physically smaller parent anchored to the bottom of the screen, pushing `RaidUtility_ShowButton` toward the middle and making install-panel content appear displaced or covered.

The fix keeps `GetPhysicalScreenSize()` physical, but makes `GetScreenWidth()` / `GetScreenHeight()` divide by `UIParent:GetEffectiveScale()`. `WowLuaEnv::set_screen_size()` also fires `DISPLAY_SIZE_CHANGED` and `UI_SCALE_CHANGED` so addons recompute layout when screenshot/GUI paths resize after addon startup.

### ElvUI Chat Hook Targets

ElvUI Chat initialized far enough to create chat frames, then aborted while installing AceHook secure hooks. The first missing target was `RedockChatWindows`, which Mists static popup definitions also call. After adding that Mists post-load function, the next missing hook target was the runtime global `GetPlayerInfoByGUID`, which Blizzard chat code and ElvUI both expect.

`RedockChatWindows` belongs in Mists post-load compatibility because it depends on Blizzard chat globals such as `FCF_DockFrame` and `GENERAL_CHAT_DOCK`. `GetPlayerInfoByGUID` belongs in the shared runtime surface because it is a WoW global used across chat, social queue, static popup, and shared unit utilities. After both were present, the ElvUI Chat `SecureHook` startup error disappeared and `ChatFrame1` was parented to visible `LeftChatPanel`.

### Font Object Metatable

ElvUI's `E:AddAPI(GameFontNormal)` adds methods such as `FontTemplate` to the Font object's metatable. The simulator previously attached a new metatable to each Font table, so `GameTooltipText` had the base Rust Font methods but not ElvUI's metatable additions. Reusing one registry-backed Font metatable matches the object-type API shape and removes the `Tooltip.lua:1089` startup failure from the full-addon Mists probe.

### Layout Dirty OnUpdate Preservation

ElvUI creates `ElvUI_StaticPopup*` frames from a template that clears the inherited Blizzard static-popup scripts, then installs `E.StaticPopup_OnUpdate`. The simulator's automatic layout invalidation called Blizzard `MarkDirty()` while processing size and text changes. `BaseLayoutMixin:MarkDirty()` optimizes dirty frames by assigning `self.OnUpdate`; when a layout child recursively dirtied its parent, that replaced the popup's ElvUI handler with `GameDialogMixin:OnUpdate`.

The fix keeps simulator-driven layout invalidation from stealing an already installed custom `OnUpdate`: before invoking the Lua layout-dirty helper, the Rust call site snapshots existing `OnUpdate` handlers for the target layout frame and its parent chain, then restores any handler that `MarkDirty()` changed. The full Mists ElvUI probe now keeps `ElvUI_StaticPopup1` on ElvUI's handler after `E:StaticPopup_Show("INCOMPATIBLE_ADDON", ...)`, and the `StaticPopup.lua:451` nil-timeleft error no longer appears.

The install close-button hitbox error was a coordinate-space mismatch. Layout rects are already scaled by the frame's effective scale, but hit-grid construction subtracted `SetHitRectInsets` values as if they were already in that scaled space. Scaling the insets by `frame.effective_scale` before building hit rectangles keeps ElvUI's skinned close buttons clickable across scaled parents.

The remaining installer/raid-control placement issue was a startup-state mismatch, not an ElvUI-specific anchor bug. `runtime_surface_bootstrap.lua` exposed `GetScreenWidth()`, `GetScreenHeight()`, and `GetPhysicalScreenSize()` as 1024x768 before `set_screen_size()` runs, but `SimState` seeded `UIParent` and `WorldFrame` with 1600x1200. ElvUI sized `ElvUIParent` from the 1024x768 screen globals, then anchored it inside the larger 1600x1200 `UIParent`, offsetting descendants and their click targets. The default `SimState` screen dimensions now match the bootstrap 1024x768 physical screen; the regression checks that a fresh env reports matching screen globals and `UIParent` size.

### Visualizer Half-Size Panel

The Mists WeakAuras panel could render at roughly half size even though the
runtime reported the expected `UIParent` effective scale of `0.53`. The Iced
viewport scale factor was `1.0`; the mismatch was in the simulator's
coordinate pipeline.

Layout resolution already applies frame effective scale to `LayoutRect`. The
Visualizer then applied `ui_scale()` a second time while emitting quads,
building hit rectangles, resolving line and mask geometry, and serializing
manifest physical geometry. A direct UIParent child was therefore reduced by
`0.53 × 0.53` instead of by `0.53` once.

The native/headless Visualizer stage now uses a 1:1 renderer viewport: all
those paths consume resolved `LayoutRect` values directly, while the manifest
records `renderer_scale: 1.0`. The existing frame effective scale remains in
layout, so this changes the renderer boundary without changing WoW API scale
semantics. The focused GPU regression in
`tests/visualizer_coordinate_raster.rs` verifies a 64x48 direct UIParent
child lands at `(40,30)` scaled once, and the screenshot manifest tests verify
the matching physical geometry.

## Sources

- [shared_bootstrap.lua](../../../src/lua_api/env_init/shared_bootstrap.lua) — trim alias compatibility
- [frame_metatable.rs](../../../src/lua_api/methods/frame_metatable.rs) — per-widget method filtering
- [mists/post_load.lua](../../../src/mists/post_load.lua) — post-load Mists AuraUtil tuple adapter
- [helpers_shared.rs](../../../src/lua_api/globals/create_frame/helpers_shared.rs) — default Slider label anchoring
- [fonts.rs](../../../src/lua_api/globals/font_strings_collection/fonts.rs) — shared Font object metatable registration
- [inventory_probes.rs](../../../src/lua_api/globals/inventory_probes.rs) — inventory item durability probe
- [size.rs](../../../src/lua_api/frame/methods/core_state/size.rs) — layout dirty `OnUpdate` snapshot/restore
- [runtime_surface_bootstrap.lua](../../../src/lua_api/env_init/runtime_surface_bootstrap.lua) — bootstrap screen-size fallback
- [env_runtime.rs](../../../src/lua_api/env_runtime.rs) — runtime screen-size globals and resize event dispatch
- [state.rs](../../../src/lua_api/state.rs) — default backing screen dimensions for built-in root frames
- [frame_collect.rs](../../../src/iced_app/frame_collect.rs) — scaled hit-rect inset conversion for hit testing
- [strata_emit.rs](../../../src/iced_app/strata_emit.rs) — native renderer viewport geometry and hit rectangles
- [rebuild.rs](../../../src/iced_app/render/rebuild.rs) — native strata batch emission
- [visualizer_coordinate_raster.rs](../../../tests/visualizer_coordinate_raster.rs) — once-scaled GPU raster regression
- [scaling-coordinates.md](../design/scaling-coordinates.md) — coordinate-space contract
- [mists/post_load.lua](../../../src/mists/post_load.lua) — Mists `RedockChatWindows` compatibility
- [PLAN.md](../../../PLAN.md) — remaining Mists full-addon error list

## See Also

- [[frame-data-flow]] — frame method/property lookup and why exposed methods affect addon probing
- [[talent-performance]] — earlier Mists full-addon startup investigation that exposed ElvUI login cost
