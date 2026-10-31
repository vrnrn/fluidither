// <fd-calendar value="2026-10-30" min max> is a month grid. Arrow keys move by day and
// week, Page keys by month, Home and End to the edges of the week, and Enter or Space
// choose. It fires `change`, with the date as `value` in YYYY-MM-DD form.

import { addMonths, fromDays, iso, monthGrid, parseIso, toDays, weekday } from '../core/dates.js';

const today = () => {
  const now = new Date();
  return { year: now.getFullYear(), month: now.getMonth() + 1, day: now.getDate() };
};

export class Calendar extends HTMLElement {
  static observedAttributes = ['value', 'min', 'max'];

  #title = null;
  #grid = null;
  #focusDate = null;

  connectedCallback() {
    if (this.#grid) return;
    this.#focusDate = parseIso(this.getAttribute('value')) ?? today();
    const head = this.appendChild(document.createElement('div'));
    head.className = 'fd-calendar-head';
    const button = (label, step) => {
      const each = document.createElement('button');
      each.type = 'button';
      each.className = 'fd-button';
      each.dataset.look = 'ghost';
      each.dataset.density = 'compact';
      each.setAttribute('aria-label', label);
      each.textContent = step < 0 ? '<' : '>';
      each.addEventListener('click', () => this.#moveTo(addMonths(this.#focusDate, step), false));
      return each;
    };
    this.#title = document.createElement('span');
    this.#title.className = 'fd-calendar-title';
    this.#title.setAttribute('aria-live', 'polite');
    head.append(button('Previous month', -1), this.#title, button('Next month', 1));
    this.#grid = this.appendChild(document.createElement('table'));
    this.#grid.className = 'fd-calendar-grid';
    this.#grid.setAttribute('role', 'grid');
    this.#grid.addEventListener('keydown', (event) => this.#onKey(event));
    this.#grid.addEventListener('click', (event) => {
      const cell = event.target.closest('button');
      if (cell && !cell.disabled) this.#choose(parseIso(cell.dataset.date));
    });
    this.#draw();
  }

  attributeChangedCallback(name) {
    if (!this.#grid) return;
    if (name === 'value') this.#focusDate = parseIso(this.getAttribute('value')) ?? this.#focusDate;
    this.#draw();
  }

  get value() {
    return this.getAttribute('value') ?? '';
  }

  set value(value) {
    this.setAttribute('value', value);
  }

  /** Put keyboard focus on the current day. */
  focusDay() {
    this.#grid.querySelector('button[tabindex="0"]')?.focus();
  }

  #allowed(date) {
    const text = iso(date);
    const min = this.getAttribute('min');
    const max = this.getAttribute('max');
    return !(min && text < min) && !(max && text > max);
  }

  #draw() {
    const { year, month } = this.#focusDate;
    const locale = document.documentElement.lang || undefined;
    const noon = (date) => new Date(date.year, date.month - 1, date.day, 12);
    this.#title.textContent = new Intl.DateTimeFormat(locale, { month: 'long', year: 'numeric' }).format(noon(this.#focusDate));
    const days = monthGrid(year, month);
    const dayName = new Intl.DateTimeFormat(locale, { weekday: 'short' });
    const longName = new Intl.DateTimeFormat(locale, { dateStyle: 'full' });
    const head = document.createElement('tr');
    head.append(...days.slice(0, 7).map((date) => Object.assign(document.createElement('th'), { scope: 'col', textContent: dayName.format(noon(date)) })));
    const chosen = this.getAttribute('value');
    const focused = iso(this.#focusDate);
    const now = iso(today());
    const rows = [head];
    for (let week = 0; week < 6; week += 1) {
      const row = document.createElement('tr');
      for (const date of days.slice(week * 7, week * 7 + 7)) {
        const text = iso(date);
        const cell = document.createElement('td');
        cell.setAttribute('role', 'gridcell');
        cell.setAttribute('aria-selected', String(text === chosen));
        const day = document.createElement('button');
        day.type = 'button';
        day.textContent = date.day;
        day.dataset.date = text;
        day.tabIndex = text === focused ? 0 : -1;
        day.disabled = !this.#allowed(date);
        day.setAttribute('aria-label', longName.format(noon(date)));
        if (date.month !== month) day.dataset.outside = '';
        if (text === now) day.setAttribute('aria-current', 'date');
        cell.append(day);
        row.append(cell);
      }
      rows.push(row);
    }
    const hadFocus = this.#grid.contains(document.activeElement);
    this.#grid.replaceChildren(...rows);
    if (hadFocus) this.focusDay();
  }

  #moveTo(date) {
    if (!this.#allowed(date)) return;
    this.#focusDate = date;
    this.#draw();
  }

  #choose(date) {
    this.#focusDate = date;
    this.value = iso(date);
    this.dispatchEvent(new Event('change', { bubbles: true }));
  }

  #onKey(event) {
    const here = this.#focusDate;
    const shift = (days) => fromDays(toDays(here) + days);
    const moves = {
      ArrowLeft: () => shift(-1),
      ArrowRight: () => shift(1),
      ArrowUp: () => shift(-7),
      ArrowDown: () => shift(7),
      Home: () => shift(-weekday(here)),
      End: () => shift(6 - weekday(here)),
      PageUp: () => addMonths(here, -1),
      PageDown: () => addMonths(here, 1),
    };
    if (moves[event.key]) {
      event.preventDefault();
      this.#moveTo(moves[event.key]());
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      this.#choose(here);
    }
  }
}

customElements.define('fd-calendar', Calendar);
