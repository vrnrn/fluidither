// Dot maps and a globe from a coarse hand-drawn outline of the land, which is a sketch
// and not geographic data. gpui/src/world.rs is the same code, and fixtures/fields.json
// holds both to the same cells.

import { worldBase64, worldHeight, worldWidth } from './world-data.js';

const bits = Uint8Array.from(atob(worldBase64), (char) => char.charCodeAt(0));

/** Whether a place is land, with longitude and latitude in degrees. */
export function isLand(longitude, latitude) {
  const x = Math.floor(((longitude + 180) / 360) * worldWidth);
  const y = Math.floor(((90 - latitude) / 180) * worldHeight);
  const column = ((x % worldWidth) + worldWidth) % worldWidth;
  const row = Math.min(Math.max(y, 0), worldHeight - 1);
  const bit = row * worldWidth + column;
  return (bits[bit >> 3] >> (bit & 7)) & 1;
}

const DOT_SPACING = 3;

/** A dotted map, wrapped sideways by `turn` degrees. */
export function fillMap(tones, width, height, turn) {
  let index = 0;
  for (let y = 0; y < height; y += 1) {
    const latitude = 90 - ((y + 0.5) / height) * 180;
    for (let x = 0; x < width; x += 1) {
      const longitude = ((x + 0.5) / width) * 360 - 180 + turn;
      tones[index] = x % DOT_SPACING === 0 && y % DOT_SPACING === 0 && isLand(longitude, latitude) ? 1 : 0;
      index += 1;
    }
  }
}

const TILT = 0.35;

/** A globe seen from outside, turned `turn` degrees, lit from the front. */
export function fillGlobe(tones, width, height, turn) {
  const radius = Math.min(width, height) / 2 - 1;
  const tiltSin = Math.sin(TILT);
  const tiltCos = Math.cos(TILT);
  let index = 0;
  for (let y = 0; y < height; y += 1) {
    const dy = (height / 2 - y - 0.5) / radius;
    for (let x = 0; x < width; x += 1) {
      const dx = (x + 0.5 - width / 2) / radius;
      const reach = dx * dx + dy * dy;
      if (reach >= 1) {
        tones[index] = 0;
      } else {
        const front = Math.sqrt(1 - reach);
        const up = dy * tiltCos + front * tiltSin;
        const depth = front * tiltCos - dy * tiltSin;
        const latitude = (Math.asin(up) * 180) / Math.PI;
        const longitude = (Math.atan2(dx, depth) * 180) / Math.PI + turn;
        tones[index] = isLand(((longitude + 180) % 360) - 180, latitude) ? 0.2 + 0.5 * front : 0.1 * front;
      }
      index += 1;
    }
  }
}
