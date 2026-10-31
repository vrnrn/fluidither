// Calendar arithmetic on plain integers, so nothing depends on time zones and both
// targets agree. gpui/src/dates.rs is the same code. A date is { year, month, day }
// with month from 1. Days are counted from 1970-01-01.

const floorDiv = (a, b) => Math.floor(a / b);

export function toDays({ year, month, day }) {
  const y = month <= 2 ? year - 1 : year;
  const era = floorDiv(y, 400);
  const yearOfEra = y - era * 400;
  const dayOfYear = floorDiv(153 * (month + (month > 2 ? -3 : 9)) + 2, 5) + day - 1;
  const dayOfEra = yearOfEra * 365 + floorDiv(yearOfEra, 4) - floorDiv(yearOfEra, 100) + dayOfYear;
  return era * 146097 + dayOfEra - 719468;
}

export function fromDays(days) {
  const shifted = days + 719468;
  const era = floorDiv(shifted, 146097);
  const dayOfEra = shifted - era * 146097;
  const yearOfEra = floorDiv(dayOfEra - floorDiv(dayOfEra, 1460) + floorDiv(dayOfEra, 36524) - floorDiv(dayOfEra, 146096), 365);
  const dayOfYear = dayOfEra - (365 * yearOfEra + floorDiv(yearOfEra, 4) - floorDiv(yearOfEra, 100));
  const shiftedMonth = floorDiv(5 * dayOfYear + 2, 153);
  const month = shiftedMonth < 10 ? shiftedMonth + 3 : shiftedMonth - 9;
  const year = yearOfEra + era * 400 + (month <= 2 ? 1 : 0);
  return { year, month, day: dayOfYear - floorDiv(153 * shiftedMonth + 2, 5) + 1 };
}

/** Day of the week, 0 for Monday. */
export const weekday = (date) => ((toDays(date) + 3) % 7 + 7) % 7;

export function addMonths({ year, month, day }, count) {
  const total = year * 12 + (month - 1) + count;
  const nextYear = floorDiv(total, 12);
  const nextMonth = total - nextYear * 12 + 1;
  const lastDay = fromDays(toDays({ year: nextMonth === 12 ? nextYear + 1 : nextYear, month: nextMonth === 12 ? 1 : nextMonth + 1, day: 1 }) - 1).day;
  return { year: nextYear, month: nextMonth, day: Math.min(day, lastDay) };
}

/** The six weeks shown for a month, Monday first. */
export function monthGrid(year, month) {
  const first = { year, month, day: 1 };
  const start = toDays(first) - weekday(first);
  return Array.from({ length: 42 }, (_, index) => fromDays(start + index));
}

export const iso = ({ year, month, day }) => `${String(year).padStart(4, '0')}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;

export function parseIso(text) {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(text ?? '');
  if (!match) return null;
  const date = { year: Number(match[1]), month: Number(match[2]), day: Number(match[3]) };
  return iso(fromDays(toDays(date))) === text ? date : null;
}
