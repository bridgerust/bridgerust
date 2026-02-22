from bridgetime import BridgeDuration, BridgeTime, supported_units


def test_supported_units():
    units = supported_units()
    assert "day" in units
    assert "month" in units


def test_parse_add_and_format():
    dt = BridgeTime.parse("2026-02-22T10:15:30Z", "UTC")
    assert dt.format("YYYY-MM-DD HH:mm:ss") == "2026-02-22 10:15:30"

    shifted = dt.add(1, "day")
    assert shifted.format("YYYY-MM-DD") == "2026-02-23"


def test_end_of_boundaries_are_correct():
    dt = BridgeTime.parse("2026-02-22T10:15:30Z", "UTC")

    end_day = dt.end_of("day")
    assert end_day.format("YYYY-MM-DD HH:mm:ss.SSS") == "2026-02-22 23:59:59.999"

    end_week = dt.end_of("week")
    assert end_week.format("YYYY-MM-DD HH:mm:ss.SSS") == "2026-02-28 23:59:59.999"

    end_month = dt.end_of("month")
    assert end_month.format("YYYY-MM-DD HH:mm:ss.SSS") == "2026-02-28 23:59:59.999"


def test_parse_format_supports_custom_patterns():
    dt = BridgeTime.parse_format("22/02/2026 10:15", "DD/MM/YYYY HH:mm", "UTC")
    assert dt.format("YYYY-MM-DD HH:mm:ss") == "2026-02-22 10:15:00"

    date_only = BridgeTime.parse_format("2026/02/22", "YYYY/MM/DD", "UTC")
    assert date_only.format("YYYY-MM-DD HH:mm:ss") == "2026-02-22 00:00:00"


def test_array_min_max_clamp_and_dst_helpers():
    dt = BridgeTime.from_array([2026, 1, 22, 10, 15, 30, 250], "UTC")
    assert dt.to_array() == [2026, 1, 22, 10, 15, 30, 250]

    overflow = BridgeTime.from_array([2026, 12, 1], "UTC")
    assert overflow.format("YYYY-MM-DD") == "2027-01-01"

    a = BridgeTime.parse("2026-02-22T10:00:00Z", "UTC")
    b = BridgeTime.parse("2026-02-22T11:00:00Z", "UTC")
    c = BridgeTime.parse("2026-02-22T12:00:00Z", "UTC")
    assert BridgeTime.min(a, b).unix_ms() == a.unix_ms()
    assert BridgeTime.max(a, b).unix_ms() == b.unix_ms()
    assert a.clamp(b, c).unix_ms() == b.unix_ms()
    assert c.clamp(a, b).unix_ms() == b.unix_ms()
    assert b.clamp(a, c).unix_ms() == b.unix_ms()

    ny_winter = BridgeTime.parse("2026-01-15T12:00:00Z", "America/New_York")
    ny_summer = BridgeTime.parse("2026-07-15T12:00:00Z", "America/New_York")
    assert ny_winter.is_dst() is False
    assert ny_summer.is_dst() is True


def test_timezone_conversion_preserves_instant():
    dt = BridgeTime.parse("2026-02-22T12:00:00Z", "UTC")
    ny = dt.to_timezone("America/New_York")
    assert dt.unix_ms() == ny.unix_ms()
    assert dt.utc_offset() == 0
    assert ny.utc_offset() == -300
    assert dt.is_utc() is True
    assert ny.is_utc() is False


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


def test_explicit_component_getters_and_setters():
    dt = BridgeTime.parse("2026-02-22T10:15:30.250Z", "UTC")

    assert dt.year() == 2026
    assert dt.month() == 1
    assert dt.date() == 22
    assert dt.day() == 0
    assert dt.hour() == 10
    assert dt.minute() == 15
    assert dt.second() == 30
    assert dt.millisecond() == 250

    shifted = (
        dt.set_year(2027)
        .set_month(3)
        .set_date(5)
        .set_hour(12)
        .set_minute(45)
        .set_second(5)
        .set_millisecond(900)
    )
    assert shifted.format("YYYY-MM-DD HH:mm:ss.SSS") == "2027-04-05 12:45:05.900"


def test_day_of_year_week_and_relative_day_helpers():
    dt = BridgeTime.parse("2026-02-22T10:15:30Z", "UTC")

    assert dt.day_of_year() == 53
    assert dt.quarter() == 1
    assert dt.iso_weekday() == 7
    assert dt.week_of_year() == 9
    assert dt.week() == dt.week_of_year()
    assert dt.iso_week() == 8
    assert dt.iso_week_year() == 2026
    assert dt.days_in_year() == 365
    assert dt.weeks_in_year() == 53
    assert dt.iso_weeks_in_year() == 53

    next_quarter = dt.set_quarter(2)
    assert next_quarter.format("YYYY-MM-DD") == "2026-05-22"

    monday = dt.set_iso_weekday(1)
    assert monday.format("YYYY-MM-DD") == "2026-02-16"

    next_day = dt.set_day_of_year(54)
    assert next_day.format("YYYY-MM-DD") == "2026-02-23"

    next_week = dt.set_week(dt.week() + 1)
    assert next_week.diff(dt, "day", True) == 7.0

    next_iso_week = dt.set_iso_week(dt.iso_week() + 1)
    assert next_iso_week.diff(dt, "day", True) == 7.0

    today = BridgeTime.now("UTC")
    assert today.is_today() is True
    assert today.add(1, "day").is_tomorrow() is True
    assert today.subtract(1, "day").is_yesterday() is True


def test_relative_time_helpers():
    base = BridgeTime.parse("2026-02-22T10:00:00Z", "UTC")
    future = BridgeTime.parse("2026-02-22T10:30:00Z", "UTC")
    past = BridgeTime.parse("2026-02-22T09:30:00Z", "UTC")

    assert future.from_time(base, False) == "in 30 minutes"
    assert past.from_time(base, False) == "30 minutes ago"
    assert future.from_time(base, True) == "30 minutes"
    assert base.to_time(future, False) == "in 30 minutes"

    now = BridgeTime.now("UTC")
    assert now.add(2, "day").from_now(False).startswith("in ")
    assert now.subtract(2, "day").to_now(False).startswith("in ")


def test_unit_aware_same_or_comparisons():
    morning = BridgeTime.parse("2026-02-22T10:15:30Z", "UTC")
    evening = BridgeTime.parse("2026-02-22T23:59:00Z", "UTC")
    next_day = BridgeTime.parse("2026-02-23T00:00:00Z", "UTC")

    assert morning.is_same_or_before_unit(evening, "day") is True
    assert evening.is_same_or_after_unit(morning, "day") is True
    assert next_day.is_same_or_before_unit(morning, "day") is False


def test_duration_helpers():
    duration = BridgeDuration(90, "minute")
    assert duration.as_milliseconds() == 5_400_000
    assert duration.as_hours() == 1.5
    assert duration.humanize(False) == "2 hours"
    assert duration.humanize(True) == "in 2 hours"

    thirty = BridgeDuration.from_minutes(30)
    sixty = duration.subtract(thirty)
    assert sixty.as_minutes() == 60.0
    assert thirty.negate().humanize(True) == "30 minutes ago"

    base = BridgeTime.parse("2026-02-22T10:00:00Z", "UTC")
    moved = base.add_duration(thirty)
    assert moved.format("YYYY-MM-DD HH:mm:ss") == "2026-02-22 10:30:00"
    back = moved.subtract_duration(thirty)
    assert back.unix_ms() == base.unix_ms()

    static_duration = BridgeTime.duration(2, "hour")
    assert static_duration.as_minutes() == 120.0
