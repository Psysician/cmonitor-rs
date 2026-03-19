use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use cmonitor_rs::report::{ActiveSessionReport, ReportState, ReportTotals};
use cmonitor_rs::runtime::theme::resolve_theme;
use cmonitor_rs::config::Theme;
use cmonitor_rs::ui::realtime::{RealtimeContext, render_realtime};
use time::macros::datetime;

/// Resolves the workspace root once so binary-driven realtime tests execute
/// under the same relative-path assumptions as the CLI. (ref: DL-004)
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Uses the compiled test binary so realtime coverage observes the terminal
/// contract through the real CLI surface. (ref: DL-004)
fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_cmonitor-rs")
}

/// Allocates isolated homes so repeated-render tests can persist state without
/// cross-test terminal or fixture leakage. (ref: DL-004)
fn unique_home(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("cmonitor-rs-{name}-{nanos}"))
}

/// Seeds a minimal live dataset so the refresh loop can render multiple frames
/// without depending on external Claude directories. (ref: DL-004)
fn seed_realtime_home(home: &Path) {
    if home.exists() {
        fs::remove_dir_all(home).expect("remove old fixture home");
    }
    let project_dir = home.join(".claude/projects/demo-project");
    fs::create_dir_all(&project_dir).expect("create realtime fixture project dir");
    let payload = concat!(
        r#"{"timestamp":"2099-01-01T00:15:00Z","message_id":"m-1","request_id":"r-1","model":"Claude-3-7-Sonnet-20250219","usage":{"input_tokens":10,"output_tokens":2},"cost":0.01}"#,
        "\n",
    );
    fs::write(project_dir.join("session.jsonl"), payload).expect("write realtime fixture");
}

fn test_context() -> RealtimeContext {
    RealtimeContext {
        plan_name: "pro".to_owned(),
        token_limit: Some(44_000),
        cost_limit: Some(18.0),
        message_limit: Some(45),
        timezone: "UTC".to_owned(),
        theme: resolve_theme(Theme::Classic),
        now: datetime!(2026-03-14 14:30 UTC),
    }
}

#[test]
fn realtime_render_snapshot_is_deterministic() {
    let report = ReportState {
        generated_at: datetime!(2026-03-14 12:30 UTC),
        blocks: Vec::new(),
        limits: Vec::new(),
        totals: ReportTotals {
            total_tokens: 12,
            input_tokens: 8,
            output_tokens: 4,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            total_cost_usd: 0.01,
            total_messages: 1,
        },
        active_session: Some(ActiveSessionReport {
            block_id: "1700000000".to_owned(),
            started_at: datetime!(2026-03-14 12:00 UTC),
            ends_at: datetime!(2026-03-14 17:00 UTC),
            totals: ReportTotals {
                total_tokens: 12,
                input_tokens: 8,
                output_tokens: 4,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                total_cost_usd: 0.01,
                total_messages: 1,
            },
            warnings: Vec::new(),
            per_model: vec![cmonitor_rs::report::ModelStats {
                model: "claude-3-5-sonnet-20241022".to_owned(),
                input_tokens: 8,
                output_tokens: 4,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                total_tokens: 12,
                cost_usd: 0.01,
            }],
            models: vec!["claude-3-5-sonnet-20241022".to_owned()],
        }),
        custom_limit: None,
        custom_cost_limit: None,
    };

    let ctx = test_context();
    insta::assert_snapshot!("realtime-render", render_realtime(&report, &ctx));
}

#[test]
/// Guards the no-data fast path so alternate-screen mode stays reserved for
/// actual realtime frames. (ref: DL-004)
fn realtime_no_data_fast_path_skips_alternate_screen() {
    let home = unique_home("realtime-empty");
    fs::create_dir_all(&home).expect("create empty home");

    let output = Command::new(bin_path())
        .current_dir(repo_root())
        .env("HOME", &home)
        .arg("--view")
        .arg("realtime")
        .output()
        .expect("realtime binary should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "No Claude data directory found\n");
    assert!(!stdout.contains("\x1b[?1049h"));
}

#[test]
/// Falls back to a plain single snapshot when the process does not own a
/// terminal, preventing ANSI control noise in unsupported environments.
fn realtime_data_path_falls_back_to_plain_snapshot_without_terminal() {
    let home = unique_home("realtime-live");
    seed_realtime_home(&home);

    let output = Command::new(bin_path())
        .current_dir(repo_root())
        .env("HOME", &home)
        .arg("--view")
        .arg("realtime")
        .arg("--plan")
        .arg("pro")
        .output()
        .expect("realtime binary should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\x1b[?1049h"));
    assert!(!stdout.contains("\x1b[H\x1b[2J"));
    assert_eq!(stdout.matches("CLAUDE CODE USAGE MONITOR").count(), 1);
}

#[test]
/// Bounds the live path by frame count so repeated renders are asserted without
/// manual interruption when terminal ownership is explicitly enabled in tests.
/// (ref: DL-004, DL-005)
fn realtime_data_path_enters_alternate_screen_and_stops_after_test_frames() {
    let home = unique_home("realtime-live-tty");
    seed_realtime_home(&home);

    let output = Command::new(bin_path())
        .current_dir(repo_root())
        .env("HOME", &home)
        .env("TERM", "xterm-256color")
        .env("CMONITOR_TEST_FORCE_TTY", "1")
        .env("CMONITOR_TEST_MAX_FRAMES", "3")
        .arg("--view")
        .arg("realtime")
        .arg("--plan")
        .arg("pro")
        .arg("--refresh-rate")
        .arg("1")
        .arg("--refresh-per-second")
        .arg("20")
        .output()
        .expect("realtime binary should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\x1b[?1049h"));
    assert!(stdout.contains("\x1b[?1049l"));
    assert!(stdout.matches("\x1b[H\x1b[2J").count() >= 3);
    assert!(stdout.matches("CLAUDE CODE USAGE MONITOR").count() >= 3);
}
