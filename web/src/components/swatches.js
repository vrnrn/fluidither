// <fd-swatches colors="#c33 #39c" value="#39c" aria-label="Accent"> is a set of real
// radio inputs, so keyboard and forms work as usual. Add name="..." to join a form.

let lastGroup = 0;

export class Swatches extends HTMLElement {
  #built = false;

  connectedCallback() {
    if (this.#built) return;
    this.#built = true;
    this.setAttribute('role', 'radiogroup');
    const group = this.getAttribute('name') ?? `fd-swatches-${(lastGroup += 1)}`;
    const value = this.getAttribute('value');
    for (const color of (this.getAttribute('colors') ?? '').trim().split(/\s+/).filter(Boolean)) {
      const swatch = document.createElement('input');
      swatch.type = 'radio';
      swatch.name = group;
      swatch.value = color;
      swatch.checked = color === value;
      swatch.setAttribute('aria-label', color);
      swatch.style.setProperty('--_color', color);
      this.append(swatch);
    }
    this.addEventListener('change', (event) => this.setAttribute('value', event.target.value));
  }

  get value() {
    return this.getAttribute('value') ?? '';
  }
}

customElements.define('fd-swatches', Swatches);
