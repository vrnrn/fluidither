// <fd-combobox name placeholder aria-label> is a text field with a filtered <fd-list>.
// The value is the chosen item's `value`. It fires `change`, and the ARIA combobox
// pattern applies: arrows move, Enter chooses, Escape closes.

import { List } from '../core/list.js';
import { follow } from '../core/overlay.js';

export { List };

/** Show the items matching `query`. Shared with the command menu. */
export function filterList(list, query) {
  const wanted = query.trim().toLowerCase();
  let shown = 0;
  for (const item of list.items) {
    item.hidden = !item.textContent.toLowerCase().includes(wanted);
    if (!item.hidden) shown += 1;
  }
  let empty = list.querySelector('fd-list-empty');
  if (shown === 0 && !empty) {
    empty = document.createElement('fd-list-empty');
    empty.textContent = list.getAttribute('empty') ?? 'No matches';
    list.append(empty);
  } else if (shown > 0) {
    empty?.remove();
  }
  return shown;
}

export class Combobox extends HTMLElement {
  static formAssociated = true;

  #internals = this.attachInternals();
  #input = null;
  #list = null;
  #unfollow = null;
  #value = '';

  connectedCallback() {
    if (this.#input) return;
    this.#list = this.querySelector(':scope > fd-list');
    const list = this.#list;
    list.popover = 'manual';
    list.prepare('listbox', 'option');

    const input = document.createElement('input');
    input.className = 'fd-input';
    input.setAttribute('role', 'combobox');
    input.setAttribute('aria-autocomplete', 'list');
    input.setAttribute('aria-expanded', 'false');
    input.setAttribute('aria-controls', list.id);
    input.autocomplete = 'off';
    input.placeholder = this.getAttribute('placeholder') ?? '';
    const label = this.getAttribute('aria-label');
    if (label) input.setAttribute('aria-label', label);
    this.removeAttribute('aria-label');
    this.prepend(input);
    this.#input = input;

    input.addEventListener('input', () => {
      if (!this.open) this.show();
      filterList(list, input.value);
      list.first();
    });
    input.addEventListener('keydown', (event) => this.#onKey(event));
    input.addEventListener('blur', () => {
      if (this.open) this.close();
    });
    list.addEventListener('activate', ({ detail: { item } }) => {
      if (item) input.setAttribute('aria-activedescendant', item.id);
      else input.removeAttribute('aria-activedescendant');
    });
    list.addEventListener('pick', ({ detail: { item } }) => this.#choose(item));
  }

  get value() {
    return this.#value;
  }

  get open() {
    return this.#list.matches(':popover-open');
  }

  show() {
    const list = this.#list;
    list.showPopover();
    list.style.minWidth = `${this.#input.offsetWidth}px`;
    this.#unfollow = follow(list, this.#input);
    this.#input.setAttribute('aria-expanded', 'true');
  }

  close() {
    this.#list.hidePopover();
    this.#unfollow?.();
    this.#input.setAttribute('aria-expanded', 'false');
    this.#list.activate(null);
  }

  #choose(item) {
    this.#value = item.getAttribute('value');
    this.#input.value = item.textContent.trim();
    this.#internals.setFormValue(this.#value);
    for (const each of this.#list.items) each.setAttribute('aria-selected', String(each === item));
    this.close();
    this.dispatchEvent(new Event('change', { bubbles: true }));
  }

  #onKey(event) {
    const list = this.#list;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (!this.open) {
        this.show();
        filterList(list, this.#input.value);
      }
      list.step(event.key === 'ArrowDown' ? 1 : -1);
    } else if (event.key === 'Enter' && this.open && list.active) {
      event.preventDefault();
      this.#choose(list.active);
    } else if (event.key === 'Escape' && this.open) {
      event.preventDefault();
      this.close();
    }
  }
}

customElements.define('fd-combobox', Combobox);
