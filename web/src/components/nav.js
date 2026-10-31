// <fd-nav> moves its thumb to the current link and its dithered peek to the hovered
// one. The links stay plain links, so they work with or without this script.

import { Glide, springs } from '../core/spring.js';

export class Nav extends HTMLElement {
  #thumb = null;
  #peek = null;
  #peeking = null;
  #sizes = null;

  connectedCallback() {
    if (!this.hasAttribute('role')) this.setAttribute('role', 'navigation');
    const setVars = (prefix) => (values) => {
      for (const [name, value] of Object.entries(values)) {
        this.style.setProperty(`--_${prefix}-${name}`, name === 'opacity' ? value : `${value}px`);
      }
    };
    this.#thumb ??= new Glide({ y: 0, height: 0 }, springs.glide, setVars('thumb'));
    this.#peek ??= new Glide({ y: 0, height: 0, opacity: 0 }, springs.quick, setVars('peek'));
    this.#sizes = new ResizeObserver(() => this.#placeThumb(true));
    this.#sizes.observe(this);
    this.addEventListener('pointerover', this.#onOver);
    this.addEventListener('pointerleave', this.#onLeave);
    this.addEventListener('click', this.#onClick);
    this.#placeThumb(true);
  }

  disconnectedCallback() {
    this.#sizes.disconnect();
    this.#thumb.stop();
    this.#peek.stop();
    this.removeEventListener('pointerover', this.#onOver);
    this.removeEventListener('pointerleave', this.#onLeave);
    this.removeEventListener('click', this.#onClick);
  }

  get #current() {
    return this.querySelector('a[aria-current]');
  }

  /** Mark a link as the current page, for apps that navigate without a reload. */
  select(link) {
    this.#current?.removeAttribute('aria-current');
    link.setAttribute('aria-current', 'page');
    this.#placeThumb(false);
    this.#show(null);
  }

  #placeThumb(instantly) {
    const link = this.#current;
    if (!link) return;
    const place = { y: link.offsetTop, height: link.offsetHeight };
    if (instantly) this.#thumb.jump(place);
    else this.#thumb.to(place);
  }

  #show(link) {
    const shown = link && link !== this.#current ? link : null;
    if (shown === this.#peeking) return;
    const wasHidden = this.#peeking === null;
    this.#peeking = shown;
    if (!shown) {
      this.#peek.to({ opacity: 0 });
      return;
    }
    const place = { y: shown.offsetTop, height: shown.offsetHeight };
    if (wasHidden) this.#peek.jump(place);
    this.#peek.to({ ...place, opacity: 1 });
  }

  #onOver = (event) => {
    if (event.pointerType !== 'touch') this.#show(event.target.closest('a'));
  };

  #onLeave = () => this.#show(null);

  #onClick = (event) => {
    const link = event.target.closest('a');
    if (link && this.hasAttribute('in-page')) {
      event.preventDefault();
      this.select(link);
    }
  };
}

customElements.define('fd-nav', Nav);
