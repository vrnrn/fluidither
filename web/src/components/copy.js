// <fd-copy value="npm i fluidither" label="Install command">: a read-only field with a
// copy button. The button reports success in words, which a live region announces.

const RESET_MS = 1500;

export class Copy extends HTMLElement {
  #timer = 0;

  connectedCallback() {
    if (this.firstElementChild) return;
    const field = document.createElement('input');
    field.className = 'fd-input';
    field.readOnly = true;
    field.value = this.getAttribute('value') ?? '';
    field.setAttribute('aria-label', this.getAttribute('label') ?? 'Text to copy');
    field.addEventListener('focus', () => field.select());
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'fd-button';
    button.dataset.look = 'soft';
    button.textContent = 'Copy';
    button.setAttribute('aria-live', 'polite');
    button.addEventListener('click', async () => {
      await navigator.clipboard.writeText(field.value);
      button.textContent = 'Copied';
      clearTimeout(this.#timer);
      this.#timer = setTimeout(() => (button.textContent = 'Copy'), RESET_MS);
      this.dispatchEvent(new Event('copy', { bubbles: true }));
    });
    this.append(field, button);
  }
}

customElements.define('fd-copy', Copy);
