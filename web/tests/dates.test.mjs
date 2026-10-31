import { test } from 'node:test';
import assert from 'node:assert/strict';
import { addMonths, fromDays, iso, monthGrid, parseIso, toDays, weekday } from '../src/core/dates.js';

test('day numbers match the platform calendar across centuries', () => {
  for (let days = -150000; days < 150000; days += 311) {
    const date = new Date(days * 86400000);
    const expected = { year: date.getUTCFullYear(), month: date.getUTCMonth() + 1, day: date.getUTCDate() };
    assert.deepEqual(fromDays(days), expected);
    assert.equal(toDays(expected), days);
    assert.equal(weekday(expected), (date.getUTCDay() + 6) % 7);
  }
});

test('a month grid starts on a Monday and holds the whole month', () => {
  const grid = monthGrid(2026, 10);
  assert.equal(grid.length, 42);
  assert.ok(grid.every((date, index) => weekday(date) === index % 7));
  assert.equal(grid.filter((date) => date.month === 10).length, 31);
});

test('adding months clamps to the end of a short month', () => {
  assert.equal(iso(addMonths({ year: 2024, month: 1, day: 31 }, 1)), '2024-02-29');
  assert.equal(iso(addMonths({ year: 2023, month: 1, day: 31 }, 1)), '2023-02-28');
  assert.equal(iso(addMonths({ year: 2024, month: 1, day: 15 }, -1)), '2023-12-15');
});

test('parsing rejects dates that do not exist', () => {
  assert.deepEqual(parseIso('2024-02-29'), { year: 2024, month: 2, day: 29 });
  assert.equal(parseIso('2023-02-29'), null);
  assert.equal(parseIso('nonsense'), null);
  assert.equal(parseIso(null), null);
});
