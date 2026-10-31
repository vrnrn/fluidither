// Labels must sit in the middle of their box. This measures rendered pixels at 4x,
// with cap-height text so descenders do not skew the result, because neither layout
// numbers nor the eye catch a label that is 1px off.
import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright-core';
import { createDemoServer } from '../../demo/serve.mjs';
import { inkOffset, readPng } from '../../../scripts/lib/png.mjs';

const SCALE = 4;
const LIMIT = 0.5; // CSS pixels

let browser;
let server;
let origin;

before(async () => {
  browser = await chromium.launch();
  server = createDemoServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  origin = `http://127.0.0.1:${server.address().port}`;
});

after(async () => {
  await browser.close();
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
});

async function measure(page, selector) {
  const element = await page.$(selector);
  const image = readPng(await element.screenshot());
  const edge = 2 * SCALE;
  const offset = inkOffset(image, { x: edge, y: edge, w: image.width - 2 * edge, h: image.height - 2 * edge });
  return { x: offset.x / SCALE, y: offset.y / SCALE };
}

const within = (offset, label) =>
  assert.ok(
    Math.abs(offset.x) <= LIMIT && Math.abs(offset.y) <= LIMIT,
    `${label} is off center by x ${offset.x.toFixed(2)}px, y ${offset.y.toFixed(2)}px`,
  );

for (const colorScheme of ['light', 'dark']) {
  test(`tab and button labels are centered in ${colorScheme}`, async (t) => {
    const context = await browser.newContext({ colorScheme, deviceScaleFactor: SCALE, viewport: { width: 900, height: 900 } });
    t.after(() => context.close());
    const page = await context.newPage();
    await page.goto(`${origin}/`);
    await page.waitForSelector('#range fd-tab[aria-selected="true"]');
    await page.waitForTimeout(150);

    // Selected, plain, and compact tabs, in both weights.
    await page.$$eval('fd-tab .fd-tab-text', (labels) =>
      labels.forEach((label) => {
        label.textContent = 'HEH';
        label.dataset.text = 'HEH';
      }),
    );
    await page.waitForTimeout(50);
    within(await measure(page, '#range fd-tab[value=week]'), 'selected tab');
    within(await measure(page, '#range fd-tab[value=month]'), 'plain tab');
    within(await measure(page, '[data-density=compact] fd-tab[value=list]'), 'compact selected tab');
    within(await measure(page, '[data-density=compact] fd-tab[value=grid]'), 'compact plain tab');

    await page.$$eval('.demo-row .fd-button', (buttons) => buttons.forEach((button) => (button.textContent = 'HEH')));
    await page.waitForTimeout(50);
    within(await measure(page, '.demo-row:nth-of-type(1) .fd-button:nth-child(1)'), 'solid button');
    within(await measure(page, '.demo-row:nth-of-type(1) .fd-button:nth-child(2)'), 'soft button');
    within(await measure(page, '[data-density=compact].demo-row .fd-button:nth-child(1)'), 'compact button');
  });
}

test('every real tab label is horizontally centered', async (t) => {
  const context = await browser.newContext({ deviceScaleFactor: SCALE, viewport: { width: 900, height: 900 } });
  t.after(() => context.close());
  const page = await context.newPage();
  await page.goto(`${origin}/`);
  await page.waitForSelector('#range fd-tab[aria-selected="true"]');
  await page.waitForTimeout(150);
  // Real words are centered by advance width, so a letter's side bearing can add
  // a little here. The strict limit above uses symmetric letters.
  for (const value of ['day', 'week', 'month', 'all']) {
    const { x } = await measure(page, `#range fd-tab[value=${value}]`);
    assert.ok(Math.abs(x) <= 1, `${value}: x ${x.toFixed(2)}px`);
  }
});
