"""Adapt the public Python API to the parity runner protocol."""

import json
import sys

if sys.argv[1:] == ["--legacy"]:
    from legacy_dates import DateParseError, normalize_date
elif len(sys.argv) == 1:
    from dates import DateParseError, normalize_date
else:
    raise SystemExit("usage: python run.py [--legacy]")

for line in sys.stdin:
    case = json.loads(line)
    try:
        outcome = {"ok": normalize_date(case["input"])}
    except DateParseError as error:
        outcome = {"error": {"code": error.code}}
    print(json.dumps({"id": case["id"], "outcome": outcome}, separators=(",", ":")))
