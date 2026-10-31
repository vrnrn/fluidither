// The form controls are real inputs, so these tests check that styling them has not
// broken what the browser gives them, and that their own states work.
import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright-core';
import { createDemoServer } from '../../demo/serve.mjs';

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
  const context = await browser.newContext({ viewport: { width: 1000, height: 1300 }, ...options });
  t.after(() => context.close());
  const page = await context.newPage();
  const problems = [];
  page.on('pageerror', (error) => problems.push(error.message));
  await page.goto(`${origin}/`);
  await page.waitForSelector('fd-slider input');
  t.after(() => assert.deepEqual(problems, []));
  return page;
}

const checked = (page, selector) => page.$eval(selector, (input) => input.checked);

test('clicking a label toggles its checkbox, and Space does too', async (t) => {
  const page = await open(t);
  const box = '.fd-choice:nth-child(2) .fd-check';
  assert.equal(await checked(page, box), false);
  await page.click('.fd-choice:nth-child(2)');
  assert.equal(await checked(page, box), true);
  await page.focus(box);
  await page.keyboard.press('Space');
  assert.equal(await checked(page, box), false);
});

test('a disabled checkbox ignores clicks', async (t) => {
  const page = await open(t);
  await page.click('.fd-choice:has(.fd-check:disabled)', { force: true });
  assert.equal(await checked(page, '.fd-check:disabled'), false);
});

test('the tick and the dash are drawn', async (t) => {
  const page = await open(t);
  const shadow = (selector) => page.$eval(selector, (input) => getComputedStyle(input, '::before').boxShadow);
  assert.notEqual(await shadow('.fd-check:checked'), 'none');
  const dash = await shadow('#mixed');
  assert.notEqual(dash, 'none');
  assert.notEqual(dash, await shadow('.fd-check:checked'));
  assert.equal(await shadow('.fd-choice:nth-child(2) .fd-check'), 'none');
});

test('arrow keys move through a radio group and skip the disabled one', async (t) => {
  const page = await open(t);
  await page.focus('.fd-radio:checked');
  await page.keyboard.press('ArrowRight');
  assert.equal(await page.$eval('.fd-radio:checked', (input) => input.parentElement.textContent.trim()), 'Yearly');
  await page.keyboard.press('ArrowRight');
  assert.equal(await page.$eval('.fd-radio:checked', (input) => input.parentElement.textContent.trim()), 'Monthly');
});

test('a switch toggles and its thumb travels', async (t) => {
  const page = await open(t);
  const off = '.fd-choice:has(.fd-switch:not(:checked))';
  const input = await page.$(`${off} .fd-switch`);
  await page.click(off);
  assert.equal(await input.evaluate((each) => each.checked), true);
  await page.waitForTimeout(450);
  const moved = await input.evaluate((each) => getComputedStyle(each, '::before').translate);
  assert.notEqual(moved, 'none');
  assert.equal(await input.getAttribute('role'), 'switch');
});

test('the slider fill follows the keyboard and code', async (t) => {
  const page = await open(t);
  const fill = () => page.$eval('fd-slider input', (input) => input.style.getPropertyValue('--_fill'));
  assert.equal(await fill(), '40%');
  await page.focus('fd-slider input');
  await page.keyboard.press('ArrowRight');
  assert.equal(await fill(), '41%');
  await page.keyboard.press('End');
  assert.equal(await fill(), '100%');
  await page.evaluate(() => {
    document.querySelector('fd-slider input').value = 10;
    document.querySelector('fd-slider').refresh();
  });
  assert.equal(await fill(), '10%');
});

test('text fields take input and show their invalid state', async (t) => {
  const page = await open(t);
  await page.fill('.fd-input[type=text]', 'Ada');
  assert.equal(await page.inputValue('.fd-input[type=text]'), 'Ada');
  const edge = await page.$eval('.fd-input[aria-invalid]', (input) => getComputedStyle(input).boxShadow);
  const red = await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--fd-red'));
  const channel = (at) => Number.parseInt(red.trim().slice(at, at + 2), 16);
  assert.ok(edge.includes(`rgb(${channel(1)}, ${channel(3)}, ${channel(5)})`), edge);
});

test('form values reach a real form', async (t) => {
  const page = await open(t);
  const values = await page.evaluate(() => {
    const form = document.createElement('form');
    form.innerHTML =
      '<input class="fd-check" type="checkbox" name="a" checked><input class="fd-switch" type="checkbox" role="switch" name="b" checked><input class="fd-input" name="c" value="x">';
    document.body.append(form);
    return Object.fromEntries(new FormData(form));
  });
  assert.deepEqual(values, { a: 'on', b: 'on', c: 'x' });
});

test('an icon sits beside a tab label and the tab stays centered around both', async (t) => {
  const page = await open(t);
  const facts = await page.$eval('[data-density=compact] fd-tab[value=list]', (tab) => {
    const icon = tab.querySelector('svg').getBoundingClientRect();
    const label = tab.querySelector('.fd-tab-text').getBoundingClientRect();
    const box = tab.getBoundingClientRect();
    return {
      sameRow: Math.abs(icon.top + icon.height / 2 - (label.top + label.height / 2)) < 1,
      left: icon.left - box.left,
      right: box.right - label.right,
    };
  });
  assert.equal(facts.sameRow, true);
  assert.ok(Math.abs(facts.left - facts.right) < 1, JSON.stringify(facts));
});
