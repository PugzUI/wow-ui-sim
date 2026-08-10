# Debug Tools

Three complementary tools for inspecting the frame hierarchy. All read from the same Rust `WidgetRegistry` — no data comes from Lua directly.

## Inspector Panel (GUI)

**Trigger**: middle-click any frame in the running GUI.

A floating panel appears at the click position showing that frame's properties. Hit testing works by searching the cached hittable list in reverse strata/level order (topmost frame first), excluding non-interactive frames like `UIParent`, `Minimap`, `WorldFrame`, and chat frames.

**Editable fields**: W, H, Alpha, Level, Visible (checkbox), Mouse-enabled (checkbox). Clicking **Apply** writes changes back to `WidgetRegistry` and invalidates the layout cache, causing an immediate re-render.

**Caveat**: W/H show the *stored* explicit dimensions, not the computed size. Frames sized by two-point anchoring (e.g., TOPLEFT + BOTTOMRIGHT) will show `0×0` — the actual size comes from anchor resolution. The `Pos` field does show computed coordinates.

## Dump Tree (CLI)

Two variants with different use cases:

**`wow-sim dump-tree`** — standalone, loads the full UI then dumps without starting the GUI. Shows *stored* frame sizes. Prints an anchor diagnostic summary first (anchored/unanchored counts, top unanchored parent keys). Indentation only, no tree connector graphics.

```bash
wow-sim --no-addons --no-saved-vars dump-tree   # Fast
wow-sim dump-tree --filter ScrollBar            # Filter by name
wow-sim dump-tree --filter-key SpellBookFrame   # Filter + full subtree
wow-sim dump-tree --visible-only
wow-sim dump-tree --delay 500                   # Wait 500ms after startup events
```

**`wow-cli dump-tree`** — connects to a running `wow-sim` via Unix socket IPC. Shows *computed* layout positions from the live renderer. Uses tree connector graphics (`+-`, `|`). Includes per-frame anchor detail lines with resolved absolute positions.

```bash
wow-cli dump-tree
wow-cli dump-tree --filter Button
wow-cli dump-tree --visible-only
```

The key difference: connected mode uses `compute_frame_rect()` for anchor-resolved positions; standalone uses stored `frame.width`/`frame.height` directly.

## Deterministic Frame Time (IPC)

Native evidence can take ownership of OnUpdate-driven frame time through the live Unix socket:

```json
{"AdvanceFrameTime":{"seconds":0.75,"steps":45}}
```

The first valid request activates manual frame-time mode for the lifetime of the process. The simulator freezes the current WoW game clock, splits `seconds` into exactly `steps` equal increments, advances `GetTime()` before each matching OnUpdate call, resolves layout after each call, and reports both the command delta and cumulative manually advanced time. Once active:

- `GetTime()` and `GetTimePreciseSec()` remain frozen between explicit requests;
- wall-clock OnUpdate ticks are suppressed;
- C_Timer processing, party-health drift, and casting progression are paused;
- rendering, cooldowns, casts, message expiry, and other game-time consumers read the manual clock;
- later Lua IPC mutations receive a zero-elapsed OnUpdate flush so state settles without moving animations;
- only another `AdvanceFrameTime` request advances the WoW game clock and OnUpdate-based animations.

A zero-second request is valid and is the recommended way to freeze time before creating or activating an evidence fixture:

```json
{"AdvanceFrameTime":{"seconds":0.0,"steps":1}}
```

Each request is bounded to 60 seconds and 3,600 steps. Process logging and profiling retain their separate monotonic wall clocks; gameplay APIs and native rendering use the deterministic WoW clock.

## Debug Overlay (Visual)

Shader-level overlays rendered over the live UI:

| Flag | Effect |
|------|--------|
| `--debug-elements` | Red borders + green anchor dots |
| `--debug-borders` | Red borders around all elements |
| `--debug-anchors` | Green dots at anchor points |

Same flags available as environment variables: `WOW_SIM_DEBUG_ELEMENTS=1`, `WOW_SIM_DEBUG_BORDERS=1`, `WOW_SIM_DEBUG_ANCHORS=1`. Environment variables override CLI flags.

## Architecture Note

```
Lua API calls ──> WidgetRegistry <──┬── Inspector Panel (live, editable)
                                    ├── dump-tree connected (live, computed)
                                    ├── dump-tree standalone (one-shot, stored)
                                    └── Debug overlay (live, shader quads)
```

## Sources

- [docs/debug-tools.md](../../debug-tools.md) — full implementation details, file references, hit test logic

## See Also

- [[scaling-coordinates]] — coordinate system that affects displayed positions
- [[architecture-overview]] — WidgetRegistry and Lua/Rust sync
