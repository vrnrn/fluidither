// Ordered dithering on the CPU. Plain arithmetic on typed arrays with no DOM, so
// it runs in Node for tests. gpui/src/dither.rs is the same algorithm, and
// fixtures/dither.json holds both to the same cells.

import { bayerRanks, halftoneRanks } from './tokens.js';

const cached = new Map();

function buildMap(size, ranks, levels) {
  // Centered in each level, so no tone lands exactly on a threshold.
  return { size, thresholds: Float64Array.from(ranks, (rank) => (rank + 0.5) / levels) };
}

/** The threshold map for `bayer2`, `bayer4`, `bayer8`, `halftone` or `blue-noise`. */
export async function thresholdMap(name) {
  if (cached.has(name)) return cached.get(name);
  let map;
  if (name === 'blue-noise') {
    const { blueNoiseBase64 } = await import('./blue-noise.js');
    const bytes = Uint8Array.from(atob(blueNoiseBase64), (char) => char.charCodeAt(0));
    map = buildMap(64, bytes, 256);
  } else if (name === 'halftone') {
    map = buildMap(8, halftoneRanks, 64);
  } else {
    const size = { bayer2: 2, bayer8: 8 }[name] ?? 4;
    map = buildMap(size, bayerRanks[size], size * size);
  }
  cached.set(name, map);
  return map;
}

// Lattice noise. Integer hashing rather than a random source, so both targets
// get the same bits.
function lattice(x, y) {
  let mix = Math.imul(x, 0x27d4eb2f) ^ Math.imul(y, 0x165667b1);
  mix = Math.imul(mix ^ (mix >>> 15), 0x2c1b3c6d);
  mix = Math.imul(mix ^ (mix >>> 12), 0x297a2d39);
  return ((mix ^ (mix >>> 15)) >>> 0) / 4294967296;
}

const ease = (t) => t * t * (3 - 2 * t);

function smoothNoise(x, y) {
  const left = Math.floor(x);
  const top = Math.floor(y);
  const across = ease(x - left);
  const down = ease(y - top);
  const upper = lattice(left, top);
  const lower = lattice(left, top + 1);
  const upperRow = upper + (lattice(left + 1, top) - upper) * across;
  const lowerRow = lower + (lattice(left + 1, top + 1) - lower) * across;
  return upperRow + (lowerRow - upperRow) * down;
}

/** Two layers of smooth noise, 0 to 1. */
export function grain(x, y) {
  return (2 * smoothNoise(x, y) + smoothNoise(2.03 * x + 41, 2.03 * y + 23)) / 3;
}

const WARP_SCALE = 2.6;
const WARP_DRIFT_X = 0.07;
const WARP_DRIFT_Y = 0.05;

/**
 * Fill `tones` (width * height) with a gradient at time `t`, following the
 * geometry of CSS gradients. `mode` is 0 linear, 1 radial, 2 conic. `angle` is in
 * CSS degrees. `centerX`, `centerY` and `radius` are shares of the box. A radius
 * of 1 reaches the farthest corner. `warp` bends the bands with drifting noise.
 */
export function fillGradient(tones, width, height, gradient, t) {
  const { mode, angle, centerX, centerY, radius, from, to, warp } = gradient;
  const originX = mode === 0 ? width / 2 : centerX * width;
  const originY = mode === 0 ? height / 2 : centerY * height;
  const radians = (angle * Math.PI) / 180;
  const sin = Math.sin(radians);
  const cos = Math.cos(radians);
  const lineLength = Math.abs(width * sin) + Math.abs(height * cos);
  const reach = Math.hypot(Math.max(originX, width - originX), Math.max(originY, height - originY)) * radius;
  const noiseScale = WARP_SCALE / height;
  const driftX = t * WARP_DRIFT_X;
  const driftY = t * WARP_DRIFT_Y;

  let index = 0;
  for (let row = 0; row < height; row += 1) {
    const y = row + 0.5 - originY;
    for (let column = 0; column < width; column += 1) {
      const x = column + 0.5 - originX;
      let along;
      if (mode === 0) along = (x * sin - y * cos) / lineLength + 0.5;
      else if (mode === 1) along = Math.hypot(x, y) / reach;
      else {
        const turn = Math.atan2(x, -y) / (2 * Math.PI) - angle / 360;
        along = turn - Math.floor(turn);
      }
      if (warp !== 0) {
        along += warp * (grain((column + 0.5) * noiseScale + driftX, (row + 0.5) * noiseScale + driftY) - 0.5);
      }
      const clamped = along < 0 ? 0 : along > 1 ? 1 : along;
      tones[index] = from + (to - from) * clamped;
      index += 1;
    }
  }
}

/**
 * Pick a palette entry per cell. A tone falls between two neighbouring entries
 * and the threshold decides which. `palette` and `out` are the same array type.
 */
export function ditherTones(tones, width, height, map, palette, out) {
  const { size, thresholds } = map;
  const wrap = size - 1;
  const last = palette.length - 1;
  let index = 0;
  for (let y = 0; y < height; y += 1) {
    const row = (y & wrap) * size;
    for (let x = 0; x < width; x += 1) {
      const scaled = tones[index] * last;
      const lower = Math.floor(scaled);
      const upper = scaled - lower > thresholds[row + (x & wrap)];
      out[index] = palette[Math.min(upper ? lower + 1 : lower, last)];
      index += 1;
    }
  }
}

/**
 * Fill `tones` with a sparkline: solid ink along the line, fading out below it.
 * `values` run 0 to 1, spread evenly across the width.
 */
export function fillSparkline(tones, width, height, values, fade) {
  const span = Math.max(values.length - 1, 1);
  let index = 0;
  const tops = new Float64Array(width);
  for (let column = 0; column < width; column += 1) {
    const along = ((column + 0.5) / width) * span;
    const left = Math.min(Math.floor(along), span - 1);
    const value = values[left] + (values[Math.min(left + 1, values.length - 1)] - values[left]) * (along - left);
    tops[column] = (1 - value) * (height - 1);
  }
  for (let row = 0; row < height; row += 1) {
    for (let column = 0; column < width; column += 1) {
      const below = row - tops[column];
      if (below < 0) tones[index] = 0;
      else if (below < 1) tones[index] = 1;
      else tones[index] = fade * (1 - below / (height - tops[column]));
      index += 1;
    }
  }
}
