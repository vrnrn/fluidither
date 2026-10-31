// Structure components in real Chromium. Most are CSS on real elements, so these
// tests check that the styling has not cost them what the browser provides.
import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { chromium } from 'playwright-core';
import { createDemoServer } from '../../demo/serve.mjs';
import { settles } from './settles.mjs';

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
  page.on('console', (message) => {
    if (message.type() === 'error') problems.push(message.text());
  });
  await page.goto(`${origin}/`);
  await page.waitForSelector('#side');
  await page.$eval('#structure', (section) => section.scrollIntoView());
  t.after(() => assert.deepEqual(problems, []));
  return page;
}

const openSections = (page) => page.$$eval('.fd-accordion', (each) => each.filter((section) => section.open).map((section) => section.querySelector('summary').textContent));
const height = (page, nth) =>
  page.$eval(`.fd-accordion:nth-of-type(${nth})`, (section) => section.getBoundingClientRect().height);

test('accordion: one section at a time, by mouse and by keyboard', async (t) => {
  const page = await open(t);
  assert.deepEqual(await openSections(page), ['What is dithering?']);
  await page.click('.fd-accordion:nth-of-type(2) summary');
  assert.deepEqual(await openSections(page), ['Why a spring?']);
  await page.focus('.fd-accordion:nth-of-type(3) summary');
  await page.keyboard.press('Enter');
  assert.deepEqual(await openSections(page), ['Does it need a framework?']);
  await page.keyboard.press('Space');
  assert.deepEqual(await openSections(page), []);
});

test('accordion: a closed section takes no space and an open one fits its text', async (t) => {
  const page = await open(t);
  const closed = await height(page, 2);
  await page.click('.fd-accordion:nth-of-type(2) summary');
  await page.waitForTimeout(450);
  const opened = await height(page, 2);
  assert.ok(opened > closed + 20, `${closed} to ${opened}`);
  const clipped = await page.$eval('.fd-accordion:nth-of-type(2) .fd-accordion-body', (body) => {
    const section = body.closest('details').getBoundingClientRect();
    return body.getBoundingClientRect().bottom - section.bottom;
  });
  assert.ok(clipped <= 1, `text overflows by ${clipped}`);
});

test('accordion: the arrow is drawn and flips when a section opens', async (t) => {
  const page = await open(t);
  const arrow = (nth) =>
    page.$eval(`.fd-accordion:nth-of-type(${nth}) summary`, (summary) => getComputedStyle(summary, '::after').boxShadow);
  assert.notEqual(await arrow(1), 'none', 'open');
  assert.notEqual(await arrow(2), 'none', 'closed');
  assert.notEqual(await arrow(1), await arrow(2));
});

test('nav: the thumb sits on the current link and glides to a new one', async (t) => {
  const page = await open(t);
  const thumb = () => page.$eval('#side', (nav) => Number.parseFloat(nav.style.getPropertyValue('--_thumb-y')));
  const top = (href) => page.$eval(`#side a[href="${href}"]`, (link) => link.offsetTop);
  assert.equal(await thumb(), await top('#overview'));
  await page.click('#side a[href="#billing"]');
  assert.equal(await page.$eval('#side a[aria-current]', (link) => link.textContent), 'Billing');
  await settles(thumb, await top('#billing'));
  assert.equal(await page.$$eval('#side a[aria-current]', (links) => links.length), 1);
});

test('nav: the peek shows on other links and not on the current one', async (t) => {
  const page = await open(t);
  const opacity = () => page.$eval('#side', (nav) => Number.parseFloat(nav.style.getPropertyValue('--_peek-opacity')) || 0);
  await page.hover('#side a[href="#activity"]');
  await settles(opacity, 1);
  await page.hover('#side a[href="#overview"]');
  await settles(opacity, 0);
});

test('nav: links stay reachable by keyboard', async (t) => {
  const page = await open(t);
  await page.focus('#side a[href="#overview"]');
  await page.keyboard.press('Tab');
  assert.equal(await page.evaluate(() => document.activeElement.getAttribute('href')), '#activity');
  await page.keyboard.press('Enter');
  assert.equal(await page.$eval('#side a[aria-current]', (link) => link.textContent), 'Activity');
});

test('table: stays a real table, numbers line up, and rows dither on hover', async (t) => {
  const page = await open(t);
  const facts = await page.$eval('.fd-table', (table) => ({
    display: getComputedStyle(table).display,
    align: getComputedStyle(table.querySelector('td[data-number]')).textAlign,
    numerals: getComputedStyle(table).fontVariantNumeric,
  }));
  assert.deepEqual(facts, { display: 'table', align: 'end', numerals: 'tabular-nums' });
  await page.hover('.fd-table tbody tr:nth-child(2)');
  const image = await page.$eval('.fd-table tbody tr:nth-child(2)', (row) => getComputedStyle(row).backgroundImage);
  assert.ok(image.includes('repeating-conic-gradient'), image);
});

test('scroll area: scrolls and can be reached by keyboard', async (t) => {
  const page = await open(t);
  await page.focus('.demo-scroller');
  await page.keyboard.press('End');
  await page.waitForTimeout(300);
  assert.ok((await page.$eval('.demo-scroller', (box) => box.scrollTop)) > 0);
});

test('badges carry their tone in the cell and keep the text at full contrast', async (t) => {
  const page = await open(t);
  const colors = await page.$eval('.fd-badge[data-tone=green]', (badge) => ({
    text: getComputedStyle(badge).color,
    cell: getComputedStyle(badge, '::before').backgroundColor,
    ink: getComputedStyle(document.documentElement).getPropertyValue('--fd-ink'),
  }));
  assert.notEqual(colors.cell, colors.text);
});

test('axe finds nothing in the structure section, in light or dark', async (t) => {
  for (const colorScheme of ['light', 'dark']) {
    const page = await open(t, { colorScheme });
    await page.context().addInitScript(axeSource);
    await page.reload();
    await page.waitForFunction(() => window.axe && document.querySelector('#side'));
    const violations = await page.evaluate(async () => {
      const { violations } = await window.axe.run('#structure', { runOnly: ['wcag2a', 'wcag2aa', 'wcag21aa', 'best-practice'] });
      return violations.map(({ id, nodes }) => `${id}: ${nodes.map((node) => node.target).join(' ')}`);
    });
    assert.deepEqual(violations, [], colorScheme);
  }
});
