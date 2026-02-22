#[allow(unused_imports)]
use bridgerust::new;
use bridgerust::{bridge, error};
use chrono::{
    DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, SecondsFormat, TimeZone,
    Timelike, Utc,
};
use chrono_tz::Tz;
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

    pub fn subtract(&self, amount: i64, unit: String) -> Result<Self, BridgeTimeError> {
        let parsed_unit = TimeUnit::parse(&unit)?;
        let negated = amount
            .checked_neg()
            .ok_or(BridgeTimeError::ArithmeticOverflow)?;
        self.add_unit(negated, parsed_unit)
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
    use super::{BridgeTime, BridgeTimeError};

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
}

#[cfg(feature = "python")]
#[bridgerust::pyo3::pymodule]
fn bridgetime(
    m: &bridgerust::pyo3::Bound<'_, bridgerust::pyo3::types::PyModule>,
) -> bridgerust::pyo3::PyResult<()> {
    m.add_class::<BridgeTime>()?;
    m.add_function(bridgerust::pyo3::wrap_pyfunction!(supported_units, m)?)?;
    Ok(())
}
