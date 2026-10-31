// <fd-sparkline values="3 5 2 8" aria-label="Sales this week">: a small dithered
// chart. Give it a size in CSS, and its label says what the numbers mean.

import { ditherTones, fillSparkline, thresholdMap } from '../core/dither.js';

const OPAQUE = 0xff000000;
const FADE = 0.6;
const live = new Set();
let themeWatch = null;

function followTheme(element) {
  live.add(element);
  if (themeWatch) return;
  themeWatch = new MutationObserver(() => live.forEach((each) => each.repaint()));
  themeWatch.observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'data-theme'] });
  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => live.forEach((each) => each.repaint()));
}

export class Sparkline extends HTMLElement {
  static observedAttributes = ['values'];

  #canvas = null;
  #sizes = null;

  connectedCallback() {
    this.setAttribute('role', 'img');
    if (!this.#canvas) this.#canvas = this.appendChild(document.createElement('canvas'));
    this.#sizes = new ResizeObserver(() => this.repaint());
    this.#sizes.observe(this);
    followTheme(this);
  }

  disconnectedCallback() {
    this.#sizes.disconnect();
    this.#sizes = null;
    live.delete(this);
  }

  attributeChangedCallback() {
    if (this.#sizes) this.repaint();
  }

  async repaint() {
    const canvas = this.#canvas;
    const { width, height } = this.getBoundingClientRect();
    if (width === 0 || height === 0) return;
    const raw = (this.getAttribute('values') ?? '').trim().split(/\s+/).map(Number).filter(Number.isFinite);
    if (raw.length === 0) return;
    const low = Math.min(...raw);
    const range = Math.max(...raw) - low || 1;
    const values = raw.map((value) => (value - low) / range);

    const scale = devicePixelRatio;
    const fallback = Number.parseFloat(getComputedStyle(this).getPropertyValue('--fd-dither-hover-cell')) || 2;
    const deviceCell = Math.max(1, Math.round(fallback * scale));
    const cell = deviceCell / scale;
    const columns = Math.ceil(width / cell);
    const rows = Math.ceil(height / cell);
    canvas.width = columns;
    canvas.height = rows;
    canvas.style.width = `${columns * cell}px`;
    canvas.style.height = `${rows * cell}px`;

    const tones = new Float64Array(columns * rows);
    const words = new Uint32Array(columns * rows);
    fillSparkline(tones, columns, rows, values, FADE);
    ditherTones(tones, columns, rows, await thresholdMap('bayer4'), Uint32Array.of(0, OPAQUE), words);
    const painter = canvas.getContext('2d');
    painter.putImageData(new ImageData(new Uint8ClampedArray(words.buffer), columns, rows), 0, 0);
    painter.globalCompositeOperation = 'source-in';
    painter.fillStyle = getComputedStyle(this).color;
    painter.fillRect(0, 0, columns, rows);
    painter.globalCompositeOperation = 'source-over';
  }
}

customElements.define('fd-sparkline', Sparkline);
