// <fd-world kind="map|globe" speed="1" aria-label="...">: a dotted world or a turning
// globe, from a coarse sketch of the land. Good for decoration, not for geography.

import { CellCanvas, followTheme, unfollowTheme } from '../core/cells.js';
import { ditherTones, thresholdMap } from '../core/dither.js';
import { animate } from '../core/motion.js';
import { dither } from '../core/tokens.js';
import { fillGlobe, fillMap } from '../core/world.js';

const DEGREES_PER_SECOND = 12;
const CELL = 2;

export class World extends HTMLElement {
  static observedAttributes = ['kind', 'speed'];

  #cells = null;
  #map = null;
  #sizes = null;
  #stop = null;
  #turn = 0;

  connectedCallback() {
    this.#cells ??= new CellCanvas(this);
    this.setAttribute('role', 'img');
    this.#sizes = new ResizeObserver(() => this.repaint());
    this.#sizes.observe(this);
    followTheme(this);
    this.#start();
  }

  disconnectedCallback() {
    this.#sizes.disconnect();
    this.#sizes = null;
    this.#stop?.();
    this.#stop = null;
    unfollowTheme(this);
  }

  attributeChangedCallback() {
    if (this.#sizes) this.#start();
  }

  async #start() {
    this.#map = await thresholdMap('bayer4');
    if (!this.#sizes) return;
    this.#stop?.();
    this.#stop = null;
    const speed = Number.parseFloat(this.getAttribute('speed') ?? '1');
    if (this.getAttribute('kind') === 'globe' && speed > 0) {
      this.#stop = animate(this, (seconds) => this.#paint(seconds * speed * DEGREES_PER_SECOND), dither.fps);
    }
    this.#paint(this.#turn);
  }

  repaint() {
    this.#paint(this.#turn);
  }

  #paint(turn) {
    this.#turn = turn;
    const cells = this.#cells;
    if (!this.#map || !cells.fit(CELL)) return;
    const { columns, rows } = cells;
    const tones = new Float64Array(columns * rows);
    const marks = new Uint8Array(columns * rows);
    (this.getAttribute('kind') === 'globe' ? fillGlobe : fillMap)(tones, columns, rows, turn);
    ditherTones(tones, columns, rows, this.#map, Uint8Array.of(0, 1), marks);
    cells.layer(marks, getComputedStyle(this).color, (mark) => mark === 1, true);
  }
}

customElements.define('fd-world', World);
