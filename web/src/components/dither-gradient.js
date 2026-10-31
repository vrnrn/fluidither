// <fd-dither-gradient>: paints a dithered gradient into a canvas. The cells become
// an alpha mask, then the element's own CSS `color` fills it, so any color the
// browser understands works and themes need no special handling here. The paper is
// the element's background. Cells snap to device pixels to stay square.

import { ditherTones, fillGradient, thresholdMap } from '../core/dither.js';
import { animate } from '../core/motion.js';
import { dither } from '../core/tokens.js';

const OPAQUE = 0xff000000;
const MODES = { linear: 0, radial: 1, conic: 2 };

const live = new Set();
let themeObserver = null;

function followTheme(element) {
  live.add(element);
  if (themeObserver) return;
  const repaintAll = () => live.forEach((each) => each.repaint());
  themeObserver = new MutationObserver(repaintAll);
  themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'data-theme'] });
  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', repaintAll);
}

export class DitherGradient extends HTMLElement {
  static observedAttributes = ['mode', 'angle', 'center', 'radius', 'from', 'to', 'warp', 'speed', 'matrix', 'cell'];

  #canvas = null;
  #painter = null;
  #sizeWatch = null;
  #stop = null;
  #map = null;
  #tones = null;
  #mask = null;
  #time = 0;

  connectedCallback() {
    if (!this.#canvas) {
      this.#canvas = this.appendChild(document.createElement('canvas'));
      this.#canvas.setAttribute('aria-hidden', 'true');
      this.#painter = this.#canvas.getContext('2d');
    }
    this.#sizeWatch = new ResizeObserver(() => this.#resize());
    this.#sizeWatch.observe(this);
    followTheme(this);
    this.#configure();
  }

  disconnectedCallback() {
    this.#sizeWatch.disconnect();
    this.#sizeWatch = null;
    this.#stop?.();
    this.#stop = null;
    live.delete(this);
  }

  attributeChangedCallback() {
    if (this.#sizeWatch) this.#configure();
  }

  repaint() {
    this.#paint(this.#time);
  }

  #number(name, fallback) {
    const value = Number.parseFloat(this.getAttribute(name));
    return Number.isNaN(value) ? fallback : value;
  }

  async #configure() {
    const wanted = this.getAttribute('matrix') ?? 'bayer4';
    const map = await thresholdMap(wanted);
    // The attribute may have changed while a table loaded, or the element gone.
    if (!this.#sizeWatch || wanted !== (this.getAttribute('matrix') ?? 'bayer4')) return;
    this.#map = map;
    this.#resize();
    this.#stop?.();
    this.#stop = null;
    const speed = this.#number('speed', 0);
    if (speed > 0) this.#stop = animate(this, (seconds) => this.#paint(seconds * speed), dither.fps);
    else this.#paint(0);
  }

  #resize() {
    if (!this.#map) return;
    const { width, height } = this.getBoundingClientRect();
    if (width === 0 || height === 0) return;
    const scale = devicePixelRatio;
    const fallback = Number.parseFloat(getComputedStyle(this).getPropertyValue('--fd-dither-cell')) || dither.cell;
    const devicePixels = Math.max(1, Math.round(this.#number('cell', fallback) * scale));
    const cell = devicePixels / scale;
    const columns = Math.ceil(width / cell);
    const rows = Math.ceil(height / cell);
    const canvas = this.#canvas;
    canvas.style.width = `${columns * cell}px`;
    canvas.style.height = `${rows * cell}px`;
    if (canvas.width === columns && canvas.height === rows) return;
    canvas.width = columns;
    canvas.height = rows;
    this.#tones = new Float64Array(columns * rows);
    const words = new Uint32Array(columns * rows);
    this.#mask = { words, image: new ImageData(new Uint8ClampedArray(words.buffer), columns, rows) };
    this.#paint(this.#time);
  }

  #gradient() {
    const [centerX = 0.5, centerY = 0.5] = (this.getAttribute('center') ?? '').trim().split(/\s+/).filter(Boolean).map(Number);
    const mode = MODES[this.getAttribute('mode')] ?? MODES.linear;
    return {
      mode,
      angle: this.#number('angle', mode === MODES.linear ? 180 : 0),
      centerX,
      centerY,
      radius: Math.max(0.01, this.#number('radius', 1)),
      from: this.#number('from', 0),
      to: this.#number('to', 1),
      warp: this.#number('warp', 0.12),
    };
  }

  #paint(time) {
    if (!this.#mask) return;
    this.#time = time;
    const { width, height } = this.#canvas;
    fillGradient(this.#tones, width, height, this.#gradient(), time);
    ditherTones(this.#tones, width, height, this.#map, Uint32Array.of(0, OPAQUE), this.#mask.words);
    const painter = this.#painter;
    painter.putImageData(this.#mask.image, 0, 0);
    painter.globalCompositeOperation = 'source-in';
    painter.fillStyle = getComputedStyle(this).color;
    painter.fillRect(0, 0, width, height);
    painter.globalCompositeOperation = 'source-over';
  }
}

customElements.define('fd-dither-gradient', DitherGradient);
