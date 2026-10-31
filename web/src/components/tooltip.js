// <fd-tooltip text="Save"> wraps one control. The text shows after a short hover, or
// at once on keyboard focus, and Escape dismisses it.

import { place } from '../core/overlay.js';

const HOVER_DELAY = 350;
let lastId = 0;

export class Tooltip extends HTMLElement {
  #tip = null;
  #timer = 0;

  connectedCallback() {
    const control = this.firstElementChild;
    if (!control || this.#tip) return;
    const tip = document.createElement('span');
    tip.className = 'fd-tooltip';
    tip.id = `fd-tooltip-${(lastId += 1)}`;
    tip.setAttribute('role', 'tooltip');
    tip.popover = 'manual';
    tip.textContent = this.getAttribute('text');
    this.append(tip);
    this.#tip = tip;
    control.setAttribute('aria-describedby', tip.id);

    this.addEventListener('pointerenter', (event) => {
      if (event.pointerType !== 'touch') this.#timer = setTimeout(() => this.show(), HOVER_DELAY);
    });
    this.addEventListener('pointerleave', () => this.hide());
    this.addEventListener('focusin', (event) => {
      if (event.target.matches(':focus-visible')) this.show();
    });
    this.addEventListener('focusout', () => this.hide());
    this.addEventListener('keydown', (event) => {
      if (event.key === 'Escape') this.hide();
    });
  }

  show() {
    clearTimeout(this.#timer);
    const tip = this.#tip;
    if (tip.matches(':popover-open')) return;
    tip.textContent = this.getAttribute('text');
    tip.showPopover();
    place(tip, this.firstElementChild, { side: 'top', align: 'center' });
  }

  hide() {
    clearTimeout(this.#timer);
    if (this.#tip.matches(':popover-open')) this.#tip.hidePopover();
  }
}

customElements.define('fd-tooltip', Tooltip);
