# BridgeTime

BridgeTime is a Rust-powered Day.js/Moment-style datetime toolkit for Python and Node.js.

## Goals

- Keep one shared Rust core for correctness and performance.
- Provide a familiar API surface (`parse`, `format`, `add`, `subtract`, `start_of`, `end_of`, `diff`).
- Support timezone-aware operations through IANA timezone names (e.g. `America/New_York`).

## Current Feature Set

- Parse ISO strings (`2026-02-22T10:15:30Z`)
- Parse local datetime strings (`YYYY-MM-DD`, `YYYY-MM-DD HH:mm:ss`)
- Day.js-style token formatting (`YYYY`, `MM`, `DD`, `HH`, `mm`, `ss`, `SSS`, `Z`)
- Immutable date arithmetic across units:
  - `millisecond`, `second`, `minute`, `hour`, `day`, `week`, `month`, `quarter`, `year`
- `start_of` / `end_of`
- `diff` (integer or float)
- Instant-preserving timezone conversion (`to_timezone`)

## API Surface

### Static methods

- `BridgeTime.now(timezone?)`
- `BridgeTime.parse(input, timezone?)`
- `BridgeTime.from_unix_ms(unix_ms, timezone?)`
- `BridgeTime.from_unix(unix_seconds, timezone?)`

### Instance methods

- `to_iso()`
- `format(pattern)`
- `unix_ms()` / `unix()` / `value_of()`
- `timezone()`
- `to_timezone(timezone)`
- `add(amount, unit)`
- `subtract(amount, unit)`
- `start_of(unit)`
- `end_of(unit)`
- `diff(other, unit, as_float?)`
- `is_before(other)` / `is_after(other)` / `is_same(other)`

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
