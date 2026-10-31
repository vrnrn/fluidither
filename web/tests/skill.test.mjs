// The skill is only useful if what it names exists, so names can't drift from the code.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const root = new URL('../../', import.meta.url).pathname;
const read = (path) => readFileSync(join(root, path), 'utf8');
const skill = ['SKILL.md', 'references/web.md', 'references/gpui.md', 'references/recipes.md', 'references/extending.md'].map((file) => `skills/fluidither/${file}`).map(read).join('\n');

const webFiles = readdirSync(join(root, 'web/src/components')).map((name) => read(`web/src/components/${name}`));
const web = [...webFiles, read('web/src/core/list.js')].join('\n');

test('the skill has valid front matter', () => {
  const head = read('skills/fluidither/SKILL.md').match(/^---\n([\s\S]*?)\n---/)[1];
  assert.match(head, /^name: fluidither$/m);
  assert.match(head, /^description: .{80,1024}$/m);
});

test('every custom element it names is defined', () => {
  for (const name of new Set(skill.match(/<fd-[a-z-]+/g).map((tag) => tag.slice(1)))) {
    const generic = name === 'fd-tab' || name === 'fd-tab-list' || name === 'fd-tab-panel';
    assert.ok(web.includes(`'${name}'`) || generic, `${name} is not defined`);
  }
});

test('every class it names has styles', () => {
  for (const name of new Set(skill.match(/\.fd-[a-z-]+/g).map((each) => each.slice(1)))) {
    assert.ok(web.includes(`.${name}`) || web.includes(`'${name}'`), `.${name} is not styled`);
  }
});

test('every file it names exists', () => {
  const files = ['components', 'core'].flatMap((folder) => readdirSync(join(root, 'web/src', folder)));
  const table = read('skills/fluidither/references/web.md');
  for (const name of new Set(table.match(/\b[a-z-]+\.(?:css|js)\b/g))) {
    assert.ok(files.includes(name) || name === 'tokens.css', `${name} is missing`);
  }
});

test('every token it names exists', () => {
  const tokens = read('web/src/tokens.css');
  for (const name of new Set(skill.match(/--fd-[a-z-]+/g))) {
    if (name === '--fd-dissolve-paper') continue;
    assert.ok(tokens.includes(name), `${name} is not a token`);
  }
});

test('every Rust type it names is exported', () => {
  const crate = read('gpui/src/lib.rs');
  const table = read('skills/fluidither/references/gpui.md');
  for (const [, name] of table.matchAll(/`([A-Z][A-Za-z]+)::new\(/g)) {
    if (name === 'Row' || name === 'Date') continue;
    assert.ok(new RegExp(`\\b${name}\\b`).test(crate), `${name} is not exported`);
  }
});
