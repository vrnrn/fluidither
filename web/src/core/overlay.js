// Keeps a floating element beside its anchor, on screen, and following it through
// scrolling and resizing. Positions are set through the CSSOM, which a strict CSP
// allows.

const EDGE = 8;

export function place(floating, anchor, { side = 'bottom', align = 'start', gap = 6 } = {}) {
  const near = anchor.getBoundingClientRect();
  const { width, height } = floating.getBoundingClientRect();
  const viewWidth = document.documentElement.clientWidth;
  const viewHeight = window.innerHeight;

  const above = near.top - height - gap;
  const below = near.bottom + gap;
  const fitsAbove = above >= EDGE;
  const fitsBelow = below + height <= viewHeight - EDGE;
  const top = side === 'top' ? (fitsAbove || !fitsBelow ? above : below) : fitsBelow || !fitsAbove ? below : above;

  const flipped = getComputedStyle(anchor).direction === 'rtl';
  const start = flipped ? near.right - width : near.left;
  const end = flipped ? near.left : near.right - width;
  const left = align === 'center' ? near.left + (near.width - width) / 2 : align === 'end' ? end : start;

  floating.style.top = `${Math.round(Math.max(EDGE, top))}px`;
  floating.style.left = `${Math.round(Math.min(Math.max(EDGE, left), Math.max(EDGE, viewWidth - width - EDGE)))}px`;
  floating.dataset.side = top < near.top ? 'top' : 'bottom';
}

/** Place now and keep it placed. Returns a function that stops following. */
export function follow(floating, anchor, options) {
  const update = () => place(floating, anchor, options);
  update();
  window.addEventListener('scroll', update, { capture: true, passive: true });
  window.addEventListener('resize', update);
  return () => {
    window.removeEventListener('scroll', update, { capture: true });
    window.removeEventListener('resize', update);
  };
}
