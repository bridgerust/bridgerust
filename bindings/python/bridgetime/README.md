# bridgetime (Python)

BridgeTime is a Rust-powered datetime toolkit with a Day.js/Moment-style API exposed to Python.

## Install

```bash
pip install bridgetime
```

## Quickstart

```python
from bridgetime import BridgeTime, supported_units

now = BridgeTime.now("UTC")
print(now.to_iso())

ny = now.to_timezone("America/New_York")
print(ny.format("YYYY-MM-DD HH:mm:ss"))

future = now.add(2, "week").start_of("day")
print(future.to_iso())

print(now.get("month"))          # 0-based month (Jan=0)
print(now.set("day", 1).to_iso())  # set weekday (Sunday=0)
print(now.days_in_month())
print(now.is_between(now.start_of("day"), now.end_of("day"), "day", "[]"))

print(supported_units())
```

## API Highlights

- Core: `parse`, `format`, `add`, `subtract`, `start_of`, `end_of`, `diff`
- Calendar helpers: `get`, `set`, `days_in_month`, `is_leap_year`, `is_valid`
- Comparison helpers: `is_before`, `is_after`, `is_same`, `is_same_or_before`, `is_same_or_after`
- Unit/range helpers: `is_before_unit`, `is_after_unit`, `is_same_unit`, `is_between`

## Build Locally

```bash
cd bindings/python/bridgetime
maturin develop
```
