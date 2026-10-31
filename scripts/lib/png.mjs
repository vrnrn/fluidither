// A small PNG reader for measuring screenshots: 8-bit RGB or RGBA, no interlace.
import { inflateSync } from 'node:zlib';

export function readPng(buffer) {
  let offset = 8;
  let width = 0;
  let height = 0;
  let channels = 0;
  const data = [];
  while (offset < buffer.length) {
    const length = buffer.readUInt32BE(offset);
    const type = buffer.toString('latin1', offset + 4, offset + 8);
    const body = buffer.subarray(offset + 8, offset + 8 + length);
    if (type === 'IHDR') {
      width = body.readUInt32BE(0);
      height = body.readUInt32BE(4);
      channels = { 2: 3, 6: 4 }[body[9]];
      if (body[8] !== 8 || !channels || body[12] !== 0) throw new Error('unsupported PNG');
    } else if (type === 'IDAT') {
      data.push(body);
    }
    offset += length + 12;
  }
  const raw = inflateSync(Buffer.concat(data));
  const stride = width * channels;
  const pixels = Buffer.alloc(stride * height);
  const paeth = (a, b, c) => {
    const p = a + b - c;
    const [pa, pb, pc] = [Math.abs(p - a), Math.abs(p - b), Math.abs(p - c)];
    if (pa <= pb && pa <= pc) return a;
    return pb <= pc ? b : c;
  };
  for (let y = 0; y < height; y += 1) {
    const filter = raw[y * (stride + 1)];
    const line = y * (stride + 1) + 1;
    for (let x = 0; x < stride; x += 1) {
      const left = x >= channels ? pixels[y * stride + x - channels] : 0;
      const up = y > 0 ? pixels[(y - 1) * stride + x] : 0;
      const corner = y > 0 && x >= channels ? pixels[(y - 1) * stride + x - channels] : 0;
      const predicted = [0, left, up, (left + up) >> 1, paeth(left, up, corner)][filter];
      pixels[y * stride + x] = (raw[line + x] + predicted) & 255;
    }
  }
  return { width, height, channels, pixels };
}

/** The box of pixels differing from `reference` (or the region's center-left edge) inside a region. */
export function inkBounds({ width, channels, pixels }, region, threshold = 60) {
  const { x, y, w, h } = region;
  const at = (px, py) => (py * width + px) * channels;
  const reference = pixels.subarray(at(x, y + (h >> 1)), at(x, y + (h >> 1)) + 3);
  let [left, right, top, bottom] = [Infinity, -Infinity, Infinity, -Infinity];
  for (let py = y; py < y + h; py += 1) {
    for (let px = x; px < x + w; px += 1) {
      const p = at(px, py);
      const distance =
        Math.abs(pixels[p] - reference[0]) + Math.abs(pixels[p + 1] - reference[1]) + Math.abs(pixels[p + 2] - reference[2]);
      if (distance > threshold) {
        left = Math.min(left, px);
        right = Math.max(right, px);
        top = Math.min(top, py);
        bottom = Math.max(bottom, py);
      }
    }
  }
  return { left, right, top, bottom };
}

/** How far ink sits from the center of its region, in region pixels. Positive is right or down. */
export function inkOffset(image, region, threshold) {
  const { left, right, top, bottom } = inkBounds(image, region, threshold);
  return {
    x: (left - region.x - (region.x + region.w - 1 - right)) / 2,
    y: (top - region.y - (region.y + region.h - 1 - bottom)) / 2,
  };
}
