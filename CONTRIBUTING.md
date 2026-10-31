# Contributing

This has been run with Node 26 and Rust 1.98.1. The Chromium tests need a browser, and the
GPUI work needs macOS, the only platform it has been built on.

```bash
npm install
```

```bash
npx playwright-core install chromium
```

## Run the tests

Unit tests for the web, then the Rust tests:

```bash
npm test
```

Components in real Chromium, under the strict CSP, with axe in both themes:

```bash
npm run test:browser
```

Label centering on GPUI, in rendered pixels. This opens a real window, because GPUI's test
platform draws nothing:

```bash
npm run check:centering
```

## Change a token

Edit `tokens/tokens.json`, never the generated files. Then:

```bash
npm run tokens
```

## Change an engine

Change the JavaScript and the Rust together, and regenerate the fixtures. The Rust parity
tests fail if the two disagree.

```bash
npm run fixtures
```

`tokens/blue-noise-64.bin` comes from `npm run blue-noise`.

## Look at it

```bash
npm run demo
```

```bash
cargo run --release --manifest-path gpui/Cargo.toml --example gallery
```

The `blocks`, `visuals` and `extras` examples show the rest. To remake the README images:

```bash
npm run images
```

## Keep to these

- No runtime dependencies on the web, and no inline styles.
- Respect reduced motion in anything that moves.
- Say what you tested. If you couldn't test something, add it to
  [docs/support.md](docs/support.md).

The [skill's extending guide](skills/fluidither/references/extending.md) walks through adding
a component or a field to both targets.
