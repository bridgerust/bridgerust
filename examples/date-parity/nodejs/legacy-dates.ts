// Reference implementation for the TypeScript backend in this example.

import { DateParseError } from "./date-errors.ts";

export { DateParseError };

const DATE = /^([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})(?:(Z)|([+-])([0-9]{2}):([0-9]{2}))?(?![\s\S])/;

export function normalizeDate(value: string): string {
  const match = DATE.exec(value);
  if (!match) throw new DateParseError("INVALID_FORMAT");

  const [year, month, day, hour, minute, second] = match.slice(1, 7).map(Number);
  const [, , , , , , , utc, sign, offsetHours, offsetMinutes] = match;
  const date = new Date(0);
  date.setUTCFullYear(year, month - 1, day);
  date.setUTCHours(hour, minute, second);
  if (year === 0 || date.getUTCFullYear() !== year || date.getUTCMonth() !== month - 1 ||
      date.getUTCDate() !== day || date.getUTCHours() !== hour ||
      date.getUTCMinutes() !== minute || date.getUTCSeconds() !== second) {
    throw new DateParseError("INVALID_DATE");
  }
  if (!utc && !sign) throw new DateParseError("MISSING_OFFSET");

  if (sign) {
    const hours = Number(offsetHours);
    const minutes = Number(offsetMinutes);
    if (hours > 23 || minutes > 59) throw new DateParseError("INVALID_DATE");
    const offset = (hours * 60 + minutes) * (sign === "+" ? 1 : -1);
    date.setTime(date.getTime() - offset * 60_000);
  }
  if (date.getUTCFullYear() < 1 || date.getUTCFullYear() > 9999) {
    throw new DateParseError("INVALID_DATE");
  }
  return date.toISOString().replace(".000Z", "Z");
}
