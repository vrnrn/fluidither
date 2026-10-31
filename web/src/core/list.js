// The list behind menus, selects, comboboxes and the command menu: <fd-list> holding
// <fd-item>s. It tracks one active item with aria-activedescendant, so focus can stay
// wherever the owner keeps it, and a dithered mark glides to that item.

import { Glide, springs } from './spring.js';

let lastId = 0;

const usable = (item) => !item.hasAttribute('disabled') && !item.hidden;

export class List extends HTMLElement {
  #mark = null;
  #active = null;
  #typed = '';
  #typedAt = 0;

  connectedCallback() {
    if (this.#mark) return;
    this.id ||= `fd-list-${(lastId += 1)}`;
    this.#mark = new Glide({ y: 0, height: 0, opacity: 0 }, springs.quick, (values) => {
      this.style.setProperty('--_mark-y', `${values.y}px`);
      this.style.setProperty('--_mark-height', `${values.height}px`);
      this.style.setProperty('--_mark-opacity', values.opacity);
    });
    this.addEventListener('pointerover', (event) => {
      if (event.pointerType === 'touch') return;
      const item = event.target.closest('fd-item');
      if (item && usable(item)) this.activate(item, { scroll: false });
    });
    this.addEventListener('pointerleave', () => this.activate(null));
    this.addEventListener('click', (event) => {
      const item = event.target.closest('fd-item');
      if (item && usable(item)) this.dispatchEvent(new CustomEvent('pick', { detail: { item } }));
    });
    // Clicking an item must not take focus away from whoever owns the list.
    this.addEventListener('pointerdown', (event) => event.preventDefault());
  }

  get items() {
    return [...this.querySelectorAll('fd-item')];
  }

  get choices() {
    return this.items.filter(usable);
  }

  get active() {
    return this.#active;
  }

  /** Give every item the role its owner needs. */
  prepare(listRole, itemRole) {
    this.setAttribute('role', listRole);
    for (const item of this.items) {
      item.id ||= `fd-item-${(lastId += 1)}`;
      item.setAttribute('role', itemRole);
      if (item.hasAttribute('disabled')) item.setAttribute('aria-disabled', 'true');
      else item.removeAttribute('aria-disabled');
    }
  }

  activate(item, { scroll = true, instantly = false } = {}) {
    this.#active?.removeAttribute('data-active');
    const wasHidden = this.#active === null;
    this.#active = item;
    if (!item) {
      this.#mark.to({ opacity: 0 });
      this.dispatchEvent(new CustomEvent('activate', { detail: { item: null } }));
      return;
    }
    item.setAttribute('data-active', '');
    if (scroll) item.scrollIntoView({ block: 'nearest' });
    const place = { y: item.offsetTop, height: item.offsetHeight };
    if (wasHidden || instantly) this.#mark.jump(place);
    this.#mark.to({ ...place, opacity: 1 });
    this.dispatchEvent(new CustomEvent('activate', { detail: { item } }));
  }

  /** Move the active item. `step` is 1 or -1, and it wraps. */
  step(step) {
    const choices = this.choices;
    if (choices.length === 0) return;
    const here = choices.indexOf(this.#active);
    const next = here < 0 ? (step > 0 ? 0 : choices.length - 1) : (here + step + choices.length) % choices.length;
    this.activate(choices[next]);
  }

  first() {
    this.activate(this.choices[0] ?? null);
  }

  last() {
    this.activate(this.choices.at(-1) ?? null);
  }

  /** Jump to the next item starting with what was typed. Pauses reset the text. */
  typeahead(character) {
    const now = performance.now();
    this.#typed = now - this.#typedAt > 600 ? character : this.#typed + character;
    this.#typedAt = now;
    const wanted = this.#typed.toLowerCase();
    const choices = this.choices;
    const from = choices.indexOf(this.#active) + (wanted.length === 1 ? 1 : 0);
    const ordered = [...choices.slice(from), ...choices.slice(0, from)];
    const match = ordered.find((item) => item.textContent.trim().toLowerCase().startsWith(wanted));
    if (match) this.activate(match);
  }
}

customElements.define('fd-list', List);
if (!customElements.get('fd-item')) customElements.define('fd-item', class extends HTMLElement {});
