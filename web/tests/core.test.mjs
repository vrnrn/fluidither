import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { ditherTones, fillGradient, fillSparkline, thresholdMap } from '../src/core/dither.js';
import { Spring, springs, stepSpring } from '../src/core/spring.js';
import { hashCells } from '../../scripts/gen-fixtures.mjs';

const fixtures = JSON.parse(readFileSync(new URL('../../fixtures/dither.json', import.meta.url)));

test('every matrix holds each rank once and thresholds stay inside (0, 1)', async () => {
  for (const [name, size] of [['bayer2', 2], ['bayer4', 4], ['bayer8', 8]]) {
    const map = await thresholdMap(name);
    assert.equal(map.size, size);
    assert.equal(new Set(map.thresholds).size, size * size);
    assert.ok(map.thresholds.every((value) => value > 0 && value < 1));
  }
  const noise = await thresholdMap('blue-noise');
  assert.equal(noise.thresholds.length, 4096);
  assert.equal(new Set(noise.thresholds).size, 256);
});

test('a flat tone inks that share of the cells', async () => {
  const map = await thresholdMap('bayer8');
  for (const tone of [0, 0.25, 0.5, 0.75, 1]) {
    const tones = new Float64Array(64).fill(tone);
    const out = new Uint32Array(64);
    ditherTones(tones, 8, 8, map, Uint32Array.of(0, 1), out);
    assert.equal(out.reduce((a, b) => a + b, 0), tone * 64);
  }
});

test('a three-color palette only mixes neighbours', async () => {
  const map = await thresholdMap('bayer4');
  const out = new Uint32Array(16);
  ditherTones(new Float64Array(16).fill(0.25), 4, 4, map, Uint32Array.of(0, 1, 2), out);
  assert.deepEqual([...new Set(out)].sort(), [0, 1]);
});

test('dither output matches the shared fixture', async () => {
  for (const { name, matrix, width, height, t, gradient, colors, hash } of fixtures.cases) {
    const tones = new Float64Array(width * height);
    const out = new Uint32Array(width * height);
    fillGradient(tones, width, height, gradient, t);
    ditherTones(tones, width, height, await thresholdMap(matrix), Uint32Array.from({ length: colors }, (_, i) => i), out);
    assert.equal(hashCells(out), hash, name);
  }
});

test('critically damped springs never overshoot and land by their duration', () => {
  for (const config of [springs.quick, springs.glide]) {
    const spring = new Spring(config, 0);
    spring.target = 1;
    let time = 0;
    while (time < config.duration) {
      spring.step(1 / 240);
      time += 1 / 240;
      assert.ok(spring.position <= 1);
    }
    assert.ok(1 - spring.position < 0.002, `${spring.position}`);
  }
});

test('the bouncy spring overshoots a little and still lands', () => {
  const spring = new Spring(springs.settle, 0);
  spring.target = 1;
  let peak = 0;
  for (let frame = 0; frame < 240; frame += 1) {
    spring.step(1 / 240);
    peak = Math.max(peak, spring.position);
  }
  assert.ok(peak > 1 && peak < 1.05, `peak ${peak}`);
  assert.ok(Math.abs(spring.position - 1) < 0.002);
});

test('the result does not depend on the frame rate', () => {
  for (const config of Object.values(springs)) {
    const [whole] = stepSpring(config, 0, 0, 1, 0.1);
    let position = 0;
    let velocity = 0;
    for (let frame = 0; frame < 10; frame += 1) {
      [position, velocity] = stepSpring(config, position, velocity, 1, 0.01);
    }
    assert.ok(Math.abs(whole - position) < 1e-9);
  }
});

test('retargeting keeps velocity', () => {
  const spring = new Spring(springs.glide, 0);
  spring.target = 100;
  spring.step(0.03);
  const before = spring.velocity;
  spring.target = 200;
  assert.equal(spring.velocity, before);
  assert.ok(before > 0);
});

test('a landed spring reports done and stops changing', () => {
  const spring = new Spring(springs.quick, 5);
  assert.equal(spring.step(1 / 60), true);
  assert.equal(spring.position, 5);
});

test('a sparkline is solid on its line, empty above, and fades below', () => {
  const tones = new Float64Array(10 * 11);
  fillSparkline(tones, 10, 11, [0.5, 0.5], 0.6);
  const column = (row) => tones[row * 10 + 3];
  assert.equal(column(4), 0, 'above the line');
  assert.equal(column(5), 1, 'on the line');
  assert.ok(column(6) > column(9) && column(9) > 0, 'fading below');
});

test('a single value still draws', () => {
  const tones = new Float64Array(4 * 4);
  fillSparkline(tones, 4, 4, [1], 0.6);
  assert.ok(tones.every(Number.isFinite));
});

test('colors convert both ways, the same as GPUI', async () => {
  const { fromHex, hsvToRgb, rgbToHsv, toHex } = await import('../src/core/color.js');
  assert.deepEqual(hsvToRgb(0, 1, 1), [1, 0, 0]);
  assert.deepEqual(hsvToRgb(120, 1, 1), [0, 1, 0]);
  assert.deepEqual(hsvToRgb(360, 1, 1), [1, 0, 0]);
  assert.equal(toHex(hsvToRgb(...rgbToHsv(...fromHex('#fc9e47')))), '#fc9e47');
  assert.equal(fromHex('nope'), null);
});

test('the nearest box wins, along the chosen axis', async () => {
  globalThis.HTMLElement ??= class {};
  globalThis.customElements ??= { define() {} };
  const { nearestIndex } = await import('../src/components/hover.js');
  const boxes = [{ left: 0, top: 0, right: 10, bottom: 10 }, { left: 0, top: 20, right: 10, bottom: 30 }];
  assert.equal(nearestIndex(boxes, 5, 14), 0);
  assert.equal(nearestIndex(boxes, 5, 17), 1);
  assert.equal(nearestIndex(boxes, 500, 17, 'y'), 1);
  assert.equal(nearestIndex([], 0, 0), -1);
});
