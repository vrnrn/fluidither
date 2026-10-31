// Deals a board: carves a grid into tiles, then gives each tile one component that suits its
// size. A seed fixes the deal. No DOM, so it runs in Node for tests.

const SHAPES = [
  { width: 1, height: 1, weight: 10 },
  { width: 2, height: 1, weight: 7 },
  { width: 1, height: 2, weight: 4 },
  { width: 2, height: 2, weight: 3 },
];

const ATTEMPTS = 200;

/** A repeatable source of numbers from 0 to 1. */
export function randomFrom(seed) {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let mix = Math.imul(state ^ (state >>> 15), state | 1);
    mix ^= mix + Math.imul(mix ^ (mix >>> 7), mix | 61);
    return ((mix ^ (mix >>> 14)) >>> 0) / 4294967296;
  };
}

function weighted(list, random) {
  let left = random() * list.reduce((sum, each) => sum + (each.weight ?? 1), 0);
  for (const each of list) {
    left -= each.weight ?? 1;
    if (left <= 0) return each;
  }
  return list.at(-1);
}

const taken = (grid, columns, x, y) => grid[y * columns + x] === 1;

function fits(grid, columns, rows, { x, y, width, height }) {
  if (x + width > columns || y + height > rows) return false;
  for (let row = y; row < y + height; row += 1) {
    for (let column = x; column < x + width; column += 1) {
      if (taken(grid, columns, column, row)) return false;
    }
  }
  return true;
}

function claim(grid, columns, { x, y, width, height }) {
  for (let row = y; row < y + height; row += 1) grid.fill(1, row * columns + x, row * columns + x + width);
}

/** Split what `placed` leaves of the grid into slots, in reading order. */
export function carve(columns, rows, random, placed = []) {
  const grid = new Uint8Array(columns * rows);
  placed.forEach((slot) => claim(grid, columns, slot));
  const slots = [];
  for (let y = 0; y < rows; y += 1) {
    for (let x = 0; x < columns; x += 1) {
      if (taken(grid, columns, x, y)) continue;
      const open = SHAPES.filter((shape) => fits(grid, columns, rows, { x, y, ...shape }));
      const { width, height } = weighted(open, random);
      const slot = { x, y, width, height };
      claim(grid, columns, slot);
      slots.push(slot);
    }
  }
  return slots;
}

function pixels(slot, { cellWidth, cellHeight, gap }) {
  return {
    width: slot.width * cellWidth + (slot.width - 1) * gap,
    height: slot.height * cellHeight + (slot.height - 1) * gap,
  };
}

function suits(item, slot, grid) {
  const { width, height } = pixels(slot, grid);
  const [mostWide, mostTall] = item.most ?? [2, 2];
  return width >= item.min[0] && height >= item.min[1] && slot.width <= mostWide && slot.height <= mostTall;
}

// The tightest slots choose first, or the small tiles use up everything that can sit in them.
// No slot stays empty: one that nothing suits takes what is left and counts as a miss.
function fill(slots, pool, grid, random) {
  const left = new Set(pool);
  const choices = (slot) => [...left].filter((item) => suits(item, slot, grid));
  const tiles = [];
  let misses = 0;
  for (const slot of [...slots].sort((a, b) => choices(a).length - choices(b).length)) {
    const fitting = choices(slot);
    if (fitting.length === 0) misses += 1;
    const item = weighted(fitting.length > 0 ? fitting : [...left], random);
    left.delete(item);
    tiles.push({ ...slot, item });
  }
  return { tiles: tiles.sort((a, b) => a.y - b.y || a.x - b.x), misses };
}

/**
 * Deal `pool` over every cell that `placed` leaves free, each item at most once. Very narrow
 * windows suit few items, so it keeps the attempt with the fewest misses.
 */
export function deal(pool, grid, random, placed = []) {
  let best = null;
  for (let attempt = 0; attempt < ATTEMPTS; attempt += 1) {
    const dealt = fill(carve(grid.columns, grid.rows, random, placed), pool, grid, random);
    if (!best || dealt.misses < best.misses) best = dealt;
    if (best.misses === 0) break;
  }
  return best.tiles;
}
