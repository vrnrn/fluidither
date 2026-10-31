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

## Run the site

The home page and docs live in `site/`:

```bash
npm run site
```

Their tests run in Chromium too:

```bash
npm run test:site
```

`npm run site:build` puts the site and the library in `site-dist/`. Once CI passes on `main`,
`deploy.yml` builds that folder and publishes it, so a merge updates the site. To publish by
hand, run `npx wrangler deploy` after building.

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

## Release

Pushing a version tag publishes both packages. The release workflow checks that the tag
matches every version number, runs the tests, then publishes the crate and the npm package.

```bash
npm run version:set 0.2.0
```

```bash
git commit -am "Release 0.2.0" && git tag v0.2.0 && git push origin main v0.2.0
```

No token is stored in the repository. Each registry trusts the workflow by name, which is set
once under the package's settings:

- **crates.io:** Settings, Trusted Publishing. Owner `vrnrn`, repository `fluidither`, workflow
  `release.yml`.
- **npm:** Settings, Trusted Publisher, GitHub Actions. The same three values, with
  `npm publish` allowed. A new npm connection expires unless it publishes within 2 days, so set
  it up just before a release.

## Keep to these

- No runtime dependencies on the web, and no inline styles.
- Respect reduced motion in anything that moves.
- Say what you tested. If you couldn't test something, add it to
  [docs/support.md](docs/support.md).

The [skill's extending guide](skills/fluidither/references/extending.md) walks through adding
a component or a field to both targets.
