// <fd-menu> holds a button and an <fd-list> of actions. The first element child opens
// it. A `pick` event carries the chosen item's `value`.

import { List } from '../core/list.js';
import { follow } from '../core/overlay.js';

export { List };

export class Menu extends HTMLElement {
  #button = null;
  #list = null;
  #unfollow = null;

  connectedCallback() {
    if (this.#button) return;
    this.#button = this.firstElementChild;
    this.#list = this.querySelector(':scope > fd-list');
    const button = this.#button;
    const list = this.#list;
    list.popover = 'auto';
    list.tabIndex = -1;
    list.prepare('menu', 'menuitem');
    button.setAttribute('aria-haspopup', 'menu');
    button.setAttribute('aria-expanded', 'false');
    button.setAttribute('aria-controls', list.id);

    button.addEventListener('click', () => (this.open ? this.close() : this.show()));
    button.addEventListener('keydown', (event) => {
      if (event.key === 'ArrowDown') this.#openAt(event, () => list.first());
      else if (event.key === 'ArrowUp') this.#openAt(event, () => list.last());
    });
    list.addEventListener('keydown', (event) => this.#onKey(event));
    list.addEventListener('activate', ({ detail: { item } }) => {
      if (item) list.setAttribute('aria-activedescendant', item.id);
      else list.removeAttribute('aria-activedescendant');
    });
    list.addEventListener('pick', ({ detail: { item } }) => this.#choose(item));
    // Light dismiss closes the popover without going through close().
    list.addEventListener('toggle', (event) => {
      if (event.newState === 'closed') {
        button.setAttribute('aria-expanded', 'false');
        this.#unfollow?.();
        list.activate(null);
      }
    });
  }

  get open() {
    return this.#list.matches(':popover-open');
  }

  show() {
    this.#list.showPopover();
    this.#button.setAttribute('aria-expanded', 'true');
    this.#unfollow = follow(this.#list, this.#button);
    this.#list.focus();
  }

  close() {
    this.#list.hidePopover();
    this.#button.setAttribute('aria-expanded', 'false');
    this.#button.focus();
  }

  #openAt(event, move) {
    event.preventDefault();
    if (!this.open) this.show();
    move();
  }

  #choose(item) {
    this.close();
    this.dispatchEvent(new CustomEvent('pick', { bubbles: true, detail: { value: item.getAttribute('value') } }));
  }

  #onKey(event) {
    const list = this.#list;
    const actions = {
      ArrowDown: () => list.step(1),
      ArrowUp: () => list.step(-1),
      Home: () => list.first(),
      End: () => list.last(),
      Enter: () => list.active && this.#choose(list.active),
      ' ': () => list.active && this.#choose(list.active),
      Escape: () => this.close(),
      Tab: () => this.close(),
    };
    const action = actions[event.key];
    if (action) {
      if (event.key !== 'Tab') event.preventDefault();
      action();
    } else if (event.key.length === 1 && !event.metaKey && !event.ctrlKey) {
      list.typeahead(event.key);
    }
  }
}

customElements.define('fd-menu', Menu);
