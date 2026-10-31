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
  const problems = [];
  page.on('pageerror', (error) => problems.push(error.message));
  await page.goto(`${origin}/`);
  await page.waitForSelector('#calendar table');
  await page.$eval('#special', (each) => each.scrollIntoView());
  t.after(() => assert.deepEqual(problems, []));
  return page;
}

const focusedDate = (page) => page.evaluate(() => document.activeElement.dataset.date);

test('calendar keys move by day, week and month, and Enter chooses', async (t) => {
  const page = await open(t);
  await page.focus('#calendar button[tabindex="0"]');
  assert.equal(await focusedDate(page), '2026-10-14');
  await page.keyboard.press('ArrowRight');
  assert.equal(await focusedDate(page), '2026-10-15');
  await page.keyboard.press('ArrowDown');
  assert.equal(await focusedDate(page), '2026-10-22');
  await page.keyboard.press('Home');
  assert.equal(await focusedDate(page), '2026-10-19', 'Monday');
  await page.keyboard.press('End');
  assert.equal(await focusedDate(page), '2026-10-25', 'Sunday');
  await page.keyboard.press('PageDown');
  assert.equal(await focusedDate(page), '2026-11-25');
  await page.keyboard.press('Enter');
  assert.equal(await page.$eval('#calendar', (each) => each.value), '2026-11-25');
  assert.equal(await page.$eval('#calendar [aria-selected="true"] button', (each) => each.dataset.date), '2026-11-25');
});

test('calendar arrows cross into the next month', async (t) => {
  const page = await open(t);
  await page.click('#calendar button[data-date="2026-10-31"]');
  await page.focus('#calendar button[tabindex="0"]');
  await page.keyboard.press('ArrowRight');
  assert.equal(await focusedDate(page), '2026-11-01');
  assert.ok((await page.textContent('.fd-calendar-title')).includes('2026'));
});

test('min and max block days and keys', async (t) => {
  const page = await open(t);
  await page.$eval('#calendar', (each) => {
    each.setAttribute('min', '2026-10-10');
    each.setAttribute('max', '2026-10-20');
  });
  assert.equal(await page.$eval('#calendar button[data-date="2026-10-09"]', (each) => each.disabled), true);
  await page.focus('#calendar button[tabindex="0"]');
  await page.keyboard.press('PageDown');
  assert.equal(await focusedDate(page), '2026-10-14', 'stays in range');
});

test('date picker opens on its date, picks one, closes, and joins forms', async (t) => {
  const page = await open(t);
  const panel = '#due .fd-date-panel';
  const open_ = () => page.$eval(panel, (each) => each.matches(':popover-open') && getComputedStyle(each).display !== 'none');
  assert.equal(await open_(), false);
  await page.click('#due .fd-select-trigger');
  assert.equal(await open_(), true);
  assert.equal(await focusedDate(page), '2026-10-30');
  await page.keyboard.press('ArrowLeft');
  await page.keyboard.press('Enter');
  assert.equal(await open_(), false);
  assert.equal(await page.$eval('#due', (each) => each.value), '2026-10-29');
  assert.equal(await page.evaluate(() => document.activeElement.className), 'fd-select-trigger');
  const values = await page.evaluate(() => {
    const form = document.createElement('form');
    document.body.append(form);
    form.append(document.querySelector('#due'));
    return Object.fromEntries(new FormData(form));
  });
  assert.deepEqual(values, { due: '2026-10-29' });
});

test('Escape closes the date picker without changing it', async (t) => {
  const page = await open(t);
  await page.click('#due .fd-select-trigger');
  await page.keyboard.press('ArrowLeft');
  await page.keyboard.press('Escape');
  assert.equal(await page.$eval('#due', (each) => each.value), '2026-10-30');
  assert.equal(await page.evaluate(() => document.activeElement.className), 'fd-select-trigger');
});

test('swatches are radios, and arrow keys change the value', async (t) => {
  const page = await open(t);
  await page.focus('#accent input:checked');
  await page.keyboard.press('ArrowRight');
  assert.equal(await page.$eval('#accent', (each) => each.value), '#7C5AC2');
});

test('copy writes to the clipboard and says so', async (t) => {
  const page = await open(t, { permissions: ['clipboard-read', 'clipboard-write'] });
  await page.click('fd-copy button');
  await page.waitForFunction(() => document.querySelector('fd-copy button').textContent === 'Copied');
  assert.equal(await page.evaluate(() => navigator.clipboard.readText()), 'npm install fluidither');
});

test('drawer slides in as a real modal and closes on Escape', async (t) => {
  const page = await open(t);
  await page.click('#open-drawer');
  await page.waitForTimeout(400);
  const box = await page.$eval('#drawer', (each) => each.getBoundingClientRect().toJSON());
  assert.ok(Math.abs(box.right - 1000) < 1, JSON.stringify(box));
  await page.keyboard.press('Escape');
  await page.waitForTimeout(300);
  assert.equal(await page.$eval('#drawer', (each) => each.open), false);
  assert.equal(await page.evaluate(() => document.activeElement.id), 'open-drawer');
});

test('axe finds nothing here, in light or dark', async (t) => {
  for (const colorScheme of ['light', 'dark']) {
    const page = await open(t, { colorScheme });
    await page.context().addInitScript(axeSource);
    await page.reload();
    await page.waitForFunction(() => window.axe && document.querySelector('#calendar table'));
    const violations = await page.evaluate(async () => {
      const { violations } = await window.axe.run('#special', { runOnly: ['wcag2a', 'wcag2aa', 'wcag21aa'] });
      return violations.map(({ id, nodes }) => `${id}: ${nodes.map((node) => node.target).join(' ')}`);
    });
    assert.deepEqual(violations, [], colorScheme);
  }
});
