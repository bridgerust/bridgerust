#[allow(unused_imports)]
use bridgerust::new;
use bridgerust::{bridge, error};
use chrono::{
    DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, Offset, SecondsFormat,
    TimeZone, Timelike, Utc,
};
use chrono_tz::{OffsetComponents, Tz};
use std::fmt::{Display, Formatter};

#[cfg(feature = "python")]
use pyo3::types::PyModuleMethods;

const AVG_DAYS_PER_MONTH: f64 = 30.436875;
const AVG_DAYS_PER_YEAR: f64 = 365.2425;
const MS_PER_SECOND: f64 = 1_000.0;
const MS_PER_MINUTE: f64 = 60_000.0;
const MS_PER_HOUR: f64 = 3_600_000.0;
const MS_PER_DAY: f64 = 86_400_000.0;
const MS_PER_WEEK: f64 = 604_800_000.0;

#[error]
#[derive(Debug, Clone)]
pub enum BridgeTimeError {
    InvalidTimezone(String),
    InvalidDateInput(String),
    InvalidTimestamp(i64),
    InvalidUnit(String),
    InvalidField(String),
    InvalidInclusivity(String),
    InvalidComponents(String),
    ArithmeticOverflow,
    NonexistentLocalTime(String, String),
}

impl Display for BridgeTimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTimezone(tz) => write!(f, "Invalid timezone: {tz}"),
            Self::InvalidDateInput(value) => write!(f, "Invalid date input: {value}"),
            Self::InvalidTimestamp(value) => write!(f, "Invalid timestamp (ms): {value}"),
            Self::InvalidUnit(unit) => write!(
                f,
                "Invalid unit: {unit}. Use one of: millisecond, second, minute, hour, day, week, month, quarter, year"
            ),
            Self::InvalidField(field) => write!(
                f,
                "Invalid field: {field}. Use one of: millisecond, second, minute, hour, day, date, month, quarter, year"
            ),
            Self::InvalidInclusivity(value) => write!(
                f,
                "Invalid inclusivity: {value}. Use one of: (), (], [), []"
            ),
            Self::InvalidComponents(value) => write!(
                f,
                "Invalid components: {value}. Use [year, month0, date, hour, minute, second, millisecond]"
            ),
            Self::ArithmeticOverflow => write!(f, "Date arithmetic overflow"),
            Self::NonexistentLocalTime(local, tz) => {
                write!(f, "Local time {local} does not exist in timezone {tz}")
            }
        }
    }
}

impl std::error::Error for BridgeTimeError {}

#[cfg(feature = "nodejs")]
impl From<BridgeTimeError> for napi::Error {
    fn from(err: BridgeTimeError) -> Self {
        napi::Error::from_reason(err.to_string())
    }
}

#[cfg(feature = "nodejs")]
impl From<BridgeTimeError> for napi::bindgen_prelude::JsError {
    fn from(err: BridgeTimeError) -> Self {
        napi::bindgen_prelude::JsError::from(napi::Error::from(err))
    }
}

#[cfg(feature = "python")]
impl From<BridgeTimeError> for pyo3::PyErr {
    fn from(err: BridgeTimeError) -> Self {
        pyo3::exceptions::PyValueError::new_err(err.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeUnit {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl TimeUnit {
    fn parse(raw: &str) -> Result<Self, BridgeTimeError> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "ms" | "millisecond" | "milliseconds" => Ok(Self::Millisecond),
            "s" | "sec" | "second" | "seconds" => Ok(Self::Second),
            "m" | "min" | "minute" | "minutes" => Ok(Self::Minute),
            "h" | "hour" | "hours" => Ok(Self::Hour),
            "d" | "day" | "days" => Ok(Self::Day),
            "w" | "week" | "weeks" => Ok(Self::Week),
            "mo" | "month" | "months" => Ok(Self::Month),
            "q" | "quarter" | "quarters" => Ok(Self::Quarter),
            "y" | "year" | "years" => Ok(Self::Year),
            _ => Err(BridgeTimeError::InvalidUnit(raw.to_string())),
        }
    }

    fn next(self) -> Option<Self> {
        match self {
            Self::Millisecond => Some(Self::Second),
            Self::Second => Some(Self::Minute),
            Self::Minute => Some(Self::Hour),
            Self::Hour => Some(Self::Day),
            Self::Day => Some(Self::Week),
            Self::Week => Some(Self::Month),
            Self::Month => Some(Self::Quarter),
            Self::Quarter => Some(Self::Year),
            Self::Year => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CalendarField {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Date,
    Month,
    Quarter,
    Year,
}

impl CalendarField {
    fn parse(raw: &str) -> Result<Self, BridgeTimeError> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "ms" | "millisecond" | "milliseconds" => Ok(Self::Millisecond),
            "s" | "sec" | "second" | "seconds" => Ok(Self::Second),
            "m" | "min" | "minute" | "minutes" => Ok(Self::Minute),
            "h" | "hour" | "hours" => Ok(Self::Hour),
            "d" | "day" | "days" | "weekday" | "week_day" | "dayofweek" | "day_of_week" => {
                Ok(Self::Day)
            }
            "date" | "dates" | "dayofmonth" | "day_of_month" => Ok(Self::Date),
            "mo" | "month" | "months" => Ok(Self::Month),
            "q" | "quarter" | "quarters" => Ok(Self::Quarter),
            "y" | "year" | "years" => Ok(Self::Year),
            _ => Err(BridgeTimeError::InvalidField(raw.to_string())),
        }
    }
}

fn parse_inclusivity(inclusivity: Option<String>) -> Result<(bool, bool), BridgeTimeError> {
    let resolved = inclusivity.unwrap_or_else(|| "()".to_string());
    let chars: Vec<char> = resolved.chars().collect();
    if chars.len() != 2 {
        return Err(BridgeTimeError::InvalidInclusivity(resolved));
    }

    let start_inclusive = match chars[0] {
        '[' => true,
        '(' => false,
        _ => return Err(BridgeTimeError::InvalidInclusivity(resolved)),
    };

    let end_inclusive = match chars[1] {
        ']' => true,
        ')' => false,
        _ => return Err(BridgeTimeError::InvalidInclusivity(resolved)),
    };

    Ok((start_inclusive, end_inclusive))
}

fn sunday_week_of_year(date: NaiveDate) -> Result<u32, BridgeTimeError> {
    let first_day =
        NaiveDate::from_ymd_opt(date.year(), 1, 1).ok_or(BridgeTimeError::ArithmeticOverflow)?;
    let first_day_offset = i64::from(first_day.weekday().num_days_from_sunday());
    let ordinal0 = i64::from(date.ordinal0());
    let aligned = ordinal0
        .checked_add(first_day_offset)
        .ok_or(BridgeTimeError::ArithmeticOverflow)?;
    let week = aligned
        .checked_div(7)
        .and_then(|value| value.checked_add(1))
        .ok_or(BridgeTimeError::ArithmeticOverflow)?;
    u32::try_from(week).map_err(|_| BridgeTimeError::ArithmeticOverflow)
}

fn format_relative_phrase(diff_millis: i64, without_suffix: bool) -> String {
    let is_future = diff_millis > 0;
    let seconds = (diff_millis.unsigned_abs() as f64) / 1_000.0;

    let phrase = if seconds < 45.0 {
        "a few seconds".to_string()
    } else if seconds < 90.0 {
        "a minute".to_string()
    } else if seconds < 45.0 * 60.0 {
        format!("{} minutes", (seconds / 60.0).round() as i64)
    } else if seconds < 90.0 * 60.0 {
        "an hour".to_string()
    } else if seconds < 22.0 * 3_600.0 {
        format!("{} hours", (seconds / 3_600.0).round() as i64)
    } else if seconds < 36.0 * 3_600.0 {
        "a day".to_string()
    } else if seconds < 26.0 * 86_400.0 {
        format!("{} days", (seconds / 86_400.0).round() as i64)
    } else if seconds < 45.0 * 86_400.0 {
        "a month".to_string()
    } else if seconds < 320.0 * 86_400.0 {
        format!("{} months", (seconds / (30.0 * 86_400.0)).round() as i64)
    } else if seconds < 548.0 * 86_400.0 {
        "a year".to_string()
    } else {
        format!("{} years", (seconds / (365.0 * 86_400.0)).round() as i64)
    };

    if without_suffix {
        phrase
    } else if is_future {
        format!("in {phrase}")
    } else {
        format!("{phrase} ago")
    }
}

fn duration_to_millis(amount: i64, unit: TimeUnit) -> Result<i64, BridgeTimeError> {
    match unit {
        TimeUnit::Millisecond => Ok(amount),
        TimeUnit::Second => amount
            .checked_mul(1_000)
            .ok_or(BridgeTimeError::ArithmeticOverflow),
        TimeUnit::Minute => amount
            .checked_mul(60_000)
            .ok_or(BridgeTimeError::ArithmeticOverflow),
        TimeUnit::Hour => amount
            .checked_mul(3_600_000)
            .ok_or(BridgeTimeError::ArithmeticOverflow),
        TimeUnit::Day => amount
            .checked_mul(86_400_000)
            .ok_or(BridgeTimeError::ArithmeticOverflow),
        TimeUnit::Week => amount
            .checked_mul(604_800_000)
            .ok_or(BridgeTimeError::ArithmeticOverflow),
        TimeUnit::Month | TimeUnit::Quarter | TimeUnit::Year => {
            let factor = match unit {
                TimeUnit::Month => AVG_DAYS_PER_MONTH * MS_PER_DAY,
                TimeUnit::Quarter => AVG_DAYS_PER_MONTH * 3.0 * MS_PER_DAY,
                TimeUnit::Year => AVG_DAYS_PER_YEAR * MS_PER_DAY,
                _ => unreachable!(),
            };
            let value = (amount as f64) * factor;
            if !value.is_finite() || value.abs() > i64::MAX as f64 {
                return Err(BridgeTimeError::ArithmeticOverflow);
            }
            Ok(value.round() as i64)
        }
    }
}

fn parse_timezone(raw: &str) -> Result<Tz, BridgeTimeError> {
    let trimmed = raw.trim();
    let normalized = if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("utc")
        || trimmed.eq_ignore_ascii_case("z")
    {
        "UTC"
    } else {
        trimmed
    };

    normalized
        .parse::<Tz>()
        .map_err(|_| BridgeTimeError::InvalidTimezone(raw.to_string()))
}

fn resolve_timezone(timezone: Option<String>) -> Result<Tz, BridgeTimeError> {
    match timezone {
        Some(value) => parse_timezone(&value),
        None => Ok(chrono_tz::UTC),
    }
}

fn utc_from_millis(millis: i64) -> Result<DateTime<Utc>, BridgeTimeError> {
    DateTime::<Utc>::from_timestamp_millis(millis).ok_or(BridgeTimeError::InvalidTimestamp(millis))
}

fn resolve_local_datetime(tz: Tz, naive: NaiveDateTime) -> Result<DateTime<Tz>, BridgeTimeError> {
    match tz.from_local_datetime(&naive) {
        LocalResult::Single(value) => Ok(value),
        LocalResult::Ambiguous(earliest, _) => Ok(earliest),
        LocalResult::None => {
            let mut probe = naive;
            for _ in 0..180 {
                probe = probe
                    .checked_add_signed(Duration::minutes(1))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                match tz.from_local_datetime(&probe) {
                    LocalResult::Single(value) => return Ok(value),
                    LocalResult::Ambiguous(earliest, _) => return Ok(earliest),
                    LocalResult::None => {}
                }
            }
            Err(BridgeTimeError::NonexistentLocalTime(
                naive.to_string(),
                tz.name().to_string(),
            ))
        }
    }
}

fn build_naive_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
) -> Result<NaiveDateTime, BridgeTimeError> {
    let date =
        NaiveDate::from_ymd_opt(year, month, day).ok_or(BridgeTimeError::ArithmeticOverflow)?;
    date.and_hms_milli_opt(hour, minute, second, millisecond)
        .ok_or(BridgeTimeError::ArithmeticOverflow)
}

fn last_day_of_month(year: i32, month: u32) -> Result<u32, BridgeTimeError> {
    let first_of_month =
        NaiveDate::from_ymd_opt(year, month, 1).ok_or(BridgeTimeError::ArithmeticOverflow)?;

    let next_month = if month == 12 {
        let next_year = year
            .checked_add(1)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        NaiveDate::from_ymd_opt(next_year, 1, 1).ok_or(BridgeTimeError::ArithmeticOverflow)?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).ok_or(BridgeTimeError::ArithmeticOverflow)?
    };

    let days = next_month.signed_duration_since(first_of_month).num_days();
    u32::try_from(days).map_err(|_| BridgeTimeError::ArithmeticOverflow)
}

fn shift_local_months(local: DateTime<Tz>, months: i32) -> Result<DateTime<Tz>, BridgeTimeError> {
    let tz = local.timezone();
    let naive = local.naive_local();

    let base_months = naive
        .year()
        .checked_mul(12)
        .and_then(|value| value.checked_add(naive.month0() as i32))
        .ok_or(BridgeTimeError::ArithmeticOverflow)?;

    let total_months = base_months
        .checked_add(months)
        .ok_or(BridgeTimeError::ArithmeticOverflow)?;

    let year = total_months.div_euclid(12);
    let month0 = total_months.rem_euclid(12);
    let month = u32::try_from(month0 + 1).map_err(|_| BridgeTimeError::ArithmeticOverflow)?;

    let day = naive.day().min(last_day_of_month(year, month)?);
    let millisecond = naive.and_utc().timestamp_subsec_millis();
    let target_naive = build_naive_datetime(
        year,
        month,
        day,
        naive.hour(),
        naive.minute(),
        naive.second(),
        millisecond,
    )?;

    resolve_local_datetime(tz, target_naive)
}

fn convert_dayjs_pattern(pattern: &str) -> String {
    let mut output = pattern.to_string();
    let replacements = [
        ("YYYY", "%Y"),
        ("YY", "%y"),
        ("MMMM", "%B"),
        ("MMM", "%b"),
        ("MM", "%m"),
        ("DD", "%d"),
        ("HH", "%H"),
        ("hh", "%I"),
        ("mm", "%M"),
        ("ss", "%S"),
        ("SSS", "%3f"),
        ("dddd", "%A"),
        ("ddd", "%a"),
        ("ZZ", "%z"),
        ("Z", "%:z"),
        ("A", "%p"),
        ("a", "%P"),
    ];

    for (from, to) in replacements {
        output = output.replace(from, to);
    }

    output
}

fn parse_datetime_input(input: &str, tz: Tz) -> Result<i64, BridgeTimeError> {
    if let Ok(value) = DateTime::parse_from_rfc3339(input) {
        return Ok(value.timestamp_millis());
    }

    let naive_formats = [
        "%Y-%m-%dT%H:%M:%S%.3f",
        "%Y-%m-%d %H:%M:%S%.3f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M",
    ];

    for fmt in naive_formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(input, fmt) {
            let local = resolve_local_datetime(tz, naive)?;
            return Ok(local.with_timezone(&Utc).timestamp_millis());
        }
    }

    if let Ok(date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        let naive = date
            .and_hms_milli_opt(0, 0, 0, 0)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        let local = resolve_local_datetime(tz, naive)?;
        return Ok(local.with_timezone(&Utc).timestamp_millis());
    }

    Err(BridgeTimeError::InvalidDateInput(input.to_string()))
}

fn parse_datetime_with_format(input: &str, pattern: &str, tz: Tz) -> Result<i64, BridgeTimeError> {
    let chrono_pattern = convert_dayjs_pattern(pattern);

    if let Ok(value) = DateTime::parse_from_str(input, &chrono_pattern) {
        return Ok(value.timestamp_millis());
    }

    if let Ok(naive) = NaiveDateTime::parse_from_str(input, &chrono_pattern) {
        let local = resolve_local_datetime(tz, naive)?;
        return Ok(local.with_timezone(&Utc).timestamp_millis());
    }

    if let Ok(date) = NaiveDate::parse_from_str(input, &chrono_pattern) {
        let naive = date
            .and_hms_milli_opt(0, 0, 0, 0)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        let local = resolve_local_datetime(tz, naive)?;
        return Ok(local.with_timezone(&Utc).timestamp_millis());
    }

    Err(BridgeTimeError::InvalidDateInput(format!(
        "{input} (format: {pattern})"
    )))
}

#[bridge]
pub fn supported_units() -> Vec<String> {
    vec![
        "millisecond".to_string(),
        "second".to_string(),
        "minute".to_string(),
        "hour".to_string(),
        "day".to_string(),
        "week".to_string(),
        "month".to_string(),
        "quarter".to_string(),
        "year".to_string(),
    ]
}

#[bridge]
pub struct BridgeDuration {
    milliseconds: i64,
}

#[bridge]
impl BridgeDuration {
    #[constructor]
    pub fn new(value: i64, unit: Option<String>) -> Result<Self, BridgeTimeError> {
        let millis = match unit {
            Some(raw_unit) => {
                let parsed_unit = TimeUnit::parse(&raw_unit)?;
                duration_to_millis(value, parsed_unit)?
            }
            None => value,
        };
        Ok(Self {
            milliseconds: millis,
        })
    }

    pub fn from_milliseconds(value: i64) -> Self {
        Self {
            milliseconds: value,
        }
    }

    pub fn from_seconds(value: i64) -> Result<Self, BridgeTimeError> {
        Self::new(value, Some("second".to_string()))
    }

    pub fn from_minutes(value: i64) -> Result<Self, BridgeTimeError> {
        Self::new(value, Some("minute".to_string()))
    }

    pub fn from_hours(value: i64) -> Result<Self, BridgeTimeError> {
        Self::new(value, Some("hour".to_string()))
    }

    pub fn from_days(value: i64) -> Result<Self, BridgeTimeError> {
        Self::new(value, Some("day".to_string()))
    }

    pub fn from_weeks(value: i64) -> Result<Self, BridgeTimeError> {
        Self::new(value, Some("week".to_string()))
    }

    pub fn from_months(value: i64) -> Result<Self, BridgeTimeError> {
        Self::new(value, Some("month".to_string()))
    }

    pub fn from_years(value: i64) -> Result<Self, BridgeTimeError> {
        Self::new(value, Some("year".to_string()))
    }

    pub fn as_milliseconds(&self) -> i64 {
        self.milliseconds
    }

    pub fn as_seconds(&self) -> f64 {
        self.milliseconds as f64 / MS_PER_SECOND
    }

    pub fn as_minutes(&self) -> f64 {
        self.milliseconds as f64 / MS_PER_MINUTE
    }

    pub fn as_hours(&self) -> f64 {
        self.milliseconds as f64 / MS_PER_HOUR
    }

    pub fn as_days(&self) -> f64 {
        self.milliseconds as f64 / MS_PER_DAY
    }

    pub fn as_weeks(&self) -> f64 {
        self.milliseconds as f64 / MS_PER_WEEK
    }

    pub fn as_months(&self) -> f64 {
        self.milliseconds as f64 / (AVG_DAYS_PER_MONTH * MS_PER_DAY)
    }

    pub fn as_years(&self) -> f64 {
        self.milliseconds as f64 / (AVG_DAYS_PER_YEAR * MS_PER_DAY)
    }

    pub fn humanize(&self, with_suffix: Option<bool>) -> String {
        format_relative_phrase(self.milliseconds, !with_suffix.unwrap_or(false))
    }

    pub fn add(&self, other: &BridgeDuration) -> Result<Self, BridgeTimeError> {
        Ok(Self {
            milliseconds: self
                .milliseconds
                .checked_add(other.milliseconds)
                .ok_or(BridgeTimeError::ArithmeticOverflow)?,
        })
    }

    pub fn subtract(&self, other: &BridgeDuration) -> Result<Self, BridgeTimeError> {
        Ok(Self {
            milliseconds: self
                .milliseconds
                .checked_sub(other.milliseconds)
                .ok_or(BridgeTimeError::ArithmeticOverflow)?,
        })
    }

    pub fn negate(&self) -> Result<Self, BridgeTimeError> {
        Ok(Self {
            milliseconds: self
                .milliseconds
                .checked_neg()
                .ok_or(BridgeTimeError::ArithmeticOverflow)?,
        })
    }

    pub fn abs(&self) -> Result<Self, BridgeTimeError> {
        if self.milliseconds >= 0 {
            return Ok(Self {
                milliseconds: self.milliseconds,
            });
        }
        self.negate()
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("BridgeDuration(milliseconds={})", self.milliseconds)
    }
}

#[bridge]
pub struct BridgeTime {
    utc_millis: i64,
    timezone: String,
}

#[bridge]
impl BridgeTime {
    #[constructor]
    pub fn new(input: Option<String>, timezone: Option<String>) -> Result<Self, BridgeTimeError> {
        match input {
            Some(value) => Self::parse(value, timezone),
            None => Self::now(timezone),
        }
    }

    pub fn now(timezone: Option<String>) -> Result<Self, BridgeTimeError> {
        let tz = resolve_timezone(timezone)?;
        Ok(Self {
            utc_millis: Utc::now().timestamp_millis(),
            timezone: tz.name().to_string(),
        })
    }

    pub fn parse(input: String, timezone: Option<String>) -> Result<Self, BridgeTimeError> {
        let tz = resolve_timezone(timezone)?;
        let utc_millis = parse_datetime_input(&input, tz)?;
        Ok(Self {
            utc_millis,
            timezone: tz.name().to_string(),
        })
    }

    pub fn parse_format(
        input: String,
        pattern: String,
        timezone: Option<String>,
    ) -> Result<Self, BridgeTimeError> {
        let tz = resolve_timezone(timezone)?;
        let utc_millis = parse_datetime_with_format(&input, &pattern, tz)?;
        Ok(Self {
            utc_millis,
            timezone: tz.name().to_string(),
        })
    }

    pub fn from_array(
        components: Vec<i64>,
        timezone: Option<String>,
    ) -> Result<Self, BridgeTimeError> {
        if components.is_empty() || components.len() > 7 {
            return Err(BridgeTimeError::InvalidComponents(format!(
                "expected 1..=7 items, got {}",
                components.len()
            )));
        }

        let tz = resolve_timezone(timezone)?;

        let year = i32::try_from(components[0]).map_err(|_| BridgeTimeError::ArithmeticOverflow)?;
        let month0 = *components.get(1).unwrap_or(&0);
        let date = *components.get(2).unwrap_or(&1);
        let hour = *components.get(3).unwrap_or(&0);
        let minute = *components.get(4).unwrap_or(&0);
        let second = *components.get(5).unwrap_or(&0);
        let millisecond = *components.get(6).unwrap_or(&0);

        let total_month = i64::from(year)
            .checked_mul(12)
            .and_then(|value| value.checked_add(month0))
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        let normalized_year = i32::try_from(total_month.div_euclid(12))
            .map_err(|_| BridgeTimeError::ArithmeticOverflow)?;
        let normalized_month = u32::try_from(total_month.rem_euclid(12) + 1)
            .map_err(|_| BridgeTimeError::ArithmeticOverflow)?;

        let mut naive = build_naive_datetime(normalized_year, normalized_month, 1, 0, 0, 0, 0)?;
        let adjustments = [
            Duration::days(
                date.checked_sub(1)
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?,
            ),
            Duration::hours(hour),
            Duration::minutes(minute),
            Duration::seconds(second),
            Duration::milliseconds(millisecond),
        ];

        for adjustment in adjustments {
            naive = naive
                .checked_add_signed(adjustment)
                .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        }

        let local = resolve_local_datetime(tz, naive)?;
        Ok(Self::from_local_datetime(local))
    }

    pub fn from_unix_ms(unix_ms: i64, timezone: Option<String>) -> Result<Self, BridgeTimeError> {
        let tz = resolve_timezone(timezone)?;
        utc_from_millis(unix_ms)?;
        Ok(Self {
            utc_millis: unix_ms,
            timezone: tz.name().to_string(),
        })
    }

    pub fn from_unix(unix_seconds: i64, timezone: Option<String>) -> Result<Self, BridgeTimeError> {
        let unix_ms = unix_seconds
            .checked_mul(1_000)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        Self::from_unix_ms(unix_ms, timezone)
    }

    pub fn duration(value: i64, unit: Option<String>) -> Result<BridgeDuration, BridgeTimeError> {
        BridgeDuration::new(value, unit)
    }

    pub fn to_iso(&self) -> Result<String, BridgeTimeError> {
        let local = self.local_datetime()?;
        Ok(local.to_rfc3339_opts(SecondsFormat::Millis, true))
    }

    pub fn format(&self, pattern: String) -> Result<String, BridgeTimeError> {
        let local = self.local_datetime()?;
        let chrono_pattern = convert_dayjs_pattern(&pattern);
        Ok(local.format(&chrono_pattern).to_string())
    }

    pub fn unix_ms(&self) -> i64 {
        self.utc_millis
    }

    pub fn unix(&self) -> i64 {
        self.utc_millis / 1_000
    }

    pub fn value_of(&self) -> i64 {
        self.utc_millis
    }

    pub fn timezone(&self) -> String {
        self.timezone.clone()
    }

    pub fn to_array(&self) -> Result<Vec<i64>, BridgeTimeError> {
        let local = self.local_datetime()?;
        let naive = local.naive_local();
        Ok(vec![
            i64::from(naive.year()),
            i64::from(naive.month0()),
            i64::from(naive.day()),
            i64::from(naive.hour()),
            i64::from(naive.minute()),
            i64::from(naive.second()),
            i64::from(naive.and_utc().timestamp_subsec_millis()),
        ])
    }

    pub fn utc_offset(&self) -> Result<i32, BridgeTimeError> {
        let local = self.local_datetime()?;
        Ok(local.offset().fix().local_minus_utc() / 60)
    }

    pub fn is_utc(&self) -> Result<bool, BridgeTimeError> {
        let tz = self.as_tz()?;
        Ok(tz.name() == "UTC")
    }

    pub fn is_dst(&self) -> Result<bool, BridgeTimeError> {
        let local = self.local_datetime()?;
        Ok(local.offset().dst_offset() != Duration::zero())
    }

    pub fn to_timezone(&self, timezone: String) -> Result<Self, BridgeTimeError> {
        let tz = parse_timezone(&timezone)?;
        Ok(Self {
            utc_millis: self.utc_millis,
            timezone: tz.name().to_string(),
        })
    }

    pub fn add(&self, amount: i64, unit: String) -> Result<Self, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        self.add_unit(amount, parsed_unit)
    }

    pub fn add_duration(&self, duration: &BridgeDuration) -> Result<Self, BridgeTimeError> {
        self.add_unit(duration.milliseconds, TimeUnit::Millisecond)
    }

    pub fn subtract(&self, amount: i64, unit: String) -> Result<Self, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        let negated = amount
            .checked_neg()
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        self.add_unit(negated, parsed_unit)
    }

    pub fn subtract_duration(&self, duration: &BridgeDuration) -> Result<Self, BridgeTimeError> {
        let negated = duration
            .milliseconds
            .checked_neg()
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        self.add_unit(negated, TimeUnit::Millisecond)
    }

    pub fn start_of(&self, unit: String) -> Result<Self, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        self.start_of_unit(parsed_unit)
    }

    pub fn end_of(&self, unit: String) -> Result<Self, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        self.end_of_unit(parsed_unit)
    }

    pub fn diff(
        &self,
        other: &BridgeTime,
        unit: String,
        as_float: Option<bool>,
    ) -> Result<f64, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        let raw_diff = self.utc_millis.saturating_sub(other.utc_millis) as f64;

        let value = match parsed_unit {
            TimeUnit::Millisecond => raw_diff,
            TimeUnit::Second => raw_diff / MS_PER_SECOND,
            TimeUnit::Minute => raw_diff / MS_PER_MINUTE,
            TimeUnit::Hour => raw_diff / MS_PER_HOUR,
            TimeUnit::Day => raw_diff / MS_PER_DAY,
            TimeUnit::Week => raw_diff / MS_PER_WEEK,
            TimeUnit::Month => raw_diff / (AVG_DAYS_PER_MONTH * MS_PER_DAY),
            TimeUnit::Quarter => raw_diff / ((AVG_DAYS_PER_MONTH * 3.0) * MS_PER_DAY),
            TimeUnit::Year => raw_diff / (AVG_DAYS_PER_YEAR * MS_PER_DAY),
        };

        if as_float.unwrap_or(false) {
            Ok(value)
        } else {
            Ok(value.trunc())
        }
    }

    pub fn is_valid(&self) -> bool {
        self.as_utc_datetime().is_ok() && self.as_tz().is_ok()
    }

    pub fn days_in_month(&self) -> Result<u32, BridgeTimeError> {
        let local = self.local_datetime()?;
        last_day_of_month(local.year(), local.month())
    }

    pub fn is_leap_year(&self) -> Result<bool, BridgeTimeError> {
        let local = self.local_datetime()?;
        Ok(NaiveDate::from_ymd_opt(local.year(), 2, 29).is_some())
    }

    pub fn get(&self, field: String) -> Result<i64, BridgeTimeError> {
        let parsed_field = CalendarField::parse(&field)?;
        self.get_field(parsed_field)
    }

    pub fn set(&self, field: String, value: i64) -> Result<Self, BridgeTimeError> {
        let parsed_field = CalendarField::parse(&field)?;
        self.set_field(parsed_field, value)
    }

    pub fn year(&self) -> Result<i64, BridgeTimeError> {
        self.get_field(CalendarField::Year)
    }

    pub fn month(&self) -> Result<i64, BridgeTimeError> {
        self.get_field(CalendarField::Month)
    }

    pub fn date(&self) -> Result<i64, BridgeTimeError> {
        self.get_field(CalendarField::Date)
    }

    pub fn day(&self) -> Result<i64, BridgeTimeError> {
        self.get_field(CalendarField::Day)
    }

    pub fn hour(&self) -> Result<i64, BridgeTimeError> {
        self.get_field(CalendarField::Hour)
    }

    pub fn minute(&self) -> Result<i64, BridgeTimeError> {
        self.get_field(CalendarField::Minute)
    }

    pub fn second(&self) -> Result<i64, BridgeTimeError> {
        self.get_field(CalendarField::Second)
    }

    pub fn millisecond(&self) -> Result<i64, BridgeTimeError> {
        self.get_field(CalendarField::Millisecond)
    }

    pub fn set_year(&self, value: i64) -> Result<Self, BridgeTimeError> {
        self.set_field(CalendarField::Year, value)
    }

    pub fn set_month(&self, value: i64) -> Result<Self, BridgeTimeError> {
        self.set_field(CalendarField::Month, value)
    }

    pub fn set_date(&self, value: i64) -> Result<Self, BridgeTimeError> {
        self.set_field(CalendarField::Date, value)
    }

    pub fn set_day(&self, value: i64) -> Result<Self, BridgeTimeError> {
        self.set_field(CalendarField::Day, value)
    }

    pub fn set_hour(&self, value: i64) -> Result<Self, BridgeTimeError> {
        self.set_field(CalendarField::Hour, value)
    }

    pub fn set_minute(&self, value: i64) -> Result<Self, BridgeTimeError> {
        self.set_field(CalendarField::Minute, value)
    }

    pub fn set_second(&self, value: i64) -> Result<Self, BridgeTimeError> {
        self.set_field(CalendarField::Second, value)
    }

    pub fn set_millisecond(&self, value: i64) -> Result<Self, BridgeTimeError> {
        self.set_field(CalendarField::Millisecond, value)
    }

    pub fn day_of_year(&self) -> Result<u32, BridgeTimeError> {
        let local = self.local_datetime()?;
        Ok(local.ordinal())
    }

    pub fn quarter(&self) -> Result<u32, BridgeTimeError> {
        let month0 = self.month()?;
        let month0_u32 = u32::try_from(month0).map_err(|_| BridgeTimeError::ArithmeticOverflow)?;
        Ok((month0_u32 / 3) + 1)
    }

    pub fn set_quarter(&self, value: i64) -> Result<Self, BridgeTimeError> {
        self.set_field(CalendarField::Quarter, value)
    }

    pub fn iso_weekday(&self) -> Result<u32, BridgeTimeError> {
        let local = self.local_datetime()?;
        Ok(local.weekday().number_from_monday())
    }

    pub fn set_iso_weekday(&self, value: i64) -> Result<Self, BridgeTimeError> {
        let local = self.local_datetime()?;
        let current = i64::from(local.weekday().number_from_monday());
        let delta = value
            .checked_sub(current)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        self.shift_local_by(Duration::days(delta))
    }

    pub fn set_day_of_year(&self, value: i64) -> Result<Self, BridgeTimeError> {
        let current = i64::from(self.day_of_year()?);
        let delta = value
            .checked_sub(current)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        self.shift_local_by(Duration::days(delta))
    }

    pub fn iso_week(&self) -> Result<u32, BridgeTimeError> {
        let local = self.local_datetime()?;
        Ok(local.iso_week().week())
    }

    pub fn set_iso_week(&self, value: i64) -> Result<Self, BridgeTimeError> {
        let current = i64::from(self.iso_week()?);
        let delta = value
            .checked_sub(current)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        self.shift_local_by(Duration::weeks(delta))
    }

    pub fn iso_week_year(&self) -> Result<i64, BridgeTimeError> {
        let local = self.local_datetime()?;
        Ok(i64::from(local.iso_week().year()))
    }

    pub fn week_of_year(&self) -> Result<u32, BridgeTimeError> {
        let local = self.local_datetime()?;
        sunday_week_of_year(local.date_naive())
    }

    pub fn week(&self) -> Result<u32, BridgeTimeError> {
        self.week_of_year()
    }

    pub fn weeks_in_year(&self) -> Result<u32, BridgeTimeError> {
        let year = i32::try_from(self.year()?).map_err(|_| BridgeTimeError::ArithmeticOverflow)?;
        let last_day =
            NaiveDate::from_ymd_opt(year, 12, 31).ok_or(BridgeTimeError::ArithmeticOverflow)?;
        sunday_week_of_year(last_day)
    }

    pub fn iso_weeks_in_year(&self) -> Result<u32, BridgeTimeError> {
        let year = i32::try_from(self.year()?).map_err(|_| BridgeTimeError::ArithmeticOverflow)?;
        let dec_28 =
            NaiveDate::from_ymd_opt(year, 12, 28).ok_or(BridgeTimeError::ArithmeticOverflow)?;
        Ok(dec_28.iso_week().week())
    }

    pub fn days_in_year(&self) -> Result<u32, BridgeTimeError> {
        if self.is_leap_year()? {
            Ok(366)
        } else {
            Ok(365)
        }
    }

    pub fn set_week(&self, value: i64) -> Result<Self, BridgeTimeError> {
        let current = i64::from(self.week_of_year()?);
        let delta = value
            .checked_sub(current)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        self.shift_local_by(Duration::weeks(delta))
    }

    pub fn is_today(&self) -> Result<bool, BridgeTimeError> {
        self.matches_relative_day(0)
    }

    pub fn is_yesterday(&self) -> Result<bool, BridgeTimeError> {
        self.matches_relative_day(-1)
    }

    pub fn is_tomorrow(&self) -> Result<bool, BridgeTimeError> {
        self.matches_relative_day(1)
    }

    pub fn is_before(&self, other: &BridgeTime) -> bool {
        self.utc_millis < other.utc_millis
    }

    pub fn is_after(&self, other: &BridgeTime) -> bool {
        self.utc_millis > other.utc_millis
    }

    pub fn is_same(&self, other: &BridgeTime) -> bool {
        self.utc_millis == other.utc_millis
    }

    pub fn is_before_unit(
        &self,
        other: &BridgeTime,
        unit: String,
    ) -> Result<bool, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        let lhs = self.start_of_unit(parsed_unit)?;
        let rhs = other.start_of_unit(parsed_unit)?;
        Ok(lhs.utc_millis < rhs.utc_millis)
    }

    pub fn is_after_unit(&self, other: &BridgeTime, unit: String) -> Result<bool, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        let lhs = self.start_of_unit(parsed_unit)?;
        let rhs = other.start_of_unit(parsed_unit)?;
        Ok(lhs.utc_millis > rhs.utc_millis)
    }

    pub fn is_same_unit(&self, other: &BridgeTime, unit: String) -> Result<bool, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        let lhs = self.start_of_unit(parsed_unit)?;
        let rhs = other.start_of_unit(parsed_unit)?;
        Ok(lhs.utc_millis == rhs.utc_millis)
    }

    pub fn is_same_or_before(&self, other: &BridgeTime) -> bool {
        self.utc_millis <= other.utc_millis
    }

    pub fn is_same_or_after(&self, other: &BridgeTime) -> bool {
        self.utc_millis >= other.utc_millis
    }

    pub fn is_same_or_before_unit(
        &self,
        other: &BridgeTime,
        unit: String,
    ) -> Result<bool, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        let lhs = self.start_of_unit(parsed_unit)?;
        let rhs = other.start_of_unit(parsed_unit)?;
        Ok(lhs.utc_millis <= rhs.utc_millis)
    }

    pub fn is_same_or_after_unit(
        &self,
        other: &BridgeTime,
        unit: String,
    ) -> Result<bool, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        let lhs = self.start_of_unit(parsed_unit)?;
        let rhs = other.start_of_unit(parsed_unit)?;
        Ok(lhs.utc_millis >= rhs.utc_millis)
    }

    pub fn is_between(
        &self,
        start: &BridgeTime,
        end: &BridgeTime,
        unit: Option<String>,
        inclusivity: Option<String>,
    ) -> Result<bool, BridgeTimeError> {
        let parsed_unit = unit.map(|raw| TimeUnit::parse(&raw)).transpose()?;
        let current = self.comparison_value(parsed_unit)?;
        let lower = start.comparison_value(parsed_unit)?;
        let upper = end.comparison_value(parsed_unit)?;
        let (start_inclusive, end_inclusive) = parse_inclusivity(inclusivity)?;

        let lower_ok = if start_inclusive {
            current >= lower
        } else {
            current > lower
        };

        let upper_ok = if end_inclusive {
            current <= upper
        } else {
            current < upper
        };

        Ok(lower_ok && upper_ok)
    }

    pub fn from_time(
        &self,
        other: &BridgeTime,
        without_suffix: Option<bool>,
    ) -> Result<String, BridgeTimeError> {
        let diff_millis = self
            .utc_millis
            .checked_sub(other.utc_millis)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        Ok(format_relative_phrase(
            diff_millis,
            without_suffix.unwrap_or(false),
        ))
    }

    pub fn to_time(
        &self,
        other: &BridgeTime,
        without_suffix: Option<bool>,
    ) -> Result<String, BridgeTimeError> {
        other.from_time(self, without_suffix)
    }

    pub fn from_now(&self, without_suffix: Option<bool>) -> Result<String, BridgeTimeError> {
        let now = Self::now(Some(self.timezone.clone()))?;
        self.from_time(&now, without_suffix)
    }

    pub fn to_now(&self, without_suffix: Option<bool>) -> Result<String, BridgeTimeError> {
        let now = Self::now(Some(self.timezone.clone()))?;
        self.to_time(&now, without_suffix)
    }

    pub fn min(first: &BridgeTime, second: &BridgeTime) -> BridgeTime {
        if first.utc_millis <= second.utc_millis {
            first.clone_time()
        } else {
            second.clone_time()
        }
    }

    pub fn max(first: &BridgeTime, second: &BridgeTime) -> BridgeTime {
        if first.utc_millis >= second.utc_millis {
            first.clone_time()
        } else {
            second.clone_time()
        }
    }

    pub fn clamp(&self, start: &BridgeTime, end: &BridgeTime) -> BridgeTime {
        let lower = Self::min(start, end);
        let upper = Self::max(start, end);

        if self.utc_millis < lower.utc_millis {
            lower
        } else if self.utc_millis > upper.utc_millis {
            upper
        } else {
            self.clone_time()
        }
    }

    pub fn clone_time(&self) -> BridgeTime {
        Self {
            utc_millis: self.utc_millis,
            timezone: self.timezone.clone(),
        }
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        match self.to_iso() {
            Ok(iso) => format!("BridgeTime(iso='{}', timezone='{}')", iso, self.timezone),
            Err(_) => format!(
                "BridgeTime(utc_ms={}, timezone='{}')",
                self.utc_millis, self.timezone
            ),
        }
    }
}

impl BridgeTime {
    fn as_tz(&self) -> Result<Tz, BridgeTimeError> {
        parse_timezone(&self.timezone)
    }

    fn as_utc_datetime(&self) -> Result<DateTime<Utc>, BridgeTimeError> {
        utc_from_millis(self.utc_millis)
    }

    fn local_datetime(&self) -> Result<DateTime<Tz>, BridgeTimeError> {
        let utc = self.as_utc_datetime()?;
        let tz = self.as_tz()?;
        Ok(utc.with_timezone(&tz))
    }

    fn from_local_datetime(local: DateTime<Tz>) -> Self {
        Self {
            utc_millis: local.with_timezone(&Utc).timestamp_millis(),
            timezone: local.timezone().name().to_string(),
        }
    }

    fn from_resolved_local(tz: Tz, naive: NaiveDateTime) -> Result<Self, BridgeTimeError> {
        let local = resolve_local_datetime(tz, naive)?;
        Ok(Self::from_local_datetime(local))
    }

    fn comparison_value(&self, unit: Option<TimeUnit>) -> Result<i64, BridgeTimeError> {
        match unit {
            Some(parsed_unit) => {
                let normalized = self.start_of_unit(parsed_unit)?;
                Ok(normalized.utc_millis)
            }
            None => Ok(self.utc_millis),
        }
    }

    fn get_field(&self, field: CalendarField) -> Result<i64, BridgeTimeError> {
        let local = self.local_datetime()?;
        let naive = local.naive_local();
        let value = match field {
            CalendarField::Millisecond => i64::from(naive.and_utc().timestamp_subsec_millis()),
            CalendarField::Second => i64::from(naive.second()),
            CalendarField::Minute => i64::from(naive.minute()),
            CalendarField::Hour => i64::from(naive.hour()),
            CalendarField::Day => i64::from(naive.weekday().num_days_from_sunday()),
            CalendarField::Date => i64::from(naive.day()),
            CalendarField::Month => i64::from(naive.month0()),
            CalendarField::Quarter => i64::from((naive.month0() / 3) + 1),
            CalendarField::Year => i64::from(naive.year()),
        };
        Ok(value)
    }

    fn set_field(&self, field: CalendarField, value: i64) -> Result<Self, BridgeTimeError> {
        let local = self.local_datetime()?;
        let tz = local.timezone();
        let naive = local.naive_local();
        let millisecond = naive.and_utc().timestamp_subsec_millis();

        match field {
            CalendarField::Year => {
                let year = i32::try_from(value).map_err(|_| BridgeTimeError::ArithmeticOverflow)?;
                let day = naive.day().min(last_day_of_month(year, naive.month())?);
                let target = build_naive_datetime(
                    year,
                    naive.month(),
                    day,
                    naive.hour(),
                    naive.minute(),
                    naive.second(),
                    millisecond,
                )?;
                Self::from_resolved_local(tz, target)
            }
            CalendarField::Month | CalendarField::Quarter => {
                let target_month0 = if field == CalendarField::Month {
                    i32::try_from(value).map_err(|_| BridgeTimeError::ArithmeticOverflow)?
                } else {
                    let quarter =
                        i32::try_from(value).map_err(|_| BridgeTimeError::ArithmeticOverflow)?;
                    let quarter_zero_based = quarter
                        .checked_sub(1)
                        .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                    let month_in_quarter = i32::try_from(naive.month0() % 3)
                        .map_err(|_| BridgeTimeError::ArithmeticOverflow)?;
                    quarter_zero_based
                        .checked_mul(3)
                        .and_then(|base| base.checked_add(month_in_quarter))
                        .ok_or(BridgeTimeError::ArithmeticOverflow)?
                };

                let base_year_month = naive
                    .year()
                    .checked_mul(12)
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                let total_month = base_year_month
                    .checked_add(target_month0)
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                let year = total_month.div_euclid(12);
                let month = u32::try_from(total_month.rem_euclid(12) + 1)
                    .map_err(|_| BridgeTimeError::ArithmeticOverflow)?;
                let day = naive.day().min(last_day_of_month(year, month)?);
                let target = build_naive_datetime(
                    year,
                    month,
                    day,
                    naive.hour(),
                    naive.minute(),
                    naive.second(),
                    millisecond,
                )?;
                Self::from_resolved_local(tz, target)
            }
            CalendarField::Date => {
                let current = i64::from(naive.day());
                let delta = value
                    .checked_sub(current)
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                let target = naive
                    .checked_add_signed(Duration::days(delta))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Self::from_resolved_local(tz, target)
            }
            CalendarField::Day => {
                let current = i64::from(naive.weekday().num_days_from_sunday());
                let delta = value
                    .checked_sub(current)
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                let target = naive
                    .checked_add_signed(Duration::days(delta))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Self::from_resolved_local(tz, target)
            }
            CalendarField::Hour => {
                let current = i64::from(naive.hour());
                let delta = value
                    .checked_sub(current)
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                let target = naive
                    .checked_add_signed(Duration::hours(delta))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Self::from_resolved_local(tz, target)
            }
            CalendarField::Minute => {
                let current = i64::from(naive.minute());
                let delta = value
                    .checked_sub(current)
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                let target = naive
                    .checked_add_signed(Duration::minutes(delta))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Self::from_resolved_local(tz, target)
            }
            CalendarField::Second => {
                let current = i64::from(naive.second());
                let delta = value
                    .checked_sub(current)
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                let target = naive
                    .checked_add_signed(Duration::seconds(delta))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Self::from_resolved_local(tz, target)
            }
            CalendarField::Millisecond => {
                let current = i64::from(millisecond);
                let delta = value
                    .checked_sub(current)
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                let target = naive
                    .checked_add_signed(Duration::milliseconds(delta))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Self::from_resolved_local(tz, target)
            }
        }
    }

    fn shift_local_by(&self, duration: Duration) -> Result<Self, BridgeTimeError> {
        let local = self.local_datetime()?;
        let tz = local.timezone();
        let shifted = local
            .naive_local()
            .checked_add_signed(duration)
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        Self::from_resolved_local(tz, shifted)
    }

    fn matches_relative_day(&self, delta_days: i64) -> Result<bool, BridgeTimeError> {
        let tz = self.as_tz()?;
        let now_local_date = Utc::now().with_timezone(&tz).date_naive();
        let target_date = now_local_date
            .checked_add_signed(Duration::days(delta_days))
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        let self_local_date = self.local_datetime()?.date_naive();
        Ok(self_local_date == target_date)
    }

    fn add_unit(&self, amount: i64, unit: TimeUnit) -> Result<Self, BridgeTimeError> {
        if amount == 0 {
            return Ok(self.clone_time());
        }

        match unit {
            TimeUnit::Millisecond => {
                let utc = self.as_utc_datetime()?;
                let next = utc
                    .checked_add_signed(Duration::milliseconds(amount))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Ok(Self {
                    utc_millis: next.timestamp_millis(),
                    timezone: self.timezone.clone(),
                })
            }
            TimeUnit::Second => {
                let utc = self.as_utc_datetime()?;
                let next = utc
                    .checked_add_signed(Duration::seconds(amount))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Ok(Self {
                    utc_millis: next.timestamp_millis(),
                    timezone: self.timezone.clone(),
                })
            }
            TimeUnit::Minute => {
                let utc = self.as_utc_datetime()?;
                let next = utc
                    .checked_add_signed(Duration::minutes(amount))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Ok(Self {
                    utc_millis: next.timestamp_millis(),
                    timezone: self.timezone.clone(),
                })
            }
            TimeUnit::Hour => {
                let utc = self.as_utc_datetime()?;
                let next = utc
                    .checked_add_signed(Duration::hours(amount))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                Ok(Self {
                    utc_millis: next.timestamp_millis(),
                    timezone: self.timezone.clone(),
                })
            }
            TimeUnit::Day | TimeUnit::Week => {
                let local = self.local_datetime()?;
                let tz = local.timezone();
                let days = if unit == TimeUnit::Week {
                    amount
                        .checked_mul(7)
                        .ok_or(BridgeTimeError::ArithmeticOverflow)?
                } else {
                    amount
                };
                let naive_shifted = local
                    .naive_local()
                    .checked_add_signed(Duration::days(days))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                let shifted = resolve_local_datetime(tz, naive_shifted)?;
                Ok(Self::from_local_datetime(shifted))
            }
            TimeUnit::Month | TimeUnit::Quarter | TimeUnit::Year => {
                let local = self.local_datetime()?;
                let months: i32 = match unit {
                    TimeUnit::Month => {
                        i32::try_from(amount).map_err(|_| BridgeTimeError::ArithmeticOverflow)?
                    }
                    TimeUnit::Quarter => {
                        let quarter_months = amount
                            .checked_mul(3)
                            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                        i32::try_from(quarter_months)
                            .map_err(|_| BridgeTimeError::ArithmeticOverflow)?
                    }
                    TimeUnit::Year => {
                        let year_months = amount
                            .checked_mul(12)
                            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
                        i32::try_from(year_months)
                            .map_err(|_| BridgeTimeError::ArithmeticOverflow)?
                    }
                    _ => 0,
                };
                let shifted = shift_local_months(local, months)?;
                Ok(Self::from_local_datetime(shifted))
            }
        }
    }

    fn start_of_unit(&self, unit: TimeUnit) -> Result<Self, BridgeTimeError> {
        if unit == TimeUnit::Millisecond {
            return Ok(self.clone_time());
        }

        let local = self.local_datetime()?;
        let tz = local.timezone();
        let naive = local.naive_local();

        let year = naive.year();
        let month = naive.month();
        let day = naive.day();
        let weekday = naive.weekday().num_days_from_sunday() as i64;

        let target_naive = match unit {
            TimeUnit::Second => build_naive_datetime(
                year,
                month,
                day,
                naive.hour(),
                naive.minute(),
                naive.second(),
                0,
            )?,
            TimeUnit::Minute => {
                build_naive_datetime(year, month, day, naive.hour(), naive.minute(), 0, 0)?
            }
            TimeUnit::Hour => build_naive_datetime(year, month, day, naive.hour(), 0, 0, 0)?,
            TimeUnit::Day => build_naive_datetime(year, month, day, 0, 0, 0, 0)?,
            TimeUnit::Week => {
                let day_start = build_naive_datetime(year, month, day, 0, 0, 0, 0)?;
                day_start
                    .checked_sub_signed(Duration::days(weekday))
                    .ok_or(BridgeTimeError::ArithmeticOverflow)?
            }
            TimeUnit::Month => build_naive_datetime(year, month, 1, 0, 0, 0, 0)?,
            TimeUnit::Quarter => {
                let quarter_start_month = ((month - 1) / 3) * 3 + 1;
                build_naive_datetime(year, quarter_start_month, 1, 0, 0, 0, 0)?
            }
            TimeUnit::Year => build_naive_datetime(year, 1, 1, 0, 0, 0, 0)?,
            TimeUnit::Millisecond => unreachable!("handled above"),
        };

        let target_local = resolve_local_datetime(tz, target_naive)?;
        Ok(Self::from_local_datetime(target_local))
    }

    fn end_of_unit(&self, unit: TimeUnit) -> Result<Self, BridgeTimeError> {
        if unit == TimeUnit::Millisecond {
            return Ok(self.clone_time());
        }

        let next_unit = unit.next().ok_or(BridgeTimeError::ArithmeticOverflow)?;
        let next_start = self.start_of_unit(next_unit)?;
        next_start.add_unit(-1, TimeUnit::Millisecond)
    }
}

#[cfg(test)]
mod tests {
    use super::{BridgeDuration, BridgeTime, BridgeTimeError};

    #[test]
    fn parses_iso_and_formats() {
        let dt = BridgeTime::parse("2026-02-22T10:15:30Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");

        let formatted = dt
            .format("YYYY-MM-DD HH:mm:ss".to_string())
            .expect("format should succeed");
        assert_eq!(formatted, "2026-02-22 10:15:30");
    }

    #[test]
    fn parse_format_supports_custom_patterns() {
        let dt = BridgeTime::parse_format(
            "22/02/2026 10:15".to_string(),
            "DD/MM/YYYY HH:mm".to_string(),
            Some("UTC".to_string()),
        )
        .expect("parse_format should succeed");

        assert_eq!(
            dt.format("YYYY-MM-DD HH:mm:ss".to_string())
                .expect("format should succeed"),
            "2026-02-22 10:15:00"
        );

        let date_only = BridgeTime::parse_format(
            "2026/02/22".to_string(),
            "YYYY/MM/DD".to_string(),
            Some("UTC".to_string()),
        )
        .expect("parse_format should succeed");
        assert_eq!(
            date_only
                .format("YYYY-MM-DD HH:mm:ss".to_string())
                .expect("format should succeed"),
            "2026-02-22 00:00:00"
        );
    }

    #[test]
    fn from_array_and_to_array_work() {
        let dt =
            BridgeTime::from_array(vec![2026, 1, 22, 10, 15, 30, 250], Some("UTC".to_string()))
                .expect("from_array should succeed");
        assert_eq!(
            dt.to_array().expect("to_array should succeed"),
            vec![2026, 1, 22, 10, 15, 30, 250]
        );

        let overflow = BridgeTime::from_array(vec![2026, 12, 1], Some("UTC".to_string()))
            .expect("from_array should succeed");
        assert_eq!(
            overflow
                .format("YYYY-MM-DD".to_string())
                .expect("format should succeed"),
            "2027-01-01"
        );
    }

    #[test]
    fn add_and_start_of_work() {
        let dt = BridgeTime::parse("2026-02-22T10:15:30Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");

        let plus_day = dt.add(1, "day".to_string()).expect("add should succeed");
        assert_eq!(plus_day.unix_ms() - dt.unix_ms(), 86_400_000);

        let start = plus_day
            .start_of("day".to_string())
            .expect("start_of should succeed");
        let formatted = start
            .format("YYYY-MM-DD HH:mm:ss".to_string())
            .expect("format should succeed");
        assert_eq!(formatted, "2026-02-23 00:00:00");
    }

    #[test]
    fn timezone_conversion_preserves_instant() {
        let utc_dt = BridgeTime::parse("2026-02-22T12:00:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");
        let ny_dt = utc_dt
            .to_timezone("America/New_York".to_string())
            .expect("timezone conversion should succeed");

        assert_eq!(utc_dt.unix_ms(), ny_dt.unix_ms());
        assert_eq!(ny_dt.timezone(), "America/New_York");
        assert_eq!(utc_dt.utc_offset().expect("utc_offset"), 0);
        assert_eq!(ny_dt.utc_offset().expect("utc_offset"), -300);
        assert!(utc_dt.is_utc().expect("is_utc"));
        assert!(!ny_dt.is_utc().expect("is_utc"));

        let ny_winter = BridgeTime::parse(
            "2026-01-15T12:00:00Z".to_string(),
            Some("America/New_York".to_string()),
        )
        .expect("parse should succeed");
        let ny_summer = BridgeTime::parse(
            "2026-07-15T12:00:00Z".to_string(),
            Some("America/New_York".to_string()),
        )
        .expect("parse should succeed");
        assert!(!ny_winter.is_dst().expect("is_dst"));
        assert!(ny_summer.is_dst().expect("is_dst"));
    }

    #[test]
    fn diff_supports_integer_and_float() {
        let a = BridgeTime::parse("2026-02-22T10:00:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");
        let b = BridgeTime::parse("2026-02-22T11:30:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");

        let int_hours = b
            .diff(&a, "hour".to_string(), Some(false))
            .expect("diff should succeed");
        let float_hours = b
            .diff(&a, "hour".to_string(), Some(true))
            .expect("diff should succeed");

        assert_eq!(int_hours, 1.0);
        assert!((float_hours - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn get_and_set_follow_dayjs_conventions() {
        let dt = BridgeTime::parse(
            "2026-02-22T10:15:30.250Z".to_string(),
            Some("UTC".to_string()),
        )
        .expect("parse should succeed");

        assert_eq!(dt.get("month".to_string()).expect("get month"), 1);
        assert_eq!(dt.get("date".to_string()).expect("get date"), 22);
        assert_eq!(dt.get("day".to_string()).expect("get day"), 0);
        assert_eq!(dt.get("quarter".to_string()).expect("get quarter"), 1);

        let shifted_month = dt.set("month".to_string(), 2).expect("set month");
        assert_eq!(
            shifted_month
                .format("YYYY-MM-DD".to_string())
                .expect("format"),
            "2026-03-22"
        );

        let shifted_day = dt.set("day".to_string(), 1).expect("set day");
        assert_eq!(
            shifted_day
                .format("YYYY-MM-DD".to_string())
                .expect("format"),
            "2026-02-23"
        );

        let shifted_ms = dt
            .set("millisecond".to_string(), 900)
            .expect("set millisecond");
        assert_eq!(shifted_ms.format("SSS".to_string()).expect("format"), "900");
    }

    #[test]
    fn calendar_helpers_work() {
        let leap = BridgeTime::parse("2024-02-10T12:00:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");
        let normal = BridgeTime::parse("2025-02-10T12:00:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");

        assert!(leap.is_valid());
        assert!(leap.is_leap_year().expect("is_leap_year"));
        assert_eq!(leap.days_in_month().expect("days_in_month"), 29);

        assert!(!normal.is_leap_year().expect("is_leap_year"));
        assert_eq!(normal.days_in_month().expect("days_in_month"), 28);
    }

    #[test]
    fn comparison_helpers_with_units_and_ranges_work() {
        let morning =
            BridgeTime::parse("2026-02-22T10:15:30Z".to_string(), Some("UTC".to_string()))
                .expect("parse should succeed");
        let evening =
            BridgeTime::parse("2026-02-22T23:59:00Z".to_string(), Some("UTC".to_string()))
                .expect("parse should succeed");
        let next_day =
            BridgeTime::parse("2026-02-23T00:00:00Z".to_string(), Some("UTC".to_string()))
                .expect("parse should succeed");

        assert!(morning.is_before(&evening));
        assert!(morning.is_same_or_before(&evening));
        assert!(evening.is_same_or_after(&morning));

        assert!(
            morning
                .is_same_unit(&evening, "day".to_string())
                .expect("is_same_unit")
        );
        assert!(
            !evening
                .is_after_unit(&morning, "day".to_string())
                .expect("is_after_unit")
        );
        assert!(
            morning
                .is_before_unit(&next_day, "day".to_string())
                .expect("is_before_unit")
        );

        assert!(
            evening
                .is_between(
                    &morning,
                    &next_day,
                    Some("day".to_string()),
                    Some("[)".to_string())
                )
                .expect("is_between")
        );

        let invalid = evening.is_between(&morning, &next_day, None, Some("invalid".to_string()));
        assert!(matches!(
            invalid,
            Err(BridgeTimeError::InvalidInclusivity(_))
        ));
    }

    #[test]
    fn explicit_component_getters_and_setters_work() {
        let dt = BridgeTime::parse(
            "2026-02-22T10:15:30.250Z".to_string(),
            Some("UTC".to_string()),
        )
        .expect("parse should succeed");

        assert_eq!(dt.year().expect("year"), 2026);
        assert_eq!(dt.month().expect("month"), 1);
        assert_eq!(dt.date().expect("date"), 22);
        assert_eq!(dt.day().expect("day"), 0);
        assert_eq!(dt.hour().expect("hour"), 10);
        assert_eq!(dt.minute().expect("minute"), 15);
        assert_eq!(dt.second().expect("second"), 30);
        assert_eq!(dt.millisecond().expect("millisecond"), 250);

        let shifted = dt
            .set_year(2027)
            .expect("set_year")
            .set_month(3)
            .expect("set_month")
            .set_date(5)
            .expect("set_date")
            .set_hour(12)
            .expect("set_hour")
            .set_minute(45)
            .expect("set_minute")
            .set_second(5)
            .expect("set_second")
            .set_millisecond(900)
            .expect("set_millisecond");

        assert_eq!(
            shifted
                .format("YYYY-MM-DD HH:mm:ss.SSS".to_string())
                .expect("format"),
            "2027-04-05 12:45:05.900"
        );
    }

    #[test]
    fn day_of_year_week_and_relative_helpers_work() {
        let dt = BridgeTime::parse("2026-02-22T10:15:30Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");

        assert_eq!(dt.day_of_year().expect("day_of_year"), 53);
        assert_eq!(dt.quarter().expect("quarter"), 1);
        assert_eq!(dt.iso_weekday().expect("iso_weekday"), 7);
        assert_eq!(dt.week_of_year().expect("week_of_year"), 9);
        assert_eq!(
            dt.week().expect("week"),
            dt.week_of_year().expect("week_of_year")
        );
        assert_eq!(dt.iso_week().expect("iso_week"), 8);
        assert_eq!(dt.iso_week_year().expect("iso_week_year"), 2026);
        assert_eq!(dt.days_in_year().expect("days_in_year"), 365);
        assert_eq!(dt.weeks_in_year().expect("weeks_in_year"), 53);
        assert_eq!(dt.iso_weeks_in_year().expect("iso_weeks_in_year"), 53);

        let next_quarter = dt.set_quarter(2).expect("set_quarter");
        assert_eq!(
            next_quarter
                .format("YYYY-MM-DD".to_string())
                .expect("format"),
            "2026-05-22"
        );

        let monday = dt.set_iso_weekday(1).expect("set_iso_weekday");
        assert_eq!(
            monday.format("YYYY-MM-DD".to_string()).expect("format"),
            "2026-02-16"
        );

        let next_day = dt.set_day_of_year(54).expect("set_day_of_year");
        assert_eq!(
            next_day.format("YYYY-MM-DD".to_string()).expect("format"),
            "2026-02-23"
        );

        let next_week = dt
            .set_week(i64::from(dt.week().expect("week")) + 1)
            .expect("set_week");
        assert_eq!(
            next_week
                .diff(&dt, "day".to_string(), Some(true))
                .expect("diff"),
            7.0
        );

        let next_iso_week = dt
            .set_iso_week(i64::from(dt.iso_week().expect("iso_week")) + 1)
            .expect("set_iso_week");
        assert_eq!(
            next_iso_week
                .diff(&dt, "day".to_string(), Some(true))
                .expect("diff"),
            7.0
        );

        let today = BridgeTime::now(Some("UTC".to_string())).expect("now should succeed");
        assert!(today.is_today().expect("is_today"));
        assert!(
            today
                .add(1, "day".to_string())
                .expect("add")
                .is_tomorrow()
                .expect("is_tomorrow")
        );
        assert!(
            today
                .subtract(1, "day".to_string())
                .expect("subtract")
                .is_yesterday()
                .expect("is_yesterday")
        );
    }

    #[test]
    fn unit_aware_same_or_comparisons_work() {
        let morning =
            BridgeTime::parse("2026-02-22T10:15:30Z".to_string(), Some("UTC".to_string()))
                .expect("parse should succeed");
        let evening =
            BridgeTime::parse("2026-02-22T23:59:00Z".to_string(), Some("UTC".to_string()))
                .expect("parse should succeed");
        let next_day =
            BridgeTime::parse("2026-02-23T00:00:00Z".to_string(), Some("UTC".to_string()))
                .expect("parse should succeed");

        assert!(
            morning
                .is_same_or_before_unit(&evening, "day".to_string())
                .expect("is_same_or_before_unit")
        );
        assert!(
            evening
                .is_same_or_after_unit(&morning, "day".to_string())
                .expect("is_same_or_after_unit")
        );
        assert!(
            !next_day
                .is_same_or_before_unit(&morning, "day".to_string())
                .expect("is_same_or_before_unit")
        );
    }

    #[test]
    fn min_max_and_clamp_work() {
        let a = BridgeTime::parse("2026-02-22T10:00:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");
        let b = BridgeTime::parse("2026-02-22T11:00:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");
        let c = BridgeTime::parse("2026-02-22T12:00:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");

        assert_eq!(BridgeTime::min(&a, &b).unix_ms(), a.unix_ms());
        assert_eq!(BridgeTime::max(&a, &b).unix_ms(), b.unix_ms());
        assert_eq!(a.clamp(&b, &c).unix_ms(), b.unix_ms());
        assert_eq!(c.clamp(&a, &b).unix_ms(), b.unix_ms());
        assert_eq!(b.clamp(&a, &c).unix_ms(), b.unix_ms());
    }

    #[test]
    fn relative_time_helpers_work() {
        let base = BridgeTime::parse("2026-02-22T10:00:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");
        let future = BridgeTime::parse("2026-02-22T10:30:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");
        let past = BridgeTime::parse("2026-02-22T09:30:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");

        assert_eq!(
            future
                .from_time(&base, Some(false))
                .expect("from_time should succeed"),
            "in 30 minutes"
        );
        assert_eq!(
            past.from_time(&base, Some(false))
                .expect("from_time should succeed"),
            "30 minutes ago"
        );
        assert_eq!(
            future
                .from_time(&base, Some(true))
                .expect("from_time should succeed"),
            "30 minutes"
        );
        assert_eq!(
            base.to_time(&future, Some(false))
                .expect("to_time should succeed"),
            "in 30 minutes"
        );

        let now = BridgeTime::now(Some("UTC".to_string())).expect("now should succeed");
        assert!(
            now.add(2, "day".to_string())
                .expect("add should succeed")
                .from_now(Some(false))
                .expect("from_now should succeed")
                .starts_with("in ")
        );
        assert!(
            now.subtract(2, "day".to_string())
                .expect("subtract should succeed")
                .to_now(Some(false))
                .expect("to_now should succeed")
                .starts_with("in ")
        );
    }

    #[test]
    fn duration_helpers_work() {
        let duration =
            BridgeDuration::new(90, Some("minute".to_string())).expect("duration should build");
        assert_eq!(duration.as_milliseconds(), 5_400_000);
        assert_eq!(duration.as_hours(), 1.5);
        assert_eq!(duration.humanize(Some(false)), "2 hours");
        assert_eq!(duration.humanize(Some(true)), "in 2 hours");

        let thirty = BridgeDuration::from_minutes(30).expect("duration");
        let sixty = duration.subtract(&thirty).expect("subtract");
        assert_eq!(sixty.as_minutes(), 60.0);

        let negative = thirty.negate().expect("negate");
        assert_eq!(negative.humanize(Some(true)), "30 minutes ago");

        let base = BridgeTime::parse("2026-02-22T10:00:00Z".to_string(), Some("UTC".to_string()))
            .expect("parse should succeed");
        let moved = base.add_duration(&thirty).expect("add_duration");
        assert_eq!(
            moved
                .format("YYYY-MM-DD HH:mm:ss".to_string())
                .expect("format should succeed"),
            "2026-02-22 10:30:00"
        );

        let back = moved.subtract_duration(&thirty).expect("subtract_duration");
        assert_eq!(back.unix_ms(), base.unix_ms());

        let static_duration =
            BridgeTime::duration(2, Some("hour".to_string())).expect("BridgeTime::duration");
        assert_eq!(static_duration.as_minutes(), 120.0);
    }
}

#[cfg(feature = "python")]
#[bridgerust::pyo3::pymodule]
fn bridgetime(
    m: &bridgerust::pyo3::Bound<'_, bridgerust::pyo3::types::PyModule>,
) -> bridgerust::pyo3::PyResult<()> {
    m.add_class::<BridgeDuration>()?;
    m.add_class::<BridgeTime>()?;
    m.add_function(bridgerust::pyo3::wrap_pyfunction!(supported_units, m)?)?;
    Ok(())
}
