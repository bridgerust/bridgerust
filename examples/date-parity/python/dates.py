"""Drop-in Python interface backed by the installed Rust extension."""

import json

from bridge_date_parity import normalize_date as _normalize
from date_errors import DateParseError


def normalize_date(value: str) -> str:
    outcome = json.loads(_normalize(value, True))
    if "error" in outcome:
        raise DateParseError(outcome["error"]["code"])
    return outcome["ok"]
