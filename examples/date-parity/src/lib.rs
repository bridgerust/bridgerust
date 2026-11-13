use bridgerust::export;
use chrono::{Datelike, FixedOffset, NaiveDate};
use serde_json::json;

#[derive(Clone, Copy)]
enum DateError {
    InvalidFormat,
    InvalidDate,
    MissingOffset,
}

impl DateError {
    fn code(self) -> &'static str {
        match self {
            Self::InvalidFormat => "INVALID_FORMAT",
            Self::InvalidDate => "INVALID_DATE",
            Self::MissingOffset => "MISSING_OFFSET",
        }
    }
}

/// Normalize a date to UTC and encode the outcome for both language adapters.
#[export]
pub fn normalize_date(input: String, allow_missing_offset: bool) -> String {
    match normalize(&input, allow_missing_offset) {
        Ok(value) => json!({ "ok": value }).to_string(),
        Err(error) => json!({ "error": { "code": error.code() } }).to_string(),
    }
}

fn normalize(input: &str, allow_missing_offset: bool) -> Result<String, DateError> {
    let (local, suffix) = input
        .as_bytes()
        .split_at_checked(19)
        .ok_or(DateError::InvalidFormat)?;
    if [local[4], local[7], local[10], local[13], local[16]] != *b"--T::" {
        return Err(DateError::InvalidFormat);
    }
    let year = digits(&local[..4])?;
    let month = digits(&local[5..7])?;
    let day = digits(&local[8..10])?;
    let hour = digits(&local[11..13])?;
    let minute = digits(&local[14..16])?;
    let second = digits(&local[17..19])?;
    let offset = match suffix {
        [] => None,
        [b'Z'] => Some(0),
        [sign @ (b'+' | b'-'), h1, h2, b':', m1, m2] => {
            let hour = digits(&[*h1, *h2])?;
            let minute = digits(&[*m1, *m2])?;
            if hour > 23 || minute > 59 {
                return Err(DateError::InvalidDate);
            }
            let seconds = (hour * 3_600 + minute * 60) as i32;
            Some(if *sign == b'-' { -seconds } else { seconds })
        }
        _ => return Err(DateError::InvalidFormat),
    };

    if year == 0 {
        return Err(DateError::InvalidDate);
    }
    let local = NaiveDate::from_ymd_opt(year as i32, month, day)
        .and_then(|date| date.and_hms_opt(hour, minute, second))
        .ok_or(DateError::InvalidDate)?;
    let offset = match offset {
        Some(seconds) => seconds,
        None if allow_missing_offset => 0,
        None => return Err(DateError::MissingOffset),
    };
    let utc = FixedOffset::east_opt(offset)
        .and_then(|offset| local.and_local_timezone(offset).single())
        .ok_or(DateError::InvalidDate)?
        .naive_utc();
    if !(1..=9_999).contains(&utc.year()) {
        return Err(DateError::InvalidDate);
    }
    Ok(utc.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

fn digits(bytes: &[u8]) -> Result<u32, DateError> {
    bytes.iter().try_fold(0, |value, digit| {
        if digit.is_ascii_digit() {
            Ok(value * 10 + u32::from(digit - b'0'))
        } else {
            Err(DateError::InvalidFormat)
        }
    })
}

#[cfg(feature = "python")]
#[bridgerust::pyo3::pymodule(gil_used = false)]
fn bridge_date_parity(
    module: &bridgerust::pyo3::Bound<'_, bridgerust::pyo3::types::PyModule>,
) -> bridgerust::pyo3::PyResult<()> {
    use bridgerust::pyo3::types::PyModuleMethods;
    module.add_function(bridgerust::pyo3::wrap_pyfunction!(normalize_date, module)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(input: &str, allow_missing_offset: bool) -> serde_json::Value {
        serde_json::from_str(&normalize_date(input.into(), allow_missing_offset)).unwrap()
    }

    #[test]
    fn applies_offsets_across_calendar_boundaries() {
        assert_eq!(
            outcome("2024-03-01T00:15:00+01:00", false),
            json!({ "ok": "2024-02-29T23:15:00Z" })
        );
        assert_eq!(
            outcome("2024-12-31T23:59:59-00:01", false),
            json!({ "ok": "2025-01-01T00:00:59Z" })
        );
        assert_eq!(
            outcome("0001-01-01T00:00:00Z", false),
            json!({ "ok": "0001-01-01T00:00:00Z" })
        );
    }

    #[test]
    fn applies_the_legacy_offset_policy_after_date_validation() {
        assert_eq!(
            outcome("2024-02-29T03:04:05", true),
            json!({ "ok": "2024-02-29T03:04:05Z" })
        );
        assert_eq!(
            outcome("2024-02-29T03:04:05", false),
            json!({ "error": { "code": "MISSING_OFFSET" } })
        );
        assert_eq!(
            outcome("2023-02-29T03:04:05", false),
            json!({ "error": { "code": "INVALID_DATE" } })
        );
    }

    #[test]
    fn distinguishes_invalid_shape_from_out_of_range_dates() {
        for input in [
            "2024-01-01T00:00:00z",
            "2024-01-01T00:00:00.0Z",
            "2024-01-01T00:00:00+1234",
            "2024-01-01T00:00:00Z\n",
            "２０２４-01-01T00:00:00Z",
        ] {
            assert_eq!(
                outcome(input, true),
                json!({ "error": { "code": "INVALID_FORMAT" } }),
                "{input:?}"
            );
        }
        for input in [
            "0000-01-01T00:00:00Z",
            "0001-01-01T00:00:00+00:01",
            "9999-12-31T23:59:59-00:01",
            "1900-02-29T00:00:00Z",
            "2024-01-01T00:00:60Z",
            "2024-01-01T00:00:00+24:00",
            "2024-01-01T00:00:00+00:60",
        ] {
            assert_eq!(
                outcome(input, true),
                json!({ "error": { "code": "INVALID_DATE" } }),
                "{input:?}"
            );
        }
    }
}
