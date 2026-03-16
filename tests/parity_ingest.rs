use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use time::macros::datetime;

use cmonitor_rs::discovery::{JsonlFile, RootSource, collect_jsonl_files, discover_roots_with};
use cmonitor_rs::parser::{decode_jsonl_file, normalize_usage_entries};

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ingest")
        .join(relative)
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("cmonitor-rs-{name}-{nanos}"))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).expect("fixture directory should be created");
}

#[test]
fn discovery_preserves_multi_root_order_and_primary_selection() {
    let base = unique_temp_dir("discover-roots");
    let missing = base.join("missing");
    let z_root = base.join("z-root");
    let a_root = base.join("a-root");
    create_dir(&z_root);
    create_dir(&a_root);

    let discovery = discover_roots_with(&[z_root.clone(), missing, a_root.clone(), z_root.clone()]);

    let discovered = discovery
        .roots
        .iter()
        .map(|root| root.path.clone())
        .collect::<Vec<_>>();
    assert_eq!(discovered, vec![a_root.clone(), z_root.clone()]);
    assert_eq!(discovery.selected.as_deref(), Some(a_root.as_path()));
    assert!(
        discovery
            .roots
            .iter()
            .all(|root| root.source == RootSource::Custom)
    );
    fs::remove_dir_all(base).expect("temp fixture tree should be removed");
}

#[test]
fn jsonl_file_collection_is_recursive_and_sorted() {
    let root = fixture_path("sample-home");
    let files = collect_jsonl_files(&root);
    let relative_paths = files
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(&root)
                .expect("collected file should be under fixture root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        relative_paths,
        vec![
            "alpha/project/nested/session-a.jsonl",
            "alpha/project/session-b.jsonl",
            "beta/another/session-c.jsonl",
        ]
    );
}

#[test]
fn malformed_lines_zero_token_filtering_dedupe_and_cutoff_match_fixture_expectations() {
    let path = fixture_path("mixed-events.jsonl");
    let decoded = decode_jsonl_file(&JsonlFile {
        root: path.parent().expect("fixture parent").to_path_buf(),
        path: path.clone(),
    })
    .expect("fixture jsonl should decode");

    assert_eq!(decoded.diagnostics.len(), 1);
    assert_eq!(decoded.diagnostics[0].line_number, 2);

    let normalized = normalize_usage_entries(decoded, Some(datetime!(2026-03-14 11:59:30 UTC)));

    assert_eq!(normalized.entries.len(), 2);
    assert_eq!(normalized.skipped_before_cutoff, 1);
    assert_eq!(normalized.skipped_zero_tokens, 1);
    assert_eq!(normalized.skipped_duplicates, 1);
    // Preserved raw warnings keep zero-token system rows visible to the
    // limit-detection pass without polluting usage totals. (ref: DL-002)
    let preserved_warnings = normalized
        .retained_raw_events
        .iter()
        .filter(|event| {
            matches!(
                event.payload.get("type").and_then(Value::as_str),
                Some("system" | "tool_result")
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(preserved_warnings.len(), 2);
    assert_eq!(normalized.entries[0].model, "claude-3-5-haiku-20241022");
    assert_eq!(normalized.entries[0].tokens.total_tokens(), 11);
    assert_eq!(normalized.entries[1].message_id.as_deref(), Some("m-1"));
    assert!(normalized.entries[0].timestamp <= normalized.entries[1].timestamp);
}

#[test]
fn parser_invariants() {
    let path = fixture_path("mixed-events.jsonl");
    let decoded = decode_jsonl_file(&JsonlFile {
        root: path.parent().expect("fixture parent").to_path_buf(),
        path,
    })
    .expect("fixture jsonl should decode");

    let normalized = normalize_usage_entries(decoded, None);

    assert!(
        normalized
            .entries
            .windows(2)
            .all(|pair| pair[0].timestamp <= pair[1].timestamp)
    );
    assert!(
        normalized
            .entries
            .iter()
            .all(|entry| entry.tokens.total_tokens() > 0)
    );
    assert!(normalized.entries.iter().all(|entry| {
        // The retained stream still carries warning-only rows so analysis sees
        // the same raw evidence that ingest observed. (ref: DL-002)
        normalized.retained_raw_events.iter().any(|event| {
            event.source_file == entry.source_file && event.line_number == entry.line_number
        })
    }));
    assert!(normalized.retained_raw_events.iter().any(|event| {
        matches!(
            event.payload.get("type").and_then(Value::as_str),
            Some("system")
        )
    }));
}
