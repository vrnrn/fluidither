# fluidither

![The word fluidither fading out in dots beside a dithered globe](https://raw.githubusercontent.com/vrnrn/fluidither/main/docs/images/hero-light.png)

Dithered, spring-driven UI components in plain HTML, CSS and JavaScript. There is no framework
and there are no runtime dependencies, and it works under a strict Content Security Policy.
Each component loads on its own: tokens plus an animated gradient come to 4.2 KB gzipped.

![Real fluidither components: a chart, a form, a calendar, tabs, a globe and a chat](https://raw.githubusercontent.com/vrnrn/fluidither/main/docs/images/showcase-light.png)

## Install

There is no build step. Copy in the components you want, along with the files they need:

```bash
npx fluidither add tabs button
```

It uses `public/fluidither` if the project has a `public` folder, and `--dir` sets another
place. It also prints the tags to paste, like these:

```html
<link rel="stylesheet" href="fluidither/tokens.css" />
<link rel="stylesheet" href="fluidither/components/button.css" />
<link rel="stylesheet" href="fluidither/components/tabs.css" />
<script type="module" src="fluidither/components/tabs.js"></script>

<fd-tabs value="week">
  <fd-tab-list aria-label="Date range">
    <fd-tab value="day">Day</fd-tab>
    <fd-tab value="week">Week</fd-tab>
  </fd-tab-list>
</fd-tabs>

<button class="fd-button" type="button">Save</button>
```

`npx fluidither list` shows every name, plain `npx fluidither` copies the whole library, and
`npx fluidither skill` installs an agent skill for building with it.

With a bundler, run `npm install fluidither` and import the pieces by name:

```js
import 'fluidither/tokens.css';
import 'fluidither/components/tabs.css';
import 'fluidither/components/tabs.js';
```

## Typography

Run `npx fluidither add typography` and link `fluidither/components/typography.css`, or import
`fluidither/components/typography.css` with a bundler. This loads the self-hosted Obel Mono
variable family for code, keyboard hints and numeric readouts. Use `.fd-mono` or
`var(--fd-font-mono)` for your own text; keep prose in `var(--fd-font-sans)`.

Add `font-src 'self'` to your CSP. Roman is about 140 KiB, italic another 154 KiB when used.
Without the stylesheet, the mono token uses local fallback fonts. Override it to use another
family. The fonts, license and attribution are copied together.

## Learn more

- [Full README, with a complete example](https://github.com/vrnrn/fluidither#readme)
- [Every component and attribute](https://github.com/vrnrn/fluidither/blob/main/skills/fluidither/references/web.md)
- [What has and hasn't been tested](https://github.com/vrnrn/fluidither/blob/main/docs/support.md)

Tested in Chromium. Firefox and Safari have not been run.

## License

MIT for the library. Obel Mono by Typographic Systems is bundled under SIL OFL 1.1;
see `src/fonts/obel-mono/OFL.txt` and `AUTHORS.txt`.
