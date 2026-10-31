// Writes the numbers both targets must reproduce. The JS engine produces them;
// the Rust tests read them back, so any drift between the two fails a test.
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { ditherTones, fillGradient, fillSparkline, grain, thresholdMap } from '../web/src/core/dither.js';
import { chartCells, CHARTS } from '../web/src/core/charts.js';
import { FIELDS, fillField, wave } from '../web/src/core/fields.js';
import { addMonths, fromDays, iso, monthGrid, toDays, weekday } from '../web/src/core/dates.js';
import { scrambled } from '../web/src/core/text.js';
import { fillGlobe, fillMap } from '../web/src/core/world.js';
import { Spring, springs, stepSpring } from '../web/src/core/spring.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

// FNV-1a over 32-bit words, so a mismatch names the case without storing cells.
export function hashCells(cells) {
  let hash = 0x811c9dc5;
  for (const cell of cells) {
    hash = Math.imul(hash ^ cell, 0x01000193) >>> 0;
  }
  return hash;
}

const base = { mode: 0, angle: 180, centerX: 0.5, centerY: 0.5, radius: 1, from: 0, to: 1, warp: 0 };

const cases = [
  { name: 'linear-down', matrix: 'bayer4', width: 64, height: 40, t: 0, gradient: base },
  { name: 'linear-bayer2', matrix: 'bayer2', width: 33, height: 17, t: 0, gradient: { ...base, angle: 90 } },
  { name: 'linear-diagonal', matrix: 'bayer8', width: 80, height: 30, t: 0, gradient: { ...base, angle: 135, from: 0.1, to: 0.9 } },
  { name: 'radial', matrix: 'bayer8', width: 50, height: 50, t: 0, gradient: { ...base, mode: 1, centerX: 0.3, centerY: 0.6, radius: 0.8 } },
  { name: 'conic', matrix: 'blue-noise', width: 70, height: 70, t: 0, gradient: { ...base, mode: 2, angle: 45 } },
  { name: 'noise-still', matrix: 'bayer4', width: 120, height: 40, t: 0, gradient: { ...base, angle: 90, to: 0.75, warp: 0.12 } },
  { name: 'noise-later', matrix: 'bayer4', width: 120, height: 40, t: 12.5, gradient: { ...base, angle: 90, to: 0.75, warp: 0.12 } },
  { name: 'three-colors', matrix: 'bayer4', width: 60, height: 20, t: 0, colors: 3, gradient: { ...base, angle: 90 } },
];

const dither = [];
for (const { name, matrix, width, height, t, gradient, colors = 2 } of cases) {
  const tones = new Float64Array(width * height);
  const out = new Uint32Array(width * height);
  fillGradient(tones, width, height, gradient, t);
  ditherTones(tones, width, height, await thresholdMap(matrix), Uint32Array.from({ length: colors }, (_, i) => i), out);
  dither.push({ name, matrix, width, height, t, colors, gradient, hash: hashCells(out), inked: out.reduce((sum, cell) => sum + (cell > 0 ? 1 : 0), 0) });
}

const sparkline = { width: 48, height: 16, fade: 0.6, matrix: 'bayer4', values: [0.1, 0.8, 0.35, 1, 0, 0.55] };
{
  const { width, height, fade, matrix, values } = sparkline;
  const tones = new Float64Array(width * height);
  const out = new Uint32Array(width * height);
  fillSparkline(tones, width, height, values, fade);
  ditherTones(tones, width, height, await thresholdMap(matrix), Uint32Array.of(0, 1), out);
  sparkline.hash = hashCells(out);
}

const fieldCases = [];
for (const kind of FIELDS) {
  for (const t of [0, 7.25]) {
    const [width, height] = [64, 40];
    const tones = new Float64Array(width * height);
    const out = new Uint32Array(width * height);
    fillField(tones, width, height, kind, t);
    ditherTones(tones, width, height, await thresholdMap('bayer4'), Uint32Array.of(0, 1), out);
    fieldCases.push({ kind, t, width, height, hash: hashCells(out) });
  }
}

const chartData = {
  bar: [[3, 7, 2, 9], [5, 1, 6, 4]],
  area: [[2, 6, 3, 8, 5], [1, 2, 2, 4, 3]],
  line: [[2, 6, 3, 8, 5], [7, 1, 6, 2, 4]],
  donut: [[5, 3, 2, 1]],
  radar: [[5, 8, 3, 7, 6], [3, 4, 6, 2, 5]],
  heatmap: [[1, 4, 2], [7, 3, 9], [2, 8, 5]],
  gauge: [[62, 100]],
};
const chartCases = [];
for (const kind of CHARTS) {
  const [width, height] = [60, 40];
  const out = new Uint32Array(width * height);
  chartCells(kind, chartData[kind], width, height, await thresholdMap('bayer4'), out);
  chartCases.push({ kind, series: chartData[kind], width, height, hash: hashCells(out), inked: out.filter(Boolean).length });
}

const worldCases = [];
{
  const [width, height] = [80, 40];
  const map = await thresholdMap('bayer4');
  for (const [kind, draw] of [['map', fillMap], ['globe', fillGlobe]]) {
    for (const turn of [0, 1.7]) {
      const tones = new Float64Array(width * height);
      const out = new Uint32Array(width * height);
      draw(tones, width, height, turn);
      ditherTones(tones, width, height, map, Uint32Array.of(0, 1), out);
      worldCases.push({ kind, turn, width, height, hash: hashCells(out), inked: out.filter(Boolean).length });
    }
  }
}

const noise = [
  [0, 0],
  [0.5, 0.25],
  [-3.75, 2.125],
  [1000.5, -2000.25],
].map(([x, y]) => ({ x, y, value: grain(x, y) }));

const spring = [];
for (const [name, config] of Object.entries(springs)) {
  // A retarget halfway through checks that velocity carries over.
  let position = 0;
  let velocity = 0;
  const trace = [];
  for (let frame = 0; frame < 30; frame += 1) {
    const target = frame < 6 ? 100 : -40;
    [position, velocity] = stepSpring(config, position, velocity, target, 1 / 120);
    trace.push([position, velocity]);
  }
  const whole = new Spring(config, 0);
  whole.target = 1;
  let frames = 0;
  while (!whole.step(1 / 60) && frames < 600) frames += 1;
  spring.push({ name, trace, framesToSettle: frames });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  mkdirSync(join(root, 'fixtures'), { recursive: true });
  writeFileSync(join(root, 'fixtures/dither.json'), `${JSON.stringify({ cases: dither, noise, sparkline }, null, 1)}\n`);
  const sampleDays = [[1970, 1, 1], [2000, 2, 29], [2024, 12, 31], [1900, 3, 1], [2026, 10, 30], [1600, 1, 1], [2100, 2, 28]];
  writeFileSync(
    join(root, 'fixtures/dates.json'),
    `${JSON.stringify(
      {
        days: sampleDays.map(([year, month, day]) => ({ date: iso({ year, month, day }), days: toDays({ year, month, day }), weekday: weekday({ year, month, day }), back: iso(fromDays(toDays({ year, month, day }))) })),
        months: [[2026, 10], [2024, 2], [2023, 1], [1999, 12]].map(([year, month]) => ({ year, month, grid: monthGrid(year, month).map(iso) })),
        shifts: [[{ year: 2024, month: 1, day: 31 }, 1], [{ year: 2023, month: 12, day: 15 }, 2], [{ year: 2024, month: 3, day: 31 }, -1], [{ year: 2020, month: 2, day: 29 }, 12]].map(([date, count]) => ({ from: iso(date), count, to: iso(addMonths(date, count)) })),
      },
      null,
      1,
    )}\n`,
  );
  writeFileSync(join(root, 'fixtures/fields.json'), `${JSON.stringify({ fields: fieldCases, charts: chartCases, world: worldCases, scramble: [[0, 1], [0.25, 4], [0.5, 9], [0.9, 2], [1, 3]].map(([progress, seed]) => ({ progress, seed, text: scrambled('Hello there, world', progress, seed) })), waves: [0, 0.1, 0.33, 0.5, 0.77, 1.25, -0.4].map((t) => ({ t, value: wave(t) })) }, null, 1)}\n`);
  writeFileSync(join(root, 'fixtures/spring.json'), `${JSON.stringify({ springs: spring }, null, 1)}\n`);
  console.log(`fixtures: ${dither.length} dither cases, ${spring.length} springs`);
}
