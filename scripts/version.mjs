// Keeps the three version numbers in step. `node scripts/version.mjs 0.2.0` sets them, and
// `--check v0.2.0` fails unless all of them match that tag.
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const manifests = ['package.json', 'web/package.json'].map((file) => join(root, file));
const crate = join(root, 'gpui/Cargo.toml');
const crateVersion = /^version = "(.+)"$/m;

const versions = () => [
  ...manifests.map((file) => JSON.parse(readFileSync(file, 'utf8')).version),
  readFileSync(crate, 'utf8').match(crateVersion)[1],
];

const [first, second] = process.argv.slice(2);

if (first === '--check') {
  const wanted = (second ?? '').replace(/^v/, '');
  const found = versions();
  if (found.some((version) => version !== wanted)) {
    console.error(`tag ${second} does not match the versions: ${found.join(', ')}`);
    process.exit(1);
  }
  console.log(`all versions are ${wanted}`);
} else if (/^\d+\.\d+\.\d+$/.test(first ?? '')) {
  for (const file of manifests) {
    const manifest = JSON.parse(readFileSync(file, 'utf8'));
    manifest.version = first;
    writeFileSync(file, `${JSON.stringify(manifest, null, 2)}\n`);
  }
  writeFileSync(crate, readFileSync(crate, 'utf8').replace(crateVersion, `version = "${first}"`));
  execFileSync('npm', ['install', '--package-lock-only', '--silent'], { cwd: root });
  execFileSync('cargo', ['update', '--offline', '--package', 'fluidither', '--manifest-path', crate], { stdio: 'ignore' });
  console.log(`set every version to ${first}`);
} else {
  console.error('usage: version.mjs 0.2.0, or version.mjs --check v0.2.0');
  process.exit(1);
}
