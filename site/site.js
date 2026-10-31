// The tile wall. Measures the window, deals a board, and swaps boards under a dither wipe. The
// first board is fixed, and every other comes from a seed.

import { deal, randomFrom } from './layout.js';
import { loadParts } from './parts.js';
import { replayButton } from './replay.js';
import { opening, POOL } from './tiles.js';
import { reveal, wipe } from './wipe.js';

const WALL = { columns: 5, rows: 3 };
// The wall takes this share of the window, down to the smallest cells that hold a calendar.
const WALL_SHARE = { width: 0.84, height: 0.72 };
const CELL_WIDTH = { least: 228, wanted: 260, most: 360 };
const CELL_HEIGHT = { least: 232, wanted: 248, most: 320 };
const WIDEST_CELL_SHAPE = 1.3;
const FULL_WALL_WIDTH = 1240;

const SCROLLING_CELL_WIDTH = 284;
const SCROLLING_CELL_HEIGHT = 276;
const SCROLLING_TILES = 12;
const SCROLLING_MOST_COLUMNS = 4;
const RESIZE_WAIT_MS = 180;

const GROUNDS = ['silk', 'ridges', 'contours', 'smoke', 'waves', 'aurora', 'horizon', 'plasma'];

const page = document.documentElement;
const board = document.getElementById('board');
const hero = document.getElementById('hero');
const ground = document.getElementById('ground');
const dice = document.getElementById('randomize');
const dark = document.getElementById('dark');
const status = document.getElementById('status');

let seed = 0;
let shape = '';
let busy = false;

const clamp = (value, least, most) => Math.min(Math.max(value, least), most);

function measure() {
  const styles = getComputedStyle(page);
  const length = (name) => Number.parseFloat(styles.getPropertyValue(name));
  const gap = length('--wall-gap');
  const edge = length('--wall-edge');
  const bar = length('--wall-bar');
  const width = page.clientWidth;
  const height = innerHeight;
  const span = (room, count) => (room - (count - 1) * gap) / count;

  if (width < FULL_WALL_WIDTH) {
    const room = width - 2 * edge;
    const columns = clamp(Math.round(room / SCROLLING_CELL_WIDTH), 1, SCROLLING_MOST_COLUMNS);
    const rows = Math.ceil(SCROLLING_TILES / columns) + 1;
    return { columns, rows, gap, cellHeight: SCROLLING_CELL_HEIGHT, cellWidth: Math.floor(span(room, columns)) };
  }

  const wantedHeight = clamp(span(height * WALL_SHARE.height, WALL.rows), CELL_HEIGHT.wanted, CELL_HEIGHT.most);
  const cellHeight = Math.max(Math.min(wantedHeight, span(height - 2 * bar, WALL.rows)), CELL_HEIGHT.least);
  const widest = Math.min(CELL_WIDTH.most, cellHeight * WIDEST_CELL_SHAPE);
  const wantedWidth = clamp(span(width * WALL_SHARE.width, WALL.columns), CELL_WIDTH.wanted, widest);
  const cellWidth = Math.max(Math.min(wantedWidth, span(width - 2 * edge, WALL.columns)), CELL_WIDTH.least);
  return { ...WALL, gap, cellHeight: Math.floor(cellHeight), cellWidth: Math.floor(cellWidth) };
}

function place(element, { x, y, width, height }) {
  element.style.gridArea = `${y + 1} / ${x + 1} / ${y + height + 1} / ${x + width + 1}`;
}

function heroSlot({ columns, rows }, random) {
  if (columns === 1) return { x: 0, y: 0, width: 1, height: 1 };
  const shapes = [[2, 1], [2, 2]].filter(([, height]) => height <= rows);
  const [width, height] = shapes[Math.floor(random() * shapes.length)];
  return {
    x: Math.floor(random() * (columns - width + 1)),
    y: Math.floor(random() * (rows - height + 1)),
    width,
    height,
  };
}

function render({ item, ...slot }, pick, grid, actions) {
  const tile = document.createElement('article');
  tile.className = 'fd-card tile';
  tile.dataset.tile = item.id;
  tile.toggleAttribute('data-bare', item.bare ?? false);
  if (item.compact && grid.cellHeight < item.compact) tile.dataset.density = 'compact';
  const content = item.render(pick, tile);
  tile.innerHTML = content;
  if (item.replay) {
    const button = replayButton(`Replay ${item.id}`, () => {
      tile.innerHTML = content;
      tile.append(button);
    });
    tile.append(button);
  }
  place(tile, slot);
  return { tile, wire: () => item.wire?.(tile, actions) };
}

function arrange(grid, random) {
  if (seed !== 0) {
    const heroAt = heroSlot(grid, random);
    return { grid, heroAt, tiles: deal(POOL, grid, random, [heroAt]) };
  }
  const plan = opening(grid.columns, WALL);
  const fixed = plan.tiles.map((slot) => ({ ...slot, item: POOL.find(({ id }) => id === slot.id) }));
  const rest = POOL.filter((item) => !plan.tiles.some(({ id }) => id === item.id));
  const sized = { ...grid, rows: plan.rows };
  const dealt = deal(rest, sized, random, [plan.hero, ...plan.tiles]);
  return { grid: sized, heroAt: plan.hero, tiles: [...fixed, ...dealt] };
}

const showWord = () => {
  const word = document.createElement('fd-dissolve');
  word.setAttribute('hold', '0 0.62');
  word.textContent = 'fluidither';
  hero.querySelector('h1').replaceChildren(word);
};

const setSize = ({ columns, cellWidth, cellHeight, gap }) => {
  page.style.setProperty('--wall-columns', columns);
  page.style.setProperty('--wall-row', `${cellHeight}px`);
  page.style.setProperty('--wall-width', `${columns * cellWidth + (columns - 1) * gap}px`);
};

async function prepare(measured) {
  const random = randomFrom(seed);
  // The first example everywhere on the opening board.
  const pick = seed === 0 ? ([first]) => first : (list) => list[Math.floor(random() * list.length)];
  const { grid, heroAt, tiles } = arrange(measured, random);
  const adds = [...new Set(tiles.flatMap(({ item }) => item.adds))];
  await loadParts(adds);
  const actions = { randomize, toggleTheme };
  const rendered = tiles.map((tile) => render(tile, pick, grid, actions));
  const kind = seed === 0 ? GROUNDS[0] : GROUNDS[Math.floor(random() * GROUNDS.length)];
  return () => {
    setSize(grid);
    place(hero, heroAt);
    board.replaceChildren(hero, ...rendered.map(({ tile }) => tile));
    rendered.forEach(({ wire }) => wire());
    showWord();
    ground.setAttribute('kind', kind);
    status.textContent = `New board with ${tiles.length} components.`;
  };
}

const describe = ({ columns, rows }) => `${columns}x${rows}`;

const center = (element) => {
  const { left, top, width, height } = element.getBoundingClientRect();
  return { x: left + width / 2, y: top + height / 2 };
};

async function randomize() {
  if (busy) return;
  busy = true;
  delete page.dataset.ready;
  seed = (Math.random() * 2 ** 32) >>> 0 || 1;
  const grid = measure();
  shape = describe(grid);
  const swap = await prepare(grid);
  await wipe(center(dice), getComputedStyle(page).getPropertyValue('--fd-paper'), swap);
  page.dataset.ready = '';
  busy = false;
}

function setTheme(theme) {
  page.dataset.theme = theme;
  dark.checked = theme === 'dark';
  window.saveTheme(theme);
}

async function toggleTheme() {
  if (busy) return;
  busy = true;
  const next = page.dataset.theme === 'dark' ? 'light' : 'dark';
  // The old ink is close to the new paper, so the page turns over through its own ink.
  await wipe(center(dark), getComputedStyle(page).getPropertyValue('--fd-ink'), () => {
    setTheme(next);
    showWord();
  });
  busy = false;
}

function refit() {
  const grid = measure();
  if (describe(grid) === shape) {
    setSize(grid);
    showWord();
    return;
  }
  shape = describe(grid);
  prepare(grid).then((swap) => swap());
}

let resizing = 0;
new ResizeObserver(() => {
  clearTimeout(resizing);
  resizing = setTimeout(refit, RESIZE_WAIT_MS);
}).observe(document.body);

dice.addEventListener('click', randomize);

dark.checked = page.dataset.theme === 'dark';
dark.addEventListener('change', () => {
  dark.checked = page.dataset.theme !== 'dark';
  toggleTheme();
});

document.addEventListener('keydown', (event) => {
  if (event.key !== 'r' || event.metaKey || event.ctrlKey || event.altKey) return;
  if (event.target.closest('input, textarea, [contenteditable], fd-combobox')) return;
  randomize();
});

const firstGrid = measure();
shape = describe(firstGrid);
const firstBoard = await prepare(firstGrid);
hero.classList.add('tile-over');
await reveal(center(dice), getComputedStyle(page).getPropertyValue('--fd-paper'), firstBoard);
hero.classList.remove('tile-over');
page.dataset.ready = '';
