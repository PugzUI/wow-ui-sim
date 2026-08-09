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

`src/render/coordinates.rs` owns the only renderer-to-raster conversion:

```text
physical_x      = renderer_x      × renderer_scale
physical_y      = renderer_y      × renderer_scale
physical_width  = renderer_width  × renderer_scale
physical_height = renderer_height × renderer_scale
```

The Visualizer renderer scale is 0.53. `src/iced_app/strata_emit.rs` uses this converter when emitting screen rectangles, and `src/iced_app/screenshot.rs` uses the same converter for manifest geometry.

Frame `effective_scale` and stage `renderer_scale` are separate values. Effective scale already participates in anchor and size resolution before a `LayoutRect` exists. Renderer scale performs the final conversion from that logical renderer rectangle into PNG pixels. They are equal for common direct UIParent children at the fixed stage, but consumers must use `renderer_scale` or `physical_geometry`, not assume equality for nested locally-scaled frames.

## Manifest Schema Version 2

Each manifest records stage dimensions, all coordinate-space identifiers, the renderer scale, and dual geometry for every region and classified ElvUI frame. Flat `x`, `y`, `width`, and `height` fields remain for compatibility but now always equal `physical_geometry`.

```json
{
  "stage": {
    "physical_width": 2560,
    "physical_height": 1440,
    "logical_width": 4830.1887,
    "logical_height": 2716.9811,
    "renderer_scale": 0.53
  },
  "region": {
    "logical_geometry": { "x": 1351.02, "y": 218.36, "width": 33.92, "height": 33.92 },
    "physical_geometry": { "x": 716.0406, "y": 115.7308, "width": 17.9776, "height": 17.9776 }
  }
}
```

Region records also include native identity, parent identity, local and effective visibility, local size and scale, effective scale, alpha, parent-relative anchors, text/media metadata, and coordinate-space version. This is sufficient to compare manifest geometry with the exact raster without consulting renderer implementation details.

## Runtime Paths

Warm IPC capture and one-shot CLI recovery both pass the active renderer scale into manifest generation. The same native batch is used for the PNG and its manifest. Cropping occurs only after full-stage rendering; evidence that requires stage correspondence must use uncropped captures.

## Regression Coverage

- Unit tests pin the 2560 × 1440, 0.53 API values.
- Coordinate tests pin `1351.02 → 716.0406` and `33.92 → 17.9776`.
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
