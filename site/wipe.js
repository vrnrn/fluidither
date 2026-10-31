// The page's one transition: a dither that spreads from a point until it covers everything,
// a change made under it, then the dither thinning away. It rides a spring, and under reduced
// motion the change simply happens.

import { CellCanvas } from './fluidither/core/cells.js';
import { ditherTones, thresholdMap } from './fluidither/core/dither.js';
import { prefersReducedMotion } from './fluidither/core/motion.js';
import { Glide } from './fluidither/core/spring.js';

const CELL = 6;
const SPREAD = { omega: 16, damping: 1 };
// How far behind its leading edge the dither reaches solid, as a share of the longest distance.
const SOFTNESS = 0.5;

const host = document.getElementById('wipe');
let cells = null;

function run(glide, target) {
  return new Promise((resolve) => {
    const report = glide.report;
    glide.report = (values) => {
      report(values);
      if (values.reach === target) resolve();
    };
    glide.to({ reach: target });
  });
}

async function sweep(origin, color, change, covered) {
  host.hidden = false;
  cells ??= new CellCanvas(host);
  cells.fit(CELL);
  const { columns, rows } = cells;
  const startX = (origin.x / innerWidth) * columns;
  const startY = (origin.y / innerHeight) * rows;
  const farthest = Math.hypot(Math.max(startX, columns - startX), Math.max(startY, rows - startY));
  const distances = new Float64Array(columns * rows);
  for (let y = 0; y < rows; y += 1) {
    for (let x = 0; x < columns; x += 1) distances[y * columns + x] = Math.hypot(x - startX, y - startY) / farthest;
  }
  const map = await thresholdMap('bayer4');
  const tones = new Float64Array(columns * rows);
  const marks = new Uint8Array(columns * rows);
  let hiding = true;

  const paint = ({ reach }) => {
    const front = reach * (1 + SOFTNESS);
    for (let index = 0; index < tones.length; index += 1) {
      const tone = Math.min(Math.max((front - distances[index]) / SOFTNESS, 0), 1);
      tones[index] = hiding ? tone : 1 - tone;
    }
    ditherTones(tones, columns, rows, map, Uint8Array.of(0, 1), marks);
    cells.painter.clearRect(0, 0, columns, rows);
    cells.layer(marks, color, (mark) => mark === 1, true);
  };

  const glide = new Glide({ reach: covered ? 1 : 0 }, SPREAD, paint);
  if (covered) {
    glide.jump({ reach: 1 });
  } else {
    glide.jump({ reach: 0 });
    await run(glide, 1);
  }
  await change();
  await new Promise(requestAnimationFrame);
  await new Promise(requestAnimationFrame);
  hiding = false;
  glide.jump({ reach: 0 });
  await run(glide, 1);
  host.hidden = true;
}

/** Cover the page from `origin` in `color`, run `change`, then uncover it. */
export async function wipe(origin, color, change) {
  if (prefersReducedMotion()) {
    await change();
    return;
  }
  await sweep(origin, color, change, false);
}

/** Uncover the page from `origin`, for the first board. */
export async function reveal(origin, color, change) {
  if (prefersReducedMotion()) {
    await change();
    return;
  }
  await sweep(origin, color, change, true);
}
