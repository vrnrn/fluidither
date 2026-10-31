// <fd-sidebar label="Menu"> holds an <fd-nav> and collapses to a thin strip. The first
// child button it adds toggles it, and a `collapsed` attribute holds the state.

export class Sidebar extends HTMLElement {
  #toggle = null;

  connectedCallback() {
    if (this.#toggle) return;
    const toggle = document.createElement('button');
    toggle.type = 'button';
    toggle.className = 'fd-sidebar-toggle';
    toggle.addEventListener('click', () => this.toggleAttribute('collapsed'));
    this.prepend(toggle);
    this.#toggle = toggle;
    this.#sync();
    new MutationObserver(() => this.#sync()).observe(this, { attributeFilter: ['collapsed'] });
  }

  #sync() {
    const collapsed = this.hasAttribute('collapsed');
    const label = this.getAttribute('label') ?? 'sidebar';
    this.#toggle.setAttribute('aria-expanded', String(!collapsed));
    this.#toggle.setAttribute('aria-label', `${collapsed ? 'Show' : 'Hide'} ${label}`);
    this.#toggle.textContent = collapsed ? '>' : '<';
    for (const child of this.children) {
      if (child !== this.#toggle) child.inert = collapsed;
    }
  }
}

customElements.define('fd-sidebar', Sidebar);
