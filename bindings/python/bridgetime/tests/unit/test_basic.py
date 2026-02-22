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
