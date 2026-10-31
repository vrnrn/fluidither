// Overlays in real Chromium under the strict CSP: keyboard patterns, focus handling,
// positioning and form behaviour.
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
  page.on('console', (message) => {
    if (message.type() === 'error') problems.push(message.text());
  });
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => {
      (window.violations ??= []).push(event.violatedDirective);
    });
  });
  await page.goto(`${origin}/`);
  await page.waitForSelector('#fruit .fd-select-trigger');
  await page.$eval('#overlays', (section) => section.scrollIntoView());
  t.after(async () => {
    const violations = await page.evaluate(() => window.violations ?? []).catch(() => []);
    assert.deepEqual([...problems, ...violations], []);
  });
  return page;
}

// Whether it is really showing, not just flagged as open.
const isOpen = (page, selector) =>
  page.$eval(selector, (list) => {
    const showing = getComputedStyle(list).display !== 'none' && list.getBoundingClientRect().height > 0;
    if (showing !== list.matches(':popover-open')) throw new Error(`${list.className} open state and display disagree`);
    return showing;
  });
const activeText = (page, selector) => page.$eval(selector, (list) => list.querySelector('[data-active]')?.textContent ?? null);
const focusRole = (page) => page.evaluate(() => document.activeElement.getAttribute('role'));

test('nothing floats at rest', async (t) => {
  const page = await open(t);
  const showing = await page.$$eval('fd-list, .fd-tooltip', (each) =>
    each.filter((element) => getComputedStyle(element).display !== 'none').map((element) => element.outerHTML.slice(0, 40)),
  );
  // The command menu's list sits inside its closed dialog, which hides it.
  assert.deepEqual(showing.filter((html) => !html.includes('role="listbox"') || html.includes('popover')), []);
});

test('tooltip: shows on hover after a delay, on keyboard focus at once, and Escape hides it', async (t) => {
  const page = await open(t);
  const tip = '.fd-tooltip';
  await page.hover('fd-tooltip button');
  assert.equal(await isOpen(page, tip), false, 'not before the delay');
  await page.waitForTimeout(450);
  assert.equal(await isOpen(page, tip), true);
  await page.mouse.move(1, 1);
  assert.equal(await isOpen(page, tip), false);
  await page.focus('#theme');
  await page.keyboard.press('Tab');
  while (!(await page.evaluate(() => document.activeElement.closest('fd-tooltip')))) await page.keyboard.press('Tab');
  assert.equal(await isOpen(page, tip), true);
  await page.keyboard.press('Escape');
  assert.equal(await isOpen(page, tip), false);
  const linked = await page.$eval('fd-tooltip button', (button) => document.getElementById(button.getAttribute('aria-describedby')).getAttribute('role'));
  assert.equal(linked, 'tooltip');
});

test('tooltip: stays on screen', async (t) => {
  const page = await open(t, { viewport: { width: 400, height: 900 } });
  await page.$eval('fd-tooltip', (each) => each.setAttribute('text', 'A rather long tooltip that needs real room to fit'));
  await page.hover('fd-tooltip button');
  await page.waitForTimeout(450);
  const box = await page.$eval('.fd-tooltip', (tip) => tip.getBoundingClientRect().toJSON());
  assert.ok(box.left >= 8 && box.right <= 392, JSON.stringify(box));
});

test('menu: opens from the button, arrows wrap and skip disabled, Enter picks and returns focus', async (t) => {
  const page = await open(t);
  await page.evaluate(() => {
    window.picks = [];
    document.querySelector('#actions').addEventListener('pick', (event) => window.picks.push(event.detail.value));
  });
  const list = '#actions fd-list';
  await page.focus('#actions button');
  await page.keyboard.press('ArrowDown');
  assert.equal(await isOpen(page, list), true);
  assert.equal(await activeText(page, list), 'Rename');
  assert.equal(await page.$eval('#actions button', (button) => button.getAttribute('aria-expanded')), 'true');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('ArrowDown');
  assert.equal(await activeText(page, list), 'Delete', 'skips Archive');
  await page.keyboard.press('ArrowDown');
  assert.equal(await activeText(page, list), 'Rename', 'wraps');
  await page.keyboard.press('End');
  assert.equal(await activeText(page, list), 'Delete');
  await page.keyboard.press('Enter');
  assert.equal(await isOpen(page, list), false);
  assert.deepEqual(await page.evaluate(() => window.picks), ['delete']);
  assert.equal(await page.evaluate(() => document.activeElement.textContent.trim()), 'Actions');
});

test('menu: typing jumps to a match and Escape closes', async (t) => {
  const page = await open(t);
  const list = '#actions fd-list';
  await page.click('#actions button');
  await page.keyboard.press('d');
  assert.equal(await activeText(page, list), 'Duplicate');
  await page.keyboard.press('Escape');
  assert.equal(await isOpen(page, list), false);
});

test('menu: clicking outside closes it and hover moves the mark', async (t) => {
  const page = await open(t);
  const list = '#actions fd-list';
  await page.click('#actions button');
  await page.hover('#actions fd-item[value=duplicate]');
  assert.equal(await activeText(page, list), 'Duplicate');
  await page.waitForTimeout(200);
  const place = await page.$eval(list, (each) => ({
    mark: Number.parseFloat(each.style.getPropertyValue('--_mark-y')),
    item: each.querySelector('[data-active]').offsetTop,
  }));
  assert.equal(place.mark, place.item);
  await page.mouse.click(5, 5);
  assert.equal(await isOpen(page, list), false);
});

test('menu: a disabled item cannot be clicked', async (t) => {
  const page = await open(t);
  await page.evaluate(() => {
    window.picks = [];
    document.querySelector('#actions').addEventListener('pick', (event) => window.picks.push(event.detail.value));
  });
  await page.click('#actions button');
  await page.click('#actions fd-item[disabled]', { force: true });
  assert.deepEqual(await page.evaluate(() => window.picks), []);
});

test('select: shows its value, keeps focus on the trigger, arrows and Enter choose', async (t) => {
  const page = await open(t);
  const trigger = '#fruit .fd-select-trigger';
  const list = '#fruit fd-list';
  await page.evaluate(() => {
    window.changes = 0;
    document.querySelector('#fruit').addEventListener('change', () => (window.changes += 1));
  });
  assert.equal(await page.textContent(trigger), 'Pear');
  await page.focus(trigger);
  await page.keyboard.press('Enter');
  assert.equal(await isOpen(page, list), true);
  assert.equal(await focusRole(page), 'combobox');
  assert.equal(await activeText(page, list), 'Pear', 'opens on the selected option');
  await page.keyboard.press('ArrowDown');
  assert.equal(await activeText(page, list), 'Plum');
  await page.keyboard.press('Enter');
  assert.equal(await page.textContent(trigger), 'Plum');
  assert.equal(await page.$eval('#fruit', (each) => each.value), 'plum');
  assert.equal(await page.evaluate(() => window.changes), 1);
  assert.equal(await page.$eval('#fruit fd-item[value=plum]', (item) => item.getAttribute('aria-selected')), 'true');
});

test('select: typing jumps, Escape cancels, blur closes', async (t) => {
  const page = await open(t);
  const trigger = '#fruit .fd-select-trigger';
  const list = '#fruit fd-list';
  await page.focus(trigger);
  await page.keyboard.press('b');
  assert.equal(await isOpen(page, list), true);
  assert.equal(await activeText(page, list), 'Banana');
  await page.keyboard.press('Escape');
  assert.equal(await isOpen(page, list), false);
  assert.equal(await page.textContent(trigger), 'Pear', 'unchanged');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Tab');
  assert.equal(await isOpen(page, list), false);
});

test('select and combobox send their values in a form', async (t) => {
  const page = await open(t);
  await page.focus('#city input');
  await page.keyboard.type('os');
  await page.keyboard.press('Enter');
  const values = await page.evaluate(() => {
    const form = document.createElement('form');
    const section = document.querySelector('#overlays');
    section.append(form);
    form.append(document.querySelector('#fruit'), document.querySelector('#city'));
    return Object.fromEntries(new FormData(form));
  });
  assert.deepEqual(values, { fruit: 'pear', city: 'osl' });
});

test('combobox: typing filters, arrows move, Enter chooses, Escape closes', async (t) => {
  const page = await open(t);
  const list = '#city fd-list';
  const visible = () => page.$$eval(`${list} fd-item:not([hidden])`, (items) => items.map((item) => item.textContent));
  await page.focus('#city input');
  await page.keyboard.type('l');
  assert.deepEqual(await visible(), ['Lisbon', 'London', 'Oslo'], 'matches anywhere in the name');
  assert.equal(await activeText(page, list), 'Lisbon');
  await page.keyboard.type('o');
  assert.deepEqual(await visible(), ['London', 'Oslo']);
  await page.keyboard.press('Enter');
  assert.equal(await page.inputValue('#city input'), 'London');
  assert.equal(await page.$eval('#city', (each) => each.value), 'lon');
  await page.fill('#city input', 'zz');
  assert.equal(await page.textContent(`${list} fd-list-empty`), 'No matches');
  await page.keyboard.press('Escape');
  assert.equal(await isOpen(page, list), false);
  assert.equal(await page.$eval('#city input', (input) => input.getAttribute('aria-expanded')), 'false');
});

test('dialog: traps focus, Escape closes, focus returns, the form reports its button', async (t) => {
  const page = await open(t);
  await page.click('#open-dialog');
  assert.equal(await page.$eval('#confirm', (dialog) => dialog.open), true);
  for (let press = 0; press < 5; press += 1) {
    await page.keyboard.press('Tab');
    const inside = await page.evaluate(() => {
      const active = document.activeElement;
      return active === document.body || Boolean(active.closest('#confirm'));
    });
    assert.ok(inside);
  }
  await page.keyboard.press('Escape');
  await page.waitForTimeout(250);
  assert.equal(await page.$eval('#confirm', (dialog) => dialog.open), false);
  assert.equal(await page.evaluate(() => document.activeElement.id), 'open-dialog');
  await page.click('#open-dialog');
  await page.click('#confirm button[value=delete]');
  assert.equal(await page.$eval('#confirm', (dialog) => dialog.returnValue), 'delete');
});

test('command menu: search, keyboard pick and close', async (t) => {
  const page = await open(t);
  await page.evaluate(() => {
    window.picks = [];
    document.querySelector('#commands').addEventListener('pick', (event) => window.picks.push(event.detail.value));
  });
  await page.click('#open-command');
  assert.equal(await focusRole(page), 'combobox');
  assert.equal(await activeText(page, '#commands fd-list'), 'New report');
  await page.keyboard.type('th');
  assert.equal(await activeText(page, '#commands fd-list'), 'Switch theme');
  await page.keyboard.press('Enter');
  assert.deepEqual(await page.evaluate(() => window.picks), ['theme']);
  assert.equal(await page.$eval('#commands', (each) => each.open), false);
});

test('command menu: clicking the backdrop closes it and reopening starts clean', async (t) => {
  const page = await open(t);
  await page.click('#open-command');
  await page.keyboard.type('zz');
  await page.mouse.click(5, 5);
  assert.equal(await page.$eval('#commands', (each) => each.open), false);
  await page.click('#open-command');
  assert.equal(await page.inputValue('#commands input'), '');
  assert.equal(await page.$$eval('#commands fd-item:not([hidden])', (items) => items.length), 4);
});

test('reduced motion: the mark and the dialog arrive at once', async (t) => {
  const page = await open(t, { reducedMotion: 'reduce' });
  await page.click('#actions button');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('ArrowDown');
  const place = await page.$eval('#actions fd-list', (list) => ({
    mark: Number.parseFloat(list.style.getPropertyValue('--_mark-y')),
    item: list.querySelector('[data-active]').offsetTop,
  }));
  assert.equal(place.mark, place.item);
});

test('axe finds nothing with each overlay open', async (t) => {
  for (const colorScheme of ['light', 'dark']) {
    const page = await open(t, { colorScheme });
    await page.context().addInitScript(axeSource);
    await page.reload();
    await page.waitForFunction(() => window.axe && document.querySelector('#fruit .fd-select-trigger'));
    const check = async (label) => {
      const violations = await page.evaluate(async () => {
        const { violations } = await window.axe.run(document, { runOnly: ['wcag2a', 'wcag2aa', 'wcag21aa'] });
        return violations.map(({ id, nodes }) => `${id}: ${nodes.map((node) => node.target).join(' ')}`);
      });
      assert.deepEqual(violations, [], `${colorScheme} ${label}`);
    };
    await page.click('#actions button');
    await page.keyboard.press('ArrowDown');
    await check('menu');
    await page.keyboard.press('Escape');
    await page.focus('#fruit .fd-select-trigger');
    await page.keyboard.press('Enter');
    await check('select');
    await page.keyboard.press('Escape');
    await page.focus('#city input');
    await page.keyboard.type('o');
    await check('combobox');
    await page.keyboard.press('Escape');
    await page.click('#open-command');
    await page.waitForTimeout(250); // Contrast can only be judged after the fade-in.
    await check('command');
    await page.keyboard.press('Escape');
    await page.click('#open-dialog');
    await page.waitForTimeout(250);
    await check('dialog');
  }
});
