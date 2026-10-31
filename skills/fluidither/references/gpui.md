# GPUI reference

A Rust crate on `gpui` 0.2.2. The crate turns on `runtime_shaders`, so building needs no
Metal toolchain.

```rust
use fluidither::{Button, Look, Tabs};

fluidither::init(cx); // once at startup: sets the theme and makes Tab move focus

Button::new("save", "Save").look(Look::Soft).on_click(|_, _, _| {});
```

Use `Theme::get(cx).colors()` for colors. Set `cx.global_mut::<Theme>().mode` to force a
theme, then call `window.refresh()`.

## Elements

Add these directly as children. They keep their own state, keyed by id, unless you pass the
value in, as with `.checked()`, `.on()`, `.selected()` or `.value()`.

| Need | Write |
| --- | --- |
| Button | `Button::new(id, label)`, `.look()`, `.density()`, `.icon(path)`, `.busy()`, `.disabled()`, `.tip()`, `.on_click()` |
| Tabs | `Tabs::new(id, ["A", "B"])`, `.selected()`, `.initial()`, `.flat_peek()`, `.underline()`, `.on_change()` |
| Checkbox | `Checkbox::new(id).label("Tick")`, `.checked(Checked::Mixed)`, `.initial()`, `.on_change()` |
| Radio group | `RadioGroup::new(id, ["A", "B"])`, `.disable(1)`, `.on_change()` |
| Switch | `Switch::new(id).label("Sync")`, `.on()`, `.initial()`, `.on_change()` |
| Slider | `Slider::new(id).range(0., 100.).step(5.)`, `.value()`, `.initial()`, `.on_change()` |
| Menu | `Menu::new(id, "Actions", rows).on_pick()` |
| Select | `Select::new(id, rows)`, `.placeholder()`, `.selected()`, `.initial()`, `.on_change()` |
| Dialog | `Dialog::new(id, open).title()`, `.text()`, `.body()`, `.action()`, `.on_close()`, `.drawer()`, `.sheet()` |
| Card | `Card::new().title()`, `.text()`, `.raised()`, `.child()` |
| Badge | `Badge::new("Live").tone(Tone::Green)` |
| Banner | `Banner::new("Text").tone(Tone::Orange).action(button)` |
| Accordion | `Accordion::new(id).section("Title", body)`, `.initial(0)`, `.multiple()` |
| Table | `Table::new(id).column("Name").number_column("Amount").row(["A", "1"])` |
| Nav | `Nav::new(id, rows)`, `.selected()`, `.initial()`, `.on_change()` |
| Scroll area | `ScrollArea::new(id).h(px(70.)).child(...)` |
| Loader | `Loader::new(id)` |
| Skeleton | `Skeleton::new(id).height(18.)` |
| Progress | `Progress::new(0.62)` |
| Divider | `Divider` |
| Stat | `Stat::new("Revenue", "$48,210").change("Up 12%")` |
| Sparkline | `Sparkline::new(id, [4., 6., 5.])` |
| Chart | `Chart::new(id, ChartKind::Bar, series).names(["A", "B"]).height(120.)` |
| Gradient | `DitherGradient::new(id, Gradient::default()).speed(0.4).h(px(120.)).w_full()` |
| Field | `DitherGradient::field(id, Field::Plasma).h(px(70.)).w_full()` |
| Map and globe | `DitherGradient::map(id).w(px(440.)).h(px(220.))`, `DitherGradient::globe(id)` |
| Image | `DitherImage::new(id, path).matrix(Matrix::Halftone).size(240., 120.)`, transparent parts stay paper, so it also makes a dithered logo |
| Text effects | `Scramble::new(id, text)`, `Dissolve::new(id, content)`, `.hold(0.1, 0.7)`, `.on_view()`, `Count::new(id, 1204.)` |
| Calendar | `Calendar::new(id, Date::new(2026, 10, 14))`, `.value()`, `.on_change()` |
| Date picker | `DatePicker::new(id, start)`, `.initial()`, `.on_change()` |
| Swatches | `Swatches::new(id, colors)`, `.initial()`, `.on_change()` |
| Color | `ColorPicker::new(id).on_change()` |
| Chat | `Message::new(text)`, `.mine()`, `Thinking::new(id, "Thinking")` |
| Steps | `Steps::new([("Read", Step::Done), ("Write", Step::Active), ("Check", Step::Waiting)])` |
| Question | `QuestionCard::new(id, "Which?", ["A", "B"]).on_answer()` |
| Sidebar | `Sidebar::new(id).width(200.).child(...)` |
| Compare | `Compare::new(id, first, second).height(160.)` |
| Marquee | `Marquee::new(id, ["A", "B"])` |
| File | `FileCard::new("report.pdf", "2.4 MB")` |
| Glow | `Bloom::new(Tone::Green, content)` |
| Missing page | `NotFound::new("404", "Gone.").action(button)` |
| Small pieces | `CopyField::new(id, text)`, `Avatar::new("AL")`, `Dots::new(id, 3, 1)`, `EmptyState::new(title, text)`, `Uptime::new(periods)` |

Rows are `Row::new("Name")`, with `.disabled(true)`. Plain strings work too.

## Views

These are views, because they need a long-lived object. Create them once and keep the
entity. Add it as a child, and subscribe to hear from it.

```rust
let name = cx.new(|cx| TextField::new(cx).placeholder("Your name"));
cx.subscribe(&name, |_, event: &TextFieldEvent, _| { /* Changed or Submitted */ }).detach();

let city = cx.new(|cx| Combobox::new(["Lisbon", "Oslo"], window, cx)); // ComboboxEvent::Chosen

let commands = cx.new(|cx| CommandMenu::new(["New", "Open"], cx)); // CommandEvent::Picked
commands.update(cx, |menu, cx| menu.show(cx));

let composer = cx.new(Composer::new); // ComposerEvent::Sent(text)
```

## Saving a dither

`fluidither::export::save_png(path, &cells, columns, rows, 3, ink)` writes dither cells at
3 pixels a cell. Make the cells with `fill_map`, a fill of your own, and `dither_tones`.

## Limits to work around or say out loud

- No accessibility API. Screen readers can't see any of this.
- `DitherGradient` defaults to the image path, which stays cheap at any size. Use
  `PaintMode::Quads` only for small areas.
- CPU budget for an animated dither is roughly 100,000 cells a frame.
- `Tooltip` shows on hover only. The text field is one line, with no undo, so `Composer`
  is one line too.
- `Marquee` redraws the window 30 times a second while it moves.
- `FileCard` shows no preview. Rendering a PDF needs a renderer this library doesn't carry.
- Tables are text only, and the scroll bar can't be dragged.
- `DitherImage` reads its file on the main thread at first draw.
- Don't use a typed `WindowHandle::update` around a handler that updates the root view, as
  it holds a lock on it. Use `AnyWindowHandle`.
- `debug_bounds` is never cleared, so tests can't use it to show something has gone.
