# Scaling and Coordinates

The simulator exposes several coordinate systems because WoW APIs, anchor resolution, renderer layout, and PNG output do not share one origin or unit. Visualizer manifest schema version 2 names each space and records both logical and physical geometry so consumers never infer a scale.

## Canonical Coordinate Spaces

| Space | Manifest identifier | Origin and axes | Purpose |
|---|---|---|---|
| Physical output | `physical_pixels` | Top-left; X right; Y down | Lossless PNG dimensions and raster-localization rectangles. |
| WoW screen | `wow_screen_units` | Bottom-left; X right; Y up | `GetScreenWidth()`, `GetScreenHeight()`, and UIParent query semantics. |
| Renderer viewport | `renderer_viewport_units` | Top-left; X right; Y down | Post-anchor, pre-raster `LayoutRect` values used to emit quads. |
| Local frame | `local_frame_units` | Frame/anchor dependent | Raw `SetSize`, local frame scale, and untransformed region dimensions. |
| Parent anchor | `parent_relative_ui_units` | X right; Y up | `SetPoint` offsets relative to the selected parent anchor. |

WoW anchor offsets are converted to the renderer convention during layout. For example, positive WoW Y offsets move upward, so top-left renderer resolution subtracts the scaled offset.

## Fixed Native Visualizer Stage

At the required 2560 × 1440 output and UI scale 0.53, the public API contract is:

| API | Value | Space |
|---|---:|---|
| `GetPhysicalScreenSize()` | `2560, 1440` | Physical output pixels. |
| `GetScreenWidth()` | `4830.188679...` | WoW screen units. |
| `GetScreenHeight()` | `2716.981132...` | WoW screen units. |
| `UIParent:GetWidth()` | `4830.188679...` | WoW screen units. |
| `UIParent:GetHeight()` | `2716.981132...` | WoW screen units. |
| `UIParent:GetScale()` | `0.53` | Local UIParent scale. |
| `UIParent:GetEffectiveScale()` | `0.53` | Effective UIParent scale. |

`GetPhysicalScreenSize()` never returns logical values. `GetScreenWidth()` and `GetScreenHeight()` divide the physical dimensions by UIParent effective scale. UIParent dimension queries resolve its rendered rectangle back into WoW units.

## Authoritative Transform

The native/headless renderer uses a 1:1 viewport for the output target. Its
authoritative transform is therefore:

```text
physical_x      = renderer_x
physical_y      = renderer_y
physical_width  = renderer_width
physical_height = renderer_height
```

`src/layout.rs` applies each frame's effective UI scale while resolving the
layout rectangle. `src/iced_app/strata_emit.rs` and the other Visualizer paths
consume that rectangle directly; applying `ui_scale()` again would shrink
direct UIParent children twice. `src/iced_app/screenshot.rs` records the same
native geometry in the manifest with `renderer_scale: 1.0`.

Frame `effective_scale` and stage `renderer_scale` are separate values. Effective scale already participates in anchor and size resolution before a `LayoutRect` exists. Renderer scale describes the final conversion from that renderer rectangle into PNG pixels and is 1.0 for the native stage. Consumers must use `renderer_scale` or `physical_geometry`, not infer either value from a frame's effective scale.

## Manifest Schema Version 2

Each manifest records stage dimensions, all coordinate-space identifiers, the renderer scale, and dual geometry for every region and classified ElvUI frame. Flat `x`, `y`, `width`, and `height` fields remain for compatibility but now always equal `physical_geometry`.

```json
{
  "stage": {
    "physical_width": 2560,
    "physical_height": 1440,
    "logical_width": 4830.1887,
    "logical_height": 2716.9811,
    "renderer_scale": 1.0
  },
  "region": {
    "logical_geometry": { "x": 1351.02, "y": 218.36, "width": 33.92, "height": 33.92 },
    "physical_geometry": { "x": 1351.02, "y": 218.36, "width": 33.92, "height": 33.92 }
  }
}
```

Region records also include native identity, parent identity, local and effective visibility, local size and scale, effective scale, alpha, parent-relative anchors, text/media metadata, and coordinate-space version. This is sufficient to compare manifest geometry with the exact raster without consulting renderer implementation details.

## Runtime Paths

Warm IPC capture and one-shot CLI recovery both pass the active renderer scale into manifest generation. The same native batch is used for the PNG and its manifest. Cropping occurs only after full-stage rendering; evidence that requires stage correspondence must use uncropped captures.

## Regression Coverage

- Unit tests pin the 2560 × 1440, 0.53 API values.
- Coordinate tests cover the generic renderer-to-physical conversion and the
  native Visualizer 1:1 raster mapping.
- Manifest tests verify every physical axis equals the corresponding logical axis multiplied by renderer scale.
- Exact WeakAuras region tests cover unnamed roots and owned FontString descendants resolved through `WeakAuras.GetRegion`.

## Sources

- `src/render/coordinates.rs` — coordinate identifiers and renderer-to-physical transform.
- `src/lua_api/env_runtime.rs` — screen APIs and UIParent scale probes.
- `src/layout.rs` — anchor resolution and effective-scale application.
- `src/iced_app/strata_emit.rs` — logical rectangle to screen rectangle emission.
- `src/iced_app/screenshot.rs` — native manifest schema and geometry serialization.

## See Also

- [[layout-system]] — anchor and multi-anchor resolution.
- [[rendering-pipeline]] — quad emission and GPU projection.
- [[mists-elvui-startup-compat]] — ElvUI screen-size compatibility history.
