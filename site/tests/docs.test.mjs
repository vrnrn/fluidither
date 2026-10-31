// The docs page in Chromium under its own CSP, and its claims against the code.
import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';
import { createSiteServer } from '../serve.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '../..');
const source = readFileSync(join(here, '../docs.html'), 'utf8');
const axeSource = readFileSync(createRequire(import.meta.url).resolve('axe-core/axe.min.js'), 'utf8');

let browser;
let server;
let origin;

before(async () => {
  browser = await chromium.launch();
  server = createSiteServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  origin = `http://127.0.0.1:${server.address().port}`;
});

after(async () => {
  await browser.close();
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
});

async function open(t, options = {}) {
  const context = await browser.newContext({ reducedMotion: 'reduce', viewport: { width: 1440, height: 900 }, ...options });
  t.after(() => context.close());
  const page = await context.newPage();
  const problems = [];
  page.on('pageerror', (error) => problems.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') problems.push(message.text());
  });
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => (window.violations ??= []).push(`${event.violatedDirective} ${event.blockedURI}`));
  });
  await page.goto(`${origin}/docs.html`);
  await page.waitForSelector('html[data-ready]');
  return { page, problems };
}

test('every install name it prints is one the command knows', () => {
  const known = new Set(execFileSync(process.execPath, [join(root, 'web/bin/fluidither.mjs'), 'list'], { encoding: 'utf8' }).trim().split('\n'));
  const named = [...source.matchAll(/data-add="([^"]+)"/g)].flatMap(([, names]) => names.split(' '));
  assert.ok(named.length > 40);
  assert.deepEqual(named.filter((name) => !known.has(name)), []);
});

test('every Rust type it names is in the crate', () => {
  const crate = readdirSync(join(root, 'gpui/src'))
    .filter((file) => file.endsWith('.rs'))
    .map((file) => readFileSync(join(root, 'gpui/src', file), 'utf8'))
    .join('\n');
  const samples = [...source.matchAll(/<template data-rust>([\s\S]*?)<\/template>/g)].map(([, code]) => code);
  assert.ok(samples.length > 40);
  const types = new Set(samples.flatMap((code) => [...code.matchAll(/\b([A-Z][A-Za-z]+)::/g)].map(([, name]) => name)));
  const missing = [...types].filter((name) => !new RegExp(`(struct|enum|type) ${name}\\b`).test(crate));
  assert.deepEqual(missing, []);
});

test('every web token it names is real', () => {
  const tokens = readFileSync(join(root, 'web/src/tokens.css'), 'utf8');
  const named = new Set([...source.matchAll(/--fd-[a-z0-9-]+/g)].map(([name]) => name));
  named.delete('--fd-chart-height');
  named.delete('--fd-dissolve-paper');
  assert.deepEqual([...named].filter((name) => !tokens.includes(`${name}:`)), []);
});

test('the page runs clean, with every element it shows defined', async (t) => {
  const { page, problems } = await open(t);
  const undefinedTags = await page.evaluate(() =>
    [...new Set([...document.querySelectorAll('.plate-view *')].map((each) => each.localName))].filter((tag) => tag.includes('-') && !customElements.get(tag)),
  );
  assert.deepEqual(undefinedTags, []);
  assert.deepEqual(problems, []);
  assert.equal(await page.evaluate(() => window.violations), undefined);
});

test('every sidebar link lands on something, and every example is live', async (t) => {
  const { page } = await open(t);
  const broken = await page.$$eval('#nav a', (links) => links.filter((link) => !document.getElementById(link.hash.slice(1))).map((link) => link.hash));
  assert.deepEqual(broken, []);
  const empty = await page.$$eval('.plate-view', (views) => views.filter((view) => view.childElementCount === 0).length);
  assert.equal(empty, 0);
  assert.ok((await page.$$eval('.plate', (plates) => plates.length)) > 40);
});

test('the source shown is what a reader would write', async (t) => {
  const { page } = await open(t);
  const code = await page.$$eval('.plate code', (blocks) => blocks.map((block) => block.textContent).join('\n'));
  assert.doesNotMatch(code, /data-opens/);
  assert.doesNotMatch(code, /class="[^"]*demo-/);
  assert.doesNotMatch(code, /=""/);
});

test('scramble, dissolve and count can run again', async (t) => {
  const { page, problems } = await open(t, { reducedMotion: 'no-preference' });
  for (const id of ['scramble', 'dissolve', 'count']) {
    const shown = page.locator(`#${id} .plate-view p`);
    const first = await shown.elementHandle();
    await page.click(`#${id} .replay`);
    assert.equal(await first.evaluate((each) => each.isConnected), false, id);
    assert.equal(await shown.count(), 1, id);
  }
  assert.equal(await page.$$eval('.replay', (buttons) => buttons.length), 6);
  assert.deepEqual(problems, []);
});

test('the same noise runs behind the docs', async (t) => {
  const { page } = await open(t);
  assert.equal(await page.getAttribute('#ground', 'kind'), 'noise');
  assert.ok(Number(await page.getAttribute('#ground', 'speed')) > 0);
});

test('search jumps to a component', async (t) => {
  const { page } = await open(t);
  await page.keyboard.press('/');
  await page.keyboard.type('calendar');
  await page.keyboard.press('Enter');
  await page.waitForFunction(() => location.hash === '#calendar');
  await page.waitForFunction(() => document.querySelector('#nav a[aria-current]').hash === '#calendar');
  const top = await page.$eval('#calendar', (part) => part.getBoundingClientRect().top);
  assert.ok(top >= 0 && top < 120, `the section sits at ${top}`);
});

test('the sidebar follows reading', async (t) => {
  const { page } = await open(t);
  await page.$eval('#charts', (part) => part.scrollIntoView());
  await page.waitForFunction(() => document.querySelector('#nav a[aria-current]').hash === '#charts');
});

test('the code tabs show the Rust', async (t) => {
  const { page } = await open(t);
  const tabs = page.locator('#button fd-tab[value=gpui]');
  await tabs.click();
  assert.match(await page.locator('#button fd-tab-panel[value=gpui] code').textContent(), /Button::new/);
});

test('it fits a phone, with the sidebar in a drawer', async (t) => {
  const { page } = await open(t, { viewport: { width: 390, height: 844 } });
  assert.ok((await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)) <= 0);
  await page.click('#menu-open');
  await page.click('#menu a[href="#tokens"]');
  await page.waitForFunction(() => location.hash === '#tokens');
  assert.equal(await page.$eval('#menu', (menu) => menu.open), false);
  await page.waitForFunction(() => document.getElementById('nav').parentElement.tagName === 'ASIDE');
});

for (const theme of ['dark', 'light']) {
  test(`axe finds nothing in ${theme}`, async (t) => {
    const { page } = await open(t);
    if (theme === 'light') {
      await page.click('#dark');
      await page.waitForFunction(() => document.documentElement.dataset.theme === 'light');
      // Colors transition after the flip, and axe must read where they land.
      await page.waitForTimeout(500);
    }
    await page.evaluate(axeSource);
    const { violations } = await page.evaluate(() => window.axe.run(document, { resultTypes: ['violations'] }));
    assert.deepEqual(violations.map(({ id, nodes }) => `${id}: ${nodes.slice(0, 4).map((node) => node.target).join(', ')}`), []);
  });
}
