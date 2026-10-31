// Real Chromium against the demo page, served with the strict CSP.
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

// Opens the demo and fails the test on any console error or CSP violation.
async function open(t, options = {}) {
  const context = await browser.newContext({ viewport: { width: 1000, height: 900 }, ...options });
  t.after(() => context.close());
  const page = await context.newPage();
  const problems = [];
  page.on('pageerror', (error) => problems.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') problems.push(message.text());
  });
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => {
      window.violations ??= [];
      window.violations.push(`${event.violatedDirective}: ${event.blockedURI}`);
    });
  });
  await page.goto(`${origin}/`);
  await page.waitForFunction(() => document.querySelector('fd-tab[aria-selected="true"]'));
  t.after(async () => {
    const violations = (await page.evaluate(() => window.violations ?? []).catch(() => [])) ?? [];
    assert.deepEqual([...problems, ...violations], []);
  });
  return page;
}

const selected = (page) => page.$eval('#range fd-tab[aria-selected="true"]', (tab) => tab.getAttribute('value'));
const focused = (page) => page.evaluate(() => document.activeElement.getAttribute('value'));
const visiblePanel = (page) =>
  page.$$eval('#range fd-tab-panel', (panels) => panels.filter((panel) => !panel.hidden).map((panel) => panel.getAttribute('value')));

async function focusRange(page) {
  await page.keyboard.press('Tab'); // theme button
  await page.keyboard.press('Tab');
  while ((await page.evaluate(() => document.activeElement.tagName)) !== 'FD-TAB') {
    await page.keyboard.press('Tab');
  }
}

test('tabs expose the ARIA pattern', async (t) => {
  const page = await open(t);
  const shape = await page.evaluate(() => {
    const list = document.querySelector('#range fd-tab-list');
    const tabs = [...list.querySelectorAll('fd-tab')];
    const panels = [...document.querySelectorAll('#range fd-tab-panel')];
    return {
      role: list.getAttribute('role'),
      label: list.getAttribute('aria-label'),
      roles: new Set(tabs.map((tab) => tab.getAttribute('role'))).size,
      tabbable: tabs.filter((tab) => tab.getAttribute('tabindex') === '0').map((tab) => tab.getAttribute('value')),
      linked: tabs.every((tab) => {
        const panel = document.getElementById(tab.getAttribute('aria-controls'));
        return panel?.getAttribute('aria-labelledby') === tab.id && panel.getAttribute('role') === 'tabpanel';
      }),
      disabled: list.querySelector('[value=year]').getAttribute('aria-disabled'),
      panelsTabbable: panels.every((panel) => panel.getAttribute('tabindex') === '0'),
    };
  });
  assert.deepEqual(shape, {
    role: 'tablist',
    label: 'Date range',
    roles: 1,
    tabbable: ['week'],
    linked: true,
    disabled: 'true',
    panelsTabbable: true,
  });
});

test('arrow keys, Home and End select, skip disabled tabs and wrap', async (t) => {
  const page = await open(t);
  await focusRange(page);
  assert.equal(await focused(page), 'week');
  await page.keyboard.press('ArrowRight');
  assert.equal(await selected(page), 'month');
  assert.deepEqual(await visiblePanel(page), ['month']);
  await page.keyboard.press('ArrowRight');
  assert.equal(await selected(page), 'all', 'skips disabled Year');
  await page.keyboard.press('ArrowRight');
  assert.equal(await selected(page), 'day', 'wraps forward');
  await page.keyboard.press('ArrowLeft');
  assert.equal(await selected(page), 'all', 'wraps backward');
  await page.keyboard.press('Home');
  assert.equal(await selected(page), 'day');
  await page.keyboard.press('End');
  assert.equal(await selected(page), 'all');
  assert.equal(await focused(page), 'all');
});

test('Tab leaves the list and lands on the panel', async (t) => {
  const page = await open(t);
  await focusRange(page);
  await page.keyboard.press('Tab');
  assert.equal(await page.evaluate(() => document.activeElement.tagName), 'FD-TAB-PANEL');
});

test('in a right-to-left page the arrow keys swap', async (t) => {
  const page = await open(t);
  await page.evaluate(() => {
    document.documentElement.dir = 'rtl';
  });
  await focusRange(page);
  await page.keyboard.press('ArrowLeft');
  assert.equal(await selected(page), 'month');
});

test('clicking selects, disabled tabs do nothing, and change fires once', async (t) => {
  const page = await open(t);
  await page.evaluate(() => {
    window.changes = [];
    document.querySelector('#range').addEventListener('change', (event) => window.changes.push(event.detail));
  });
  await page.click('#range fd-tab[value=day]');
  await page.click('#range fd-tab[value=year]', { force: true });
  assert.equal(await selected(page), 'day');
  assert.deepEqual(await page.evaluate(() => window.changes), [{ value: 'day', index: 0 }]);
});

test('setting value from code selects without firing change', async (t) => {
  const page = await open(t);
  await page.evaluate(() => {
    window.changes = 0;
    const tabs = document.querySelector('#range');
    tabs.addEventListener('change', () => (window.changes += 1));
    tabs.value = 'all';
  });
  await page.waitForFunction(() => document.querySelector('#range fd-tab[value=all]').getAttribute('aria-selected') === 'true');
  assert.equal(await page.evaluate(() => window.changes), 0);
});

test('tabs added later join in', async (t) => {
  const page = await open(t);
  await page.evaluate(() => {
    const tab = document.createElement('fd-tab');
    tab.setAttribute('value', 'new');
    tab.textContent = 'New';
    document.querySelector('#range fd-tab-list').append(tab);
  });
  await page.waitForSelector('#range fd-tab[value=new][role=tab]');
  await page.click('#range fd-tab[value=new]');
  assert.equal(await selected(page), 'new');
});

const listVars = (page) =>
  page.$eval('#range fd-tab-list', (list) => ({
    thumbX: Number.parseFloat(list.style.getPropertyValue('--_thumb-x')),
    thumbWidth: Number.parseFloat(list.style.getPropertyValue('--_thumb-width')),
    peekX: Number.parseFloat(list.style.getPropertyValue('--_peek-x')),
    peekOpacity: Number.parseFloat(list.style.getPropertyValue('--_peek-opacity')) || 0,
  }));
const tabBox = (page, value) =>
  page.$eval(`#range fd-tab[value=${value}]`, (tab) => ({ x: tab.offsetLeft, width: tab.offsetWidth }));

test('the thumb lands on the selected tab', async (t) => {
  const page = await open(t);
  await page.click('#range fd-tab[value=month]');
  const tab = await tabBox(page, 'month');
  const landed = async () => {
    const { thumbX, thumbWidth } = await listVars(page);
    return thumbX === tab.x && thumbWidth === tab.width;
  };
  for (let tries = 0; tries < 60 && !(await landed()); tries += 1) await page.waitForTimeout(50);
  assert.ok(await landed(), JSON.stringify(await listVars(page)));
});

test('a tab keeps its width when its weight changes', async (t) => {
  const page = await open(t);
  const widths = () => page.$$eval('#range fd-tab', (tabs) => tabs.map((tab) => tab.offsetWidth));
  const before = await widths();
  await page.click('#range fd-tab[value=month]');
  await page.waitForTimeout(250);
  assert.deepEqual(await widths(), before);
});

test('the peek fades in on the hovered tab, glides, and fades out in place', async (t) => {
  const page = await open(t);
  assert.equal((await listVars(page)).peekOpacity, 0);
  await page.hover('#range fd-tab[value=day]');
  // It appears on the tab, without sliding over from the selection.
  assert.equal((await listVars(page)).peekX, (await tabBox(page, 'day')).x);
  await page.waitForTimeout(250);
  assert.equal((await listVars(page)).peekOpacity, 1);
  await page.hover('#range fd-tab[value=month]');
  await page.waitForTimeout(250);
  assert.equal((await listVars(page)).peekX, (await tabBox(page, 'month')).x);
  await page.hover('h1');
  await page.waitForTimeout(250);
  const after = await listVars(page);
  assert.equal(after.peekOpacity, 0);
  assert.equal(after.peekX, (await tabBox(page, 'month')).x);
});

test('the peek stays off the selected and disabled tabs', async (t) => {
  const page = await open(t);
  await page.hover('#range fd-tab[value=week]');
  await page.waitForTimeout(150);
  assert.equal((await listVars(page)).peekOpacity, 0);
  await page.hover('#range fd-tab[value=year]', { force: true });
  await page.waitForTimeout(150);
  assert.equal((await listVars(page)).peekOpacity, 0);
});

test('the focus ring shows for the keyboard and not for the mouse', async (t) => {
  const page = await open(t);
  const outline = () => page.evaluate(() => getComputedStyle(document.activeElement).outlineStyle);
  await page.click('#range fd-tab[value=month]');
  assert.equal(await outline(), 'none');
  await page.keyboard.press('ArrowRight');
  assert.equal(await outline(), 'solid');
});

test('a page at rest requests no animation frames from the tabs', async (t) => {
  const page = await open(t);
  await page.evaluate(() => document.querySelector('#band').remove());
  await page.evaluate(() => document.querySelectorAll('fd-dither-gradient').forEach((each) => each.remove()));
  await page.click('#range fd-tab[value=day]');
  await page.mouse.move(1, 1);
  await page.waitForTimeout(500);
  const frames = await page.evaluate(
    () =>
      new Promise((resolve) => {
        let count = 0;
        const original = window.requestAnimationFrame;
        window.requestAnimationFrame = (callback) => {
          count += 1;
          return original.call(window, callback);
        };
        setTimeout(() => resolve(count), 300);
      }),
  );
  assert.equal(frames, 0);
});

test('reduced motion: springs jump and the gradient draws one frame', async (t) => {
  const page = await open(t, { reducedMotion: 'reduce' });
  await page.click('#range fd-tab[value=month]');
  assert.equal((await listVars(page)).thumbX, (await tabBox(page, 'month')).x);
  const pixels = () =>
    page.$eval('#band canvas', (canvas) => canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data.join());
  const first = await pixels();
  await page.waitForTimeout(300);
  assert.equal(await pixels(), first);
});

test('the gradient animates, and stops when scrolled offscreen', async (t) => {
  const page = await open(t);
  const hash = () =>
    page.$eval('#band canvas', (canvas) => canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data.join());
  const first = await hash();
  await page.waitForTimeout(2000);
  assert.notEqual(await hash(), first, 'frames change while visible');
  await page.evaluate(() => {
    document.querySelector('#band').style.marginBottom = '4000px';
    window.scrollTo(0, 3000);
  });
  await page.waitForTimeout(400);
  const parked = await hash();
  await page.waitForTimeout(2000);
  assert.equal(await hash(), parked, 'no frames while offscreen');
});

test('the gradient follows the theme and the ink color', async (t) => {
  const page = await open(t, { colorScheme: 'light' });
  const inkOf = (selector) =>
    page.$eval(selector, (element) => {
      const canvas = element.querySelector('canvas');
      const { data } = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height);
      for (let i = 0; i < data.length; i += 4) if (data[i + 3] > 0) return [data[i], data[i + 1], data[i + 2]];
      return null;
    });
  assert.deepEqual(await inkOf('.demo-tile'), [0x1b, 0x1a, 0x17]);
  await page.click('#theme');
  await page.waitForTimeout(100);
  assert.deepEqual(await inkOf('.demo-tile'), [0xf3, 0xf1, 0xea]);
  const orange = await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--fd-orange'));
  const hex = orange.trim().slice(1);
  assert.deepEqual(await inkOf('.demo-tile-accent'), [0, 2, 4].map((at) => Number.parseInt(hex.slice(at, at + 2), 16)));
});

test('cells stay whole device pixels at a fractional scale', async (t) => {
  const page = await open(t, { deviceScaleFactor: 1.5 });
  const { width, columns } = await page.$eval('#band canvas', (canvas) => ({
    width: Number.parseFloat(canvas.style.width),
    columns: canvas.width,
  }));
  const device = (width * 1.5) / columns;
  assert.equal(device, Math.round(device));
});

test('a button is a real button: keyboard, disabled and link forms work', async (t) => {
  const page = await open(t);
  const facts = await page.evaluate(() => {
    const disabled = document.querySelector('.fd-button[disabled]:not([aria-busy])');
    const loading = document.querySelector('.fd-button[aria-busy]');
    return { disabled: disabled.matches(':disabled'), busy: loading.matches(':disabled') };
  });
  assert.deepEqual(facts, { disabled: true, busy: true });
  await page.focus('#theme');
  const before = await page.evaluate(() => document.documentElement.dataset.theme ?? '');
  await page.keyboard.press('Enter');
  assert.notEqual(await page.evaluate(() => document.documentElement.dataset.theme ?? ''), before);
});

test('pressing moves a button down one pixel and releasing brings it back', async (t) => {
  const page = await open(t);
  const settlesOn = (value) =>
    page.waitForFunction((value) => getComputedStyle(document.querySelector('.fd-button')).translate === value, value, { timeout: 2000 });
  const box = await (await page.$('.fd-button')).boundingBox();
  await page.mouse.move(box.x + 10, box.y + 10);
  await page.mouse.down();
  await settlesOn('0px 1px');
  await page.mouse.up();
  await settlesOn('none');
});

test('axe finds no accessibility problems in light or dark', async (t) => {
  for (const colorScheme of ['light', 'dark']) {
    const page = await open(t, { colorScheme });
    const context = page.context();
    await context.addInitScript(axeSource);
    await page.reload();
    await page.waitForFunction(() => window.axe && document.querySelector('fd-tab[aria-selected="true"]'));
    const violations = await page.evaluate(async () => {
      const { violations } = await window.axe.run(document, { runOnly: ['wcag2a', 'wcag2aa', 'wcag21aa', 'best-practice'] });
      return violations.map(({ id, nodes }) => `${id}: ${nodes.map((node) => node.target).join(' ')}`);
    });
    assert.deepEqual(violations, [], colorScheme);
  }
});
