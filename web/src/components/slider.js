// <fd-slider> keeps the filled part of the track in step with its range input.
// Everything else, including the keyboard, is the browser's own slider.

export class Slider extends HTMLElement {
  #input = null;

  connectedCallback() {
    this.#input = this.querySelector('input[type=range]');
    this.addEventListener('input', this.refresh);
    this.refresh();
  }

  disconnectedCallback() {
    this.removeEventListener('input', this.refresh);
  }

  /** Call after setting the input's value from code. */
  refresh = () => {
    const input = this.#input;
    if (!input) return;
    const low = Number(input.min || 0);
    const high = Number(input.max || 100);
    const share = high > low ? ((Number(input.value) - low) / (high - low)) * 100 : 0;
    input.style.setProperty('--_fill', `${share}%`);
  };
}

customElements.define('fd-slider', Slider);
