// <fd-date-picker name value placeholder aria-label> opens an <fd-calendar> in a
// popover and joins forms like a native control.

import { parseIso } from '../core/dates.js';
import { follow } from '../core/overlay.js';
import './calendar.js';

export class DatePicker extends HTMLElement {
  static formAssociated = true;
  static observedAttributes = ['value'];

  #internals = this.attachInternals();
  #trigger = null;
  #panel = null;
  #calendar = null;
  #unfollow = null;

  connectedCallback() {
    if (this.#trigger) return;
    const trigger = document.createElement('button');
    trigger.type = 'button';
    trigger.className = 'fd-select-trigger';
    trigger.setAttribute('aria-haspopup', 'dialog');
    trigger.setAttribute('aria-expanded', 'false');
    const label = this.getAttribute('aria-label');
    if (label) trigger.setAttribute('aria-label', label);
    this.removeAttribute('aria-label');
    this.#trigger = this.appendChild(trigger);

    const panel = document.createElement('div');
    panel.className = 'fd-date-panel';
    panel.popover = 'auto';
    panel.setAttribute('role', 'dialog');
    panel.setAttribute('aria-label', label ?? 'Choose a date');
    this.#calendar = document.createElement('fd-calendar');
    if (this.getAttribute('min')) this.#calendar.setAttribute('min', this.getAttribute('min'));
    if (this.getAttribute('max')) this.#calendar.setAttribute('max', this.getAttribute('max'));
    panel.append(this.#calendar);
    this.#panel = this.appendChild(panel);

    trigger.addEventListener('click', () => (panel.matches(':popover-open') ? panel.hidePopover() : this.show()));
    panel.addEventListener('toggle', (event) => {
      if (event.newState === 'closed') {
        this.#unfollow?.();
        trigger.setAttribute('aria-expanded', 'false');
      }
    });
    panel.addEventListener('keydown', (event) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        panel.hidePopover();
        trigger.focus();
      }
    });
    this.#calendar.addEventListener('change', (event) => {
      event.stopPropagation();
      this.value = this.#calendar.value;
      panel.hidePopover();
      trigger.focus();
      this.dispatchEvent(new Event('change', { bubbles: true }));
    });
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

  show() {
    this.#panel.showPopover();
    this.#trigger.setAttribute('aria-expanded', 'true');
    this.#unfollow = follow(this.#panel, this.#trigger);
    this.#calendar.focusDay();
  }

  #sync() {
    const date = parseIso(this.value);
    if (date) this.#calendar.value = this.value;
    const shown = date
      ? new Intl.DateTimeFormat(document.documentElement.lang || undefined, { dateStyle: 'medium' }).format(new Date(date.year, date.month - 1, date.day, 12))
      : (this.getAttribute('placeholder') ?? 'Choose a date');
    this.#trigger.replaceChildren(Object.assign(document.createElement('span'), { textContent: shown }));
    this.#trigger.toggleAttribute('data-placeholder', !date);
    this.#internals.setFormValue(date ? this.value : null);
  }
}

customElements.define('fd-date-picker', DatePicker);
