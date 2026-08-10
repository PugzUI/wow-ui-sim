# Scalpel Native Visualizer Streaming

Scalpel uses two distinct native capture contracts. They share the same live `wow-ui-sim` process and authoritative WoW stage, but they do not substitute for each other.

## Exact Evidence Capture

The `Screenshot` IPC request renders the full native stage at 2560 × 1440, optionally writes manifest schema version 2, and supports lossless PNG output. It is the only path used for coordinate, geometry, anchor, Condition, animation, and media certification.

Exact capture always rebuilds the current registry representation and never consumes a reduced stream frame as evidence.

## Interactive Stream Frame

The `StreamFrame` IPC request renders the current native stage into a reduced 16:9 output target for browser presentation:

```json
{
  "StreamFrame": {
    "output": "/session/render/stream.jpg",
    "width": 512,
    "height": 288,
    "quality": 65
  }
}
```

The response is JSON inside the standard `Output` envelope and reports:

```json
{
  "output": "/session/render/stream.jpg",
  "stage_width": 2560,
  "stage_height": 1440,
  "width": 512,
  "height": 288,
  "render_ms": 7.2,
  "encode_ms": 3.1,
  "total_ms": 10.5,
  "sim_time": 42.4,
  "cached_strata": true,
  "dirty_strata_mask": 12
}
```

JPEG is selected by a `.jpg` or `.jpeg` extension. Other extensions currently use WebP. Output is written through a temporary file and atomically renamed, so readers never observe a partial frame.

## Reused Native Resources

A warm Visualizer process owns one reusable headless WGPU context. It retains:

- device and queue;
- shader pipeline and GPU texture atlases;
- reduced render target and mapped-readback allocation;
- previously uploaded per-strata vertex/index buffers;
- the glyph-atlas revision last uploaded to the independent offscreen pipeline.

The live Iced renderer already stores one `Arc<QuadBatch>` per WoW frame strata. `StreamFrame` synchronizes dirty cached strata, compares retained `Arc` identities, and uploads only replaced batches. Existing texture paths are not decoded or uploaded again. New glyph pixels are uploaded only when the shared glyph-atlas revision changes.

The cache retains prior `Arc` batches rather than raw pointer values, preventing allocator address reuse from being mistaken for an unchanged scene.

## Correctness Fallback

If no live strata cache is available, `StreamFrame` rebuilds the current full registry batch before rendering. Reduced output dimensions change only the GPU target and viewport; geometry and projection remain anchored to the authoritative physical stage.

The exact `Screenshot` path clears stream strata before its merged render, and the next stream frame repopulates all current strata. This prevents stale or duplicate batches when switching between interactive and evidence captures.

## Performance Contract

The supported interactive profile is:

| Setting | Value |
|---|---:|
| Native stage | 2560 × 1440 |
| UI scale | 0.53 |
| Stream output | 512 × 288 JPEG |
| JPEG quality | 65 |
| Visualizer command tick | 8 ms |
| Target | at least 30 presented FPS after warm-up |

Performance certification must use a representative native scene for at least 30 seconds, record render/encode/round-trip latency, simulator-time advancement, CPU, memory, frame count, and an independent lossless 2560 × 1440 capture. The first stream request is a warm-up allocation and is recorded separately rather than counted as steady-state throughput.

Thirty FPS is a streaming target. It does not alter the exact evidence contract or permit synthetic SVG fallback.

## Sources

- `src/lua_server.rs` — `StreamFrame` and `Screenshot` IPC schemas.
- `src/iced_app/screenshot.rs` — exact and reduced native frame construction and encoding.
- `src/render/headless.rs` — persistent offscreen GPU pipeline, target, readback, texture, glyph, and strata reuse.
- `src/iced_app/render.rs` — authoritative live per-strata quad cache.
- [[scaling-coordinates]] — fixed stage and manifest coordinate contract.
