// <fd-command> is a searchable list of actions in a modal: <fd-command aria-label>
// around an <fd-list>. Call .show() from your own shortcut. A `pick` event carries the
// chosen `value`.

import { List } from '../core/list.js';
import { filterList } from './combobox.js';

export { List };

export class Command extends HTMLElement {
  #dialog = null;
  #input = null;
  #list = null;

  connectedCallback() {
    if (this.#dialog) return;
    const list = this.querySelector(':scope > fd-list');
    this.#list = list;
    list.prepare('listbox', 'option');

    const dialog = document.createElement('dialog');
    dialog.className = 'fd-dialog fd-command';
    const label = this.getAttribute('aria-label') ?? 'Commands';
    this.removeAttribute('aria-label');
    dialog.setAttribute('aria-label', label);
    const input = document.createElement('input');
    input.className = 'fd-input';
    input.placeholder = this.getAttribute('placeholder') ?? 'Search';
    input.setAttribute('role', 'combobox');
    input.setAttribute('aria-label', label);
    input.setAttribute('aria-autocomplete', 'list');
    input.setAttribute('aria-expanded', 'true');
    input.setAttribute('aria-controls', list.id);
    input.autocomplete = 'off';
    dialog.append(input, list);
    this.append(dialog);
    this.#dialog = dialog;
    this.#input = input;

    input.addEventListener('input', () => {
      filterList(list, input.value);
      list.first();
    });
    input.addEventListener('keydown', (event) => this.#onKey(event));
    list.addEventListener('activate', ({ detail: { item } }) => {
      if (item) input.setAttribute('aria-activedescendant', item.id);
      else input.removeAttribute('aria-activedescendant');
    });
    list.addEventListener('pick', ({ detail: { item } }) => this.#choose(item));
    dialog.addEventListener('click', (event) => {
      if (event.target === dialog) this.close();
    });
  }

  get open() {
    return this.#dialog.open;
  }

  show() {
    this.#input.value = '';
    filterList(this.#list, '');
    this.#dialog.showModal();
    this.#input.focus();
    this.#list.first();
  }

  close() {
    this.#dialog.close();
  }

  #choose(item) {
    this.close();
    this.dispatchEvent(new CustomEvent('pick', { bubbles: true, detail: { value: item.getAttribute('value') } }));
  }

  #onKey(event) {
    const list = this.#list;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      list.step(event.key === 'ArrowDown' ? 1 : -1);
    } else if (event.key === 'Enter' && list.active) {
      event.preventDefault();
      this.#choose(list.active);
    }
  }
}

customElements.define('fd-command', Command);
