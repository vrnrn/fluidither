# Architecture

## One source, two targets

```text
tokens/tokens.json ──► npm run tokens ──┬─► web/src/tokens.css
tokens/blue-noise-64.bin                ├─► web/src/core/tokens.js
tokens/world.json                       ├─► web/src/core/blue-noise.js, world-data.js
                                        └─► gpui/src/generated.rs, blue-noise-64.bin
```

Colors, sizes, springs, pixel glyphs and dither tables live in `tokens/`. The generated
files are checked in, so people can copy `web/src` or depend on the crate without running
anything. Never edit them by hand.

## The engines are written twice

Everything that decides which cells get ink exists in JavaScript and in Rust: gradients,
fields, charts, sparklines, the world map, scramble text, dates and springs.

Keeping these in step is the main design constraint, and it works like this:

1. Field code uses `wave`, a plain arithmetic curve, and an integer-hash noise, not `sin` per
   cell, so both languages produce the same bits.
2. `npm run fixtures` runs the JavaScript and writes hashes of the results to `fixtures/`.
3. `gpui/tests/parity.rs` reads those files and fails if Rust produces a different cell.

Each engine file names its twin at the top.

## How a dither is drawn

A tone from 0 to 1 is worked out for every cell, then compared with a threshold from a
matrix: Bayer, a clustered halftone screen, or blue noise from a void-and-cluster run. A
cell above its threshold gets ink, so the output is a flat grid of ink and paper.

- **Web:** cells go into a canvas as a mask, and `source-in` compositing colors them from
  the element's CSS `color`.
- **GPUI:** cells become an image at device resolution, because that stays cheap at any
  size. A few hundred merged quads are used for small hover areas instead.

[Performance](performance.md) has the numbers behind this choice.

## Web

- **Real elements first.** A button is `<button class="fd-button">`, and dialogs and
  accordions are `<dialog>` and `<details>`, so the keyboard and accessibility come from the
  browser. Custom elements exist only where the platform has nothing, and they render into
  the light DOM.
- **Strict CSP.** The library works under `default-src 'none'`, `script-src 'self'`,
  `style-src 'self'` and `img-src 'self'`: no inline styles, no `<style>` tags and no `data:`
  images. Styles set from JavaScript go through the CSSOM. The demo server sends this policy,
  so a violation fails the browser tests.
- **Load only what you use.** Each component is its own CSS file and, when it needs one, its
  own module.

## GPUI

Components are `RenderOnce` elements that keep state with `window.use_keyed_state`. Text
fields, the combobox, the command menu and the composer are views, because they need a
long-lived object for system text input or subscriptions.

Dithers are computed on the CPU, and springs are analytic, so a target can change mid-flight
without a jump. Slow changes step on a timer, because any redraw repaints the whole window.

## Repository layout

| Path | Holds |
| --- | --- |
| `tokens/` | The shared design source |
| `web/src/` | The web library: `tokens.css`, `components/`, `core/` |
| `web/bin/` | The `npx fluidither` command |
| `web/demo/` | The demo and showcase pages, and a server with the strict CSP |
| `web/tests/` | Unit tests, and Chromium tests in `browser/` |
| `gpui/src/` | The Rust crate |
| `gpui/examples/` | Galleries, the centering check and benchmarks |
| `gpui/tests/` | Parity with JavaScript, and tests on GPUI's test platform |
| `fixtures/` | Results from the JavaScript engines that Rust is held to |
| `skills/fluidither/` | The agent skill |
| `scripts/` | Generators, measurements and the README images |
| `docs/` | These pages and their images |
