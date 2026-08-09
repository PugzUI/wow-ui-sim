# FontString Ancestor Movement Cache

A Lua-created FontString could report correct text, font, geometry, visibility,
alpha, and effective scale while producing no screenshot pixels after its parent
moved. The shared layout cache updated the region's rectangle, but the render
snapshot cache retained glyph vertices at the old absolute coordinates.

## Content

FontString glyph quads are emitted in absolute screen coordinates and cached per
widget. `SetPoint` on a parent marks that parent as a rect-dirty layout root.
`ensure_layout_rects` then recomputes the entire descendant subtree, so geometry
queries see the new FontString rectangle. Previously, only the dirty root was
also marked visually dirty. A descendant FontString could therefore reuse its
old cached snapshot; if the old parent position was offscreen, the manifest was
correct while the screenshot contained no text pixels.

The shared invariant is now enforced in `recompute_layout_subtree`: whenever a
widget's resolved `layout_rect` changes, that widget is marked visually dirty.
This invalidates exactly the snapshots whose absolute vertices changed while
preserving O(1) dirty-root marking and avoiding a blanket subtree invalidation.

The raster regression creates an unnamed Lua parent and unnamed FontString,
uses `Fonts\\FRIZQT__.TTF`, uploads the real glyph atlas to the headless GPU
renderer, checks non-background pixels, moves the parent, verifies the
FontString dirty ID, and checks that raster pixels follow the movement.

## Sources

- [state_render.rs](../../../src/lua_api/state_render.rs) — layout subtree recomputation and visual invalidation
- [fontstring_raster.rs](../../../tests/fontstring_raster.rs) — Lua/API-to-GPU raster regression

## See Also

- [[rendering-pipeline]] — cached quad snapshots and glyph rendering
- [[layout-system]] — dirty-root layout resolution and descendant geometry
- [[frame-data-flow]] — Lua widget state mirrored into Rust rendering state
