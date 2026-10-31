// Serves the site with the library at /fluidither, where a deploy would copy web/src. The page
// carries its own Content Security Policy, so there is none to add here.
import { createReadStream, existsSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { dirname, extname, join, normalize, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const site = dirname(fileURLToPath(import.meta.url));
const library = join(site, '../web/src');

const types = {
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.svg': 'image/svg+xml',
};

export function createSiteServer() {
  return createServer((request, response) => {
    const path = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
    const inLibrary = path.startsWith('/fluidither/');
    const root = inLibrary ? library : site;
    const relative = inLibrary ? path.slice('/fluidither'.length) : path === '/' ? '/index.html' : path;
    const file = normalize(join(root, relative));
    if (!file.startsWith(root + sep) || !existsSync(file) || !statSync(file).isFile()) {
      response.writeHead(404).end('Not found');
      return;
    }
    response.writeHead(200, { 'Content-Type': types[extname(file)] ?? 'application/octet-stream' });
    createReadStream(file).pipe(response);
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const port = Number(process.env.PORT ?? 4174);
  createSiteServer().listen(port, '127.0.0.1', () => console.log(`http://127.0.0.1:${port}`));
}
