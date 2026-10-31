// Loads only the parts on the board, as `npx fluidither add` would copy them. The names and
// files match the table in web/bin/fluidither.mjs.

const PARTS = {
  accordion: ['accordion.css'],
  badge: ['badge.css'],
  banner: ['banner.css', 'button.css', 'banner.js'],
  bloom: ['extras.css'],
  button: ['button.css'],
  calendar: ['calendar.css', 'calendar.js'],
  card: ['card.css'],
  chart: ['chart.css', 'field.css', 'chart.js'],
  chat: ['misc.css', 'input.css', 'button.css', 'feedback.css'],
  check: ['check.css'],
  'check-group': ['check.css'],
  color: ['misc.css'],
  'color-picker': ['misc.css', 'input.css', 'color-picker.js'],
  combobox: ['list.css', 'combobox.css', 'combobox.js'],
  command: ['list.css', 'dialog.css', 'command.css', 'command.js'],
  compare: ['extras.css', 'compare.js'],
  composer: ['misc.css', 'input.css', 'button.css', 'composer.js'],
  'date-picker': ['calendar.css', 'select.css', 'list.css', 'date-picker.js'],
  dialog: ['dialog.css'],
  drawer: ['dialog.css', 'misc.css'],
  feedback: ['feedback.css'],
  field: ['field.css', 'field.js'],
  file: ['extras.css'],
  gradient: ['dither-gradient.css', 'dither-gradient.js'],
  hover: ['hover.css', 'hover.js'],
  image: ['field.css', 'dither-image.js'],
  input: ['input.css'],
  'input-group': ['input.css'],
  kbd: ['misc.css'],
  marquee: ['extras.css', 'marquee.js'],
  menu: ['list.css', 'menu.js'],
  misc: ['misc.css'],
  nav: ['nav.css', 'nav.js'],
  'not-found': ['extras.css'],
  question: ['extras.css', 'check.css', 'button.css'],
  questions: ['extras.css', 'check.css', 'button.css', 'questions.js'],
  scroll: ['scroll.css'],
  select: ['list.css', 'select.css', 'select.js'],
  sidebar: ['extras.css', 'sidebar.js'],
  slider: ['slider.css', 'slider.js'],
  sparkline: ['sparkline.css', 'sparkline.js'],
  steps: ['extras.css'],
  swatches: ['misc.css', 'swatches.js'],
  switch: ['switch.css', 'check.css'],
  table: ['table.css'],
  tabs: ['tabs.css', 'tabs.js'],
  text: ['text-effects.css', 'text-effects.js'],
  'thinking-steps': ['extras.css', 'badge.css'],
  tooltip: ['tooltip.css', 'tooltip.js'],
  world: ['field.css', 'extras.css', 'world.js'],
};

const loaded = new Map();

function loadStyle(file) {
  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = `fluidither/components/${file}`;
  const done = new Promise((resolve, reject) => {
    link.addEventListener('load', resolve);
    link.addEventListener('error', reject);
  });
  // Page styles come last in the head and must keep winning over component styles.
  document.head.querySelector('link[href="site.css"]').before(link);
  return done;
}

function loadFile(file) {
  if (!loaded.has(file)) {
    loaded.set(file, file.endsWith('.css') ? loadStyle(file) : import(`./fluidither/components/${file}`));
  }
  return loaded.get(file);
}

export const loadParts = (names) => Promise.all(names.flatMap((name) => PARTS[name]).map(loadFile));
