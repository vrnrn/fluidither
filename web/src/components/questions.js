// <fd-questions> shows the .fd-question fieldsets inside one at a time, with radios named per
// question. A click or a number key answers and moves on, arrow keys choose and Next moves on,
// so a keyboard is never rushed past an answer. Checkboxes in place of radios take several
// answers and wait for Next. A text input inside an option makes its text the answer. Skip
// leaves it null. `answer` fires with `detail: { name, value }` each time, and `done` with
// `detail: { answers }` at the end.

export class Questions extends HTMLElement {
  #questions = [];
  #at = 0;
  #answers = {};
  #count = null;
  #back = null;
  #next = null;

  connectedCallback() {
    if (this.#count) return;
    this.#questions = [...this.querySelectorAll(':scope > .fd-question')];
    this.#count = document.createElement('p');
    this.#count.className = 'fd-questions-count';
    this.#count.setAttribute('aria-live', 'polite');
    this.prepend(this.#count);
    const actions = document.createElement('div');
    actions.className = 'fd-questions-actions';
    const button = (text, look, run) => {
      const made = document.createElement('button');
      made.type = 'button';
      made.className = 'fd-button';
      if (look) made.dataset.look = look;
      made.dataset.density = 'compact';
      made.textContent = text;
      made.addEventListener('click', run);
      return made;
    };
    this.#back = button('Back', 'ghost', () => this.#go(this.#at - 1));
    this.#next = button('Next', null, () => this.#record());
    actions.append(this.#back, button('Skip', 'soft', () => this.#record(null)), this.#next);
    this.append(actions);
    this.addEventListener('click', (event) => {
      // A real click has a detail. Space on a focused radio does not, and stays put.
      const own = event.target.closest('.fd-choice')?.querySelector('input:not([type])');
      if (event.detail > 0 && event.target.matches('input[type="radio"]') && !own) this.#record();
      if (own && event.target.matches('input[type="radio"]')) own.focus();
    });
    this.addEventListener('keydown', (event) => this.#onKey(event));
    this.#go(0);
  }

  get #question() {
    return this.#questions[this.#at];
  }

  #go(index) {
    this.#at = Math.max(index, 0);
    this.#questions.forEach((question, each) => {
      question.hidden = each !== this.#at;
    });
    this.#count.textContent = `Question ${this.#at + 1} of ${this.#questions.length}`;
    this.#back.disabled = this.#at === 0;
    this.#next.textContent = this.#at === this.#questions.length - 1 ? 'Finish' : 'Next';
  }

  #record(forced) {
    const name = this.#question.querySelector('input').name;
    const value = forced === undefined ? this.#read() : forced;
    this.#answers[name] = value;
    this.dispatchEvent(new CustomEvent('answer', { bubbles: true, detail: { name, value } }));
    if (this.#at < this.#questions.length - 1) this.#go(this.#at + 1);
    else this.#finish();
  }

  #read() {
    const chosen = [...this.#question.querySelectorAll('input:checked')].map((input) => {
      const own = input.closest('.fd-choice').querySelector('input:not([type])');
      return own ? own.value.trim() : input.value;
    });
    if (this.#question.querySelector('input[type="checkbox"]')) return chosen;
    return chosen[0] ?? null;
  }

  #finish() {
    const closing = document.createElement('p');
    closing.className = 'fd-questions-done';
    closing.setAttribute('role', 'status');
    closing.textContent = this.getAttribute('done') ?? 'Thanks.';
    this.dispatchEvent(new CustomEvent('done', { bubbles: true, detail: { answers: { ...this.#answers } } }));
    this.replaceChildren(closing);
  }

  #onKey(event) {
    if (event.target.matches('input:not([type])')) {
      if (event.key === 'Enter') this.#record();
      return;
    }
    const number = Number(event.key);
    const options = this.#question?.querySelectorAll('input[type="radio"], input[type="checkbox"]') ?? [];
    if (!Number.isInteger(number) || number < 1 || number > options.length) return;
    const option = options[number - 1];
    if (option.type === 'checkbox') option.checked = !option.checked;
    else option.click();
    if (option.type === 'radio' && !option.closest('.fd-choice').querySelector('input:not([type])')) this.#record();
  }
}

customElements.define('fd-questions', Questions);
