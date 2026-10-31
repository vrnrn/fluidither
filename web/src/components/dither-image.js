// <fd-dither-image src="photo.jpg" alt="..." matrix="halftone" cell="3">: a picture
// redrawn in dither cells. The image must be same-origin so its pixels can be read.
// Under a strict CSP it also needs to come from an allowed `img-src`.

import { CellCanvas, followTheme, unfollowTheme } from '../core/cells.js';
import { ditherTones, thresholdMap } from '../core/dither.js';

// Ink is laid where the picture matches it, so a dark theme's light ink falls on the
// bright parts and the picture never turns into a negative.
function isLight(color) {
  const probe = new OffscreenCanvas(1, 1).getContext('2d', { willReadFrequently: true });
  probe.fillStyle = color;
  probe.fillRect(0, 0, 1, 1);
  const [red, green, blue] = probe.getImageData(0, 0, 1, 1).data;
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue > 127;
}

export class DitherImage extends HTMLElement {
  static observedAttributes = ['src', 'matrix', 'cell', 'alt'];

  #cells = null;
  #picture = null;
  #map = null;
  #sizes = null;

  connectedCallback() {
    this.#cells ??= new CellCanvas(this);
    this.setAttribute('role', 'img');
    this.#label();
    this.#sizes = new ResizeObserver(() => this.repaint());
    this.#sizes.observe(this);
    followTheme(this);
    this.#load();
  }

  disconnectedCallback() {
    this.#sizes.disconnect();
    this.#sizes = null;
    unfollowTheme(this);
  }

  attributeChangedCallback(name) {
    if (!this.#sizes) return;
    if (name === 'alt') this.#label();
    else this.#load();
  }

  #label() {
    this.setAttribute('aria-label', this.getAttribute('alt') ?? '');
  }

  async #load() {
    const source = this.getAttribute('src');
    this.#map = await thresholdMap(this.getAttribute('matrix') ?? 'halftone');
    if (!source) return;
    const picture = new Image();
    picture.src = source;
    await picture.decode();
    this.#picture = picture;
    this.dispatchEvent(new Event('load'));
    this.repaint();
  }

  repaint() {
    const cells = this.#cells;
    const size = Number.parseFloat(this.getAttribute('cell') ?? '') || 3;
    if (!this.#picture || !this.#map || !cells.fit(size)) return;
    const { columns, rows } = cells;
    const sample = new OffscreenCanvas(columns, rows).getContext('2d', { willReadFrequently: true });
    sample.drawImage(this.#picture, 0, 0, columns, rows);
    const { data } = sample.getImageData(0, 0, columns, rows);
    const ink = getComputedStyle(this).color;
    const lightInk = isLight(ink);
    const tones = new Float64Array(columns * rows);
    for (let index = 0; index < tones.length; index += 1) {
      const at = index * 4;
      const brightness = (0.2126 * data[at] + 0.7152 * data[at + 1] + 0.0722 * data[at + 2]) / 255;
      // See-through parts stay paper, so logos on transparency work.
      tones[index] = (lightInk ? brightness : 1 - brightness) * (data[at + 3] / 255);
    }
    const marks = new Uint8Array(tones.length);
    ditherTones(tones, columns, rows, this.#map, Uint8Array.of(0, 1), marks);
    cells.layer(marks, ink, (mark) => mark === 1, true);
  }
}

customElements.define('fd-dither-image', DitherImage);
