# Support

What has been tested, what hasn't, and where the two targets differ.

## Platforms

| Target | Tested | Not tested |
| --- | --- | --- |
| Web | Chromium, on macOS | Firefox and Safari have not been run. Their slider and checkbox styles are written but have never been seen. `linear()` easing needs Safari 17.2 or later. |
| GPUI | macOS on Apple Silicon, Rust 1.98.1, `gpui` 0.2.2 | Linux and Windows have not been built. |

## Accessibility

- **Web:** components use real elements and ARIA, and axe passes on the demo in both
  themes. No one has tried it with a real screen reader yet.
- **GPUI:** `gpui` 0.2.2 has no accessibility API, so VoiceOver can't see these components
  at all. This can't be fixed from this library. Don't use the GPUI target alone for
  anything that must be accessible.

## Known limits on GPUI

- **Text field:** one line, with no undo, so the composer is one line too. The caret doesn't
  blink. Input methods go through the system path but haven't been tried with a real IME.
- **Overlays:** tooltips show on hover, not on keyboard focus. Long lists scroll, but
  keyboard moves don't bring the active row into view yet.
- **Forms:** Select and Combobox have no form to join, so read their value from events.
- **Scroll area:** the bar shows position but can't be dragged.
- **Tables:** text only, and columns share the width equally.
- **Reduced motion:** read from macOS once at startup, and that path hasn't been exercised.
- **Right-to-left text:** not handled.
- **Font weight:** snaps instead of animating.

These inputs haven't been driven in a real window yet: a real drag on Compare, Marquee
pausing on hover, the hue slider inside ColorPicker, and `Dissolve::on_view` scrolling into
view. Live hover, focus, clicks and typing were checked with synthetic events sent to the
window, not hardware input.

## Missing on both targets

- Vertical tabs, sortable tables and submenus.
- PDF previews. File cards show the type and size, because a thumbnail needs a PDF renderer
  and that would break the no-dependency rule.

GPUI also has no multi-line text and no link-style Button.

## Differences between the targets

| Part | Web | GPUI |
| --- | --- | --- |
| Color | The browser's color input, plus swatches | ColorPicker, a saturation field over a hue slider, plus swatches |
| Saving a PNG | `savePng` saves any drawn element | `save_png` takes cells you made yourself |
| Chart axis labels | `labels` | Not available |
| Map and globe | `<fd-world>` | `DitherGradient::map` and `DitherGradient::globe` |
| Accordion animation | Needs `interpolate-size` and `::details-content`, or sections open at once | Animated |

## Things to know

- **The world map is a sketch.** It comes from land outlines drawn by hand, so it looks right
  but isn't geographic data. Don't use it to show where something really is.
- **How motion feels** has been verified by numbers, not judged by eye across devices.
- **Screenshots of GPUI** are in the display's P3 color space, so saturated colors look
  different from Chromium captures even though the values are right.
