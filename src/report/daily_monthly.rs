use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

use crate::report::ReportState;
use crate::report::model::ModelStats;

static DAY_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");
static MONTH_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]");
static OFFSET_FORMAT: &[FormatItem<'static>] =
    format_description!("[offset_hour sign:mandatory]:[offset_minute]");

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AggregateRow {
    pub label: String,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub models: Vec<String>,
    pub per_model: Vec<ModelStats>,
}

/// Carries either fixed offsets or named regions so grouping can follow the
/// resolved CLI timezone across UTC, offset, and region inputs. (ref: DL-003)
enum GroupingTimezone {
    Fixed(UtcOffset),
    Named(Tz),
}

/// Groups blocks on the requested day boundary so daily labels agree with the
/// timezone named in table output. (ref: DL-003)
pub fn build_daily_rows(report: &ReportState, timezone: &str) -> Vec<AggregateRow> {
    build_rows(report, DAY_FORMAT, "%Y-%m-%d", timezone)
}

/// Groups blocks on the requested month boundary so monthly rollups follow the
/// same timezone contract as daily output. (ref: DL-003)
pub fn build_monthly_rows(report: &ReportState, timezone: &str) -> Vec<AggregateRow> {
    build_rows(report, MONTH_FORMAT, "%Y-%m", timezone)
}

/// Converts timestamps at aggregation entry so labels and totals share one
/// calendar boundary for the entire table build. (ref: DL-003)
fn build_rows(
    report: &ReportState,
    formatter: &[FormatItem<'static>],
    chrono_formatter: &str,
    timezone: &str,
) -> Vec<AggregateRow> {
    let mut grouped = BTreeMap::<String, AggregateRow>::new();
    let timezone = resolve_timezone(timezone);

    for block in &report.blocks {
        if block.is_gap {
            continue;
        }

        let label = format_label(block.start_time, formatter, chrono_formatter, &timezone);
        let row = grouped
            .entry(label.clone())
            .or_insert_with(|| AggregateRow {
                label,
                total_tokens: 0,
                total_cost_usd: 0.0,
                models: Vec::new(),
                per_model: Vec::new(),
            });
        row.total_tokens += block.tokens.total_tokens();
        row.total_cost_usd += block.cost_usd;
        for model in &block.models {
            if !row.models.iter().any(|existing| existing == model) {
                row.models.push(model.clone());
            }
        }
        for ms in &block.model_stats {
            if let Some(existing) = row.per_model.iter_mut().find(|e| e.model == ms.model) {
                existing.input_tokens += ms.input_tokens;
                existing.output_tokens += ms.output_tokens;
                existing.cache_creation_tokens += ms.cache_creation_tokens;
                existing.cache_read_tokens += ms.cache_read_tokens;
                existing.total_tokens += ms.total_tokens;
                existing.cost_usd += ms.cost_usd;
            } else {
                row.per_model.push(ms.clone());
            }
        }
    }

    grouped.into_values().collect()
}

/// Resolves UTC, fixed offsets, and named regions into one grouping model that
/// preserves the CLI timezone contract. (ref: DL-003)
fn resolve_timezone(timezone: &str) -> GroupingTimezone {
    if timezone.eq_ignore_ascii_case("utc") || timezone.eq_ignore_ascii_case("z") {
        GroupingTimezone::Fixed(UtcOffset::UTC)
    } else if let Ok(offset) = UtcOffset::parse(timezone, OFFSET_FORMAT) {
        GroupingTimezone::Fixed(offset)
    } else if let Ok(named) = timezone.parse::<Tz>() {
        GroupingTimezone::Named(named)
    } else {
        GroupingTimezone::Fixed(UtcOffset::UTC)
    }
}

/// Keeps grouping keys and rendered labels on the same converted timestamp.
/// (ref: DL-003)
fn format_label(
    timestamp: OffsetDateTime,
    formatter: &[FormatItem<'static>],
    chrono_formatter: &str,
    timezone: &GroupingTimezone,
) -> String {
    match timezone {
        GroupingTimezone::Fixed(offset) => timestamp
            .to_offset(*offset)
            .format(formatter)
            .unwrap_or_else(|_| "unknown".to_owned()),
        GroupingTimezone::Named(named) => chrono_label(timestamp, *named, chrono_formatter),
    }
}

/// Uses chrono-tz only where named-region DST rules affect day and month
/// boundaries. (ref: DL-003)
fn chrono_label(timestamp: OffsetDateTime, timezone: Tz, formatter: &str) -> String {
    DateTime::<Utc>::from_timestamp(timestamp.unix_timestamp(), timestamp.nanosecond())
        .map(|value| value.with_timezone(&timezone).format(formatter).to_string())
        .unwrap_or_else(|| "unknown".to_owned())
}
