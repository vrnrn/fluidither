// <fd-tabs> holds <fd-tab-list> with <fd-tab> items, and <fd-tab-panel>s matched by
// `value`. The roles, the keyboard (arrows, Home, End, with selection following
// focus) and the panel links follow the WAI-ARIA tabs pattern.

import { Glide, springs } from '../core/spring.js';

let lastId = 0;
const nextId = (kind) => `fd-${kind}-${(lastId += 1)}`;

const canUse = (tab) => !tab.hasAttribute('disabled');

// Tabs take plain text. Text moves into a span so the width of its stronger
// weight can be reserved, and any icon stays beside it.
function wrapLabel(tab) {
  let label = tab.querySelector(':scope > .fd-tab-text');
  const loose = [...tab.childNodes].filter((node) => node.nodeType === Node.TEXT_NODE && node.data.trim());
  if (loose.length > 0) {
    label ??= document.createElement('span');
    label.className = 'fd-tab-text';
    label.textContent = loose.map((node) => node.data.trim()).join(' ');
    loose.forEach((node) => node.remove());
    tab.append(label);
  }
  if (label) label.dataset.text = label.textContent;
}

export class Tabs extends HTMLElement {
  static observedAttributes = ['value'];

  #list = null;
  #watcher = null;
  #sizes = null;
  #thumb = null;
  #peek = null;
  #peeking = null;
  #placed = false;

  get #tabs() {
    return this.#list ? [...this.#list.querySelectorAll('fd-tab')] : [];
  }

  get #current() {
    const tabs = this.#tabs;
    return tabs.find((tab) => tab.getAttribute('value') === this.getAttribute('value')) ?? tabs.find(canUse) ?? null;
  }

  get value() {
    return this.#current?.getAttribute('value') ?? '';
  }

  set value(value) {
    this.setAttribute('value', value);
  }

  connectedCallback() {
    this.#sizes = new ResizeObserver(() => this.#placeThumb(true));
    this.#watcher = new MutationObserver(() => this.#update());
    this.#watcher.observe(this, {
      attributeFilter: ['value', 'disabled'],
      characterData: true,
      childList: true,
      subtree: true,
    });
    this.#update();
  }

  disconnectedCallback() {
    this.#watcher.disconnect();
    this.#sizes.disconnect();
    this.#sizes = null;
    this.#thumb?.stop();
    this.#peek?.stop();
    this.#placed = false;
  }

  attributeChangedCallback() {
    if (this.#sizes) this.#update();
  }

  #update() {
    const list = this.querySelector(':scope > fd-tab-list');
    if (!list) return;
    if (list !== this.#list) this.#adopt(list);
    const current = this.#current;
    const panels = [...this.querySelectorAll(':scope > fd-tab-panel')];
    for (const tab of this.#tabs) {
      wrapLabel(tab);
      this.#sizes.observe(tab);
      tab.id ||= nextId('tab');
      tab.setAttribute('role', 'tab');
      tab.setAttribute('aria-selected', String(tab === current));
      tab.tabIndex = tab === current ? 0 : -1;
      if (canUse(tab)) tab.removeAttribute('aria-disabled');
      else tab.setAttribute('aria-disabled', 'true');
      const panel = panels.find((each) => each.getAttribute('value') === tab.getAttribute('value'));
      if (panel) {
        panel.id ||= nextId('panel');
        panel.setAttribute('role', 'tabpanel');
        panel.setAttribute('aria-labelledby', tab.id);
        panel.tabIndex = 0;
        panel.hidden = tab !== current;
        tab.setAttribute('aria-controls', panel.id);
      }
    }
    this.#placeThumb(!this.#placed);
  }

  #adopt(list) {
    this.#list = list;
    list.setAttribute('role', 'tablist');
    this.#sizes.observe(list);
    const setVars = (prefix) => (values) => {
      for (const [name, value] of Object.entries(values)) {
        list.style.setProperty(`--_${prefix}-${name}`, name === 'opacity' ? value : `${value}px`);
      }
    };
    this.#thumb = new Glide({ x: 0, width: 0 }, springs.glide, setVars('thumb'));
    this.#peek = new Glide({ x: 0, width: 0, opacity: 0 }, springs.quick, setVars('peek'));
    list.addEventListener('click', (event) => this.#choose(event.target.closest('fd-tab')));
    list.addEventListener('keydown', (event) => this.#onKey(event));
    list.addEventListener('pointerover', (event) => {
      if (event.pointerType !== 'touch') this.#showPeek(event.target.closest('fd-tab'));
    });
    list.addEventListener('pointerleave', () => this.#showPeek(null));
  }

  #placeThumb(instantly) {
    const tab = this.#current;
    if (!tab || tab.offsetWidth === 0) return;
    const place = { x: tab.offsetLeft, width: tab.offsetWidth };
    if (instantly) this.#thumb.jump(place);
    else this.#thumb.to(place);
    this.#placed = true;
    if (this.#peeking === tab) this.#showPeek(null);
  }

  // The peek fades in where the pointer lands, glides between tabs after that,
  // and fades out where it is.
  #showPeek(tab) {
    const shown = tab && canUse(tab) && tab !== this.#current ? tab : null;
    if (shown === this.#peeking) return;
    const wasHidden = this.#peeking === null;
    this.#peeking = shown;
    if (!shown) {
      this.#peek.to({ opacity: 0 }, springs.quickOut);
      return;
    }
    const place = { x: shown.offsetLeft, width: shown.offsetWidth };
    if (wasHidden) this.#peek.jump(place);
    this.#peek.to({ ...place, opacity: 1 });
  }

  #choose(tab) {
    if (!tab || !canUse(tab)) return;
    tab.focus();
    if (tab === this.#current) return;
    const value = tab.getAttribute('value');
    this.setAttribute('value', value);
    this.dispatchEvent(new CustomEvent('change', { bubbles: true, detail: { value, index: this.#tabs.indexOf(tab) } }));
  }

  #onKey(event) {
    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const usable = this.#tabs.filter(canUse);
    const here = usable.indexOf(event.target.closest('fd-tab'));
    if (here < 0) return;
    const flipped = getComputedStyle(this.#list).direction === 'rtl';
    const step = { ArrowRight: flipped ? -1 : 1, ArrowLeft: flipped ? 1 : -1 }[event.key];
    let target = null;
    if (step) target = usable[(here + step + usable.length) % usable.length];
    else if (event.key === 'Home') target = usable[0];
    else if (event.key === 'End') target = usable.at(-1);
    else if (event.key === 'Enter' || event.key === ' ') target = usable[here];
    if (target) {
      event.preventDefault();
      this.#choose(target);
    }
  }
}

for (const name of ['fd-tab-list', 'fd-tab', 'fd-tab-panel']) {
  customElements.define(name, class extends HTMLElement {});
}
customElements.define('fd-tabs', Tabs);
