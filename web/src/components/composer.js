// <fd-composer suggestion="..."> wraps a .fd-composer form with a textarea. Tab takes the
// suggestion into an empty box, Enter sends and Shift and Enter makes a new line. A `send`
// event carries `detail: { text }`.

export class Composer extends HTMLElement {
  #box = null;
  #chip = null;

  connectedCallback() {
    if (this.#box) return;
    this.#box = this.querySelector('textarea');
    const suggestion = this.getAttribute('suggestion');
    if (suggestion) {
      this.#box.placeholder = suggestion;
      this.#chip = document.createElement('span');
      this.#chip.className = 'fd-composer-tab';
      this.#chip.textContent = 'Tab';
      this.#chip.setAttribute('aria-hidden', 'true');
      this.#box.after(this.#chip);
    }
    this.#box.addEventListener('input', () => this.#sync());
    this.#box.addEventListener('keydown', (event) => this.#onKey(event));
    this.querySelector('form').addEventListener('submit', (event) => {
      event.preventDefault();
      this.#send();
    });
  }

  #sync() {
    if (this.#chip) this.#chip.hidden = this.#box.value !== '';
  }

  #onKey(event) {
    const suggestion = this.getAttribute('suggestion');
    if (event.key === 'Tab' && !event.shiftKey && suggestion && !this.#box.value) {
      event.preventDefault();
      this.#box.value = suggestion;
      this.#sync();
    } else if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      this.#send();
    }
  }

  #send() {
    const text = this.#box.value.trim();
    if (!text) return;
    this.#box.value = '';
    this.#sync();
    this.dispatchEvent(new CustomEvent('send', { bubbles: true, detail: { text } }));
  }
}

customElements.define('fd-composer', Composer);
