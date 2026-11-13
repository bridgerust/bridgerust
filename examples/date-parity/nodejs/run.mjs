// Adapt the public TypeScript API to the parity runner protocol.
import { createInterface } from "node:readline";

const args = process.argv.slice(2);
const legacy = args.length === 1 && args[0] === "--legacy";
if (args.length && !legacy) throw new Error("usage: node run.mjs [--legacy]");

const { DateParseError, normalizeDate } = await import(legacy ? "./legacy-dates.ts" : "./dates.ts");
for await (const line of createInterface({ input: process.stdin, crlfDelay: Infinity })) {
  const { id, input } = JSON.parse(line);
  let outcome;
  try {
    outcome = { ok: normalizeDate(input) };
  } catch (error) {
    if (!(error instanceof DateParseError)) throw error;
    outcome = { error: { code: error.code } };
  }
  if (!legacy && id === "missing-offset" && process.env.BRIDGE_PARITY_DEMO_REGRESSION === "1") {
    outcome = { ok: `${input}Z` };
  }
  process.stdout.write(`${JSON.stringify({ id, outcome })}\n`);
}
