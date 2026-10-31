# Extending fluidither

These steps assume a clone of the repository, because they use its generators and tests.

## Change a token

1. Edit `tokens/tokens.json`.
2. Run `npm run tokens`. This rewrites the CSS, the JS tables and `gpui/src/generated.rs`.
3. Run `npm test`.

In a project that only copied `web/src`, don't edit the generated files. Override the
`--fd-*` custom properties in your own stylesheet instead.

## Add a dither field to both targets

1. Add the name to `FIELDS` and a case to `toneAt` in `web/src/core/fields.js`.
2. Add the variant to `Field` and an arm to `tone_at` in `gpui/src/fields.rs`, with the same
   arithmetic line for line. Use `wave` and `grain`, not `sin` or `cos`.
3. Add the name to the lookup in `gpui/tests/parity.rs`.
4. Add a `<figure>` with an `<fd-field>` of that kind to `web/demo/index.html`, and a tile to
   `gpui/examples/visuals.rs`.
5. Run `npm run fixtures`, then `npm test`. The parity test compares every cell at two
   moments.
6. Run `npm run test:browser`. It takes the names from `FIELDS` and looks for each one in the
   demo.
7. Add the name to the field kinds in `references/web.md`.

## Add a web component

- Use a real element with a class if one exists. Add a CSS file to `web/src/components`.
- If it needs behavior, add a module beside it with a custom element named with the `fd-`
  prefix. Render into the light DOM.
- Stay inside the strict CSP: no inline `style` attributes and no `data:` images. Setting
  styles from JavaScript through the CSSOM is fine.
- Use tokens for every color and size, and honor `prefers-reduced-motion`.
- Add it to `COMPONENTS` in `web/bin/fluidither.mjs`, so `npx fluidither add` can copy it. A
  test fails if a file in `web/src/components` belongs to no component.
- Add it to `web/demo/index.html`. The demo server sends the strict CSP, so a violation shows
  up there.
- Add a test to `web/tests/browser`, with axe if it has content people read.

## Add a GPUI element

- Make a struct with `#[derive(IntoElement)]` and `impl RenderOnce`, and give it an `id` if
  it keeps state.
- Keep state with `window.use_keyed_state`. Make it a view only if it needs system text input
  or subscriptions.
- Read colors from `Theme::get(cx).colors()`, and check `reduced_motion` before animating.
- Export it from `gpui/src/lib.rs`, and add it to an example so it can be seen.
- Add a test to `gpui/tests`. GPUI's test platform has real layout, focus and input, but it
  draws nothing, so check the look in a real window too.

## Before you finish

```bash
npm test
```

```bash
npm run test:browser
```

```bash
npm run check:centering
```

Add anything you couldn't test to `docs/support.md`.
