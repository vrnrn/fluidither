// <fd-color-picker value="#fc9e47" aria-label="Accent">: a field of saturation and value in
// cells, a hue slider and the hex to type. Arrow keys move the mark, and `change` fires with
// `.value` set. The same cell grid as ColorPicker on GPUI.

import { fromHex, hsvToRgb, rgbToHsv, toHex } from '../core/color.js';

const COLUMNS = 32;
const ROWS = 18;
const KEY_STEP = 1 / 32;

const clamp01 = (value) => Math.min(Math.max(value, 0), 1);

export class ColorPicker extends HTMLElement {
  #hue = 0;
  #saturation = 1;
  #bright = 1;
  #painter = null;
  #field = null;
  #mark = null;
  #slider = null;
  #hex = null;

  get value() {
    return toHex(hsvToRgb(this.#hue, this.#saturation, this.#bright));
  }

  set value(text) {
    const channels = fromHex(text);
    if (!channels) return;
    [this.#hue, this.#saturation, this.#bright] = rgbToHsv(...channels);
    this.#draw();
  }

  connectedCallback() {
    if (this.#field) return;
    const label = this.getAttribute('aria-label') ?? 'Color';
    this.removeAttribute('aria-label');
    this.setAttribute('role', 'group');
    this.setAttribute('aria-label', label);

    this.#field = document.createElement('div');
    this.#field.className = 'fd-color-field';
    this.#field.tabIndex = 0;
    this.#field.setAttribute('role', 'slider');
    this.#field.setAttribute('aria-label', `${label} saturation and brightness`);
    const canvas = document.createElement('canvas');
    canvas.width = COLUMNS;
    canvas.height = ROWS;
    this.#painter = canvas.getContext('2d');
    this.#mark = document.createElement('span');
    this.#mark.className = 'fd-color-mark';
    this.#field.append(canvas, this.#mark);

    this.#slider = document.createElement('input');
    this.#slider.type = 'range';
    this.#slider.className = 'fd-color-hue';
    this.#slider.min = '0';
    this.#slider.max = '360';
    this.#slider.setAttribute('aria-label', `${label} hue`);

    this.#hex = document.createElement('input');
    this.#hex.className = 'fd-input';
    this.#hex.spellcheck = false;
    this.#hex.setAttribute('aria-label', `${label} hex`);
    this.append(this.#field, this.#slider, this.#hex);

    const fromPointer = (event) => {
      const { left, top, width, height } = this.#field.getBoundingClientRect();
      this.#saturation = clamp01((event.clientX - left) / width);
      this.#bright = 1 - clamp01((event.clientY - top) / height);
      this.#commit();
    };
    this.#field.addEventListener('pointerdown', (event) => {
      this.#field.setPointerCapture(event.pointerId);
      fromPointer(event);
    });
    this.#field.addEventListener('pointermove', (event) => {
      if (this.#field.hasPointerCapture(event.pointerId)) fromPointer(event);
    });
    this.#field.addEventListener('keydown', (event) => this.#onKey(event));
    this.#slider.addEventListener('input', () => {
      this.#hue = Number(this.#slider.value);
      this.#commit();
    });
    this.#hex.addEventListener('change', () => {
      this.value = this.#hex.value;
      this.#commit();
    });
    this.value = this.getAttribute('value') ?? '#ff0000';
  }

  #onKey(event) {
    const moves = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, 1], ArrowDown: [0, -1] };
    const move = moves[event.key];
    if (!move) return;
    event.preventDefault();
    this.#saturation = clamp01(this.#saturation + move[0] * KEY_STEP);
    this.#bright = clamp01(this.#bright + move[1] * KEY_STEP);
    this.#commit();
  }

  #commit() {
    this.#draw();
    this.dispatchEvent(new Event('change', { bubbles: true }));
  }

  #draw() {
    if (!this.#painter) return;
    for (let row = 0; row < ROWS; row += 1) {
      for (let column = 0; column < COLUMNS; column += 1) {
        this.#painter.fillStyle = toHex(hsvToRgb(this.#hue, (column + 0.5) / COLUMNS, 1 - (row + 0.5) / ROWS));
        this.#painter.fillRect(column, row, 1, 1);
      }
    }
    this.#mark.style.setProperty('--_x', `${this.#saturation * 100}%`);
    this.#mark.style.setProperty('--_y', `${(1 - this.#bright) * 100}%`);
    this.#slider.value = this.#hue;
    this.#hex.value = this.value;
    this.#field.setAttribute('aria-valuetext', this.value);
  }
}

customElements.define('fd-color-picker', ColorPicker);
