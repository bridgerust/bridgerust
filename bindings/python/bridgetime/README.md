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

print(supported_units())
```

## Build Locally

```bash
cd bindings/python/bridgetime
maturin develop
```
