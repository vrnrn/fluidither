// What one dither frame costs in real Chromium, and what resting pages do.
import { chromium } from 'playwright-core';
import { createDemoServer } from '../web/demo/serve.mjs';

const server = createDemoServer();
await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1000, height: 900 } });
await page.goto(`${origin}/`);

const rows = await page.evaluate(async () => {
  const { ditherTones, fillGradient, thresholdMap } = await import('/src/core/dither.js');
  const map = await thresholdMap('bayer4');
  const gradient = { mode: 0, angle: 90, centerX: 0.5, centerY: 0.5, radius: 1, from: 0, to: 0.75, noise: 0.12 };
  const palette = Uint32Array.of(0, 0xff171717);
  const sizes = [
    ['band 848x120 @3px', 283, 40],
    ['panel 1200x300 @3px', 400, 100],
    ['large 1200x700 @3px', 400, 234],
    ['fine 1200x700 @1px', 1200, 700],
  ];
  const results = [];
  for (const [label, width, height] of sizes) {
    const tones = new Float64Array(width * height);
    const pixels = new Uint32Array(width * height);
    const image = new ImageData(new Uint8ClampedArray(pixels.buffer), width, height);
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext('2d');
    const samples = [];
    for (let frame = 0; frame < 300; frame += 1) {
      const start = performance.now();
      fillGradient(tones, width, height, gradient, frame * 0.013);
      ditherTones(tones, width, height, map, palette, pixels);
      context.putImageData(image, 0, 0);
      samples.push(performance.now() - start);
    }
    samples.sort((a, b) => a - b);
    results.push({ label, cells: width * height, p50: samples[150], p95: samples[285] });
  }
  return results;
});

console.log('size'.padEnd(24) + 'cells'.padStart(9) + 'p50 ms'.padStart(9) + 'p95 ms'.padStart(9));
for (const { label, cells, p50, p95 } of rows) {
  console.log(label.padEnd(24) + String(cells).padStart(9) + p50.toFixed(2).padStart(9) + p95.toFixed(2).padStart(9));
}

// Main-thread share while the demo's two animated gradients run. Headless, so
// this excludes real compositing.
const client = await page.context().newCDPSession(page);
await client.send('Performance.enable');
const taskSeconds = async () =>
  (await client.send('Performance.getMetrics')).metrics.find(({ name }) => name === 'TaskDuration').value;
const before = await taskSeconds();
await page.waitForTimeout(4000);
console.log(`\nmain thread while animating: ${(((await taskSeconds()) - before) / 4 * 100).toFixed(1)}% of a core`);

await browser.close();
server.closeAllConnections();
server.close();
