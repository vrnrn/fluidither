import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { generate } from '../../scripts/gen-blue-noise.mjs';

const SIZE = 64;
const table = new Uint8Array(readFileSync(new URL('../../tokens/blue-noise-64.bin', import.meta.url)));

// A repeatable white noise tile with the same value counts, to compare against.
function shuffled(values) {
  const copy = Uint8Array.from(values);
  let state = 12345;
  for (let i = copy.length - 1; i > 0; i -= 1) {
    state = (Math.imul(state, 1103515245) + 12345) & 0x7fffffff;
    const j = state % (i + 1);
    [copy[i], copy[j]] = [copy[j], copy[i]];
  }
  return copy;
}

// Share of the pattern's energy below `limit` cycles per tile. Blue noise keeps
// this low, which is why it looks even and has no visible clumps.
function lowFrequencyShare(tile, level, limit) {
  const mean = [...tile].filter((value) => value < level).length / tile.length;
  let low = 0;
  let total = 0;
  for (let v = -32; v < 32; v += 1) {
    for (let u = -32; u < 32; u += 1) {
      if (u === 0 && v === 0) continue;
      let re = 0;
      let im = 0;
      for (let y = 0; y < SIZE; y += 1) {
        for (let x = 0; x < SIZE; x += 1) {
          const ink = (tile[y * SIZE + x] < level ? 1 : 0) - mean;
          const phase = (2 * Math.PI * (u * x + v * y)) / SIZE;
          re += ink * Math.cos(phase);
          im += ink * Math.sin(phase);
        }
      }
      const power = re * re + im * im;
      total += power;
      if (Math.hypot(u, v) <= limit) low += power;
    }
  }
  return low / total;
}

function nearestNeighbour(tile, level) {
  const points = [];
  tile.forEach((value, index) => {
    if (value < level) points.push([index % SIZE, index >> 6]);
  });
  let nearest = Infinity;
  for (let i = 0; i < points.length; i += 1) {
    for (let j = i + 1; j < points.length; j += 1) {
      const dx = Math.abs(points[i][0] - points[j][0]);
      const dy = Math.abs(points[i][1] - points[j][1]);
      nearest = Math.min(nearest, Math.hypot(Math.min(dx, SIZE - dx), Math.min(dy, SIZE - dy)));
    }
  }
  return nearest;
}

test('the checked-in table is what the script generates', () => {
  assert.deepEqual(table, generate());
});

test('every level holds the same number of cells', () => {
  const counts = new Array(256).fill(0);
  for (const value of table) counts[value] += 1;
  assert.ok(counts.every((count) => count === 16));
});

test('energy stays out of the low frequencies at every density', () => {
  const white = shuffled(table);
  for (const level of [32, 64, 128, 192]) {
    const blue = lowFrequencyShare(table, level, 8);
    const noise = lowFrequencyShare(white, level, 8);
    assert.ok(blue < noise * 0.15, `level ${level}: ${blue.toFixed(4)} against white ${noise.toFixed(4)}`);
  }
});

test('sparse points keep their distance', () => {
  // At 1/32 density an even grid would put points about 5.7 cells apart.
  assert.ok(nearestNeighbour(table, 8) >= 3.5, `nearest ${nearestNeighbour(table, 8)}`);
  assert.ok(nearestNeighbour(shuffled(table), 8) < 2);
});

test('any window holds close to its share of ink', () => {
  for (let y = 0; y < SIZE; y += 1) {
    for (let x = 0; x < SIZE; x += 1) {
      let inked = 0;
      for (let dy = 0; dy < 16; dy += 1) {
        for (let dx = 0; dx < 16; dx += 1) {
          if (table[((y + dy) % SIZE) * SIZE + ((x + dx) % SIZE)] < 128) inked += 1;
        }
      }
      assert.ok(Math.abs(inked - 128) <= 12, `window at ${x},${y} has ${inked}`);
    }
  }
});
