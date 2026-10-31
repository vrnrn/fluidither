import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { chromium } from 'playwright-core';
import { createDemoServer } from '../../demo/serve.mjs';
import { readPng } from '../../../scripts/lib/png.mjs';

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
  const context = await browser.newContext({ viewport: { width: 1000, height: 900 }, acceptDownloads: true, ...options });
  t.after(() => context.close());
  const page = await context.newPage();
  const problems = [];
  page.on('pageerror', (error) => problems.push(error.message));
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => (window.violations ??= []).push(`${event.violatedDirective} ${event.blockedURI}`));
  });
  await page.goto(`${origin}/`);
  await page.waitForSelector('#map canvas');
  await page.$eval('#extras', (each) => each.scrollIntoView());
  await page.waitForTimeout(150);
  t.after(async () => {
    const violations = await page.evaluate(() => window.violations ?? []).catch(() => []);
    assert.deepEqual([...problems, ...violations], []);
  });
  return page;
}

const pixels = (page, selector) =>
  page.$eval(`${selector} canvas`, (canvas) => canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data.join());

test('the map has land on it and the oceans stay empty', async (t) => {
  const page = await open(t);
  const shares = await page.$eval('#map canvas', (canvas) => {
    const share = (x, y) => {
      const size = Math.floor(canvas.height / 5);
      const { data } = canvas.getContext('2d').getImageData(Math.floor(x * canvas.width), Math.floor(y * canvas.height), size, size);
      let count = 0;
      for (let i = 3; i < data.length; i += 4) if (data[i] > 0) count += 1;
      return count;
    };
    return { africa: share(0.52, 0.48), pacific: share(0.03, 0.55) };
  });
  assert.ok(shares.africa > 0 && shares.pacific === 0, JSON.stringify(shares));
});

test('the globe turns and holds still under reduced motion', async (t) => {
  const page = await open(t);
  const first = await pixels(page, '#globe');
  await page.waitForTimeout(700);
  assert.notEqual(await pixels(page, '#globe'), first);
  const still = await open(t, { reducedMotion: 'reduce' });
  const frame = await pixels(still, '#globe');
  await still.waitForTimeout(400);
  assert.equal(await pixels(still, '#globe'), frame);
});

test('saving writes a real PNG under the strict CSP', async (t) => {
  const page = await open(t);
  const [download] = await Promise.all([page.waitForEvent('download'), page.click('#save-map')]);
  assert.equal(download.suggestedFilename(), 'map.png');
  const file = readFileSync(await download.path());
  const image = readPng(file);
  const size = await page.$eval('#map canvas', (canvas) => [canvas.width * 3, canvas.height * 3]);
  assert.deepEqual([image.width, image.height], size);
});

test('the sidebar collapses, and its links leave the tab order', async (t) => {
  const page = await open(t);
  const toggle = '#side-menu .fd-sidebar-toggle';
  assert.equal(await page.getAttribute(toggle, 'aria-expanded'), 'true');
  await page.click(toggle);
  assert.equal(await page.getAttribute(toggle, 'aria-expanded'), 'false');
  assert.equal(await page.getAttribute(toggle, 'aria-label'), 'Show menu');
  assert.equal(await page.$eval('#side-menu fd-nav', (nav) => nav.inert), true);
  await page.waitForTimeout(400);
  assert.equal(await page.$eval('#side-menu', (each) => each.offsetWidth), 34);
  await page.click(toggle);
  assert.equal(await page.$eval('#side-menu fd-nav', (nav) => nav.inert), false);
});

test('compare moves its split with the keyboard', async (t) => {
  const page = await open(t);
  const split = () => page.$eval('#compare', (each) => each.style.getPropertyValue('--_split'));
  assert.equal(await split(), '50%');
  await page.focus('#compare input');
  await page.keyboard.press('ArrowRight');
  assert.equal(await split(), '51%');
});

test('the marquee repeats for a seamless loop, hidden from assistive tech, and pauses', async (t) => {
  const page = await open(t);
  const hidden = await page.$$eval('fd-marquee .fd-marquee-track', (each) => each.map((track) => track.getAttribute('aria-hidden')));
  assert.deepEqual(hidden, [null, 'true']);
  await page.hover('fd-marquee');
  assert.equal(await page.$eval('.fd-marquee-track', (track) => getComputedStyle(track).animationPlayState), 'paused');
  const still = await open(t, { reducedMotion: 'reduce' });
  assert.equal(await still.$eval('.fd-marquee-track', (track) => getComputedStyle(track).animationName), 'none');
});

test('underline tabs turn the thumb into a line and still select', async (t) => {
  const page = await open(t);
  const list = 'fd-tabs[look=underline] fd-tab-list';
  assert.equal(await page.$eval(list, (each) => getComputedStyle(each, '::before').height), '2px');
  await page.click('fd-tabs[look=underline] fd-tab[value=open]');
  assert.equal(await page.$eval('fd-tabs[look=underline]', (each) => each.value), 'open');
});

test('held dither text keeps its veil and its real text', async (t) => {
  const page = await open(t);
  await page.waitForTimeout(800);
  assert.ok(await page.$('#held .fd-dissolve-veil canvas'));
  assert.equal((await page.textContent('#held')).trim(), 'Held dither text');
});

test('on-view dissolve waits for the scroll', async (t) => {
  const page = await open(t);
  const veil = () => page.$('#late .fd-dissolve-veil');
  assert.equal(await veil(), null, 'waiting offscreen');
  await page.$eval('#late', (each) => each.scrollIntoView());
  await page.waitForFunction(() => document.querySelector('#late .fd-dissolve-veil'));
  await page.waitForFunction(() => !document.querySelector('#late .fd-dissolve-veil'));
  assert.equal(await page.textContent('#late'), 'Revealed on scroll');
});

test('a transparent logo keeps its see-through parts as paper', async (t) => {
  const page = await open(t);
  const inked = await page.evaluate(async () => {
    const element = document.createElement('fd-dither-image');
    element.setAttribute('matrix', 'bayer2');
    element.style.width = '40px';
    element.style.height = '40px';
    element.setAttribute('src', '/demo/mark.svg');
    document.querySelector('#extras').append(element);
    await new Promise((resolve) => element.addEventListener('load', resolve));
    await new Promise((resolve) => setTimeout(resolve, 100));
    const canvas = element.querySelector('canvas');
    const { data } = canvas.getContext('2d').getImageData(0, 0, 3, 3);
    return data.some(Boolean);
  });
  assert.equal(inked, false, 'the corner of the logo is transparent');
});

test('axe finds nothing here, in light or dark', async (t) => {
  for (const colorScheme of ['light', 'dark']) {
    const page = await open(t, { colorScheme });
    await page.context().addInitScript(axeSource);
    await page.reload();
    await page.waitForFunction(() => window.axe && document.querySelector('#map canvas'));
    await page.waitForTimeout(800);
    const violations = await page.evaluate(async () => {
      const { violations } = await window.axe.run('#extras', { runOnly: ['wcag2a', 'wcag2aa', 'wcag21aa'] });
      return violations.map(({ id, nodes }) => `${id}: ${nodes.map((node) => node.target).join(' ')}`);
    });
    assert.deepEqual(violations, [], colorScheme);
  }
});
