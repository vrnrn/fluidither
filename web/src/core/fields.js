// Animated tone fields. Plain arithmetic with no trig, so JS and Rust produce the same
// bits: `wave` stands in for a sine. gpui/src/fields.rs is the same code, and
// fixtures/fields.json holds both to the same cells.

import { grain } from './dither.js';

export const FIELDS = [
  'noise', 'waves', 'plasma', 'ripples', 'tunnel', 'metaballs', 'orb', 'stars', 'aurora', 'contours',
  'beams', 'eclipse', 'horizon', 'mesh', 'grid', 'ridges', 'silk', 'smoke',
];

/** A smooth wave with period 1, from -1 to 1. */
export function wave(t) {
  const turn = t - Math.floor(t);
  const half = turn < 0.5 ? turn * 2 : (turn - 0.5) * 2;
  const shape = 4 * half * (1 - half);
  return turn < 0.5 ? shape : -shape;
}

const clamp01 = (value) => (value < 0 ? 0 : value > 1 ? 1 : value);

function starAt(x, y, t) {
  const cellX = Math.floor(x * 14);
  const cellY = Math.floor(y * 14);
  const seed = grain(cellX * 7.31, cellY * 3.17);
  if (seed < 0.52) return 0;
  const dx = x * 14 - cellX - 0.5;
  const dy = y * 14 - cellY - 0.5;
  const glow = 1 - Math.sqrt(dx * dx + dy * dy) * 3;
  return clamp01(glow) * (0.65 + 0.35 * wave(t * 0.3 + seed * 5));
}

// The tone at (x, y). Both run 0 to 1 along the height, so x can pass 1 in wide boxes.
function toneAt(kind, x, y, wide, t) {
  const dx = x - wide / 2;
  const dy = y - 0.5;
  const distance = Math.sqrt(dx * dx + dy * dy);
  switch (kind) {
    case 'waves':
      return 0.5 + 0.5 * wave(y * 3 + 0.3 * wave(x * 1.1 + t * 0.09) + t * 0.05);
    case 'plasma':
      return 0.5 + 0.125 * (wave(x * 1.3 + t * 0.07) + wave(y * 1.9 - t * 0.05) + wave((x + y) * 1.1 + t * 0.04) + wave(distance * 2.7 - t * 0.09));
    case 'ripples':
      return (0.5 + 0.5 * wave(distance * 5 - t * 0.3)) * clamp01(1.25 - distance * 1.4);
    case 'tunnel':
      return (0.5 + 0.5 * wave(0.35 / (distance + 0.04) + t * 0.25)) * clamp01(distance * 2.4);
    case 'metaballs': {
      let sum = 0;
      for (let ball = 0; ball < 3; ball += 1) {
        const bx = wide / 2 + wave(t * 0.04 + ball * 0.31) * wide * 0.3;
        const by = 0.5 + wave(t * 0.055 + ball * 0.47) * 0.3;
        sum += 0.035 / ((x - bx) * (x - bx) + (y - by) * (y - by) + 0.002);
      }
      return clamp01((sum - 0.6) * 0.5);
    }
    case 'orb': {
      const radius = 0.42;
      if (distance >= radius) return 0;
      const depth = Math.sqrt(radius * radius - distance * distance) / radius;
      const lightX = wave(t * 0.04) * 0.55;
      return clamp01(0.08 + 0.92 * (depth * 0.75 + (dx / radius) * lightX - (dy / radius) * 0.45));
    }
    case 'stars':
      return starAt(x, y, t);
    case 'aurora': {
      const line = 0.42 + 0.14 * wave(x * 0.7 + t * 0.03);
      return clamp01(1 - Math.abs(y - line) * 3.2) * (0.25 + 0.75 * grain(x * 3 + t * 0.09, y * 0.6 - t * 0.02));
    }
    case 'contours': {
      const height = grain(x * 2.2 + t * 0.02, y * 2.2) * 9;
      return height - Math.floor(height) < 0.16 ? 0.9 : 0.06;
    }
    case 'beams': {
      let light = 0;
      for (let beam = 0; beam < 3; beam += 1) {
        const center = wide * (0.5 + 0.4 * wave(t * 0.03 + beam * 0.29));
        const reach = 1 - Math.abs(x - center + (y - 0.5) * 0.4) * 7;
        if (reach > light) light = reach;
      }
      return clamp01(light) * (1.1 - y * 0.8);
    }
    case 'eclipse': {
      const disc = 0.3;
      if (distance < disc) return 0;
      const flicker = 0.75 + 0.5 * grain(dx * 5 + t * 0.1, dy * 5);
      return clamp01((1 - (distance - disc) * 4.5) * flicker);
    }
    case 'horizon': {
      if (y < 0.5) return 0.1 + (0.5 - y) * 0.2;
      const depth = 0.25 / (y - 0.45) + t * 0.15;
      return (depth - Math.floor(depth) < 0.5 ? 0.85 : 0.15) * clamp01((y - 0.5) * 4);
    }
    case 'mesh': {
      let sum = 0;
      for (let point = 0; point < 3; point += 1) {
        const px = wide * (0.5 + 0.42 * wave(t * 0.025 + point * 0.37));
        const py = 0.5 + 0.42 * wave(t * 0.03 + point * 0.61);
        sum += 0.45 / (1 + 7 * ((x - px) * (x - px) + (y - py) * (y - py)));
      }
      return clamp01(sum);
    }
    case 'grid': {
      if (y < 0.42) return 0.05;
      const rise = y - 0.4;
      const across = dx / rise;
      const along = 0.3 / rise + t * 0.2;
      const line = across * 2 - Math.floor(across * 2) < 0.1 || along - Math.floor(along) < 0.14;
      return line ? clamp01(rise * 3.5) : 0.03;
    }
    case 'ridges': {
      const row = y * 8;
      const band = Math.floor(row);
      const peak = grain(x * 3 + band * 5, t * 0.05) * 0.85;
      const height = row - band - (1 - peak);
      if (height < 0) return 0;
      return height < 0.09 ? 1 : 0.14;
    }
    case 'silk':
      return 0.5 + 0.5 * wave(x * 1.2 + 0.4 * wave(y * 1.7 + t * 0.05) + 0.3 * wave(x * 0.7 + y * 0.9 - t * 0.04));
    case 'smoke': {
      const bend = grain(x * 2 + t * 0.06, y * 2 - t * 0.1);
      return grain(x * 3 + bend * 2, y * 3 + bend * 2 - t * 0.08) * (1.25 - y * 0.9);
    }
    default:
      return grain(x * 4 + t * 0.12, y * 4 - t * 0.08);
  }
}

/** Fill `tones` (width * height) with a named field at time `t`. */
export function fillField(tones, width, height, kind, t) {
  const wide = width / height;
  let index = 0;
  for (let row = 0; row < height; row += 1) {
    const y = (row + 0.5) / height;
    for (let column = 0; column < width; column += 1) {
      tones[index] = toneAt(kind, ((column + 0.5) / width) * wide, y, wide, t);
      index += 1;
    }
  }
}
