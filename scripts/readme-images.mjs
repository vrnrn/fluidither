// Makes the README's images from the library itself: an animated hero and a showcase of
// real components from Chromium, in both themes, and the GPUI window on macOS.
import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { deflateSync } from 'node:zlib';
import { chromium } from 'playwright-core';
import { createDemoServer } from '../web/demo/serve.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const out = join(root, 'docs/images');
mkdirSync(out, { recursive: true });

const COLUMNS = 420;
const ROWS = 120;
const SCALE = 4;
const FRAMES = 120;
const FRAME_MS = 66;

const tokens = JSON.parse(readFileSync(join(root, 'tokens/tokens.json'), 'utf8'));

let table;
function crc(bytes) {
  table ??= Array.from({ length: 256 }, (_, n) => {
    let c = n;
    for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    return c >>> 0;
  });
  let c = 0xffffffff;
  for (const byte of bytes) c = table[(c ^ byte) & 255] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, body) {
  const head = Buffer.alloc(8);
  head.writeUInt32BE(body.length, 0);
  head.write(type, 4, 'latin1');
  const tail = Buffer.alloc(4);
  tail.writeUInt32BE(crc(Buffer.concat([head.subarray(4), body])), 0);
  return Buffer.concat([head, body, tail]);
}

// One bit a pixel, and a repeated row costs almost nothing with the "up" filter.
function packed(cells, width, box) {
  const stride = Math.ceil((box.width * SCALE) / 8);
  const rows = [];
  for (let y = 0; y < box.height; y += 1) {
    const row = Buffer.alloc(stride);
    for (let x = 0; x < box.width; x += 1) {
      if (!cells[(y + box.y) * width + x + box.x]) continue;
      for (let bit = x * SCALE; bit < (x + 1) * SCALE; bit += 1) row[bit >> 3] |= 128 >> (bit & 7);
    }
    rows.push(Buffer.concat([Buffer.of(0), row]));
    for (let copy = 1; copy < SCALE; copy += 1) rows.push(Buffer.concat([Buffer.of(2), Buffer.alloc(stride)]));
  }
  return deflateSync(Buffer.concat(rows), { level: 9 });
}

function frameControl(sequence, box) {
  const body = Buffer.alloc(26);
  body.writeUInt32BE(sequence, 0);
  body.writeUInt32BE(box.width * SCALE, 4);
  body.writeUInt32BE(box.height * SCALE, 8);
  body.writeUInt32BE(box.x * SCALE, 12);
  body.writeUInt32BE(box.y * SCALE, 16);
  body.writeUInt16BE(FRAME_MS, 20);
  body.writeUInt16BE(1000, 22);
  return chunk('fcTL', body);
}

// Only the globe moves, so every frame after the first holds just its square.
function animatedPng(frames, box, colors) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(COLUMNS * SCALE, 0);
  header.writeUInt32BE(ROWS * SCALE, 4);
  header[8] = 1;
  header[9] = 3;
  const control = Buffer.alloc(8);
  control.writeUInt32BE(frames.length, 0);
  const palette = Buffer.from(colors.flatMap((hex) => [1, 3, 5].map((at) => parseInt(hex.slice(at, at + 2), 16))));
  const parts = [
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk('IHDR', header),
    chunk('acTL', control),
    chunk('PLTE', palette),
    frameControl(0, { x: 0, y: 0, width: COLUMNS, height: ROWS }),
    chunk('IDAT', packed(frames[0], COLUMNS, { x: 0, y: 0, width: COLUMNS, height: ROWS })),
  ];
  let sequence = 1;
  for (const frame of frames.slice(1)) {
    parts.push(frameControl(sequence, box));
    sequence += 1;
    const number = Buffer.alloc(4);
    number.writeUInt32BE(sequence, 0);
    parts.push(chunk('fdAT', Buffer.concat([number, packed(frame, COLUMNS, box)])));
    sequence += 1;
  }
  parts.push(chunk('IEND', Buffer.alloc(0)));
  return Buffer.concat(parts);
}

const server = createDemoServer();
await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch();

const page = await browser.newPage({ viewport: { width: 880, height: 700 } });
await page.goto(`${origin}/demo/showcase.html`);
const hero = await page.evaluate(
  async ({ columns, rows, count, font, START_TURN }) => {
    const { ditherTones, thresholdMap } = await import('/src/core/dither.js');
    const { fillGlobe } = await import('/src/core/world.js');
    const map = await thresholdMap('bayer4');
    const canvas = new OffscreenCanvas(columns, rows);
    const painter = canvas.getContext('2d', { willReadFrequently: true });
    painter.font = `700 66px ${font}`;
    painter.textBaseline = 'middle';
    painter.fillText('fluidither', 8, rows / 2);
    const { width: wordWidth } = painter.measureText('fluidither');
    const { data } = painter.getImageData(0, 0, columns, rows);
    const wordTones = new Float64Array(columns * rows);
    for (let index = 0; index < wordTones.length; index += 1) {
      const fade = 1 - Math.max(index % columns - wordWidth * 0.2, 0) / (wordWidth * 1.15);
      wordTones[index] = data[index * 4 + 3] > 127 ? fade : 0;
    }
    const word = new Uint8Array(columns * rows);
    ditherTones(wordTones, columns, rows, map, Uint8Array.of(0, 1), word);

    const size = rows - 4;
    const left = columns - size - 2;
    const top = 2;
    const tones = new Float64Array(size * size);
    const cells = new Uint8Array(size * size);
    const frames = [];
    for (let frame = 0; frame < count; frame += 1) {
      fillGlobe(tones, size, size, (frame / count) * 360 + START_TURN);
      ditherTones(tones, size, size, map, Uint8Array.of(0, 1), cells);
      const whole = word.slice();
      for (let y = 0; y < size; y += 1) {
        for (let x = 0; x < size; x += 1) whole[(y + top) * columns + x + left] = cells[y * size + x];
      }
      frames.push([...whole]);
    }
    return { frames, box: { x: left, y: top, width: size, height: size } };
  },
  { columns: COLUMNS, rows: ROWS, count: FRAMES, font: tokens.font.sans, START_TURN: 20 },
);
for (const [name, theme] of Object.entries(tokens.themes)) {
  writeFileSync(join(out, `hero-${name}.png`), animatedPng(hero.frames, hero.box, [theme.paper, theme.ink]));
}
await page.close();

for (const colorScheme of ['light', 'dark']) {
  const shot = await browser.newPage({ colorScheme, deviceScaleFactor: 2, viewport: { width: 880, height: 900 } });
  await shot.goto(`${origin}/demo/showcase.html`);
  await shot.waitForTimeout(600);
  await shot.screenshot({ fullPage: true, path: join(out, `showcase-${colorScheme}.png`) });
  await shot.close();
}

await browser.close();
server.closeAllConnections();
server.close();

if (process.platform === 'darwin') {
  const manifest = join(root, 'gpui/Cargo.toml');
  for (const theme of ['light', 'dark']) {
    execFileSync('cargo', ['run', '--quiet', '--release', '--manifest-path', manifest, '--example', 'extras'], {
      env: { ...process.env, FD_THEME: theme, FD_CAPTURE: join(out, `gpui-${theme}.png`) },
      stdio: 'inherit',
    });
  }
}
