"""Reference implementation for the Python workers in this example."""

import re
from datetime import datetime, timedelta, timezone

from date_errors import DateParseError


_DATE = re.compile(
    r"([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})"
    r"(?:(Z)|([+-])([0-9]{2}):([0-9]{2}))?"
)


def normalize_date(value: str) -> str:
    match = _DATE.fullmatch(value)
    if not match:
        raise DateParseError("INVALID_FORMAT")

    year, month, day, hour, minute, second = map(int, match.groups()[:6])
    sign, offset_hours, offset_minutes = match.groups()[7:]
    try:
        date = datetime(year, month, day, hour, minute, second)
        if sign:
            hours, minutes = int(offset_hours), int(offset_minutes)
            if hours > 23 or minutes > 59:
                raise ValueError("invalid offset")
            offset = timedelta(hours=hours, minutes=minutes)
            date = date.replace(tzinfo=timezone(-offset if sign == "-" else offset))
        else:
            date = date.replace(tzinfo=timezone.utc)
        utc = date.astimezone(timezone.utc)
    except (ValueError, OverflowError):
        raise DateParseError("INVALID_DATE") from None

    return utc.replace(tzinfo=None).isoformat(timespec="seconds") + "Z"
