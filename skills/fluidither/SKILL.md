---
name: fluidither
description: Build interfaces with fluidither, a dithered, spring-driven UI library for the plain web (HTML, CSS, JS, no framework) and GPUI (Rust). Use whenever someone wants fluidither components, a dithered or print-like interface, ordered dithering, halftone or blue noise effects, dithered charts or backgrounds, spring-based hover and selection motion, or UI that must work under a strict Content Security Policy with no framework. Also use when porting a screen between the web and GPUI, because the two targets share tokens and produce the same dither cells.
---

# fluidither

Two targets, one set of tokens. The web target uses real elements with a class wherever
it can, and small custom elements elsewhere. The GPUI target is a Rust crate.

Before writing code, open the reference for your target:

- Web: [references/web.md](references/web.md)
- GPUI: [references/gpui.md](references/gpui.md)
- Screens and patterns: [references/recipes.md](references/recipes.md)
- Adding to the library itself: [references/extending.md](references/extending.md)

## Get the library

Check whether the project already has it before fetching it.

**Web:** with no bundler, copy what you need into the folder you serve. This also prints the
tags to paste into the page:

```bash
npx fluidither add tabs button gradient --dir public/fluidither
```

`npx fluidither list` shows the names. Rerun it to add more or to update. With a bundler, run
`npm install fluidither` and import by name, such as `fluidither/tokens.css` and
`fluidither/components/tabs.js`. Modules need a web server, so pages won't work from `file://`.

**GPUI:** add the crate, with `gpui` at the same version.

```toml
[dependencies]
fluidither = "0.1"
gpui = { version = "=0.2.2", features = ["runtime_shaders"] }
```

Call `fluidither::init(cx)` once at startup.

## The design in one minute

- **Ink and paper.** Warm neutrals, with color used to carry meaning.
- **Dither instead of tint.** Quiet hover states, dialog screens, the unfilled part of a
  bar and the shading in charts are all a dither. Never put a dither behind body text.
- **Springs instead of durations.** `quick` for hover, `glide` for movement, `settle` for
  arrivals. Springs keep their speed when the target changes.
- **Pixel marks.** Ticks, dashes and arrows are small cell drawings from the tokens.
- **Two densities.** `regular` and `compact`.

## Rules that keep work correct

1. **Use tokens.** Web: `var(--fd-ink)`, `var(--fd-gap)`. GPUI: `Theme::get(cx).colors()`
   and `Density::tokens()`. Never hard-code a color or size that a token covers.
2. **Change tokens in one place.** In a clone of the library, edit `tokens/tokens.json` and
   run `npm run tokens`. Never edit `tokens.css`, `core/tokens.js` or `generated.rs`. In a
   project that copied the source, override `--fd-*` custom properties in its own CSS.
3. **Prefer the real element on the web.** `.fd-button` on `<button>`, `.fd-input` on
   `<input>`, `.fd-accordion` on `<details>`, `.fd-dialog` on `<dialog>`. They bring the
   keyboard and accessibility with them.
4. **Stay inside the strict CSP.** No inline `style` attributes, no `<style>` tags and no
   `data:` images. Setting styles from JS through the CSSOM is fine. Dither CSS masks built
   from data URLs are blocked, so use canvas, or a `repeating-conic-gradient`.
5. **Respect reduced motion.** Components already do. New motion must too.
6. **Keep the targets in step.** The engines in `web/src/core` and `gpui/src` must match
   exactly. After changing one, change the other, run `npm run fixtures`, and run both test
   suites. Avoid `sin` and `cos` per cell. Use `wave`, which is plain arithmetic.
7. **Don't animate with frames on GPUI unless something is really moving.** Any redraw
   repaints the window, so use a slow timer for slow changes.
8. **Label every picture.** A chart needs `aria-label`. Only `<fd-chart>` carries its
   numbers for screen readers.

## What to tell people

- GPUI 0.2.2 has no accessibility API, so screen readers can't see GPUI components. Say so
  whenever you build something on that target that must be accessible.
- Only Chromium and macOS on Apple Silicon have been tested. Firefox, Safari, Linux and
  Windows have not, so say so if a project targets them.
- The world map is a hand-drawn sketch, not geographic data.

## Check your work

In a clone of the library:

```bash
npm test
```

```bash
npm run test:browser
```

```bash
npm run check:centering
```

The last one opens a real window, because GPUI's test platform draws nothing. See how it
looks with `npm run demo` for the web, and the `gallery`, `blocks`, `visuals` and `extras`
examples for GPUI.

In a project using the library, render the page under its real CSP and check the browser
console for violations.
