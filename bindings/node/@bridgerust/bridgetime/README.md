# @bridgerust/bridgetime

BridgeTime is a Rust-powered datetime toolkit with a Day.js/Moment-style API for Node.js.

## Install

```bash
npm install @bridgerust/bridgetime
```

## Quickstart

```ts
import { BridgeTime, supportedUnits } from "@bridgerust/bridgetime";

const now = BridgeTime.now("UTC");
console.log(now.toIso());

const custom = BridgeTime.parseFormat("22/02/2026 10:15", "DD/MM/YYYY HH:mm", "UTC");
console.log(custom.toIso());

const ny = now.toTimezone("America/New_York");
console.log(ny.format("YYYY-MM-DD HH:mm:ss"));

const next = now.add(1, "month").startOf("day");
console.log(next.toIso());

console.log(now.get("month")); // 0-based month (Jan=0)
console.log(now.set("day", 1).toIso()); // set weekday (Sunday=0)
console.log(now.daysInMonth());
console.log(now.quarter(), now.isoWeekday());
console.log(now.dayOfYear(), now.week(), now.isoWeek());
console.log(now.isToday());
console.log(now.add(30, "minute").fromNow()); // in 30 minutes
console.log(now.isBetween(now.startOf("day"), now.endOf("day"), "day", "[]"));

console.log(supportedUnits());
```

## API Highlights

- Core: `parse`, `format`, `add`, `subtract`, `startOf`, `endOf`, `diff`
- Custom parse helper: `parseFormat(input, pattern, timezone?)`
- Calendar helpers: `get`, `set`, component getters/setters (`year`, `setYear`, etc), `daysInMonth`, `isLeapYear`, `isValid`
- Week/day helpers: `dayOfYear`, `setDayOfYear`, `quarter`, `setQuarter`, `isoWeekday`, `setIsoWeekday`, `week`, `weekOfYear`, `setWeek`, `isoWeek`, `setIsoWeek`
- Relative-day helpers: `isToday`, `isYesterday`, `isTomorrow`
- Relative-time helpers: `fromTime`, `toTime`, `fromNow`, `toNow`
- Comparison helpers: `isBefore`, `isAfter`, `isSame`, `isSameOrBefore`, `isSameOrAfter`, `isSameOrBeforeUnit`, `isSameOrAfterUnit`
- Unit/range helpers: `isBeforeUnit`, `isAfterUnit`, `isSameUnit`, `isBetween`

## Build Locally

```bash
cd bindings/node/@bridgerust/bridgetime
npm install
npm run build
```
