# BridgeTime

BridgeTime is a Rust-powered Day.js/Moment-style datetime toolkit for Python and Node.js.

## Goals

- Keep one shared Rust core for correctness and performance.
- Provide a familiar API surface (`parse`, `format`, `add`, `subtract`, `start_of`, `end_of`, `diff`).
- Support timezone-aware operations through IANA timezone names (e.g. `America/New_York`).

## Current Feature Set

- Parse ISO strings (`2026-02-22T10:15:30Z`)
- Parse local datetime strings (`YYYY-MM-DD`, `YYYY-MM-DD HH:mm:ss`)
- Parse custom formats with Day.js-style tokens (`parse_format`)
- Parse/build via component arrays (`from_array`, `to_array`)
- Day.js-style token formatting (`YYYY`, `MM`, `DD`, `HH`, `mm`, `ss`, `SSS`, `Z`)
- Immutable date arithmetic across units:
  - `millisecond`, `second`, `minute`, `hour`, `day`, `week`, `month`, `quarter`, `year`
- `start_of` / `end_of`
- `diff` (integer or float)
- Instant-preserving timezone conversion (`to_timezone`)
- Timezone offset helpers: `utc_offset()` / `is_utc()`
- DST helper: `is_dst()`
- Calendar helpers:
  - `get(field)` / `set(field, value)`
  - explicit component getters/setters (`year`, `month`, `date`, `day`, `hour`, `minute`, `second`, `millisecond`)
  - `set_year` / `set_month` / `set_date` / `set_day` / `set_hour` / `set_minute` / `set_second` / `set_millisecond`
  - `days_in_month()` / `is_leap_year()` / `is_valid()`
- Day/week helpers:
  - `day_of_year()` / `set_day_of_year(value)`
  - `quarter()` / `set_quarter(value)`
  - `iso_weekday()` / `set_iso_weekday(value)`
  - `week()` / `week_of_year()` / `set_week(value)`
  - `iso_week()` / `set_iso_week(value)`
  - `iso_week_year()` / `days_in_year()` / `weeks_in_year()` / `iso_weeks_in_year()`
- Relative-day helpers:
  - `is_today()` / `is_yesterday()` / `is_tomorrow()`
- Relative-time helpers:
  - `from_time(other, without_suffix?)` / `to_time(other, without_suffix?)`
  - `from_now(without_suffix?)` / `to_now(without_suffix?)`
- Range helpers:
  - `clamp(start, end)`
- Static ordering helpers:
  - `BridgeTime.min(a, b)` / `BridgeTime.max(a, b)`
- Comparison and range helpers:
  - `is_before` / `is_after` / `is_same`
  - `is_before_unit` / `is_after_unit` / `is_same_unit`
  - `is_same_or_before` / `is_same_or_after`
  - `is_same_or_before_unit` / `is_same_or_after_unit`
  - `is_between(start, end, unit?, inclusivity?)`

## Day.js Parity Snapshot

- Shared parity between Python and Node.js is high because both bindings use the same Rust core.
- Core datetime operations are covered.
- Full Day.js plugin parity is not complete yet (for example locale packs, durations, and calendar-style formatting helpers).

## API Surface

### Static methods

- `BridgeTime.now(timezone?)`
- `BridgeTime.parse(input, timezone?)`
- `BridgeTime.parse_format(input, pattern, timezone?)`
- `BridgeTime.from_array(components, timezone?)`
- `BridgeTime.from_unix_ms(unix_ms, timezone?)`
- `BridgeTime.from_unix(unix_seconds, timezone?)`

### Instance methods

- `to_iso()`
- `format(pattern)`
- `unix_ms()` / `unix()` / `value_of()`
- `timezone()`
- `to_array()`
- `utc_offset()` / `is_utc()`
- `is_dst()`
- `to_timezone(timezone)`
- `add(amount, unit)`
- `subtract(amount, unit)`
- `start_of(unit)`
- `end_of(unit)`
- `diff(other, unit, as_float?)`
- `is_before(other)` / `is_after(other)` / `is_same(other)`
- `is_before_unit(other, unit)` / `is_after_unit(other, unit)` / `is_same_unit(other, unit)`
- `is_same_or_before(other)` / `is_same_or_after(other)`
- `is_between(start, end, unit?, inclusivity?)`
- `get(field)` / `set(field, value)`
- `year()` / `month()` / `date()` / `day()` / `hour()` / `minute()` / `second()` / `millisecond()`
- `set_year(value)` / `set_month(value)` / `set_date(value)` / `set_day(value)` / `set_hour(value)` / `set_minute(value)` / `set_second(value)` / `set_millisecond(value)`
- `day_of_year()` / `set_day_of_year(value)`
- `quarter()` / `set_quarter(value)`
- `iso_weekday()` / `set_iso_weekday(value)`
- `week()` / `week_of_year()` / `set_week(value)`
- `iso_week()` / `set_iso_week(value)`
- `iso_week_year()` / `days_in_year()` / `weeks_in_year()` / `iso_weeks_in_year()`
- `is_today()` / `is_yesterday()` / `is_tomorrow()`
- `from_time(other, without_suffix?)` / `to_time(other, without_suffix?)`
- `from_now(without_suffix?)` / `to_now(without_suffix?)`
- `is_same_or_before_unit(other, unit)` / `is_same_or_after_unit(other, unit)`
- `clamp(start, end)`
- `days_in_month()` / `is_leap_year()` / `is_valid()`

## Local Development

### Rust

```bash
cargo test -p bridgetime-bridge
cargo check -p bridgetime-bridge --features python
cargo check -p bridgetime-bridge --features nodejs
```

### Python

```bash
cd bindings/python/bridgetime
maturin develop
```

### Node.js

```bash
cd bindings/node/@bridgerust/bridgetime
npm install
npm run build
```
