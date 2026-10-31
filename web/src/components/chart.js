// <fd-chart kind="bar" series="3 7 2; 5 1 6" names="Sales; Costs" labels="Mon Tue Wed"
// aria-label="..."> draws a dithered chart. Separate labels with ";" if they hold spaces. Kinds: bar, area, line, donut, radar,
// heatmap, gauge. For a gauge, series is "value max". A hidden table carries the same
// numbers for screen readers, so the data is never locked in the picture.

import { chartCells } from '../core/charts.js';
import { CellCanvas, followTheme, unfollowTheme } from '../core/cells.js';
import { thresholdMap } from '../core/dither.js';

const COLORS = ['--fd-ink', '--fd-orange', '--fd-blue', '--fd-green', '--fd-violet', '--fd-pink', '--fd-red'];
const CHART_CELL = 2;

const list = (text, split = /\s+/) => (text ?? '').trim().split(split).filter(Boolean);
const labelList = (text) => list(text, text?.includes(';') ? /\s*;\s*/ : /\s+/);

export class Chart extends HTMLElement {
  static observedAttributes = ['kind', 'series', 'names', 'labels'];

  #plot = null;
  #cells = null;
  #legend = null;
  #table = null;
  #map = null;
  #sizes = null;

  connectedCallback() {
    if (!this.#plot) {
      this.setAttribute('role', 'figure');
      this.#plot = this.appendChild(document.createElement('div'));
      this.#plot.className = 'fd-chart-plot';
      this.#cells = new CellCanvas(this.#plot);
      this.#legend = this.appendChild(document.createElement('div'));
      this.#legend.className = 'fd-chart-legend';
      this.#table = this.appendChild(document.createElement('table'));
      this.#table.className = 'fd-chart-data';
    }
    this.#sizes = new ResizeObserver(() => this.repaint());
    this.#sizes.observe(this.#plot);
    followTheme(this);
    thresholdMap('bayer4').then((map) => {
      this.#map = map;
      this.#update();
    });
  }

  disconnectedCallback() {
    this.#sizes.disconnect();
    this.#sizes = null;
    unfollowTheme(this);
  }

  attributeChangedCallback() {
    if (this.#sizes && this.#map) this.#update();
  }

  get #series() {
    return list(this.getAttribute('series'), /\s*;\s*/).map((row) => list(row).map(Number));
  }

  get #kind() {
    return this.getAttribute('kind') ?? 'bar';
  }

  // A donut colors each slice, and everything else each series.
  #keys(series) {
    const names = list(this.getAttribute('names'), /\s*;\s*/);
    const labels = labelList(this.getAttribute('labels'));
    if (this.#kind === 'donut') return labels.length ? labels : series[0].map((_, index) => `Slice ${index + 1}`);
    if (this.#kind === 'heatmap' || this.#kind === 'gauge') return [];
    return names.length ? names : series.map((_, index) => `Series ${index + 1}`);
  }

  #update() {
    const series = this.#series;
    if (series.length === 0) return;
    const keys = this.#keys(series);
    this.#legend.replaceChildren(
      ...keys.map((key, index) => {
        const entry = document.createElement('span');
        entry.textContent = key;
        entry.style.setProperty('--_key', `var(${COLORS[index % COLORS.length]})`);
        return entry;
      }),
    );
    this.#fillTable(series);
    this.repaint();
  }

  #fillTable(series) {
    const labels = labelList(this.getAttribute('labels'));
    const names = list(this.getAttribute('names'), /\s*;\s*/);
    const rows = series.map((values, index) => {
      const row = document.createElement('tr');
      const head = document.createElement('th');
      head.scope = 'row';
      head.textContent = names[index] ?? `Series ${index + 1}`;
      row.append(head, ...values.map((value) => Object.assign(document.createElement('td'), { textContent: value })));
      return row;
    });
    if (labels.length) {
      const header = document.createElement('tr');
      header.append(document.createElement('td'), ...labels.map((label) => Object.assign(document.createElement('th'), { scope: 'col', textContent: label })));
      rows.unshift(header);
    }
    this.#table.replaceChildren(...rows);
  }

  repaint() {
    const series = this.#series;
    const cells = this.#cells;
    if (!this.#map || series.length === 0 || !cells.fit(CHART_CELL)) return;
    const { columns, rows } = cells;
    const marks = new Uint8Array(columns * rows);
    chartCells(this.#kind, series, columns, rows, this.#map, marks);
    const styles = getComputedStyle(this);
    const count = Math.max(...marks);
    cells.painter.clearRect(0, 0, columns, rows);
    for (let which = 1; which <= count; which += 1) {
      const color = styles.getPropertyValue(COLORS[(which - 1) % COLORS.length]);
      cells.layer(marks, color, (mark) => mark === which, false);
    }
  }
}

customElements.define('fd-chart', Chart);
