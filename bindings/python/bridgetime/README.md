# bridgetime (Python)

BridgeTime is a Rust-powered datetime toolkit with a Day.js/Moment-style API exposed to Python.

## Install

```bash
pip install bridgetime
```

## Quickstart

```python
from bridgetime import bridge_duration, bridge_time, supported_units

now = bridge_time.now("UTC")
print(now.to_iso())

custom = bridge_time.parse_format("22/02/2026 10:15", "DD/MM/YYYY HH:mm", "UTC")
print(custom.to_iso())
from_array = bridge_time.from_array([2026, 1, 22, 10, 15, 30, 250], "UTC")
print(from_array.to_array())
duration = bridge_duration(90, "minute")
print(duration.humanize(True))  # in 2 hours

ny = now.to_timezone("America/New_York")
print(ny.format("YYYY-MM-DD HH:mm:ss"))
print(now.utc_offset(), now.is_utc(), ny.is_dst())

future = now.add(2, "week").start_of("day")
print(future.to_iso())

print(now.get("month"))          # 0-based month (Jan=0)
print(now.set("day", 1).to_iso())  # set weekday (Sunday=0)
print(now.days_in_month())
print(now.quarter(), now.iso_weekday())
print(now.day_of_year(), now.week(), now.iso_week())
print(now.iso_week_year(), now.days_in_year(), now.weeks_in_year(), now.iso_weeks_in_year())
print(now.is_today())
print(now.add(30, "minute").from_now())  # in 30 minutes
print(now.add_duration(bridge_duration.from_minutes(30)).to_iso())
print(now.is_between(now.start_of("day"), now.end_of("day"), "day", "[]"))
print(bridge_time.min(now, future).to_iso(), bridge_time.max(now, future).to_iso())
print(now.clamp(now.subtract(1, "day"), now.add(1, "day")).to_iso())

print(supported_units())
```

## API Highlights

- Core: `parse`, `format`, `add`, `subtract`, `start_of`, `end_of`, `diff`
- Duration helpers: `bridge_duration`, `bridge_time.duration(value, unit?)`, `add_duration`, `subtract_duration`
- Custom parse helpers: `parse_format(input, pattern, timezone?)`, `from_array(components, timezone?)`, `to_array()`
- Timezone helpers: `timezone`, `utc_offset`, `is_utc`, `is_dst`, `to_timezone`
- Calendar helpers: `get`, `set`, component getters/setters (`year`, `set_year`, etc), `days_in_month`, `is_leap_year`, `is_valid`
- Week/day helpers: `day_of_year`, `set_day_of_year`, `quarter`, `set_quarter`, `iso_weekday`, `set_iso_weekday`, `week`, `week_of_year`, `set_week`, `iso_week`, `set_iso_week`, `iso_week_year`, `days_in_year`, `weeks_in_year`, `iso_weeks_in_year`
- Relative-day helpers: `is_today`, `is_yesterday`, `is_tomorrow`
- Relative-time helpers: `from_time`, `to_time`, `from_now`, `to_now`
- Min/max helpers: `bridge_time.min(a, b)`, `bridge_time.max(a, b)`, `clamp(start, end)`
- Comparison helpers: `is_before`, `is_after`, `is_same`, `is_same_or_before`, `is_same_or_after`, `is_same_or_before_unit`, `is_same_or_after_unit`
- Unit/range helpers: `is_before_unit`, `is_after_unit`, `is_same_unit`, `is_between`

Uppercase exports (`BridgeTime`, `BridgeDuration`) are no longer part of the public Python API.

## Build Locally

```bash
cd bindings/python/bridgetime
maturin develop
```
