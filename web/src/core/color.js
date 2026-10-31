// Color math. hsv_to_rgb in gpui/src/extras.rs is the same.

/** Hue 0 to 360, saturation and value 0 to 1. Returns red, green and blue from 0 to 1. */
export function hsvToRgb(hue, saturation, value) {
  const turn = (((hue % 360) + 360) % 360) / 60;
  const chroma = value * saturation;
  const rising = chroma * (1 - Math.abs((turn % 2) - 1));
  const [red, green, blue] = [[chroma, rising, 0], [rising, chroma, 0], [0, chroma, rising], [0, rising, chroma], [rising, 0, chroma], [chroma, 0, rising]][Math.min(Math.floor(turn), 5)];
  const base = value - chroma;
  return [red + base, green + base, blue + base];
}

/** The other way. Returns hue, saturation and value. */
export function rgbToHsv(red, green, blue) {
  const most = Math.max(red, green, blue);
  const spread = most - Math.min(red, green, blue);
  if (spread === 0) return [0, 0, most];
  const sector = most === red ? ((green - blue) / spread + 6) % 6 : most === green ? (blue - red) / spread + 2 : (red - green) / spread + 4;
  return [sector * 60, spread / most, most];
}

export const toHex = (channels) => `#${channels.map((channel) => Math.round(channel * 255).toString(16).padStart(2, '0')).join('')}`;

/** Red, green and blue from 0 to 1, or null if it isn't a six digit hex color. */
export function fromHex(text) {
  const match = /^#?([0-9a-f]{6})$/i.exec(text.trim());
  if (!match) return null;
  return [0, 2, 4].map((at) => Number.parseInt(match[1].slice(at, at + 2), 16) / 255);
}
