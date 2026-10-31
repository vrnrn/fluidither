#!/usr/bin/env node
// `npx fluidither` copies the whole library into a project. `npx fluidither add tabs button`
// copies those components and the files they import, and prints the tags to paste. It uses
// nothing but Node, like the library it copies.
import { cpSync, existsSync, mkdirSync, readFileSync } from 'node:fs';
import { basename, dirname, join, posix, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const packageRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const source = join(packageRoot, 'src');
const DEFAULT_SKILL_DIR = '.agents/skills/fluidither';

const COMPONENTS = {
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
  copy: ['misc.css', 'input.css', 'button.css', 'copy.js'],
  'date-picker': ['calendar.css', 'select.css', 'date-picker.js'],
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
  reveal: ['reveal.css', 'reveal.js'],
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

const USAGE = `fluidither: dithered, spring-driven UI with no build step

  npx fluidither                  Copy the whole library
  npx fluidither add <names...>   Copy only these components and what they need
  npx fluidither list             Show the component names
  npx fluidither skill            Install the agent skill
  npx fluidither help             Show this

Options:
  --dir <path>   Where to copy to. The default is public/fluidither if the
                 project has a public folder, otherwise fluidither. For skill,
                 it is ${DEFAULT_SKILL_DIR}.
`;

const fail = (message) => {
  console.error(`fluidither: ${message}`);
  process.exit(1);
};

// Follows relative imports, so a component brings the engine files it runs on.
function withImports(start) {
  const found = new Set();
  const visit = (file) => {
    if (found.has(file)) return;
    found.add(file);
    if (!file.endsWith('.js')) return;
    const text = readFileSync(join(source, file), 'utf8');
    for (const [, path] of text.matchAll(/(?:from\s+|import\s*\(?\s*)['"](\.{1,2}\/[^'"]+)['"]/g)) {
      visit(posix.join(posix.dirname(file), path));
    }
  };
  start.forEach(visit);
  return [...found];
}

function copy(files, target) {
  for (const file of files) {
    mkdirSync(dirname(join(target, file)), { recursive: true });
    cpSync(join(source, file), join(target, file));
  }
}

function printTags(start, target) {
  const base = basename(target);
  console.log('\nAdd these to your page:\n');
  for (const file of start.filter((each) => each.endsWith('.css'))) {
    console.log(`<link rel="stylesheet" href="${base}/${file}" />`);
  }
  for (const file of start.filter((each) => each.endsWith('.js'))) {
    console.log(`<script type="module" src="${base}/${file}"></script>`);
  }
}

function parse(argv) {
  const names = [];
  let dir = null;
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === '--dir') {
      index += 1;
      dir = argv[index] ?? fail('--dir needs a path');
    } else {
      names.push(argv[index]);
    }
  }
  return { names, dir };
}

const defaultTarget = () => (existsSync('public') ? 'public/fluidither' : 'fluidither');

function add(names, dir) {
  if (names.length === 0) fail('name at least one component. See `npx fluidither list`.');
  const unknown = names.filter((name) => !COMPONENTS[name]);
  if (unknown.length > 0) fail(`unknown component: ${unknown.join(', ')}. See \`npx fluidither list\`.`);
  const chosen = [...new Set(names.flatMap((name) => COMPONENTS[name]))].map((file) => `components/${file}`);
  const start = ['tokens.css', ...chosen];
  const target = resolve(dir ?? defaultTarget());
  const files = withImports(start);
  copy(files, target);
  console.log(`Copied ${files.length} files to ${relative('.', target) || '.'}`);
  printTags(start, target);
}

function init(dir) {
  const target = resolve(dir ?? defaultTarget());
  cpSync(source, target, { recursive: true });
  console.log(`Copied the library to ${relative('.', target) || '.'}`);
  console.log(`\nLink ${basename(target)}/tokens.css, then the components you use.`);
}

function skill(dir) {
  const packaged = join(packageRoot, 'skill');
  const from = existsSync(packaged) ? packaged : join(packageRoot, '../skills/fluidither');
  const target = resolve(dir ?? DEFAULT_SKILL_DIR);
  cpSync(from, target, { recursive: true });
  console.log(`Installed the skill to ${relative('.', target) || '.'}`);
}

// With no command, or only `--dir`, it copies everything.
const given = process.argv.slice(2);
const [command, ...rest] = given.length === 0 || given[0] === '--dir' ? ['init', ...given] : given;
const { names, dir } = parse(rest);

if (command === 'add') add(names, dir);
else if (command === 'init') init(dir);
else if (command === 'list') console.log(Object.keys(COMPONENTS).join('\n'));
else if (command === 'skill') skill(dir);
else if (command === 'help' || command === '--help' || command === '-h') console.log(USAGE);
else fail(`unknown command: ${command}\n\n${USAGE}`);
