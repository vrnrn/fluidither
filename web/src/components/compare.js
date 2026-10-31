// <fd-compare aria-label="Before and after"> holds two images or dither elements. A real
// range input moves the split, so it works with the keyboard.

export class Compare extends HTMLElement {
  #range = null;

  connectedCallback() {
    if (this.#range) return;
    const range = document.createElement('input');
    range.type = 'range';
    range.value = '50';
    range.setAttribute('aria-label', this.getAttribute('aria-label') ?? 'Compare');
    this.removeAttribute('aria-label');
    range.addEventListener('input', () => this.#set(range.value));
    this.append(range);
    this.#range = range;
    this.#set(range.value);
  }

  #set(share) {
    this.style.setProperty('--_split', `${share}%`);
  }
}

customElements.define('fd-compare', Compare);
