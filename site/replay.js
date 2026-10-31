// A small button that runs a one-shot animation again.

export function replayButton(label, run) {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'fd-button replay';
  button.dataset.look = 'ghost';
  button.dataset.density = 'compact';
  button.toggleAttribute('data-icon-only', true);
  button.setAttribute('aria-label', label);
  button.title = 'Replay';
  button.innerHTML = '<svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><path d="M5 1h6v2h2v2h2v6h-2v2h-2v2H5v-2H3v-2h2v2h6v-2h2V5h-2V3H5v2H3v3H1V5h2V3h2z" /><path d="M0 7h6L3 11z" /></svg>';
  button.addEventListener('click', run);
  return button;
}
