// A canvas whose pixels are dither cells: square, on whole device pixels, and filling
// its host. Each element that paints cells keeps one of these.

const OPAQUE = 0xff000000;

export class CellCanvas {
  columns = 0;
  rows = 0;

  constructor(host) {
    this.host = host;
    this.canvas = host.appendChild(document.createElement('canvas'));
    this.canvas.setAttribute('aria-hidden', 'true');
    this.painter = this.canvas.getContext('2d');
  }

  /** Size to the host. Returns false while the host has no size. */
  fit(cellSize) {
    const { width, height } = this.host.getBoundingClientRect();
    if (width === 0 || height === 0) return false;
    const scale = devicePixelRatio;
    const deviceCell = Math.max(1, Math.round(cellSize * scale));
    const cell = deviceCell / scale;
    const columns = Math.ceil(width / cell);
    const rows = Math.ceil(height / cell);
    this.canvas.style.width = `${columns * cell}px`;
    this.canvas.style.height = `${rows * cell}px`;
    if (columns !== this.columns || rows !== this.rows) {
      this.columns = columns;
      this.rows = rows;
      this.canvas.width = columns;
      this.canvas.height = rows;
      this.words = new Uint32Array(columns * rows);
      this.image = new ImageData(new Uint8ClampedArray(this.words.buffer), columns, rows);
    }
    return true;
  }

  /** Paint the cells that `pick(cell)` accepts, in one CSS color. */
  layer(cells, color, pick, first) {
    const { words, painter, columns, rows } = this;
    for (let index = 0; index < words.length; index += 1) words[index] = pick(cells[index]) ? OPAQUE : 0;
    const target = first ? painter : (this.spare ??= new OffscreenCanvas(columns, rows).getContext('2d'));
    if (!first) {
      target.canvas.width = columns;
      target.canvas.height = rows;
    }
    target.putImageData(this.image, 0, 0);
    target.globalCompositeOperation = 'source-in';
    target.fillStyle = color;
    target.fillRect(0, 0, columns, rows);
    target.globalCompositeOperation = 'source-over';
    if (!first) painter.drawImage(target.canvas, 0, 0);
  }
}

const watchers = new Set();
let watching = false;

/** Call `repaint` on the element whenever the theme changes. */
export function followTheme(element) {
  watchers.add(element);
  if (watching) return;
  watching = true;
  const notify = () => watchers.forEach((each) => each.repaint());
  new MutationObserver(notify).observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'data-theme'] });
  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', notify);
}

export const unfollowTheme = (element) => watchers.delete(element);
