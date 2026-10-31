# fluidither

![The word fluidither fading out in dots beside a dithered globe](https://raw.githubusercontent.com/vrnrn/fluidither/main/docs/images/hero-light.png)

Dithered, spring-driven UI components in plain HTML, CSS and JavaScript. There is no framework
and there are no runtime dependencies, and it works under a strict Content Security Policy.
Each component loads on its own: tokens plus an animated gradient come to 4.2 KB gzipped.

![Real fluidither components: a chart, a form, a calendar, tabs, a globe and a chat](https://raw.githubusercontent.com/vrnrn/fluidither/main/docs/images/showcase-light.png)

## Install

```bash
npm install fluidither
```

There is no build step. Copy the files into the folder you serve:

```bash
mkdir -p public/fluidither && cp -R node_modules/fluidither/src/. public/fluidither/
```

Then link what you use:

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

With a bundler, import the pieces by name:

```js
import 'fluidither/tokens.css';
import 'fluidither/components/tabs.css';
import 'fluidither/components/tabs.js';
```

## Learn more

- [Full README, with a complete example](https://github.com/vrnrn/fluidither#readme)
- [Every component and attribute](https://github.com/vrnrn/fluidither/blob/main/skills/fluidither/references/web.md)
- [What has and hasn't been tested](https://github.com/vrnrn/fluidither/blob/main/docs/support.md)

Tested in Chromium. Firefox and Safari have not been run.

## License

MIT
