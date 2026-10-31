# Performance

Measured on an Apple M1 Pro with a 120 Hz Retina display, on macOS, Rust 1.98.1, Node 26 and
headless Chromium 156. Your numbers will differ, so each section says how to reproduce them.

## Bundle size on the web

```bash
npm run size
```

Pages load only what they use, so the sizes that matter are the ones for a single piece.

| What | Gzipped |
| --- | --- |
| `tokens.css`, once per page | 1.2 KB |
| Most CSS-only components | under 1 KB each |
| Tokens plus an animated gradient | 4.2 KB |
| The overlays, such as Select and the command menu | 3 to 5 KB each |
| Every file, compressed separately | about 65 KB |

The last figure counts shared code more than once. There are no runtime dependencies.

## Dither frame cost on GPUI

```bash
cargo run --release --manifest-path gpui/Cargo.toml --example frame_bench
```

A real window, repainting every frame. Compute and paint are main-thread CPU.

| Size | Cells | Path | Compute p50 | Paint p50 | Frame interval p50 / p99 |
| --- | --- | --- | --- | --- | --- |
| 848x120 @3px | 11,320 | image | 0.28 ms | 0.30 ms | 8.3 / 10.8 ms |
| 848x120 @3px | 11,320 | quads | 0.17 ms | 2.48 ms | 8.3 / 9.6 ms |
| 1200x300 @3px | 40,000 | image | 0.80 ms | 0.93 ms | 8.3 / 11.7 ms |
| 1200x300 @3px | 40,000 | quads | 0.53 ms | 13.45 ms | 14.4 / 15.0 ms |
| 1200x700 @3px | 93,600 | image | 1.83 ms | 1.83 ms | 8.3 / 9.4 ms |
| 1200x700 @3px | 93,600 | quads | 1.22 ms | 40.38 ms | 42.4 / 61.5 ms |
| 1200x700 @1px | 840,000 | image | 11.66 ms | 1.86 ms | 13.7 / 14.1 ms |
| 1200x700 @1px | 840,000 | quads | 11.05 ms | 897 ms | about 1 frame per second |

- The image path is the default, because quad cost grows faster than the quad count. Quads
  stay in use only for small hover areas, at a few hundred.
- Memory stayed flat within each image run, so frames don't leak in the atlas.
- GPU time isn't measured directly. It would only show up as a longer frame interval.

The `gallery` example, with two animated gradients and a busy button, uses about 10% of a
core. The `extras` example, with a turning globe and a marquee, uses about the same. The
first launch opens a window in about 640 ms and later launches in about 180 ms, because
shaders compile at start.

## Dither frame cost on the web

```bash
npm run cost
```

Fill, dither and `putImageData` in headless Chromium.

| Size | Cells | p50 | p95 |
| --- | --- | --- | --- |
| 848x120 @3px | 11,320 | 0.40 ms | 0.50 ms |
| 1200x300 @3px | 40,000 | 1.30 ms | 1.50 ms |
| 1200x700 @3px | 93,600 | 3.10 ms | 3.30 ms |
| 1200x700 @1px | 840,000 | 27.60 ms | 28.10 ms |

The demo's main thread runs at about 8% of a core. That is headless and leaves out
compositing, so it can't be compared with the GPUI figure.
