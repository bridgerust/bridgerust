// Drop-in TypeScript interface backed by the installed native package.
import { createRequire } from "node:module";
import { DateParseError } from "./date-errors.ts";

const require = createRequire(import.meta.url);
const { normalizeDate: normalize } = require("bridge-date-parity") as {
  normalizeDate(value: string, allowMissingOffset: boolean): string;
};

type Outcome = { ok: string } | { error: { code: string } };

export { DateParseError };

export function normalizeDate(value: string): string {
  const outcome: Outcome = JSON.parse(normalize(value, false));
  if ("error" in outcome) throw new DateParseError(outcome.error.code);
  return outcome.ok;
}
