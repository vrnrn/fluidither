// Font loading and actual shaping under the demo's strict CSP, including failure.
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
  await browser?.close();
  server?.closeAllConnections();
  if (server) await new Promise((resolve) => server.close(resolve));
});

for (const colorScheme of ['light', 'dark']) {
  test(`Obel Mono renders from local files in ${colorScheme}, including italic and variable weight`, async (t) => {
    const context = await browser.newContext({ colorScheme, reducedMotion: 'reduce' });
    t.after(() => context.close());
    const page = await context.newPage();
    const problems = [];
    const fonts = [];
    page.on('console', (message) => { if (message.type() === 'error') problems.push(message.text()); });
    page.on('response', (response) => {
      if (response.url().endsWith('.woff2')) fonts.push({ url: response.url(), status: response.status() });
    });
    await page.goto(origin);
    const result = await page.evaluate(async () => {
      const roman = await document.fonts.load('560 16px "Obel Mono"');
      const italic = await document.fonts.load('italic 560 16px "Obel Mono"');
      const specimen = document.createElement('p');
      specimen.id = 'font-proof';
      specimen.className = 'fd-mono';
      specimen.textContent = 'Il1 O0 rn 0123456789';
      document.body.append(specimen);
      const canvas = document.createElement('canvas');
      const painter = canvas.getContext('2d');
      painter.font = '560 16px "Obel Mono"';
      const widths = [...'iW01'].map((letter) => painter.measureText(letter.repeat(10)).width);
      const selectors = ['.fd-stat-value', '.fd-kbd', 'fd-copy .fd-input', '.fd-table [data-number]', 'fd-count'];
      return {
        loaded: [roman.length, italic.length],
        widths,
        roles: selectors.map((selector) => [selector, getComputedStyle(document.querySelector(selector)).fontFamily]),
        body: getComputedStyle(document.body).fontFamily,
      };
    });
    assert.deepEqual(result.loaded, [1, 1]);
    assert.ok(result.widths.every((width) => Math.abs(width - 96) < 0.1), JSON.stringify(result.widths));
    for (const [selector, family] of result.roles) assert.match(family, /Obel Mono/, selector);
    assert.doesNotMatch(result.body, /Obel Mono/);
    const cdp = await context.newCDPSession(page);
    await cdp.send('DOM.enable');
    await cdp.send('CSS.enable');
    const { root } = await cdp.send('DOM.getDocument');
    const { nodeId } = await cdp.send('DOM.querySelector', { nodeId: root.nodeId, selector: '#font-proof' });
    const actual = await cdp.send('CSS.getPlatformFontsForNode', { nodeId });
    assert.ok(actual.fonts.some((font) => font.isCustomFont && font.familyName === 'Obel Mono' && font.glyphCount > 0));
    assert.equal(fonts.length, 2);
    assert.ok(fonts.every((font) => font.status === 200 && font.url.startsWith(origin)));
    assert.deepEqual(problems, []);
  });
}

test('readouts remain visible when the font cannot load', async (t) => {
  const context = await browser.newContext({ reducedMotion: 'reduce' });
  t.after(() => context.close());
  const page = await context.newPage();
  await page.route('**/*.woff2', (route) => route.abort());
  await page.goto(origin);
  await page.evaluate(() => document.fonts.ready);
  const value = page.locator('.fd-stat-value').first();
  assert.ok(await value.isVisible());
  assert.ok((await value.textContent()).trim().length > 0);
  const cdp = await context.newCDPSession(page);
  await cdp.send('DOM.enable');
  await cdp.send('CSS.enable');
  const { root } = await cdp.send('DOM.getDocument');
  const { nodeId } = await cdp.send('DOM.querySelector', { nodeId: root.nodeId, selector: '.fd-stat-value' });
  const actual = await cdp.send('CSS.getPlatformFontsForNode', { nodeId });
  assert.ok(actual.fonts.some((font) => font.glyphCount > 0 && !font.isCustomFont));
});
