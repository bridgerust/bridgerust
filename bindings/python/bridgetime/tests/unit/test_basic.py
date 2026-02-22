from bridgetime import BridgeTime, supported_units


def test_supported_units():
    units = supported_units()
    assert "day" in units
    assert "month" in units


def test_parse_add_and_format():
    dt = BridgeTime.parse("2026-02-22T10:15:30Z", "UTC")
    assert dt.format("YYYY-MM-DD HH:mm:ss") == "2026-02-22 10:15:30"

    shifted = dt.add(1, "day")
    assert shifted.format("YYYY-MM-DD") == "2026-02-23"


def test_timezone_conversion_preserves_instant():
    dt = BridgeTime.parse("2026-02-22T12:00:00Z", "UTC")
    ny = dt.to_timezone("America/New_York")
    assert dt.unix_ms() == ny.unix_ms()


def test_get_set_and_calendar_helpers():
    dt = BridgeTime.parse("2026-02-22T10:15:30.250Z", "UTC")

    assert dt.get("month") == 1
    assert dt.get("date") == 22
    assert dt.get("day") == 0
    assert dt.get("quarter") == 1

    shifted_month = dt.set("month", 2)
    assert shifted_month.format("YYYY-MM-DD") == "2026-03-22"

    shifted_day = dt.set("day", 1)
    assert shifted_day.format("YYYY-MM-DD") == "2026-02-23"

    shifted_ms = dt.set("millisecond", 900)
    assert shifted_ms.format("SSS") == "900"

    assert dt.is_valid() is True
    assert dt.days_in_month() == 28
    assert dt.is_leap_year() is False


def test_unit_comparisons_and_between():
    morning = BridgeTime.parse("2026-02-22T10:15:30Z", "UTC")
    evening = BridgeTime.parse("2026-02-22T23:59:00Z", "UTC")
    next_day = BridgeTime.parse("2026-02-23T00:00:00Z", "UTC")

    assert morning.is_before(evening) is True
    assert morning.is_same_or_before(evening) is True
    assert evening.is_same_or_after(morning) is True

    assert morning.is_same_unit(evening, "day") is True
    assert evening.is_after_unit(morning, "day") is False
    assert morning.is_before_unit(next_day, "day") is True

    assert evening.is_between(morning, next_day, "day", "[)") is True
