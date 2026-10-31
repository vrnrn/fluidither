// Puts the site and the library together in site-dist/, ready to serve from any static host.
import { cpSync, readdirSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const out = join(root, 'site-dist');
const skipped = new Set(['tests', 'serve.mjs']);

rmSync(out, { recursive: true, force: true });
for (const name of readdirSync(join(root, 'site'))) {
  if (!skipped.has(name)) cpSync(join(root, 'site', name), join(out, name), { recursive: true });
}
cpSync(join(root, 'web/src'), join(out, 'fluidither'), { recursive: true });
console.log(`Built the site in ${out}`);
