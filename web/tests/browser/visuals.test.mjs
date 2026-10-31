// Charts, fields, text effects and image dither in real Chromium, strict CSP.
import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { chromium } from 'playwright-core';
import { createDemoServer } from '../../demo/serve.mjs';
import { FIELDS } from '../../src/core/fields.js';

const axeSource = readFileSync(createRequire(import.meta.url).resolve('axe-core/axe.min.js'), 'utf8');

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

async function open(t, options = {}, section = '#charts') {
  const context = await browser.newContext({ viewport: { width: 1000, height: 900 }, ...options });
  t.after(() => context.close());
  const page = await context.newPage();
  const problems = [];
  page.on('pageerror', (error) => problems.push(error.message));
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => (window.violations ??= []).push(`${event.violatedDirective} ${event.blockedURI}`));
  });
  await page.goto(`${origin}/`);
  await page.waitForSelector('fd-chart canvas');
  await page.$eval(section, (each) => each.scrollIntoView());
  await page.waitForTimeout(200);
  t.after(async () => {
    const violations = await page.evaluate(() => window.violations ?? []).catch(() => []);
    assert.deepEqual([...problems, ...violations], []);
  });
  return page;
}

const inked = (page, selector) =>
  page.$eval(`${selector} canvas`, (canvas) => {
    const { data } = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height);
    const colors = new Set();
    let count = 0;
    for (let i = 0; i < data.length; i += 4) {
      if (data[i + 3] > 0) {
        count += 1;
        colors.add(`${data[i]},${data[i + 1]},${data[i + 2]}`);
      }
    }
    return { count, colors: colors.size, total: data.length / 4 };
  });

for (const kind of ['bar', 'area', 'line', 'donut', 'radar', 'heatmap', 'gauge']) {
  test(`${kind} chart draws`, async (t) => {
    const page = await open(t);
    const { count, total } = await inked(page, `fd-chart[kind=${kind}]`);
    assert.ok(count > total * 0.01 && count < total, `${count} of ${total}`);
  });
}

test('each series gets its own color and a legend entry', async (t) => {
  const page = await open(t);
  const { colors } = await inked(page, '#chart-bar');
  assert.equal(colors, 2);
  assert.deepEqual(await page.$$eval('#chart-bar .fd-chart-legend span', (each) => each.map((entry) => entry.textContent)), ['Sales', 'Costs']);
});

test('a hidden table carries the numbers for screen readers', async (t) => {
  const page = await open(t);
  const rows = await page.$$eval('#chart-bar .fd-chart-data tr', (each) => each.map((row) => row.textContent));
  assert.deepEqual(rows, ['MonTueWedThuFri', 'Sales37295', 'Costs51647']);
});

test('changing the data redraws the chart, the legend and the table', async (t) => {
  const page = await open(t);
  const before = (await inked(page, '#chart-bar')).count;
  await page.$eval('#chart-bar', (chart) => chart.setAttribute('series', '9 9 9 9 9; 1 1 1 1 1'));
  await page.waitForTimeout(50);
  assert.notEqual((await inked(page, '#chart-bar')).count, before);
  assert.ok((await page.textContent('#chart-bar .fd-chart-data')).includes('99999'));
});

test('charts follow the theme', async (t) => {
  const page = await open(t, { colorScheme: 'light' });
  const ink = () => page.$eval('fd-chart[kind=gauge] canvas', (canvas) => {
    const { data } = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height);
    for (let i = 0; i < data.length; i += 4) if (data[i + 3] > 0) return data[i];
  });
  const light = await ink();
  await page.click('#theme');
  await page.waitForTimeout(100);
  assert.notEqual(await ink(), light);
});

for (const kind of FIELDS) {
  test(`${kind} field draws and moves`, async (t) => {
    const page = await open(t, {}, '#fields');
    const frame = () => page.$eval(`fd-field[kind=${kind}] canvas`, (canvas) => canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data.join());
    const first = await frame();
    assert.ok(first.includes('255'));
    await page.waitForTimeout(900);
    assert.notEqual(await frame(), first);
  });
}

test('fields hold still under reduced motion', async (t) => {
  const page = await open(t, { reducedMotion: 'reduce' }, '#fields');
  const frame = () => page.$eval('fd-field[kind=plasma] canvas', (canvas) => canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data.join());
  const first = await frame();
  await page.waitForTimeout(500);
  assert.equal(await frame(), first);
});

test('scramble settles on the real text, which stays available throughout', async (t) => {
  const page = await open(t, {}, '#effects');
  await page.$eval('fd-scramble', (each) => each.run());
  assert.equal(await page.getAttribute('fd-scramble', 'aria-label'), 'Scrambled into place');
  await page.waitForTimeout(900);
  assert.equal(await page.textContent('fd-scramble'), 'Scrambled into place');
});

test('dissolve clears its veil and leaves plain text', async (t) => {
  const page = await open(t, {}, '#effects');
  const dissolve = page.locator('#effects fd-dissolve');
  await dissolve.evaluate((each) => each.run());
  await page.waitForTimeout(150);
  assert.equal(await dissolve.locator('.fd-dissolve-veil').count(), 1);
  await dissolve.locator('.fd-dissolve-veil').waitFor({ state: 'detached', timeout: 2000 });
  assert.equal(await dissolve.textContent(), 'Dissolved into view');
});

test('a count springs to its value and retargets', async (t) => {
  const page = await open(t, {}, '#effects');
  const settlesOn = (text) =>
    page.waitForFunction((text) => document.querySelector('#count').textContent === text, text, { timeout: 3000 });
  await settlesOn(new Intl.NumberFormat().format(1204));
  await page.$eval('#count', (each) => each.setAttribute('value', '50'));
  await settlesOn('50');
});

test('an image redraws in dots with its alt text as the label', async (t) => {
  const page = await open(t, {}, '#effects');
  await page.waitForFunction(() => {
    const canvas = document.querySelector('fd-dither-image canvas');
    return canvas.width > 0 && canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data.some(Boolean);
  });
  const { count, total } = await inked(page, 'fd-dither-image');
  assert.ok(count > total * 0.1 && count < total * 0.9, `${count} of ${total}`);
  assert.equal(await page.getAttribute('fd-dither-image', 'aria-label'), 'A soft circle on a gradient');
});

test('an image stays a positive in the dark theme', async (t) => {
  const page = await open(t, { colorScheme: 'dark' }, '#effects');
  await page.waitForFunction(() => {
    const canvas = document.querySelector('fd-dither-image canvas');
    return canvas.width > 0 && canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data.some(Boolean);
  });
  // The picture is a dark circle on a light ground, so in light ink the middle is sparse.
  const share = await page.$eval('fd-dither-image canvas', (canvas) => {
    const inked = (x, y, size) => {
      const { data } = canvas.getContext('2d').getImageData(x, y, size, size);
      let count = 0;
      for (let i = 3; i < data.length; i += 4) if (data[i] > 0) count += 1;
      return count / (size * size);
    };
    const size = Math.floor(canvas.height / 4);
    return { middle: inked(Math.floor(canvas.width / 2 - size / 2), Math.floor(canvas.height / 2 - size / 2), size), corner: inked(0, 0, size) };
  });
  assert.ok(share.middle < share.corner, JSON.stringify(share));
});

test('axe finds nothing in these sections', async (t) => {
  for (const colorScheme of ['light', 'dark']) {
    const page = await open(t, { colorScheme });
    await page.context().addInitScript(axeSource);
    await page.reload();
    await page.waitForFunction(() => window.axe && document.querySelector('fd-chart canvas'));
    await page.waitForTimeout(900);
    for (const section of ['#charts', '#fields', '#effects']) {
      const violations = await page.evaluate(async (target) => {
        const { violations } = await window.axe.run(target, { runOnly: ['wcag2a', 'wcag2aa', 'wcag21aa'] });
        return violations.map(({ id, nodes }) => `${id}: ${nodes.map((node) => node.target).join(' ')}`);
      }, section);
      assert.deepEqual(violations, [], `${colorScheme} ${section}`);
    }
  }
});
