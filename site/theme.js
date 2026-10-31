// A plain script in the head, so a saved theme is set before the first paint.
(() => {
  const KEY = 'fluidither-theme';
  window.saveTheme = (theme) => {
    try {
      localStorage.setItem(KEY, theme);
    } catch {
      // Private windows can refuse storage. The theme then lasts as long as the page.
    }
  };
  try {
    const saved = localStorage.getItem(KEY);
    if (saved === 'light' || saved === 'dark') document.documentElement.dataset.theme = saved;
  } catch {
    // The default in the markup stands.
  }
})();
