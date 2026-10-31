// Dithered charts as a pure function: data in, cells out. A cell holds 0 for empty or
// a series number from 1. gpui/src/charts.rs is the same code, and fixtures/charts.json
// holds both to the same cells.

export const CHARTS = ['bar', 'area', 'line', 'donut', 'radar', 'heatmap', 'gauge'];

const TAU = 2 * Math.PI;

function topOf(column, width, height, values, top) {
  const span = Math.max(values.length - 1, 1);
  const along = ((column + 0.5) / width) * span;
  const left = Math.min(Math.floor(along), span - 1);
  const right = Math.min(left + 1, values.length - 1);
  const value = values[left] + (values[right] - values[left]) * (along - left);
  return (1 - value / top) * (height - 1);
}

function largest(series) {
  let top = 0;
  for (const values of series) for (const value of values) top = Math.max(top, value);
  return top > 0 ? top : 1;
}

function bars(series, width, height, top, inked) {
  const groups = series[0].length;
  const groupWidth = width / groups;
  const barWidth = (groupWidth * 0.78) / series.length;
  for (let x = 0; x < width; x += 1) {
    const group = Math.min(Math.floor(x / groupWidth), groups - 1);
    const inset = x + 0.5 - group * groupWidth - groupWidth * 0.11;
    const which = Math.floor(inset / barWidth);
    if (inset < 0 || which >= series.length) continue;
    if (inset - which * barWidth > barWidth * 0.88) continue;
    const tall = (series[which][group] / top) * height;
    for (let y = height - 1; y >= height - tall; y -= 1) {
      inked(x, y, which, 0.55 + 0.45 * ((y - (height - tall)) / tall < 0.5 ? 1 : 0.6));
    }
  }
}

function area(series, width, height, top, inked) {
  series.forEach((values, which) => {
    for (let x = 0; x < width; x += 1) {
      const line = topOf(x, width, height, values, top);
      for (let y = Math.ceil(line); y < height; y += 1) {
        inked(x, y, which, y - line < 1 ? 1 : 0.7 * (1 - (y - line) / (height - line)));
      }
    }
  });
}

function line(series, width, height, top, inked) {
  series.forEach((values, which) => {
    let previous = topOf(0, width, height, values, top);
    for (let x = 0; x < width; x += 1) {
      const here = topOf(x, width, height, values, top);
      const from = Math.floor(Math.min(previous, here));
      const to = Math.ceil(Math.max(previous, here));
      for (let y = from; y <= to && y < height; y += 1) inked(x, y, which, 1);
      previous = here;
    }
  });
}

function donut(series, width, height, _top, inked) {
  const values = series[0];
  const total = values.reduce((sum, value) => sum + value, 0) || 1;
  const radius = Math.min(width, height) / 2 - 0.5;
  const hole = radius * 0.62;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const dx = x + 0.5 - width / 2;
      const dy = y + 0.5 - height / 2;
      const distance = Math.sqrt(dx * dx + dy * dy);
      if (distance > radius || distance < hole) continue;
      let turn = Math.atan2(dx, -dy) / TAU;
      if (turn < 0) turn += 1;
      let sum = 0;
      let slice = 0;
      for (; slice < values.length - 1; slice += 1) {
        sum += values[slice] / total;
        if (turn < sum) break;
      }
      inked(x, y, slice, 0.5 + 0.5 * ((distance - hole) / (radius - hole)));
    }
  }
}

// The straight edge of a regular polygon, as a share of its corner radius.
function polygonEdge(share, count) {
  const half = Math.PI / count;
  return Math.cos(half) / Math.cos((share * 2 - 1) * half);
}

function radar(series, width, height, top, inked) {
  const radius = Math.min(width, height) / 2 - 1;
  // A faint web first, so the shapes have something to be read against.
  const corners = series[0].length;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const dx = x + 0.5 - width / 2;
      const dy = y + 0.5 - height / 2;
      let turn = Math.atan2(dx, -dy) / TAU;
      if (turn < 0) turn += 1;
      const along = turn * corners;
      const edge = radius * polygonEdge(along - Math.floor(along), corners);
      const distance = Math.sqrt(dx * dx + dy * dy);
      if (Math.abs(distance - edge) < 0.7 || Math.abs(distance - edge / 2) < 0.7) inked(x, y, 0, 0.3);
    }
  }
  series.forEach((values, which) => {
    const count = values.length;
    for (let y = 0; y < height; y += 1) {
      for (let x = 0; x < width; x += 1) {
        const dx = x + 0.5 - width / 2;
        const dy = y + 0.5 - height / 2;
        const distance = Math.sqrt(dx * dx + dy * dy);
        let turn = Math.atan2(dx, -dy) / TAU;
        if (turn < 0) turn += 1;
        const along = turn * count;
        const left = Math.floor(along) % count;
        const share = along - Math.floor(along);
        const a = values[left] / top;
        const b = values[(left + 1) % count] / top;
        // Straight edges between spokes, which a lerp of the reach would bow outward.
        const reach = radius * (a + (b - a) * share) * polygonEdge(share, count);
        if (distance > reach) continue;
        inked(x, y, which, reach - distance < 1 ? 1 : 0.4);
      }
    }
  });
}

function heatmap(series, width, height, top, inked) {
  const rows = series.length;
  const columns = series[0].length;
  const cellWidth = width / columns;
  const cellHeight = height / rows;
  for (let y = 0; y < height; y += 1) {
    const row = Math.min(Math.floor(y / cellHeight), rows - 1);
    if (y + 1 - row * cellHeight >= cellHeight) continue;
    for (let x = 0; x < width; x += 1) {
      const column = Math.min(Math.floor(x / cellWidth), columns - 1);
      if (x + 1 - column * cellWidth >= cellWidth) continue;
      inked(x, y, 0, series[row][column] / top);
    }
  }
}

function gauge(series, width, height, top, inked) {
  const share = Math.min(series[0][0] / (series[0][1] ?? top), 1);
  const radius = Math.min(width / 2, height) - 1;
  const hole = radius * 0.7;
  const centerY = height - 0.5;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const dx = x + 0.5 - width / 2;
      const dy = y + 0.5 - centerY;
      const distance = Math.sqrt(dx * dx + dy * dy);
      if (distance > radius || distance < hole || dy > 0) continue;
      const turn = Math.atan2(dy, dx) / Math.PI + 1;
      inked(x, y, 0, turn <= share ? 1 : 0.18);
    }
  }
}

const DRAW = { bar: bars, area, line, donut, radar, heatmap, gauge };

/**
 * Fill `out` (width * height) for a chart. `series` is a list of value lists. A gauge
 * takes one list of [value, max]. A heatmap reads its lists as rows. A donut's slices
 * are its first list, and each slice counts as a series for color.
 */
export function chartCells(kind, series, width, height, map, out) {
  out.fill(0);
  const wrap = map.size - 1;
  const inked = (x, y, which, tone) => {
    if (tone > map.thresholds[(y & wrap) * map.size + (x & wrap)]) out[y * width + x] = which + 1;
  };
  DRAW[kind](series, width, height, largest(series), inked);
}
