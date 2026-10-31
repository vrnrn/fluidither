// <fd-marquee aria-label="Our customers"> scrolls its children sideways for ever. It
// repeats them once for a seamless loop, hidden from assistive tech, and stops when
// hovered or focused, and under reduced motion.

export class Marquee extends HTMLElement {
  connectedCallback() {
    if (this.querySelector(':scope > .fd-marquee-track')) return;
    const track = document.createElement('div');
    track.className = 'fd-marquee-track';
    track.append(...this.childNodes);
    const copy = track.cloneNode(true);
    copy.setAttribute('aria-hidden', 'true');
    this.append(track, copy);
  }
}

customElements.define('fd-marquee', Marquee);
