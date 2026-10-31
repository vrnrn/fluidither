# Web reference

No framework and no runtime dependencies. Copy parts in with `npx fluidither add <names>`, then
link only what you use.
Every page needs `tokens.css`. Scripts are ES modules.

`tokens.css` only defines values, so style the page yourself:

```css
body {
  background: var(--fd-paper);
  color: var(--fd-ink);
  font-family: var(--fd-font-sans);
}
```

```html
<link rel="stylesheet" href="fluidither/tokens.css" />
<link rel="stylesheet" href="fluidither/components/button.css" />
```

Set `data-theme="light"` or `"dark"` on `<html>` to force a theme. Otherwise it follows the
system. `data-density="compact"` works on any element and applies to what is inside it.

## Components

CSS means no JavaScript is needed. "+ JS" means load the script of that name as well.

| Need | Write | Files |
| --- | --- | --- |
| Button | `<button class="fd-button" data-look="soft">` | button.css |
| Busy button | `aria-busy="true" disabled` on the button | button.css |
| Tabs | `<fd-tabs value="a"><fd-tab-list aria-label="..."><fd-tab value="a">A</fd-tab></fd-tab-list><fd-tab-panel value="a">...</fd-tab-panel></fd-tabs>`, `look="underline"` for a lighter form | tabs.css + JS |
| Text field | `<input class="fd-input">`, or a `<textarea>` | input.css |
| Input group | `<div class="fd-input-group"><label class="fd-input-row"><span>Name</span><input class="fd-input"></label></div>` | input.css |
| Labeled field | `<label class="fd-field">Name <input class="fd-input"><span class="fd-field-hint">Hint</span></label>`. For `<fd-select>` and `<fd-combobox>`, use a `<div class="fd-field">`, because a label's click doesn't reach them | input.css |
| Checkbox | `<label class="fd-choice"><input class="fd-check" type="checkbox"> Label</label>` | check.css |
| Check group | `<fieldset class="fd-check-group">` holding `.fd-choice` rows. Rows checked side by side join | check.css |
| Radio | `<input class="fd-radio" type="radio" name="plan">` in a `.fd-choice` | check.css |
| Switch | `<input class="fd-switch" type="checkbox" role="switch">` in a `.fd-choice` | switch.css, check.css for the label |
| Slider | `<fd-slider><input type="range" aria-label="Volume"></fd-slider>` | slider.css + JS |
| Tooltip | `<fd-tooltip text="Save"><button>...</button></fd-tooltip>` | tooltip.css + JS |
| Menu | `<fd-menu><button>Open</button><fd-list><fd-item value="a">A</fd-item></fd-list></fd-menu>` | list.css, menu.js |
| Select | `<fd-select name="f" value="a" aria-label="Fruit"><fd-list><fd-item value="a">A</fd-item></fd-list></fd-select>` | list.css, select.css + JS |
| Combobox | `<fd-combobox name="c" aria-label="City"><fd-list><fd-item value="a">A</fd-item></fd-list></fd-combobox>` | list.css, combobox.css + JS |
| Dialog | `<dialog class="fd-dialog">`, opened with `showModal()` | dialog.css |
| Drawer | `<dialog class="fd-dialog fd-drawer">`, `data-side="bottom"` for a sheet | dialog.css, misc.css |
| Command menu | `<fd-command aria-label="Commands"><fd-list>...</fd-list></fd-command>`, call `.show()` | list.css, dialog.css, command.css + JS |
| Card | `<article class="fd-card">` holding `.fd-card-title` and `.fd-card-text`, `data-raised` to lift it | card.css |
| Badge | `<span class="fd-badge" data-tone="green">Live</span>` | badge.css |
| Banner | `<div class="fd-banner" data-tone="orange" role="status">`, with `<fd-dismiss>` inside to let it close | banner.css, banner.js for `<fd-dismiss>`, button.css |
| Key | `<kbd class="fd-kbd">Tab</kbd>` | misc.css |
| Accordion | `<details class="fd-accordion" name="group"><summary>Title</summary>...</details>` | accordion.css |
| Table | `<table class="fd-table">`, `data-number` on numeric cells | table.css |
| Navigation | `<fd-nav aria-label="Sections"><a href="#a" aria-current="page">A</a></fd-nav>` | nav.css + JS |
| Scroll area | `<div class="fd-scroll" tabindex="0" role="region" aria-label="...">` | scroll.css |
| Loader | `<span class="fd-loader" role="status" aria-label="Loading"></span>` | feedback.css |
| Skeleton | `<span class="fd-skeleton" aria-hidden="true"></span>` | feedback.css |
| Progress | `<progress class="fd-progress" value="62" max="100" aria-label="Upload">` | feedback.css |
| Divider | `<hr class="fd-divider">` | feedback.css |
| Stat | `.fd-stat` holding `.fd-stat-label`, `.fd-stat-value`, `.fd-stat-change` | feedback.css |
| Sparkline | `<fd-sparkline values="3 5 2" aria-label="...">` | sparkline.css + JS |
| Chart | `<fd-chart kind="bar" series="3 7; 5 1" names="A; B" labels="Mon Tue" aria-label="...">` | chart.css, field.css + JS |
| Gradient | `<fd-dither-gradient angle="90" from="0" to="0.75" speed="0.4">` | dither-gradient.css + JS |
| Field | `<fd-field kind="plasma" speed="1">` | field.css + JS |
| Map and globe | `<fd-world kind="map" aria-label="...">`, `kind="globe"` | field.css, extras.css, world.js |
| Image | `<fd-dither-image src="a.jpg" alt="..." matrix="halftone">`, transparent parts stay paper, so it also makes a dithered logo | field.css, dither-image.js |
| Text effects | `<fd-scramble>`, `<fd-dissolve>`, `<fd-count value="1204">`, `<fd-shimmer words="Thinking; Reading">`. On `<fd-dissolve>`, `hold="0.1 0.7"` keeps the dither and `on-view` waits until it is seen | text-effects.css + JS |
| Gliding hover | `<fd-hover nearest axis="y">` around any list. `items="tbody tr"` picks what counts | hover.css + JS |
| Reveal | `<fd-reveal subtle>` around what should rise in order. `.run()` plays it again | reveal.css + JS |
| Calendar | `<fd-calendar value="2026-10-30" min max>` | calendar.css + JS |
| Date picker | `<fd-date-picker name="due" value="2026-10-30" aria-label="Due">` | calendar.css, select.css, date-picker.js |
| Color | `<fd-swatches colors="#c33 #39c" aria-label="Accent">`, `<input class="fd-color" type="color">` | misc.css, swatches.js |
| Chat | `<p class="fd-message" data-from="me">`, `.fd-thinking`, `.fd-composer` | misc.css |
| Suggested reply | `<fd-composer suggestion="Make it shorter">` around a `.fd-composer` form. Tab fills, Enter sends, and `send` carries `detail: { text }` | misc.css, input.css, button.css, composer.js |
| Color picker | `<fd-color-picker value="#fc9e47" aria-label="Accent">`, fires `change`, reads `.value` | misc.css, input.css, color-picker.js |
| Copy field | `<fd-copy value="..." label="...">` | misc.css, input.css, button.css, copy.js |
| Small pieces | `.fd-avatar`, `.fd-empty`, `.fd-dots`, `.fd-uptime` | misc.css |
| Steps | `<ol class="fd-steps"><li data-state="done">...</li><li data-state="active">...</li><li>...</li></ol>` | extras.css |
| Question | `<fieldset class="fd-question"><legend>Which?</legend>...</fieldset>` | extras.css |
| Question flow | `<fd-questions done="Thanks.">` around `.fd-question` fieldsets of named radios, or checkboxes for several answers. A text input in an option makes its text the answer. Fires `answer` and `done` | extras.css, check.css, button.css, questions.js |
| Thinking steps | `<details class="fd-thinking-steps" open><summary>Research</summary><ol class="fd-steps">...</ol></details>`, `.fd-sources` for badges | extras.css, badge.css |
| Sidebar | `<fd-sidebar label="menu">...</fd-sidebar>` | extras.css, sidebar.js |
| Compare | `<fd-compare aria-label="...">`, around two elements | extras.css, compare.js |
| Marquee | `<fd-marquee aria-label="..."><span>A</span><span>B</span></fd-marquee>` | extras.css, marquee.js |
| File | `.fd-file` holding `.fd-file-type` and `.fd-file-size` | extras.css |
| Glow | `<span class="fd-bloom" data-tone="orange">...</span>` | extras.css |
| Missing page | `.fd-not-found` holding `.fd-not-found-code` | extras.css |

## Tokens

| Use | Tokens |
| --- | --- |
| Page | `--fd-paper`, `--fd-ink`, `--fd-ink-soft` |
| Surfaces | `--fd-surface-0` to `--fd-surface-3`, each with `--fd-shadow-0` to `--fd-shadow-3` |
| Lines and fills | `--fd-line`, `--fd-edge`, `--fd-focus`, `--fd-soft`, `--fd-wash`, `--fd-screen` |
| Color | `--fd-red`, `--fd-orange`, `--fd-green`, `--fd-blue`, `--fd-violet`, `--fd-pink` |
| Text | `--fd-font-sans`, `--fd-font-size`, `--fd-line-height`, `--fd-weight-regular`, `--fd-weight-strong` |
| Spacing | `--fd-gap` is 6px, for inside a control or row. `--fd-pad` is 14px, for cards and page layout |
| Sizes | `--fd-control`, `--fd-segment`, `--fd-icon`, `--fd-inset` |
| Corners | `--fd-radius-control`, `--fd-radius-pill` |
| Motion | `--fd-spring-quick`, `--fd-spring-glide`, `--fd-spring-settle`, and exits a step faster: `--fd-spring-quick-out`, `--fd-spring-glide-out`, `--fd-spring-settle-out` |

## Details

- Button looks: `solid` (default), `soft`, `outline`, `ghost`. Icons: put an `<svg>` inside.
- Tones: `red`, `orange`, `green`, `blue`, `violet`, `pink`, or none for neutral.
- Matrices: `bayer2`, `bayer4`, `bayer8`, `halftone`, `blue-noise`.
- Chart kinds: `bar`, `area`, `line`, `donut`, `radar`, `heatmap`, `gauge`. A gauge's series
  is `value max`. A heatmap's series are rows. A donut takes its slice names from `labels`.
- Field kinds: `noise`, `waves`, `plasma`, `ripples`, `tunnel`, `metaballs`, `orb`, `stars`,
  `aurora`, `contours`, `beams`, `eclipse`, `horizon`, `mesh`, `grid`, `ridges`, `silk`,
  `smoke`.
- Gradient modes: `linear`, `radial`, `conic`, with `angle`, `center="0.5 0.5"`, `radius`,
  `from`, `to` and `warp`.
- A canvas element's ink is its CSS `color`, and the paper is its `background-color`. Give
  gradients, fields and images a size in CSS.
- Set a chart's height with `--fd-chart-height`, not `height`. The default is 160px.
- Chart `labels` split on spaces. Separate them with `;` if they hold spaces, as in
  `labels="New York; Oslo"`.
- A canvas keeps its width while it lays out, so give a chart's grid track `minmax(0, 1fr)`, or
  `min-width: 0` on a flex or grid child, or it won't shrink.
- Events: tabs fire `change` with `detail: { value, index }`. Menus and the command menu
  fire `pick` with `detail: { value }`. Select, combobox, calendar and date picker fire
  `change`, and read `.value`.
- Select, combobox and date picker join forms. Give them a `name`.
- After setting a slider's value from code, call `.refresh()` on the `<fd-slider>`.
- In `<fd-dissolve>` on a surface other than the page, set `--fd-dissolve-paper`.
- To save a canvas element as a PNG, call `savePng(element, 'name.png')` from `core/export.js`.
