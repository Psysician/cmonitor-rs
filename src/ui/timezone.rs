use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

static OFFSET_FORMAT: &[FormatItem<'static>] =
    format_description!("[offset_hour sign:mandatory]:[offset_minute]");

enum DisplayTimezone {
    Fixed(UtcOffset),
    Named(Tz),
}

pub fn format_datetime_seconds(timestamp: OffsetDateTime, timezone: &str) -> String {
    format_datetime(timestamp, timezone, true)
}

pub fn format_datetime_minutes(timestamp: OffsetDateTime, timezone: &str) -> String {
    format_datetime(timestamp, timezone, false)
}

fn format_datetime(timestamp: OffsetDateTime, timezone: &str, include_seconds: bool) -> String {
    match resolve_timezone(timezone) {
        DisplayTimezone::Fixed(offset) => {
            let local = timestamp.to_offset(offset);
            if include_seconds {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    local.year(),
                    local.month() as u8,
                    local.day(),
                    local.hour(),
                    local.minute(),
                    local.second(),
                )
            } else {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}",
                    local.year(),
                    local.month() as u8,
                    local.day(),
                    local.hour(),
                    local.minute(),
                )
            }
        }
        DisplayTimezone::Named(named) => chrono_datetime(timestamp, named)
            .map(|local| {
                if include_seconds {
                    local.format("%Y-%m-%d %H:%M:%S").to_string()
                } else {
                    local.format("%Y-%m-%d %H:%M").to_string()
                }
            })
            .unwrap_or_else(|| "unknown".to_owned()),
    }
}

fn resolve_timezone(timezone: &str) -> DisplayTimezone {
    if timezone.eq_ignore_ascii_case("utc") || timezone.eq_ignore_ascii_case("z") {
        DisplayTimezone::Fixed(UtcOffset::UTC)
    } else if let Ok(offset) = UtcOffset::parse(timezone, OFFSET_FORMAT) {
        DisplayTimezone::Fixed(offset)
    } else if let Ok(named) = timezone.parse::<Tz>() {
        DisplayTimezone::Named(named)
    } else {
        DisplayTimezone::Fixed(UtcOffset::UTC)
    }
}

fn chrono_datetime(timestamp: OffsetDateTime, timezone: Tz) -> Option<DateTime<Tz>> {
    DateTime::<Utc>::from_timestamp(timestamp.unix_timestamp(), timestamp.nanosecond())
        .map(|value| value.with_timezone(&timezone))
}
