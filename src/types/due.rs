use crate::types::frontmatter;
use chrono::Datelike;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Due(Option<dto::DateTime>);

impl Due {
    pub const fn has_date(&self) -> bool {
        self.0.is_some()
    }
}

impl From<Option<dto::DateTime>> for Due {
    fn from(value: Option<dto::DateTime>) -> Self {
        Self(value)
    }
}

impl From<Due> for Option<dto::DateTime> {
    fn from(value: Due) -> Self {
        value.0
    }
}

impl Display for Due {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self.0.map_or_else(
            || "None".to_string(),
            |d| d.format(frontmatter::DATE_FMT_STR).to_string(),
        );
        write!(f, "{str}")
    }
}

impl TryFrom<&str> for Due {
    type Error = color_eyre::Report;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_due_date(value)
            .map(Into::into)
            .map_err(|e| color_eyre::eyre::eyre!(e))
    }
}

//NOTE: everything after this was written by claude because I am
// way too fucking lazy to do all this due date parsing bs, I did audit it though.

const fn naive_date_to_dt(d: chrono::NaiveDate) -> chrono::NaiveDateTime {
    d.and_hms_opt(23, 59, 59).unwrap()
}

#[expect(clippy::cast_possible_truncation)]
fn parse_relative_offset(lower: &str, today: chrono::NaiveDate) -> Option<chrono::NaiveDate> {
    let lower = lower.strip_prefix("in ")?.trim();
    let mut parts = lower.splitn(2, ' ');
    let n: i64 = parts.next()?.parse().ok()?;
    let unit = parts.next()?.trim_end_matches('s');
    match unit {
        "day" => Some(today + chrono::Duration::days(n)),
        "week" => Some(today + chrono::Duration::weeks(n)),
        "month" => {
            let total_months = today.month0().cast_signed() + n as i32;
            let year = today.year() + total_months / 12;
            let month = (total_months % 12).cast_unsigned() + 1;
            chrono::NaiveDate::from_ymd_opt(year, month, today.day()).or_else(|| {
                chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).and_then(|d| d.pred_opt())
            })
        }
        "year" => {
            chrono::NaiveDate::from_ymd_opt(today.year() + n as i32, today.month(), today.day())
        }
        _ => None,
    }
}

fn parse_weekday(lower: &str, today: chrono::NaiveDate) -> Option<chrono::NaiveDate> {
    use chrono::Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed};
    let (force_next, word) = lower
        .strip_prefix("next ")
        .map_or((false, lower), |rest| (true, rest.trim()));
    let target = match word {
        "monday" | "mon" => Mon,
        "tuesday" | "tue" | "tues" => Tue,
        "wednesday" | "wed" => Wed,
        "thursday" | "thu" | "thur" | "thurs" => Thu,
        "friday" | "fri" => Fri,
        "saturday" | "sat" => Sat,
        "sunday" | "sun" => Sun,
        _ => return None,
    };
    let mut days_ahead = i64::from(target.num_days_from_monday())
        - i64::from(today.weekday().num_days_from_monday());
    if days_ahead <= 0 || force_next {
        days_ahead += 7;
    }
    Some(today + chrono::Duration::days(days_ahead))
}

#[expect(clippy::too_many_lines)]
fn parse_due_date(s: &str) -> Result<Option<dto::DateTime>, String> {
    use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};

    let s = s.trim();

    // 1. Empty / explicit "no due date" sentinels
    if s.is_empty()
        || matches!(
            s.to_lowercase().as_str(),
            "none" | "never" | "n/a" | "na" | "-" | "--" | "null" | "nil" | "no" | "no due date"
        )
    {
        return Ok(None);
    }

    let lower = s.to_lowercase();
    let today = Local::now().date_naive();

    // 2. Relative human words
    let relative: Option<NaiveDate> = match lower.as_str() {
        "tomorrow" | "tmrw" | "tmr" => Some(today + chrono::Duration::days(1)),
        "yesterday" => Some(today - chrono::Duration::days(1)),
        "today" | "now" | "eod" | "end of day" => Some(today),
        "eow" | "end of week" => {
            let days = (7 - today.weekday().num_days_from_sunday()) % 7;
            Some(today + chrono::Duration::days(i64::from(days)))
        }
        "eom" | "end of month" => {
            let next = if today.month() == 12 {
                NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
            };
            next.and_then(|d| d.pred_opt())
        }
        "eoy" | "end of year" => NaiveDate::from_ymd_opt(today.year(), 12, 31),
        _ => None,
    };
    if let Some(d) = relative {
        return Ok(Some(naive_date_to_dt(d)));
    }

    // 3. "in N days/weeks/months/years"
    if let Some(d) = parse_relative_offset(&lower, today) {
        return Ok(Some(naive_date_to_dt(d)));
    }

    // 4. Named weekdays
    if let Some(d) = parse_weekday(&lower, today) {
        return Ok(Some(naive_date_to_dt(d)));
    }

    let s_noz = s.trim_end_matches('Z');

    // 5. Datetime formats
    let datetime_fmts: &[&str] = &[
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y %H:%M",
        "%d-%m-%Y %H:%M",
        "%m-%d-%Y %H:%M",
        "%d %b %Y %H:%M",
        "%d %B %Y %H:%M",
        "%b %d %Y %H:%M",
        "%B %d %Y %H:%M",
        "%b %d, %Y %H:%M",
        "%B %d, %Y %H:%M",
    ];
    for fmt in datetime_fmts {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s_noz, fmt) {
            return Ok(Some(dt));
        }
    }

    // 6. Date-only formats
    let date_fmts: &[&str] = &[
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%Y.%m.%d",
        "%d/%m/%Y",
        "%m/%d/%Y",
        "%d-%m-%Y",
        "%m-%d-%Y",
        "%d.%m.%Y",
        "%d %b %Y",
        "%d %B %Y",
        "%b %d %Y",
        "%B %d %Y",
        "%b %d, %Y",
        "%B %d, %Y",
        "%b %Y",
        "%B %Y",
        "%Y%m%d",
    ];
    for fmt in date_fmts {
        if let Ok(d) = NaiveDate::parse_from_str(s_noz, fmt) {
            return Ok(Some(naive_date_to_dt(d)));
        }
    }

    // 6b. Yearless shorthand e.g. "4/13" or "4-13" — fill in current year
    let yearless_fmts: &[&str] = &["%m/%d", "%m-%d"];
    for fmt in yearless_fmts {
        if let Ok(d) = NaiveDate::parse_from_str(
            &format!("{}/{}", s_noz, today.year()),
            &format!("{}/{}", fmt, "%Y"),
        ) {
            return Ok(Some(naive_date_to_dt(d)));
        }
    }

    // 7. Unix timestamp (seconds or milliseconds)
    if let Ok(n) = s.parse::<i64>() {
        let secs = if n > 10_000_000_000 { n / 1000 } else { n };
        if let Some(dt) = chrono::DateTime::from_timestamp(secs, 0) {
            return Ok(Some(dt.naive_utc()));
        }
    }

    // 8. Strip noise and retry date-only
    let stripped: String = s
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '/' || *c == ' ')
        .collect();
    let stripped = stripped.trim();
    for fmt in date_fmts {
        if let Ok(d) = NaiveDate::parse_from_str(stripped, fmt) {
            return Ok(Some(naive_date_to_dt(d)));
        }
    }

    Err(format!("could not parse {s:?} as a due date"))
}
