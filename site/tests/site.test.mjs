// The dealer on its own, then real boards in Chromium under the page's own CSP.
import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { chromium } from 'playwright-core';
import { deal, randomFrom } from '../layout.js';
import { createSiteServer } from '../serve.mjs';
import { POOL } from '../tiles.js';

const axeSource = readFileSync(createRequire(import.meta.url).resolve('axe-core/axe.min.js'), 'utf8');
const BOARDS = 24;
const WALLS = [
  { width: 1440, height: 900 },
  { width: 1440, height: 810 },
  { width: 1680, height: 1050 },
  { width: 1920, height: 1080 },
  { width: 2560, height: 1440 },
  { width: 1280, height: 720 },
];
const SCROLLING = [
  { width: 1024, height: 768 },
  { width: 768, height: 1024 },
  { width: 390, height: 844 },
];
const OPENING = ['area', 'field', 'form', 'calendar', 'bars', 'glide', 'proof', 'globe', 'chat'];

let browser;
let server;
let origin;

before(async () => {
  browser = await chromium.launch();
  server = createSiteServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  origin = `http://127.0.0.1:${server.address().port}`;
});

after(async () => {
  await browser.close();
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
});

async function open(t, options) {
  const context = await browser.newContext({ reducedMotion: 'reduce', ...options });
  t.after(() => context.close());
  const page = await context.newPage();
  const problems = [];
  page.on('pageerror', (error) => problems.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') problems.push(message.text());
  });
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => (window.violations ??= []).push(`${event.violatedDirective} ${event.blockedURI}`));
    // Repeatable boards, so a failure can be found again.
    let step = 0;
    Math.random = () => {
      step += 1;
      return (step * 0.61803398875) % 1;
    };
  });
  await page.goto(`${origin}/`);
  await page.waitForSelector('html[data-ready]');
  return { page, problems };
}

async function nextBoard(page) {
  await page.click('#randomize');
  await page.waitForSelector('html[data-ready]');
}

// Each tile's contents, against the tile's own box.
const spills = (page) =>
  page.evaluate(() =>
    [...document.querySelectorAll('.tile')]
      .filter((tile) => tile.scrollHeight > tile.clientHeight + 1 || tile.scrollWidth > tile.clientWidth + 1)
      .map((tile) => `${tile.dataset.tile ?? 'hero'}: ${tile.scrollWidth}x${tile.scrollHeight} in ${tile.clientWidth}x${tile.clientHeight}`),
  );

const shown = (page) => page.$$eval('.tile[data-tile]', (tiles) => tiles.map((tile) => tile.dataset.tile));

// The board as columns, rows, and the cells each tile takes.
const coverage = (page) =>
  page.evaluate(() => {
    const spans = [...document.querySelectorAll('.tile')].map((tile) => {
      const [top, left, bottom, right] = tile.style.gridArea.split('/').map(Number);
      return { left, top, width: right - left, height: bottom - top };
    });
    const columns = getComputedStyle(document.getElementById('board')).gridTemplateColumns.split(' ').length;
    const rows = Math.max(...spans.map(({ top, height }) => top + height - 1));
    const cells = new Set();
    for (const { left, top, width, height } of spans) {
      for (let y = top; y < top + height; y += 1) {
        for (let x = left; x < left + width; x += 1) cells.add(`${x},${y}`);
      }
    }
    return { columns, rows, filled: cells.size, spans };
  });

async function checkBoard(page, found, name) {
  for (const spill of await spills(page)) found.push(`${name}: ${spill}`);
  const { columns, rows, filled, spans } = await coverage(page);
  assert.equal(filled, columns * rows, `${name} leaves a cell empty`);
  assert.ok(spans.every(({ width, height }) => width <= 2 && height <= 2), `${name} has a tile past 2x2`);
  const ids = await shown(page);
  assert.equal(new Set(ids).size, ids.length, `${name} repeats a tile`);
  const sideways = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
  assert.ok(sideways <= 0, `${name} scrolls sideways by ${sideways}`);
  return { columns, rows };
}

test('every deal fills the grid, with one of each thing at most', () => {
  for (let seed = 1; seed < 400; seed += 1) {
    const grid = { columns: 5, rows: 3, cellWidth: 256, cellHeight: 224, gap: 12 };
    const placed = [{ x: 0, y: 0, width: 2, height: 1 }];
    const tiles = deal(POOL, grid, randomFrom(seed), placed);
    const ids = tiles.map(({ item }) => item.id);
    assert.equal(new Set(ids).size, ids.length);
    const area = [...tiles, ...placed].reduce((sum, { width, height }) => sum + width * height, 0);
    assert.equal(area, 15);
    assert.ok(tiles.every(({ width, height }) => width <= 2 && height <= 2));
  }
});

test('a seed always gives the same deal', () => {
  const grid = { columns: 5, rows: 3, cellWidth: 300, cellHeight: 262, gap: 12 };
  const ids = () => deal(POOL, grid, randomFrom(77)).map(({ item }) => item.id);
  assert.deepEqual(ids(), ids());
});

test('the first board is the same on every visit', async (t) => {
  const { page, problems } = await open(t, { viewport: { width: 1440, height: 900 } });
  assert.deepEqual((await shown(page)).sort(), [...OPENING].sort());
  await nextBoard(page);
  await page.reload();
  await page.waitForSelector('html[data-ready]');
  assert.deepEqual((await shown(page)).sort(), [...OPENING].sort());
  assert.deepEqual(problems, []);
  assert.equal(await page.evaluate(() => window.violations), undefined);
});

for (const viewport of WALLS) {
  test(`a full 5 by 3 wall at ${viewport.width} by ${viewport.height}`, async (t) => {
    const { page, problems } = await open(t, { viewport });
    const found = [];
    for (let board = 0; board < BOARDS; board += 1) {
      const { columns, rows } = await checkBoard(page, found, `board ${board}`);
      assert.deepEqual([columns, rows], [5, 3]);
      await nextBoard(page);
    }
    assert.deepEqual(found, []);
    assert.deepEqual(problems, []);
    assert.equal(await page.evaluate(() => window.violations), undefined);
  });
}

for (const viewport of SCROLLING) {
  test(`a full stack at ${viewport.width} by ${viewport.height}`, async (t) => {
    const { page, problems } = await open(t, { viewport });
    const found = [];
    for (let board = 0; board < BOARDS; board += 1) {
      await checkBoard(page, found, `board ${board}`);
      await nextBoard(page);
    }
    assert.deepEqual(found, []);
    assert.deepEqual(problems, []);
  });
}

test('chat bubbles sit on opposite sides', async (t) => {
  const { page } = await open(t, { viewport: { width: 1440, height: 900 } });
  const [theirs, mine] = await page.$$eval('.tile-chat .fd-message', (messages) => messages.map((each) => each.getBoundingClientRect()).map(({ left, right }) => ({ left, right })));
  const log = await page.$eval('.tile-chat', (each) => each.getBoundingClientRect().toJSON());
  assert.ok(Math.abs(theirs.left - log.left) <= 1);
  assert.ok(Math.abs(mine.right - log.right) <= 1);
});

test('the GitHub logo is dithered in both themes', async (t) => {
  const { page } = await open(t, { viewport: { width: 1440, height: 900 } });
  // Ink and bare cells both, or it is a solid mark or an empty one.
  const cells = () =>
    page.evaluate(async () => {
      const logo = [...document.querySelectorAll('.logo')].find((each) => getComputedStyle(each).display !== 'none');
      const canvas = logo.querySelector('canvas');
      const { data } = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height);
      let inked = 0;
      for (let at = 3; at < data.length; at += 4) inked += data[at] > 0 ? 1 : 0;
      return { inked, total: data.length / 4, label: logo.getAttribute('aria-label') };
    });
  const dark = await cells();
  assert.ok(dark.inked > 0 && dark.inked < dark.total);
  await page.click('#dark');
  await page.waitForFunction(() => document.documentElement.dataset.theme === 'light');
  await page.waitForFunction(() => document.querySelector('.logo-light canvas')?.width > 0);
  await page.waitForTimeout(100);
  const light = await cells();
  assert.ok(light.inked > 0 && light.inked < light.total);
});

test('the wall sits centered with room around it', async (t) => {
  const { page } = await open(t, { viewport: { width: 1920, height: 1080 } });
  const box = await page.$eval('#board', (board) => {
    const { left, right, top, bottom } = board.getBoundingClientRect();
    return { left, right: innerWidth - right, top, bottom: innerHeight - bottom };
  });
  assert.ok(Math.abs(box.left - box.right) <= 1);
  assert.ok(Math.abs(box.top - box.bottom) <= 1);
  assert.ok(box.left >= 100 && box.top >= 100);
  assert.equal(await page.evaluate(() => document.documentElement.scrollHeight), 1080);
});

test('randomize changes the board and keeps the short install command', async (t) => {
  const { page, problems } = await open(t, { reducedMotion: 'no-preference', viewport: { width: 1440, height: 900 } });
  const first = await shown(page);
  await nextBoard(page);
  assert.notDeepEqual(await shown(page), first);
  assert.equal(await page.$eval('#hero fd-copy input', (field) => field.value), 'npx fluidither');
  assert.deepEqual(problems, []);
});

test('the background moves, and starts on silk', async (t) => {
  const { page } = await open(t, { reducedMotion: 'no-preference', viewport: { width: 1440, height: 900 } });
  const sky = () =>
    page.$eval('#ground canvas', (canvas) => {
      const { data } = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height);
      let hash = 0;
      for (let at = 3; at < data.length; at += 4) hash = (hash * 31 + data[at]) | 0;
      return hash;
    });
  const first = await sky();
  await page.waitForTimeout(800);
  assert.notEqual(await sky(), first);
  assert.equal(await page.getAttribute('#ground', 'kind'), 'silk');
});

test('one-shot tiles can run their animation again', async (t) => {
  const { page, problems } = await open(t, { viewport: { width: 1440, height: 900 } });
  const before = await page.$('.tile[data-tile=proof] fd-count');
  await page.click('.tile[data-tile=proof] .replay');
  assert.notEqual(await before.evaluate((count) => count.isConnected), true);
  assert.equal(await page.locator('.tile[data-tile=proof] fd-count').count(), 1);
  assert.equal(await page.locator('.tile[data-tile=proof] .replay').count(), 1);
  await page.click('.tile[data-tile=proof] .replay');
  assert.equal(await page.locator('.tile[data-tile=proof] .replay').count(), 1);
  assert.deepEqual(problems, []);
});

test('the docs button sits just before GitHub and leads to the docs', async (t) => {
  const { page } = await open(t, { viewport: { width: 1440, height: 900 } });
  const links = await page.$$eval('.bar-tools a', (each) => each.map((link) => link.getAttribute('href')));
  assert.deepEqual(links, ['docs.html', 'https://github.com/vrnrn/fluidither']);
});

test('the theme switch flips the page and is remembered', async (t) => {
  const { page } = await open(t, { reducedMotion: 'no-preference', viewport: { width: 1440, height: 900 } });
  assert.equal(await page.getAttribute('html', 'data-theme'), 'dark');
  await page.click('#dark');
  await page.waitForFunction(() => document.documentElement.dataset.theme === 'light');
  assert.equal(await page.isChecked('#dark'), false);
  await page.reload();
  await page.waitForSelector('html[data-ready]');
  assert.equal(await page.getAttribute('html', 'data-theme'), 'light');
});

for (const theme of ['dark', 'light']) {
  test(`axe finds nothing on six boards in ${theme}`, async (t) => {
    const { page } = await open(t, { viewport: { width: 1440, height: 900 } });
    if (theme === 'light') {
      await page.click('#dark');
      await page.waitForFunction(() => document.documentElement.dataset.theme === 'light');
      // Colors transition after the flip, and axe must read where they land.
      await page.waitForTimeout(500);
    }
    await page.evaluate(axeSource);
    const found = [];
    for (let board = 0; board < 6; board += 1) {
      const { violations } = await page.evaluate(() => window.axe.run(document, { resultTypes: ['violations'] }));
      for (const { id, nodes } of violations) found.push(`board ${board}: ${id}: ${nodes.map((node) => node.target).join(', ')}`);
      await nextBoard(page);
    }
    assert.deepEqual(found, []);
  });
}
