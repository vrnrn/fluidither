// The motion-led parts in real Chromium, under the strict CSP, with axe.
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
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => (window.violations ??= []).push(`${event.violatedDirective} ${event.blockedURI}`));
  });
  await page.goto(`${origin}/`);
  await page.waitForSelector('#survey .fd-questions-count');
  await page.$eval('#motion', (each) => each.scrollIntoView());
  await page.waitForTimeout(100);
  t.after(() => {
    assert.deepEqual(problems, []);
  });
  return page;
}

const markTop = (page, id) => page.$eval(id, (host) => Number.parseFloat(host.style.getPropertyValue('--_mark-y')));

test('one highlight glides between items and skips disabled ones', async (t) => {
  const page = await open(t);
  await page.hover('#glide > div:nth-of-type(1)');
  await page.waitForFunction(() => document.querySelector('#glide > div:nth-of-type(1)').hasAttribute('data-hover'));
  await page.hover('#glide > div:nth-of-type(2)');
  await page.waitForFunction(() => document.querySelector('#glide > div:nth-of-type(2)').hasAttribute('data-hover'));
  assert.equal(await page.$$eval('#glide [data-hover]', (lit) => lit.length), 1);
  assert.equal(await page.$$eval('#glide .fd-hover-mark', (marks) => marks.length), 1);
  await page.hover('#glide > div:nth-of-type(3)');
  await page.waitForFunction(() => !document.querySelector('#glide [data-hover]'));
  await page.mouse.move(0, 0);
  await page.waitForFunction(() => Number(document.querySelector('#glide').style.getPropertyValue('--_mark-opacity')) === 0);
});

test('nearest keeps something lit across a gap', async (t) => {
  const page = await open(t);
  const box = await page.locator('#glide-near').boundingBox();
  await page.mouse.move(box.x + 20, box.y + box.height - 2);
  await page.waitForFunction(() => document.querySelector('#glide-near > div:nth-of-type(2)').hasAttribute('data-hover'));
  const first = await page.locator('#glide-near > div:nth-of-type(1)').boundingBox();
  await page.mouse.move(box.x + 20, first.y + first.height + 2);
  await page.waitForFunction(() => document.querySelector('#glide-near > div:nth-of-type(1)').hasAttribute('data-hover'));
  assert.ok((await markTop(page, '#glide-near')) >= 0);
});

test('checked neighbours join into one shape', async (t) => {
  const page = await open(t);
  await page.waitForFunction(() => {
    const [first, second] = document.querySelectorAll('#fruit-group .fd-choice');
    return Number.parseFloat(getComputedStyle(first).borderBottomLeftRadius) === 0 && Number.parseFloat(getComputedStyle(second).borderTopLeftRadius) === 0;
  });
  await page.click('#fruit-group .fd-choice:nth-of-type(2)');
  await page.waitForFunction(() => Number.parseFloat(getComputedStyle(document.querySelector('#fruit-group .fd-choice')).borderBottomLeftRadius) > 0);
});

test('shimmer keeps its label, draws, and holds still under reduced motion', async (t) => {
  const page = await open(t, { reducedMotion: 'reduce' });
  assert.equal(await page.getAttribute('#shimmer', 'aria-label'), 'Thinking');
  assert.ok(await page.$eval('#shimmer canvas', (canvas) => canvas.width > 0));
});

test('a question flow answers by click and number, skips, and reports', async (t) => {
  const page = await open(t);
  await page.$eval('#survey', (flow) => flow.addEventListener('done', (event) => (window.answers = event.detail.answers)));
  assert.equal(await page.textContent('#survey .fd-questions-count'), 'Question 1 of 2');
  await page.click('#survey .fd-question:not([hidden]) .fd-choice:nth-of-type(2)');
  assert.equal(await page.textContent('#survey .fd-questions-count'), 'Question 2 of 2');
  await page.locator('#survey').getByRole('button', { name: 'Back' }).click();
  assert.equal(await page.textContent('#survey .fd-questions-count'), 'Question 1 of 2');
  await page.focus('#survey .fd-question:not([hidden]) input');
  await page.keyboard.press('1');
  await page.locator('#survey').getByRole('button', { name: 'Skip' }).click();
  assert.deepEqual(await page.evaluate(() => window.answers), { role: 'design', team: null });
  assert.equal(await page.textContent('#survey'), 'All set.');
});

test('checkboxes take several answers, and an option can hold your own', async (t) => {
  const page = await open(t);
  await page.$eval('#many', (flow) => flow.addEventListener('done', (event) => (window.many = event.detail.answers)));
  await page.click('#many .fd-question:not([hidden]) .fd-choice:nth-of-type(1)');
  await page.focus('#many .fd-question:not([hidden]) input');
  await page.keyboard.press('2');
  assert.equal(await page.textContent('#many .fd-questions-count'), 'Question 1 of 2');
  await page.locator('#many').getByRole('button', { name: 'Next' }).click();
  await page.click('#many .fd-question:not([hidden]) .fd-choice:nth-of-type(2) .fd-radio');
  assert.equal(await page.evaluate(() => document.activeElement.getAttribute('aria-label')), 'Something else');
  await page.keyboard.type('Time');
  await page.keyboard.press('Enter');
  assert.deepEqual(await page.evaluate(() => window.many), { features: ['dark', 'speed'], block: 'Time' });
});

test('a banner can be dismissed, and says so', async (t) => {
  const page = await open(t);
  await page.$eval('#motion', (section) => section.addEventListener('dismiss', () => (window.dismissed = true)));
  await page.click('#news button');
  await page.waitForFunction(() => window.dismissed === true);
  assert.equal(await page.locator('#news').count(), 0);
});

test('arrow keys choose without moving on', async (t) => {
  const page = await open(t);
  await page.focus('#survey .fd-question:not([hidden]) input');
  await page.keyboard.press('ArrowDown');
  assert.equal(await page.textContent('#survey .fd-questions-count'), 'Question 1 of 2');
});

test('tab takes the suggestion, enter sends it', async (t) => {
  const page = await open(t);
  await page.$eval('#suggest', (box) => box.addEventListener('send', (event) => (window.sent = event.detail.text)));
  await page.focus('#suggest textarea');
  await page.keyboard.press('Tab');
  assert.equal(await page.inputValue('#suggest textarea'), 'Why is it stiff?');
  assert.equal(await page.isHidden('#suggest .fd-composer-tab'), true);
  await page.keyboard.press('Enter');
  assert.equal(await page.evaluate(() => window.sent), 'Why is it stiff?');
  assert.equal(await page.inputValue('#suggest textarea'), '');
});

test('a reveal ends with everything shown, and can run again', async (t) => {
  const page = await open(t);
  await page.$eval('#rise', (each) => each.scrollIntoView());
  await page.waitForFunction(() => document.querySelector('#rise').dataset.state === 'shown');
  await page.$eval('#rise', (each) => each.run());
  await page.waitForFunction(() => document.querySelector('#rise').dataset.state === 'shown');
  await page.waitForFunction(() => getComputedStyle(document.querySelector('#rise p:last-child')).opacity === '1');
});

test('reduced motion shows a reveal at once', async (t) => {
  const page = await open(t, { reducedMotion: 'reduce' });
  assert.equal(await page.$eval('#rise', (each) => each.dataset.state ?? 'none'), 'none');
});

test('the color picker follows keys, hue and typed hex', async (t) => {
  const page = await open(t);
  assert.equal(await page.$eval('#picker', (picker) => picker.value), '#fc9e47');
  await page.$eval('#picker', (picker) => picker.addEventListener('change', () => (window.changed = picker.value)));
  await page.focus('#picker .fd-color-field');
  await page.keyboard.press('ArrowLeft');
  assert.notEqual(await page.evaluate(() => window.changed), undefined);
  await page.fill('#picker .fd-input', '#00ff00');
  await page.keyboard.press('Enter');
  assert.equal(await page.$eval('#picker', (picker) => picker.value), '#00ff00');
  assert.equal(await page.inputValue('#picker .fd-color-hue'), '120');
});

test('a dialog closes faster than it opens', async (t) => {
  const page = await open(t);
  const timing = await page.evaluate(() => {
    const dialog = document.querySelector('#confirm');
    const closed = getComputedStyle(dialog).transitionDuration.split(',')[0];
    dialog.showModal();
    const opened = getComputedStyle(dialog).transitionDuration.split(',')[0];
    dialog.close();
    return [Number.parseFloat(closed), Number.parseFloat(opened)];
  });
  assert.ok(timing[0] < timing[1]);
});

for (const colorScheme of ['light', 'dark']) {
  test(`axe passes on the motion parts in ${colorScheme}`, async (t) => {
    const page = await open(t, { colorScheme, reducedMotion: 'reduce' });
    await page.evaluate(axeSource);
    const { violations } = await page.evaluate(() => window.axe.run('#motion', { resultTypes: ['violations'] }));
    assert.deepEqual(violations.map(({ id, nodes }) => `${id}: ${nodes.map((node) => node.target).join(', ')}`), []);
    assert.equal(await page.evaluate(() => window.violations), undefined);
  });
}
