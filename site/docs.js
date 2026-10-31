// Turns each demo template into a live example with its source beside it, so the two can't
// drift, and adds search, the current section in the sidebar, and the theme switch.

import './fluidither/components/command.js';
import { replayButton } from './replay.js';
import { wipe } from './wipe.js';

const page = document.documentElement;
const nav = document.getElementById('nav');
const side = document.querySelector('.side');
const menu = document.getElementById('menu');
const dark = document.getElementById('dark');

const escaped = (text) => text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

function dedent(text) {
  const lines = text.replace(/^\s*\n/, '').trimEnd().split('\n');
  const indents = lines.filter((line) => line.trim()).map((line) => line.match(/^ */)[0].length);
  const least = Math.min(...indents);
  return lines.map((line) => line.slice(least)).join('\n');
}

// What a reader would write: no empty boolean attributes, and none of this page's wiring or
// its own classes for sizing an example.
const sourceOf = (template) =>
  dedent(template.innerHTML)
    .replace(/ data-opens="[^"]*"/g, '')
    .replace(/ class="demo-[a-z]+"/g, '')
    .replace(/ demo-[a-z]+(?=")/g, '')
    .replace(/ ([a-z-]+)=""/g, ' $1');

function copyButton(text) {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'fd-button copy-code';
  button.dataset.look = 'ghost';
  button.dataset.density = 'compact';
  button.textContent = 'Copy';
  button.addEventListener('click', async () => {
    await navigator.clipboard.writeText(text);
    button.textContent = 'Copied';
    setTimeout(() => (button.textContent = 'Copy'), 1500);
  });
  return button;
}

function finishCode(pre, label) {
  pre.tabIndex = 0;
  pre.setAttribute('role', 'region');
  pre.setAttribute('aria-label', label);
  pre.classList.add('fd-scroll');
  const holder = document.createElement('div');
  holder.className = 'code-holder';
  pre.replaceWith(holder);
  holder.append(pre, copyButton(pre.textContent));
  return holder;
}

function codePanel(text, label) {
  const pre = document.createElement('pre');
  pre.className = 'code';
  pre.innerHTML = `<code>${escaped(text)}</code>`;
  const place = document.createElement('div');
  place.append(pre);
  finishCode(pre, label);
  return place;
}

function buildPlate(part, index) {
  const name = part.querySelector('h3').textContent;
  const demo = part.querySelector('template[data-demo]');
  const rust = part.querySelector('template[data-rust]');
  const plate = document.createElement('div');
  plate.className = 'plate';

  const view = document.createElement('div');
  view.className = 'plate-view';
  view.dataset.layout = demo.dataset.layout ?? 'row';
  // Imported rather than cloned, so a list is ready before the select around it connects.
  view.append(document.importNode(demo.content, true));

  const tabs = document.createElement('fd-tabs');
  tabs.setAttribute('value', 'web');
  tabs.setAttribute('look', 'underline');
  tabs.innerHTML = `
    <fd-tab-list aria-label="${name} code">
      <fd-tab value="web">Web</fd-tab>
      ${rust ? '<fd-tab value="gpui">GPUI</fd-tab>' : ''}
    </fd-tab-list>`;
  const addPanel = (value, text, label) => {
    const panel = document.createElement('fd-tab-panel');
    panel.setAttribute('value', value);
    panel.hidden = value !== 'web';
    panel.append(codePanel(text, label));
    tabs.append(panel);
  };
  addPanel('web', sourceOf(demo), `${name} for the web`);
  if (rust) addPanel('gpui', dedent(rust.content.textContent), `${name} for GPUI`);

  const add = document.createElement('p');
  add.className = 'plate-add';
  add.innerHTML = `Just this one: <code>npx fluidither add ${part.dataset.add}</code>`;

  plate.append(view, tabs, add);
  if ('replay' in demo.dataset) {
    const again = () => view.replaceChildren(document.importNode(demo.content, true));
    plate.append(replayButton(`Replay ${name}`, again));
  }
  demo.replaceWith(plate);
  rust?.remove();
  plate.dataset.index = index;
}

document.querySelectorAll('.part[data-add]').forEach(buildPlate);
document.querySelectorAll('pre.code:not(.fd-scroll)').forEach((pre) => finishCode(pre, pre.dataset.label ?? 'Code'));

document.addEventListener('click', (event) => {
  const opener = event.target.closest('[data-opens]');
  if (!opener) return;
  const target = document.getElementById(opener.dataset.opens);
  if (target.showModal) target.showModal();
  else target.show();
});

// Search over everything the sidebar lists.
const search = document.createElement('fd-command');
search.setAttribute('aria-label', 'Search the docs');
search.setAttribute('placeholder', 'Search the docs');
const results = document.createElement('fd-list');
for (const link of nav.querySelectorAll('a')) {
  const item = document.createElement('fd-item');
  item.setAttribute('value', link.hash.slice(1));
  item.textContent = link.textContent;
  results.append(item);
}
search.append(results);
document.body.append(search);

search.addEventListener('pick', ({ detail }) => {
  location.hash = detail.value;
});

document.getElementById('search').addEventListener('click', () => search.show());

document.addEventListener('keydown', (event) => {
  const typing = event.target.closest('input, textarea, [contenteditable]');
  const asked = (event.key === 'k' && (event.metaKey || event.ctrlKey)) || (event.key === '/' && !typing);
  if (!asked || search.open) return;
  event.preventDefault();
  search.show();
});

// The sidebar follows the reading position.
const links = new Map([...nav.querySelectorAll('a')].map((link) => [link.hash.slice(1), link]));

const reading = new IntersectionObserver(
  (entries) => {
    const entry = entries.filter(({ isIntersecting }) => isIntersecting).at(-1);
    const link = entry && links.get(entry.target.id);
    if (!link || link.hasAttribute('aria-current')) return;
    nav.select(link);
    if (nav.parentElement === side) link.scrollIntoView({ block: 'nearest' });
  },
  { rootMargin: '-8% 0px -84% 0px' },
);

for (const id of links.keys()) reading.observe(document.getElementById(id));

// On a narrow window the sidebar's links live in a drawer.
document.getElementById('menu-open').addEventListener('click', () => {
  menu.append(nav);
  menu.showModal();
});

menu.addEventListener('close', () => side.append(nav));
nav.addEventListener('click', (event) => {
  if (event.target.closest('a') && menu.open) menu.close();
});
menu.addEventListener('click', (event) => {
  if (event.target === menu) menu.close();
});

// A held dissolve keeps the paper it first saw, so it is drawn again after a theme change.
function redrawHeld() {
  for (const held of document.querySelectorAll('fd-dissolve[hold]')) {
    held.querySelector('.fd-dissolve-veil')?.remove();
    held.replaceWith(held.cloneNode(true));
  }
}

const center = (element) => {
  const { left, top, width, height } = element.getBoundingClientRect();
  return { x: left + width / 2, y: top + height / 2 };
};

dark.checked = page.dataset.theme === 'dark';
dark.addEventListener('change', async () => {
  const next = page.dataset.theme === 'dark' ? 'light' : 'dark';
  dark.checked = next === 'light';
  // The old ink is close to the new paper, so the page turns over through its own ink.
  await wipe(center(dark), getComputedStyle(page).getPropertyValue('--fd-ink'), () => {
    page.dataset.theme = next;
    dark.checked = next === 'dark';
    window.saveTheme(next);
    redrawHeld();
  });
});

page.dataset.ready = '';
