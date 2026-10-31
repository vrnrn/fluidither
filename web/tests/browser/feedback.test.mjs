import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { chromium } from 'playwright-core';
import { createDemoServer } from '../../demo/serve.mjs';

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

async function open(t, options = {}) {
  const context = await browser.newContext({ viewport: { width: 1000, height: 900 }, ...options });
  t.after(() => context.close());
  const page = await context.newPage();
  await page.goto(`${origin}/`);
  await page.waitForSelector('fd-sparkline canvas');
  await page.$eval('#feedback', (section) => section.scrollIntoView());
  await page.waitForTimeout(100);
  return page;
}

const inked = (page) =>
  page.$eval('fd-sparkline canvas', (canvas) => {
    const { data } = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height);
    let count = 0;
    for (let i = 3; i < data.length; i += 4) if (data[i] > 0) count += 1;
    return { count, total: data.length / 4 };
  });

test('a sparkline draws, and redraws when its values change', async (t) => {
  const page = await open(t);
  const before = await inked(page);
  assert.ok(before.count > 0 && before.count < before.total, JSON.stringify(before));
  await page.$eval('fd-sparkline', (each) => each.setAttribute('values', '9 1 9 1 9 1'));
  await page.waitForTimeout(100);
  assert.notEqual((await inked(page)).count, before.count);
});

test('a sparkline is an image with a label', async (t) => {
  const page = await open(t);
  const facts = await page.$eval('fd-sparkline', (each) => [each.getAttribute('role'), Boolean(each.getAttribute('aria-label'))]);
  assert.deepEqual(facts, ['img', true]);
});

test('a flat or empty sparkline does not break', async (t) => {
  const page = await open(t);
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.$eval('fd-sparkline', (each) => each.setAttribute('values', '5 5 5'));
  await page.$eval('fd-sparkline', (each) => each.setAttribute('values', ''));
  await page.$eval('fd-sparkline', (each) => each.setAttribute('values', '7'));
  await page.waitForTimeout(100);
  assert.deepEqual(errors, []);
});

test('reduced motion stills the loader and skeleton', async (t) => {
  const page = await open(t, { reducedMotion: 'reduce' });
  const names = await page.$$eval('.fd-loader, .fd-skeleton', (each) => each.map((element) => getComputedStyle(element).animationName));
  assert.ok(names.every((name) => name === 'none'), names.join());
});

test('progress stays a real progress bar and its value drives the fill', async (t) => {
  const page = await open(t);
  const facts = await page.$eval('.fd-progress', (bar) => [bar.tagName, bar.position]);
  assert.deepEqual(facts, ['PROGRESS', 0.62]);
});

test('axe finds nothing here, in light or dark', async (t) => {
  for (const colorScheme of ['light', 'dark']) {
    const page = await open(t, { colorScheme });
    await page.context().addInitScript(axeSource);
    await page.reload();
    await page.waitForFunction(() => window.axe && document.querySelector('fd-sparkline canvas'));
    const violations = await page.evaluate(async () => {
      const { violations } = await window.axe.run('#feedback', { runOnly: ['wcag2a', 'wcag2aa', 'wcag21aa', 'best-practice'] });
      return violations.map(({ id, nodes }) => `${id}: ${nodes.map((node) => node.target).join(' ')}`);
    });
    assert.deepEqual(violations, [], colorScheme);
  }
});
