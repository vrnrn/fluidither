// Serves web/ with the strictest policy we promise to work under, so any inline
// style or data: image shows up as a failure in the demo and the browser tests.
import { createReadStream, existsSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { dirname, extname, join, normalize, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

const types = {
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.svg': 'image/svg+xml',
};

export const contentSecurityPolicy = [
  "default-src 'none'",
  "script-src 'self'",
  "style-src 'self'",
  "img-src 'self'",
  "base-uri 'none'",
  "form-action 'none'",
].join('; ');

export function createDemoServer() {
  return createServer((request, response) => {
    let path = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
    if (path === '/') path = '/demo/index.html';
    const file = normalize(join(root, path));
    if (!file.startsWith(root + sep) || !existsSync(file) || !statSync(file).isFile()) {
      response.writeHead(404).end('Not found');
      return;
    }
    response.writeHead(200, {
      'Content-Security-Policy': contentSecurityPolicy,
      'Content-Type': types[extname(file)] ?? 'application/octet-stream',
    });
    createReadStream(file).pipe(response);
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const port = Number(process.env.PORT ?? 4173);
  createDemoServer().listen(port, '127.0.0.1', () => console.log(`http://127.0.0.1:${port}`));
}
