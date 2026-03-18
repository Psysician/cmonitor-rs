use std::path::PathBuf;

use time::macros::datetime;

use cmonitor_rs::analysis::transform_to_blocks;
use cmonitor_rs::config::Theme;
use cmonitor_rs::domain::{TokenUsage, UsageEntry};
use cmonitor_rs::report::{ReportState, build_daily_rows, build_monthly_rows};
use cmonitor_rs::runtime::theme::resolve_theme;
use cmonitor_rs::ui::{summary, table};

fn entry(timestamp: time::OffsetDateTime, total: u64) -> UsageEntry {
    UsageEntry {
        timestamp,
        model: "claude-3-7-sonnet-20250219".to_owned(),
        message_id: None,
        request_id: None,
        tokens: TokenUsage {
            input_tokens: total,
            output_tokens: 0,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
        },
        cost_usd: Some(0.01),
        source_file: PathBuf::from("fixture.jsonl"),
        line_number: 1,
    }
}

#[test]
fn daily_table_snapshot_is_deterministic() {
    let blocks = transform_to_blocks(
        &[entry(datetime!(2026-03-14 12:15 UTC), 20)],
        datetime!(2026-03-14 18:00 UTC),
    );
    let report = ReportState::from_blocks(datetime!(2026-03-14 18:00 UTC), blocks, Vec::new());
    let theme = resolve_theme(Theme::Classic);
    let output = format!(
        "{}\n{}",
        summary::render_summary(&report),
        table::render_table("daily usage", &build_daily_rows(&report, "UTC"), &theme)
    );

    insta::assert_snapshot!("daily-table", output);
}

#[test]
fn monthly_table_snapshot_is_deterministic() {
    let blocks = transform_to_blocks(
        &[entry(datetime!(2026-03-14 12:15 UTC), 20)],
        datetime!(2026-03-14 18:00 UTC),
    );
    let report = ReportState::from_blocks(datetime!(2026-03-14 18:00 UTC), blocks, Vec::new());
    let theme = resolve_theme(Theme::Classic);
    let output = format!(
        "{}\n{}",
        summary::render_summary(&report),
        table::render_table("monthly usage", &build_monthly_rows(&report, "UTC"), &theme)
    );

    insta::assert_snapshot!("monthly-table", output);
}

#[test]
/// Cross-midnight fixtures prove named timezone grouping keeps rendered labels
/// on the requested day and month boundaries. (ref: DL-003, DL-005)
fn daily_and_monthly_rows_use_requested_timezone_name() {
    let blocks = transform_to_blocks(
        &[entry(datetime!(2026-03-31 23:30 UTC), 20)],
        datetime!(2026-04-01 01:00 UTC),
    );
    let report = ReportState::from_blocks(datetime!(2026-04-01 01:00 UTC), blocks, Vec::new());

    let daily = build_daily_rows(&report, "Europe/Berlin");
    let monthly = build_monthly_rows(&report, "Europe/Berlin");

    assert_eq!(daily[0].label, "2026-04-01");
    assert_eq!(monthly[0].label, "2026-04");
}

#[test]
fn empty_state_stays_explicit() {
    assert_eq!(
        summary::render_empty_state("daily"),
        "no claude usage data available for daily"
    );
}
