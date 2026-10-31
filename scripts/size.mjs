// Real bundle sizes: each component minified with its own dependencies, then
// compressed. esbuild is a dev dependency used here only; nothing ships it.
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { brotliCompressSync, gzipSync } from 'node:zlib';
import { build } from 'esbuild';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const src = join(root, 'web/src');

async function minify(entry, loader) {
  const result = await build({
    bundle: true,
    entryPoints: [join(src, entry)],
    external: ['./blue-noise.js'],
    format: 'esm',
    loader: { '.css': 'css' },
    minify: true,
    write: false,
    ...(loader ? { loader } : {}),
  });
  return result.outputFiles[0].contents;
}

const sizes = (bytes) => ({
  raw: bytes.length,
  gzip: gzipSync(bytes, { level: 9 }).length,
  brotli: brotliCompressSync(bytes).length,
});

const rows = [
  ['tokens.css', [['tokens.css']]],
  ['button.css (Button needs no JS)', [['components/button.css']]],
  ['tabs.js + tabs.css', [['components/tabs.js'], ['components/tabs.css']]],
  ['dither-gradient.js + .css', [['components/dither-gradient.js'], ['components/dither-gradient.css']]],
  ['input.css', [['components/input.css']]],
  ['check.css', [['components/check.css']]],
  ['switch.css', [['components/switch.css']]],
  ['slider.js + slider.css', [['components/slider.js'], ['components/slider.css']]],
  ['tooltip.js + .css', [['components/tooltip.js'], ['components/tooltip.css']]],
  ['menu.js + list.css', [['components/menu.js'], ['components/list.css']]],
  ['select.js + .css, with list.css', [['components/select.js'], ['components/select.css'], ['components/list.css']]],
  ['combobox.js + .css, with list.css', [['components/combobox.js'], ['components/combobox.css'], ['components/list.css']]],
  ['dialog.css', [['components/dialog.css']]],
  ['command.js + .css, with list and dialog', [['components/command.js'], ['components/command.css'], ['components/list.css'], ['components/dialog.css']]],
  ['card.css', [['components/card.css']]],
  ['badge.css', [['components/badge.css']]],
  ['banner.css', [['components/banner.css']]],
  ['accordion.css', [['components/accordion.css']]],
  ['table.css', [['components/table.css']]],
  ['nav.js + nav.css', [['components/nav.js'], ['components/nav.css']]],
  ['scroll.css', [['components/scroll.css']]],
  ['feedback.css', [['components/feedback.css']]],
  ['sparkline.js + .css', [['components/sparkline.js'], ['components/sparkline.css']]],
  ['field.js + .css', [['components/field.js'], ['components/field.css']]],
  ['chart.js + .css', [['components/chart.js'], ['components/chart.css']]],
  ['dither-image.js + field.css', [['components/dither-image.js'], ['components/field.css']]],
  ['text-effects.js + .css', [['components/text-effects.js'], ['components/text-effects.css']]],
  ['calendar.js + .css', [['components/calendar.js'], ['components/calendar.css']]],
  ['date-picker.js + calendar.css', [['components/date-picker.js'], ['components/calendar.css']]],
  ['swatches.js', [['components/swatches.js']]],
  ['copy.js', [['components/copy.js']]],
  ['misc.css', [['components/misc.css']]],
  ['world.js + extras.css', [['components/world.js'], ['components/extras.css']]],
  ['sidebar.js', [['components/sidebar.js']]],
  ['compare.js', [['components/compare.js']]],
  ['marquee.js', [['components/marquee.js']]],
  ['core/export.js', [['core/export.js']]],
  ['blue-noise.js (only if asked for)', [['core/blue-noise.js']]],
];

const total = { raw: 0, gzip: 0, brotli: 0 };
const pad = (value, width) => String(value).padStart(width);
console.log(`${'piece'.padEnd(36)}${pad('min', 8)}${pad('gzip', 8)}${pad('brotli', 8)}`);
for (const [label, entries] of rows) {
  const row = { raw: 0, gzip: 0, brotli: 0 };
  for (const [entry] of entries) {
    const measured = sizes(await minify(entry));
    for (const key of Object.keys(row)) row[key] += measured[key];
  }
  for (const key of Object.keys(row)) total[key] += row[key];
  console.log(`${label.padEnd(36)}${pad(row.raw, 8)}${pad(row.gzip, 8)}${pad(row.brotli, 8)}`);
}
console.log(`${'everything, compressed per file'.padEnd(36)}${pad(total.raw, 8)}${pad(total.gzip, 8)}${pad(total.brotli, 8)}`);
