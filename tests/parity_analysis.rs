use std::path::PathBuf;

use time::macros::datetime;

use cmonitor_rs::analysis::{
    calculate_custom_cost_limit, calculate_custom_limit, detect_limit_events, transform_to_blocks,
};
use cmonitor_rs::discovery::JsonlFile;
use cmonitor_rs::domain::{TokenUsage, UsageEntry};
use cmonitor_rs::parser::{decode_jsonl_file, normalize_usage_entries};
use cmonitor_rs::report::{ReportState, build_daily_rows, build_monthly_rows};

/// Shares the ingest fixture with analysis tests so preserved-warning coverage
/// cannot drift between normalization and block-level limit detection. (ref: DL-002)
fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ingest")
        .join(relative)
}

fn usage_entry(timestamp: time::OffsetDateTime, total: u64) -> UsageEntry {
    usage_entry_with_cost(timestamp, total, 0.01)
}

fn usage_entry_with_cost(timestamp: time::OffsetDateTime, total: u64, cost: f64) -> UsageEntry {
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
        cost_usd: Some(cost),
        source_file: PathBuf::from("fixture.jsonl"),
        line_number: 1,
    }
}

#[test]
fn block_builder_rounds_to_utc_and_inserts_gaps() {
    let entries = vec![
        usage_entry(datetime!(2026-03-14 12:15 UTC), 10),
        usage_entry(datetime!(2026-03-14 12:45 UTC), 20),
        usage_entry(datetime!(2026-03-14 18:15 UTC), 30),
    ];

    let blocks = transform_to_blocks(&entries, datetime!(2026-03-14 19:00 UTC));

    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].start_time, datetime!(2026-03-14 12:00 UTC));
    assert!(blocks[1].is_gap);
    assert!(blocks[2].is_active);
}

#[test]
fn limit_detection_assigns_warnings_to_block_ranges() {
    let entries = vec![usage_entry(datetime!(2026-03-14 12:15 UTC), 10)];
    let mut blocks = transform_to_blocks(&entries, datetime!(2026-03-14 12:30 UTC));
    let limits = detect_limit_events(
        &[cmonitor_rs::parser::RawUsageEvent {
            source_file: PathBuf::from("fixture.jsonl"),
            line_number: 10,
            payload: serde_json::json!({
                "timestamp": "2026-03-14T12:20:00Z",
                "type": "system",
                "content": "Rate limit reached until later"
            }),
        }],
        &mut blocks,
    );

    assert_eq!(limits.len(), 1);
    assert_eq!(blocks[0].limits.len(), 1);
}

#[test]
fn custom_limit_uses_completed_non_gap_p90() {
    let entries = vec![
        usage_entry(datetime!(2026-03-14 00:05 UTC), 50_000),
        usage_entry(datetime!(2026-03-14 06:05 UTC), 60_000),
        usage_entry(datetime!(2026-03-14 12:05 UTC), 90_000),
    ];

    let blocks = transform_to_blocks(&entries, datetime!(2026-03-15 00:00 UTC));
    let limit = calculate_custom_limit(&blocks).expect("limit should be calculated");

    assert!(limit >= 50_000);
}

#[test]
fn report_rows_share_one_total_pipeline() {
    let entries = vec![usage_entry(datetime!(2026-03-14 12:15 UTC), 10)];
    let blocks = transform_to_blocks(&entries, datetime!(2026-03-14 18:00 UTC));
    let report = ReportState::from_blocks(datetime!(2026-03-14 18:00 UTC), blocks, Vec::new());
    let rows = build_daily_rows(&report, "UTC");

    assert_eq!(rows[0].total_tokens, report.totals.total_tokens);
}

#[test]
/// Uses the mixed-event fixture end to end so warning preservation and limit
/// attachment stay coupled in one regression seam. (ref: DL-002, DL-005)
fn limit_detection_uses_shared_mixed_event_fixture() {
    let path = fixture_path("mixed-events.jsonl");
    let decoded = decode_jsonl_file(&JsonlFile {
        root: path.parent().expect("fixture parent").to_path_buf(),
        path: path.clone(),
    })
    .expect("fixture jsonl should decode");
    let normalized = normalize_usage_entries(decoded, Some(datetime!(2026-03-14 11:59:30 UTC)));
    let mut blocks = transform_to_blocks(&normalized.entries, datetime!(2026-03-14 12:30 UTC));
    let limits = detect_limit_events(&normalized.retained_raw_events, &mut blocks);
    let report = ReportState::from_blocks(datetime!(2026-03-14 12:30 UTC), blocks, limits.clone());

    assert_eq!(limits.len(), 2);
    assert_eq!(report.limits.len(), 2);
    assert_eq!(report.blocks[0].limits.len(), 2);
    assert!(
        limits
            .iter()
            .any(|limit| limit.message.contains("Rate limit"))
    );
    assert!(
        limits
            .iter()
            .any(|limit| limit.message.contains("Usage limit"))
    );
}

#[test]
/// Checks report-row boundaries at the shared aggregation seam so timezone
/// coverage does not depend on table rendering. (ref: DL-003, DL-005)
fn report_rows_use_requested_timezone_boundary_before_table_rendering() {
    let entries = vec![usage_entry(datetime!(2026-03-31 23:30 UTC), 10)];
    let blocks = transform_to_blocks(&entries, datetime!(2026-04-01 01:00 UTC));
    let report = ReportState::from_blocks(datetime!(2026-04-01 01:00 UTC), blocks, Vec::new());
    let daily = build_daily_rows(&report, "Europe/Berlin");
    let monthly = build_monthly_rows(&report, "Europe/Berlin");

    assert_eq!(daily[0].label, "2026-04-01");
    assert_eq!(monthly[0].label, "2026-04");
    assert_eq!(daily[0].total_tokens, report.totals.total_tokens);
    assert_eq!(monthly[0].total_tokens, report.totals.total_tokens);
}

#[test]
fn p90_invariants() {
    let entries = vec![
        usage_entry(datetime!(2026-03-14 00:05 UTC), 10_000),
        usage_entry(datetime!(2026-03-14 06:05 UTC), 20_000),
        usage_entry(datetime!(2026-03-14 12:05 UTC), 30_000),
    ];

    let blocks = transform_to_blocks(&entries, datetime!(2026-03-15 00:00 UTC));
    let limit = calculate_custom_limit(&blocks).expect("limit should be calculated");

    assert!(limit >= 44_000);
}

#[test]
fn custom_cost_limit_uses_completed_non_gap_p90() {
    let entries = vec![
        usage_entry_with_cost(datetime!(2026-03-14 00:05 UTC), 50_000, 20.0),
        usage_entry_with_cost(datetime!(2026-03-14 06:05 UTC), 60_000, 25.0),
        usage_entry_with_cost(datetime!(2026-03-14 12:05 UTC), 90_000, 30.0),
    ];

    let blocks = transform_to_blocks(&entries, datetime!(2026-03-15 00:00 UTC));
    let limit = calculate_custom_cost_limit(&blocks).expect("cost limit should be calculated");

    assert!(limit >= 20.0);
}

#[test]
fn cost_p90_invariants() {
    let entries = vec![
        usage_entry_with_cost(datetime!(2026-03-14 00:05 UTC), 10_000, 2.0),
        usage_entry_with_cost(datetime!(2026-03-14 06:05 UTC), 20_000, 5.0),
        usage_entry_with_cost(datetime!(2026-03-14 12:05 UTC), 30_000, 8.0),
    ];

    let blocks = transform_to_blocks(&entries, datetime!(2026-03-15 00:00 UTC));
    let limit = calculate_custom_cost_limit(&blocks).expect("cost limit should be calculated");

    assert!(limit >= 18.0);
}
