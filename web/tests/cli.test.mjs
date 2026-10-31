// The npx command, run the way a project runs it, in throwaway folders.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, posix } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const cli = join(here, '../bin/fluidither.mjs');
const run = (cwd, ...args) => execFileSync(process.execPath, [cli, ...args], { cwd, encoding: 'utf8' });

function inFolder(work) {
  const folder = mkdtempSync(join(tmpdir(), 'fluidither-'));
  try {
    work(folder);
  } finally {
    rmSync(folder, { recursive: true });
  }
}

const allFiles = (folder, prefix = '') =>
  readdirSync(join(folder, prefix), { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? allFiles(folder, posix.join(prefix, entry.name)) : [posix.join(prefix, entry.name)],
  );

const brokenImports = (root) =>
  allFiles(root)
    .filter((file) => file.endsWith('.js'))
    .flatMap((file) =>
      [...readFileSync(join(root, file), 'utf8').matchAll(/(?:from\s+|import\s*\(?\s*)['"](\.{1,2}\/[^'"]+)['"]/g)]
        .map(([, path]) => posix.join(posix.dirname(file), path))
        .filter((path) => !existsSync(join(root, path))),
    );

test('add copies a component with tokens and its imports, and prints the tags', () => {
  inFolder((folder) => {
    const output = run(folder, 'add', 'tabs', 'button');
    const root = join(folder, 'fluidither');
    assert.ok(existsSync(join(root, 'tokens.css')));
    assert.ok(existsSync(join(root, 'components/tabs.js')));
    assert.ok(!existsSync(join(root, 'components/chart.js')), 'only what was asked for');
    assert.deepEqual(brokenImports(root), []);
    assert.match(output, /<link rel="stylesheet" href="fluidither\/components\/button\.css" \/>/);
    assert.match(output, /<script type="module" src="fluidither\/components\/tabs\.js"><\/script>/);
  });
});

test('it uses the public folder when there is one, and --dir wins', () => {
  inFolder((folder) => {
    mkdirSync(join(folder, 'public'));
    run(folder, 'add', 'card');
    assert.ok(existsSync(join(folder, 'public/fluidither/components/card.css')));
    run(folder, 'add', 'card', '--dir', 'site/ui');
    assert.ok(existsSync(join(folder, 'site/ui/components/card.css')));
  });
});

test('every component copies with no broken imports, and every file belongs to one', () => {
  const names = run(here, 'list').trim().split('\n');
  const covered = new Set();
  inFolder((folder) => {
    for (const name of names) {
      const target = join(folder, name);
      run(folder, 'add', name, '--dir', target);
      assert.deepEqual(brokenImports(target), [], name);
      for (const file of allFiles(target)) covered.add(file);
    }
  });
  const missing = readdirSync(join(here, '../src/components')).filter((file) => !covered.has(`components/${file}`));
  assert.deepEqual(missing, []);
});

test('init copies everything', () => {
  inFolder((folder) => {
    run(folder, 'init');
    assert.ok(existsSync(join(folder, 'fluidither/components/world.js')));
    assert.ok(existsSync(join(folder, 'fluidither/core/blue-noise.js')));
  });
});

test('skill installs a skill an agent can find', () => {
  inFolder((folder) => {
    run(folder, 'skill');
    const skill = readFileSync(join(folder, '.agents/skills/fluidither/SKILL.md'), 'utf8');
    assert.match(skill, /^name: fluidither$/m);
    assert.ok(existsSync(join(folder, '.agents/skills/fluidither/references/web.md')));
  });
});

test('mistakes explain themselves and fail', () => {
  inFolder((folder) => {
    const unknown = spawnSync(process.execPath, [cli, 'add', 'buton'], { cwd: folder, encoding: 'utf8' });
    assert.equal(unknown.status, 1);
    assert.match(unknown.stderr, /unknown component: buton/);
    assert.equal(spawnSync(process.execPath, [cli, 'add'], { cwd: folder }).status, 1);
    assert.equal(spawnSync(process.execPath, [cli, 'nonsense'], { cwd: folder }).status, 1);
    assert.deepEqual(readdirSync(folder), [], 'nothing is copied after a mistake');
  });
});
