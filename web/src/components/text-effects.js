// Text that arrives with some character. Each one keeps the real text available to
// assistive tech, and shows it at once under reduced motion.
// <fd-scramble>: letters settle from stand-in glyphs.
// <fd-dissolve>: text appears through a thinning dither. With `on-view` it waits until
// it scrolls into view. With hold="0.1 0.8" it never clears and keeps a dither across
// the text, from 0.1 at the start to 0.8 at the end, for dithered headings and logos.
// <fd-count value="1204">: a number that springs to its value.

import { CellCanvas } from '../core/cells.js';
import { ditherTones, fillGradient, thresholdMap } from '../core/dither.js';
import { prefersReducedMotion } from '../core/motion.js';
import { Glide, springs } from '../core/spring.js';
import { scrambled } from '../core/text.js';

const SCRAMBLE_MS = 700;
const SCRAMBLE_STEP_MS = 40;
const DISSOLVE_MS = 500;

export class Scramble extends HTMLElement {
  #timer = 0;
  #text = null;

  connectedCallback() {
    this.run();
  }

  disconnectedCallback() {
    clearInterval(this.#timer);
    this.textContent = this.#text;
  }

  /** Play it again, optionally with new text. */
  run(text = this.#text ?? this.textContent) {
    clearInterval(this.#timer);
    this.#text = text;
    this.setAttribute('aria-label', text);
    this.setAttribute('role', 'text');
    if (prefersReducedMotion()) {
      this.textContent = text;
      return;
    }
    const started = performance.now();
    let seed = 0;
    this.#timer = setInterval(() => {
      const progress = (performance.now() - started) / SCRAMBLE_MS;
      seed += 1;
      this.textContent = progress >= 1 ? text : scrambled(text, progress, seed);
      if (progress >= 1) clearInterval(this.#timer);
    }, SCRAMBLE_STEP_MS);
  }
}

export class Dissolve extends HTMLElement {
  #frame = 0;
  #watch = null;

  connectedCallback() {
    if (this.hasAttribute('hold')) {
      this.#hold();
    } else if (this.hasAttribute('on-view')) {
      this.#watch = new IntersectionObserver(([entry]) => {
        if (!entry.isIntersecting) return;
        this.#watch.disconnect();
        this.run();
      });
      this.#watch.observe(this);
    } else {
      this.run();
    }
  }

  disconnectedCallback() {
    cancelAnimationFrame(this.#frame);
    this.#watch?.disconnect();
  }

  #veil() {
    const overlay = document.createElement('span');
    overlay.className = 'fd-dissolve-veil';
    overlay.setAttribute('aria-hidden', 'true');
    this.append(overlay);
    const cells = new CellCanvas(overlay);
    if (!cells.fit(2)) {
      overlay.remove();
      return null;
    }
    const paper = getComputedStyle(overlay).backgroundColor;
    overlay.style.backgroundColor = 'transparent';
    return { overlay, cells, paper };
  }

  async #hold() {
    const [from = 0.5, to = from] = this.getAttribute('hold').trim().split(/\s+/).map(Number);
    const veil = this.#veil();
    if (!veil) return;
    const map = await thresholdMap('bayer4');
    const { columns, rows } = veil.cells;
    const tones = new Float64Array(columns * rows);
    const marks = new Uint8Array(columns * rows);
    fillGradient(tones, columns, rows, { mode: 0, angle: 90, centerX: 0.5, centerY: 0.5, radius: 1, from, to, warp: 0 }, 0);
    ditherTones(tones, columns, rows, map, Uint8Array.of(0, 1), marks);
    veil.cells.layer(marks, veil.paper, (mark) => mark === 1, true);
  }

  /** Play it again. */
  async run() {
    if (prefersReducedMotion() || this.hasAttribute('hold')) return;
    cancelAnimationFrame(this.#frame);
    this.querySelector(':scope > .fd-dissolve-veil')?.remove();
    const veil = this.#veil();
    if (!veil) return;
    const { overlay, cells, paper } = veil;
    const map = await thresholdMap('bayer4');
    if (!overlay.isConnected) return;
    const { columns, rows } = cells;
    const tones = new Float64Array(columns * rows);
    const marks = new Uint8Array(columns * rows);
    const started = performance.now();
    const step = (now) => {
      const left = 1 - (now - started) / DISSOLVE_MS;
      if (left <= 0) {
        overlay.remove();
        return;
      }
      tones.fill(left);
      ditherTones(tones, columns, rows, map, Uint8Array.of(0, 1), marks);
      cells.painter.clearRect(0, 0, columns, rows);
      cells.layer(marks, paper, (mark) => mark === 1, true);
      this.#frame = requestAnimationFrame(step);
    };
    this.#frame = requestAnimationFrame(step);
  }
}

export class Count extends HTMLElement {
  static observedAttributes = ['value'];

  #glide = null;

  connectedCallback() {
    const format = new Intl.NumberFormat(undefined, { maximumFractionDigits: Number(this.getAttribute('digits') ?? 0) });
    this.#glide ??= new Glide({ value: 0 }, springs.settle, ({ value }) => {
      this.textContent = format.format(value);
    });
    this.#glide.to({ value: Number(this.getAttribute('value') ?? 0) });
  }

  attributeChangedCallback() {
    if (this.#glide) this.#glide.to({ value: Number(this.getAttribute('value') ?? 0) });
  }
}

customElements.define('fd-scramble', Scramble);
customElements.define('fd-dissolve', Dissolve);
customElements.define('fd-count', Count);
