# Recipes

Web first, and the GPUI equivalent where the pieces differ.

## A settings page

Use `.fd-card` sections holding `.fd-field` rows, with `.fd-switch` for on and off and
`<fd-select>` for choices. Put a solid button last, with a soft one for cancel. Use
`data-density="compact"` on the page if there's a lot of it.

## A dashboard

A row of cards, each with a `.fd-stat` and an `<fd-sparkline>`, above one large
`<fd-chart>` and a `.fd-table`. Keep charts to two or three series, because the shading
already adds texture. Use color for series, and tones for status with `.fd-badge`.

## A hero band

```html
<fd-dither-gradient class="band" angle="90" from="0" to="0.75" speed="0.4"></fd-dither-gradient>
```

```css
.band {
  height: 120px;
  border-radius: 8px;
}
```

Use `<fd-field kind="aurora">` for something livelier. Both pause offscreen and hold still
under reduced motion. Keep text off busy fields.

## A confirm dialog

```html
<dialog class="fd-dialog" id="confirm" aria-labelledby="title">
  <h3 class="fd-dialog-title" id="title">Delete this report?</h3>
  <p class="fd-dialog-text">This can't be undone.</p>
  <form class="fd-dialog-actions" method="dialog">
    <button class="fd-button" data-look="soft" value="cancel">Cancel</button>
    <button class="fd-button" value="delete">Delete</button>
  </form>
</dialog>
```

Open it with `showModal()`, and read `returnValue` on `close`. On GPUI, keep an `open` bool
in your view and clear it in `on_close`.

## A command palette

Add `<fd-command>` once, and call `.show()` from your own shortcut. On GPUI, create a
`CommandMenu` view, add it as a child, and call `show`.

## A chat screen

A column of `.fd-message`, with `data-from="me"` on the user's, then `.fd-thinking` while
waiting and `.fd-composer` at the bottom. On GPUI, the same column is `Message`, `Thinking`
and a `Composer` view.

## Dither a new shape of your own

Fill a tone array, then threshold it. A tone of 0 is paper and 1 is ink.

```js
import { ditherTones, thresholdMap } from './core/dither.js';

const map = await thresholdMap('bayer4');
const tones = new Float64Array(width * height).map((_, index) => (index % width) / width);
const cells = new Uint8Array(width * height);
ditherTones(tones, width, height, map, Uint8Array.of(0, 1), cells);
```

```rust
use fluidither::dither::{dither_tones, Matrix};

let tones: Vec<f64> = (0..width * height).map(|index| (index % width) as f64 / width as f64).collect();
let mut cells = vec![0u8; width * height];
dither_tones(&tones, width, height, Matrix::Bayer4.map(), &[0u8, 1], &mut cells);
```

Matrices: Bayer looks crisp and patterned, `blue-noise` looks organic, and `halftone` looks
like print. Cells of 2 to 3px read as texture. Larger cells read as pixel art.

## Add a dithered hover to something new

```css
.thing:hover {
  --_dots: color-mix(in srgb, currentColor var(--fd-dither-hover-ink), transparent);
  background-image: repeating-conic-gradient(var(--_dots) 0 25%, transparent 0 50%);
  background-size: calc(var(--fd-dither-hover-cell) * 2) calc(var(--fd-dither-hover-cell) * 2);
}
```

On GPUI, use `paint::checker(fade(colors.ink, DITHER_HOVER_INK), DITHER_HOVER_CELL, radius)`
as a child. Keep the area small, because each cell is a quad.

## Move something on a spring

```js
import { Glide, springs } from './core/spring.js';

const glide = new Glide({ x: 0 }, springs.glide, ({ x }) => element.style.setProperty('--x', `${x}px`));
glide.to({ x: 120 });
```

It stops by itself and honors reduced motion. On GPUI, hold a `Spring`, set `target`, call
`step(dt)` in render, and call `window.request_animation_frame()` until it reports landed.
