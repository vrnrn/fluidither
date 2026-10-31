// <fd-dismiss> goes inside a .fd-banner and adds a close button. The banner leaves on the fast
// exit spring, then fires `dismiss` from the document and is removed.

import { prefersReducedMotion } from '../core/motion.js';

export class Dismiss extends HTMLElement {
  connectedCallback() {
    if (this.firstElementChild) return;
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'fd-button';
    button.dataset.look = 'ghost';
    button.dataset.density = 'compact';
    button.toggleAttribute('data-icon-only', true);
    button.setAttribute('aria-label', this.getAttribute('label') ?? 'Dismiss');
    button.innerHTML = '<svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><path d="M3 3h2v2H3zM11 3h2v2h-2zM5 5h2v2H5zM9 5h2v2H9zM7 7h2v2H7zM5 9h2v2H5zM9 9h2v2H9zM3 11h2v2H3zM11 11h2v2h-2z" /></svg>';
    button.addEventListener('click', () => this.#close());
    this.append(button);
  }

  async #close() {
    const banner = this.closest('.fd-banner');
    banner.dataset.leaving = '';
    if (!prefersReducedMotion()) await Promise.allSettled(banner.getAnimations().map((each) => each.finished));
    const parent = banner.parentElement;
    banner.remove();
    parent?.dispatchEvent(new Event('dismiss', { bubbles: true }));
  }
}

customElements.define('fd-dismiss', Dismiss);
