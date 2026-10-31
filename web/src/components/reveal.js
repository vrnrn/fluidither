// <fd-reveal>: its children rise into place one after another when it scrolls into view. `subtle`
// makes the move smaller. Content is visible until this runs, so nothing hides without
// JavaScript, and reduced motion shows it at once. `.run()` plays it again.

import { prefersReducedMotion } from '../core/motion.js';

export class Reveal extends HTMLElement {
  #watch = null;

  connectedCallback() {
    if (prefersReducedMotion()) return;
    [...this.children].forEach((child, index) => child.style.setProperty('--_index', index));
    this.dataset.state = 'waiting';
    this.#watch = new IntersectionObserver(([entry]) => {
      if (!entry.isIntersecting) return;
      this.#watch.disconnect();
      this.dataset.state = 'shown';
    });
    this.#watch.observe(this);
  }

  disconnectedCallback() {
    this.#watch?.disconnect();
  }

  /** Play it again. */
  run() {
    if (prefersReducedMotion()) return;
    this.dataset.state = 'waiting';
    this.getBoundingClientRect();
    requestAnimationFrame(() => {
      this.dataset.state = 'shown';
    });
  }
}

customElements.define('fd-reveal', Reveal);
