// <fd-select name value placeholder aria-label> holds an <fd-list> of options and
// behaves as the ARIA select-only combobox. It joins forms like a native control and
// fires `change`.

import { List } from '../core/list.js';
import { follow } from '../core/overlay.js';

export { List };

export class Select extends HTMLElement {
  static formAssociated = true;
  static observedAttributes = ['value'];

  #internals = this.attachInternals();
  #trigger = null;
  #text = null;
  #list = null;
  #unfollow = null;

  connectedCallback() {
    if (this.#trigger) return;
    this.#list = this.querySelector(':scope > fd-list');
    const list = this.#list;
    list.popover = 'manual';
    list.prepare('listbox', 'option');

    const trigger = document.createElement('button');
    trigger.type = 'button';
    trigger.className = 'fd-select-trigger';
    trigger.setAttribute('role', 'combobox');
    trigger.setAttribute('aria-haspopup', 'listbox');
    trigger.setAttribute('aria-expanded', 'false');
    trigger.setAttribute('aria-controls', list.id);
    const label = this.getAttribute('aria-label');
    if (label) trigger.setAttribute('aria-label', label);
    this.removeAttribute('aria-label');
    this.#text = document.createElement('span');
    trigger.append(this.#text);
    this.prepend(trigger);
    this.#trigger = trigger;

    trigger.addEventListener('click', () => (this.open ? this.close() : this.show()));
    trigger.addEventListener('keydown', (event) => this.#onKey(event));
    trigger.addEventListener('blur', () => {
      if (this.open) this.close();
    });
    list.addEventListener('activate', ({ detail: { item } }) => {
      if (item) trigger.setAttribute('aria-activedescendant', item.id);
      else trigger.removeAttribute('aria-activedescendant');
    });
    list.addEventListener('pick', ({ detail: { item } }) => this.#choose(item));
    this.#sync();
  }

  attributeChangedCallback() {
    if (this.#trigger) this.#sync();
  }

  get value() {
    return this.getAttribute('value') ?? '';
  }

  set value(value) {
    this.setAttribute('value', value);
  }

  get open() {
    return this.#list.matches(':popover-open');
  }

  #selected() {
    return this.#list.items.find((item) => item.getAttribute('value') === this.getAttribute('value')) ?? null;
  }

  #sync() {
    const selected = this.#selected();
    for (const item of this.#list.items) item.setAttribute('aria-selected', String(item === selected));
    this.#text.textContent = selected?.textContent.trim() ?? this.getAttribute('placeholder') ?? '';
    this.#trigger.toggleAttribute('data-placeholder', !selected);
    this.#internals.setFormValue(selected ? this.value : null);
  }

  show() {
    const list = this.#list;
    list.showPopover();
    list.style.minWidth = `${this.#trigger.offsetWidth}px`;
    this.#unfollow = follow(list, this.#trigger);
    this.#trigger.setAttribute('aria-expanded', 'true');
    list.activate(this.#selected() ?? list.choices[0] ?? null, { instantly: true });
  }

  close() {
    this.#list.hidePopover();
    this.#unfollow?.();
    this.#trigger.setAttribute('aria-expanded', 'false');
    this.#list.activate(null);
  }

  #choose(item) {
    const changed = item.getAttribute('value') !== this.value;
    this.value = item.getAttribute('value');
    this.close();
    if (changed) this.dispatchEvent(new Event('change', { bubbles: true }));
  }

  #onKey(event) {
    const list = this.#list;
    if (!this.open) {
      if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(event.key)) {
        event.preventDefault();
        this.show();
      } else if (event.key.length === 1 && !event.metaKey && !event.ctrlKey) {
        this.show();
        list.typeahead(event.key);
      }
      return;
    }
    const actions = {
      ArrowDown: () => list.step(1),
      ArrowUp: () => list.step(-1),
      Home: () => list.first(),
      End: () => list.last(),
      Enter: () => list.active && this.#choose(list.active),
      ' ': () => list.active && this.#choose(list.active),
      Escape: () => this.close(),
    };
    if (event.key === 'Tab') {
      if (list.active) this.#choose(list.active);
    } else if (actions[event.key]) {
      event.preventDefault();
      actions[event.key]();
    } else if (event.key.length === 1 && !event.metaKey && !event.ctrlKey) {
      list.typeahead(event.key);
    }
  }
}

customElements.define('fd-select', Select);
