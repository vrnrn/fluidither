# Web reference

No framework and no runtime dependencies. Copy `web/src`, then link only what you use.
Every page needs `tokens.css`. Scripts are ES modules.

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
| Labeled field | `<label class="fd-field">Name <input class="fd-input"><span class="fd-field-hint">Hint</span></label>` | input.css |
| Checkbox | `<label class="fd-choice"><input class="fd-check" type="checkbox"> Label</label>` | check.css |
| Radio | `<input class="fd-radio" type="radio" name="plan">` in a `.fd-choice` | check.css |
| Switch | `<input class="fd-switch" type="checkbox" role="switch">` in a `.fd-choice` | switch.css, check.css for the label |
| Slider | `<fd-slider><input type="range" aria-label="Volume"></fd-slider>` | slider.css + JS |
| Tooltip | `<fd-tooltip text="Save"><button>...</button></fd-tooltip>` | tooltip.css + JS |
| Menu | `<fd-menu><button>Open</button><fd-list><fd-item value="a">A</fd-item></fd-list></fd-menu>` | list.css, menu.js |
| Select | `<fd-select name="f" value="a" aria-label="Fruit"><fd-list>...</fd-list></fd-select>` | list.css, select.css + JS |
| Combobox | `<fd-combobox name="c" aria-label="City"><fd-list>...</fd-list></fd-combobox>` | list.css, combobox.css + JS |
| Dialog | `<dialog class="fd-dialog">`, opened with `showModal()` | dialog.css |
| Drawer | `<dialog class="fd-dialog fd-drawer">`, `data-side="bottom"` for a sheet | dialog.css, misc.css |
| Command menu | `<fd-command aria-label="Commands"><fd-list>...</fd-list></fd-command>`, call `.show()` | list.css, dialog.css, command.css + JS |
| Card | `<article class="fd-card">`, `data-raised` to lift it | card.css |
| Badge | `<span class="fd-badge" data-tone="green">Live</span>` | badge.css |
| Banner | `<div class="fd-banner" data-tone="orange" role="status">` | banner.css |
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
| Text effects | `<fd-scramble>`, `<fd-dissolve>`, `<fd-count value="1204">`. On `<fd-dissolve>`, `hold="0.1 0.7"` keeps the dither and `on-view` waits until it is seen | text-effects.css + JS |
| Calendar | `<fd-calendar value="2026-10-30" min max>` | calendar.css + JS |
| Date picker | `<fd-date-picker name="due" value="2026-10-30" aria-label="Due">` | calendar.css, select.css, date-picker.js |
| Color | `<fd-swatches colors="#c33 #39c" aria-label="Accent">`, `<input class="fd-color" type="color">` | misc.css, swatches.js |
| Chat | `<p class="fd-message" data-from="me">`, `.fd-thinking`, `.fd-composer` | misc.css |
| Copy field | `<fd-copy value="..." label="...">` | misc.css, input.css, button.css, copy.js |
| Small pieces | `.fd-avatar`, `.fd-empty`, `.fd-dots`, `.fd-uptime` | misc.css |
| Steps | `<ol class="fd-steps"><li data-state="done">...</li><li data-state="active">...</li><li>...</li></ol>` | extras.css |
| Question | `<fieldset class="fd-question"><legend>Which?</legend>...</fieldset>` | extras.css |
| Sidebar | `<fd-sidebar label="menu">...</fd-sidebar>` | extras.css, sidebar.js |
| Compare | `<fd-compare aria-label="...">`, around two elements | extras.css, compare.js |
| Marquee | `<fd-marquee aria-label="..."><span>A</span><span>B</span></fd-marquee>` | extras.css, marquee.js |
| File | `.fd-file` holding `.fd-file-type` and `.fd-file-size` | extras.css |
| Glow | `<span class="fd-bloom" data-tone="orange">...</span>` | extras.css |
| Missing page | `.fd-not-found` holding `.fd-not-found-code` | extras.css |

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
- Events: tabs fire `change` with `detail: { value, index }`. Menus and the command menu
  fire `pick` with `detail: { value }`. Select, combobox, calendar and date picker fire
  `change`, and read `.value`.
- Select, combobox and date picker join forms. Give them a `name`.
- After setting a slider's value from code, call `.refresh()` on the `<fd-slider>`.
- In `<fd-dissolve>` on a surface other than the page, set `--fd-dissolve-paper`.
- To save a canvas element as a PNG, call `savePng(element, 'name.png')` from `core/export.js`.
