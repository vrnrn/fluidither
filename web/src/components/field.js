// <fd-field kind="plasma" speed="1" matrix="bayer4" cell="3">: an animated dithered
// background. Kinds: noise, waves, plasma, ripples, tunnel, metaballs, orb, stars,
// aurora, contours, beams, eclipse, horizon, mesh, grid, ridges, silk, smoke.
// Decorative, so it is hidden from assistive tech. The ink is its CSS `color`, and a
// speed of 0 draws one still frame, as reduced motion does.

import { ditherTones, thresholdMap } from '../core/dither.js';
import { CellCanvas, followTheme, unfollowTheme } from '../core/cells.js';
import { fillField } from '../core/fields.js';
import { animate } from '../core/motion.js';
import { dither } from '../core/tokens.js';

export class Field extends HTMLElement {
  static observedAttributes = ['kind', 'speed', 'matrix', 'cell'];

  #cells = null;
  #map = null;
  #sizes = null;
  #stop = null;
  #time = 0;

  connectedCallback() {
    this.#cells ??= new CellCanvas(this);
    this.setAttribute('aria-hidden', 'true');
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
    const wanted = this.getAttribute('matrix') ?? 'bayer4';
    const map = await thresholdMap(wanted);
    if (!this.#sizes || wanted !== (this.getAttribute('matrix') ?? 'bayer4')) return;
    this.#map = map;
    this.#stop?.();
    this.#stop = null;
    const speed = Number.parseFloat(this.getAttribute('speed') ?? '1');
    if (speed > 0) this.#stop = animate(this, (seconds) => this.#paint(seconds * speed), dither.fps);
    this.#paint(0);
  }

  repaint() {
    this.#paint(this.#time);
  }

  #paint(time) {
    this.#time = time;
    const cells = this.#cells;
    const size = Number.parseFloat(this.getAttribute('cell') ?? '') || Number.parseFloat(getComputedStyle(this).getPropertyValue('--fd-dither-cell')) || dither.cell;
    if (!this.#map || !cells.fit(size)) return;
    const { columns, rows } = cells;
    const count = columns * rows;
    if (this.tones?.length !== count) {
      this.tones = new Float64Array(count);
      this.marks = new Uint8Array(count);
    }
    fillField(this.tones, columns, rows, this.getAttribute('kind') ?? 'noise', time);
    ditherTones(this.tones, columns, rows, this.#map, Uint8Array.of(0, 1), this.marks);
    cells.layer(this.marks, getComputedStyle(this).color, (mark) => mark === 1, true);
  }
}

customElements.define('fd-field', Field);
