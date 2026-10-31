// <fd-hover>: one dithered highlight that glides between the items inside, in place of each
// item lighting up on its own. It wraps any list, row of tabs or grid of cards. `items` is a
// selector for what counts, and the children by default. `nearest` keeps the closest item lit
// across the gaps, measured along `axis`: y, x or xy.

import { Glide, springs } from '../core/spring.js';

const usable = (item) => !item.hasAttribute('disabled') && item.getAttribute('aria-disabled') !== 'true';

/** The index of the box closest to a point, along `axis`. Boxes have left, top, right, bottom. */
export function nearestIndex(boxes, x, y, axis = 'xy') {
  let best = -1;
  let bestDistance = Infinity;
  boxes.forEach((box, index) => {
    const across = axis === 'y' ? 0 : Math.max(box.left - x, 0, x - box.right);
    const down = axis === 'x' ? 0 : Math.max(box.top - y, 0, y - box.bottom);
    const distance = Math.hypot(across, down);
    if (distance < bestDistance) {
      best = index;
      bestDistance = distance;
    }
  });
  return best;
}

export class Hover extends HTMLElement {
  #mark = null;
  #place = null;
  #fade = null;
  #item = null;

  connectedCallback() {
    if (this.#mark) return;
    this.#mark = document.createElement('span');
    this.#mark.className = 'fd-hover-mark';
    this.#mark.setAttribute('aria-hidden', 'true');
    this.prepend(this.#mark);
    const setVars = (values) => {
      for (const [name, value] of Object.entries(values)) this.style.setProperty(`--_mark-${name}`, name === 'opacity' ? value : `${value}px`);
    };
    this.#place = new Glide({ x: 0, y: 0, width: 0, height: 0 }, springs.glide, setVars);
    this.#fade = new Glide({ opacity: 0 }, springs.quick, setVars);
    this.addEventListener('pointermove', (event) => {
      if (event.pointerType !== 'touch') this.#follow(event);
    });
    this.addEventListener('pointerleave', () => this.#show(null));
    this.addEventListener('focusin', (event) => {
      if (event.target.matches(':focus-visible')) this.#show(this.#items.find((item) => item.contains(event.target)) ?? null);
    });
    this.addEventListener('focusout', () => this.#show(null));
  }

  get #items() {
    const selector = this.getAttribute('items') ?? ':scope > :not(.fd-hover-mark)';
    return [...this.querySelectorAll(selector)].filter(usable);
  }

  #follow({ clientX, clientY, target }) {
    const items = this.#items;
    if (!this.hasAttribute('nearest')) {
      this.#show(items.find((item) => item.contains(target)) ?? null);
      return;
    }
    const boxes = items.map((item) => item.getBoundingClientRect());
    this.#show(items[nearestIndex(boxes, clientX, clientY, this.getAttribute('axis') ?? 'xy')] ?? null);
  }

  #show(item) {
    if (item === this.#item) return;
    const wasHidden = this.#item === null;
    this.#item?.removeAttribute('data-hover');
    this.#item = item;
    if (!item) {
      this.#fade.to({ opacity: 0 }, springs.quickOut);
      return;
    }
    item.setAttribute('data-hover', '');
    const own = this.getBoundingClientRect();
    const box = item.getBoundingClientRect();
    const place = { x: box.left - own.left, y: box.top - own.top, width: box.width, height: box.height };
    if (wasHidden) this.#place.jump(place);
    else this.#place.to(place);
    this.#fade.to({ opacity: 1 }, springs.quick);
  }
}

customElements.define('fd-hover', Hover);
