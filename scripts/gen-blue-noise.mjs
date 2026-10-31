// Generates tokens/blue-noise-64.bin with the void-and-cluster method (Ulichney,
// 1993). A seeded random start makes the output reproducible.
//
// Points repel each other through a Gaussian that wraps at the edges. The
// "tightest cluster" is the point with the most neighbours and the "largest
// void" is the empty cell with the fewest. Ranking cells by removing clusters
// and filling voids gives a threshold map that looks even at every density.
import { writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const SIZE = 64;
const CELLS = SIZE * SIZE;
const SIGMA = 1.9;
const START_DENSITY = 0.1;
const SEED = 0x46d17;

function seededRandom(seed) {
  let state = seed;
  return () => {
    state = (state + 0x6d2b79f5) | 0;
    let mix = Math.imul(state ^ (state >>> 15), state | 1);
    mix ^= mix + Math.imul(mix ^ (mix >>> 7), mix | 61);
    return ((mix ^ (mix >>> 14)) >>> 0) / 4294967296;
  };
}

// How strongly a point at each offset pushes, wrapping around the tile.
const push = new Float64Array(CELLS);
for (let y = 0; y < SIZE; y += 1) {
  for (let x = 0; x < SIZE; x += 1) {
    const dx = Math.min(x, SIZE - x);
    const dy = Math.min(y, SIZE - y);
    push[y * SIZE + x] = Math.exp(-(dx * dx + dy * dy) / (2 * SIGMA * SIGMA));
  }
}

class Field {
  points = new Uint8Array(CELLS);
  energy = new Float64Array(CELLS);
  count = 0;

  set(cell, on) {
    this.points[cell] = on ? 1 : 0;
    this.count += on ? 1 : -1;
    const sign = on ? 1 : -1;
    const cy = cell >> 6;
    const cx = cell & 63;
    for (let y = 0; y < SIZE; y += 1) {
      const row = ((y - cy + SIZE) % SIZE) * SIZE;
      for (let x = 0; x < SIZE; x += 1) {
        this.energy[y * SIZE + x] += sign * push[row + ((x - cx + SIZE) % SIZE)];
      }
    }
  }

  tightestCluster() {
    let best = -1;
    for (let cell = 0; cell < CELLS; cell += 1) {
      if (this.points[cell] && (best < 0 || this.energy[cell] > this.energy[best])) best = cell;
    }
    return best;
  }

  largestVoid() {
    let best = -1;
    for (let cell = 0; cell < CELLS; cell += 1) {
      if (!this.points[cell] && (best < 0 || this.energy[cell] < this.energy[best])) best = cell;
    }
    return best;
  }

  clone() {
    const copy = new Field();
    copy.points.set(this.points);
    copy.energy.set(this.energy);
    copy.count = this.count;
    return copy;
  }
}

export function generate() {
  const random = seededRandom(SEED);
  const start = new Field();
  while (start.count < CELLS * START_DENSITY) {
    const cell = Math.floor(random() * CELLS);
    if (!start.points[cell]) start.set(cell, true);
  }

  // Even out the start: move the tightest point into the largest void until
  // the point just moved would be the next to go.
  for (;;) {
    const from = start.tightestCluster();
    start.set(from, false);
    const to = start.largestVoid();
    start.set(to, true);
    if (to === from) break;
  }

  const ranks = new Uint16Array(CELLS);

  // Below the start density: take points away, densest first.
  const shrinking = start.clone();
  while (shrinking.count > 0) {
    const cell = shrinking.tightestCluster();
    shrinking.set(cell, false);
    ranks[cell] = shrinking.count;
  }

  // Above it: fill the emptiest cell until the tile is full.
  const growing = start.clone();
  while (growing.count < CELLS) {
    const cell = growing.largestVoid();
    ranks[cell] = growing.count;
    growing.set(cell, true);
  }

  return Uint8Array.from(ranks, (rank) => Math.floor((rank * 256) / CELLS));
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..');
  writeFileSync(join(root, 'tokens/blue-noise-64.bin'), generate());
  console.log('blue noise: wrote tokens/blue-noise-64.bin');
}
