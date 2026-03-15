use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_cmonitor-rs")
}

fn unique_home(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("cmonitor-rs-{name}-{nanos}"))
}

fn seed_home(home: &Path, last_used: Option<&str>) {
    if home.exists() {
        fs::remove_dir_all(home).expect("remove old fixture home");
    }
    fs::create_dir_all(home).expect("create fixture home");
    if let Some(payload) = last_used {
        let path = home.join(".claude-monitor/last_used.json");
        fs::create_dir_all(path.parent().expect("config dir")).expect("create config dir");
        fs::write(path, payload).expect("seed last_used");
    }
}

fn scrub_last_used(value: &mut Value) {
    if let Some(last_used) = value.get_mut("last_used").and_then(Value::as_object_mut) {
        last_used.remove("timestamp");
    }
}

fn run_oracle(scenario: &str, home: &Path) -> Value {
    let output = Command::new("python3")
        .current_dir(repo_root())
        .arg("tests/support/oracle_runner.py")
        .arg("--scenario")
        .arg(scenario)
        .arg("--fixture-home")
        .arg(home)
        .output()
        .expect("oracle runner should execute");
    assert!(
        output.status.success(),
        "oracle stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut payload: Value =
        serde_json::from_slice(&output.stdout).expect("oracle output should be valid json");
    scrub_last_used(&mut payload);
    // Bundle mode keeps CLI parity anchored to executable upstream behavior so
    // import drift fails the harness and emulation cannot mask it. (ref: DL-001)
    assert_eq!(
        payload["oracle_mode"],
        json!("bundle"),
        "oracle runner must execute the vendored upstream bundle"
    );
    payload
}

fn run_rust(args: &[&str], home: &Path) -> Value {
    let output = Command::new(bin_path())
        .current_dir(repo_root())
        .env("HOME", home)
        .args(args)
        .output()
        .expect("rust binary should execute");

    let last_used_path = home.join(".claude-monitor/last_used.json");
    let last_used = if last_used_path.exists() {
        Some(
            serde_json::from_slice::<Value>(&fs::read(last_used_path).expect("read last_used"))
                .expect("last_used should be json"),
        )
    } else {
        None
    };

    let mut payload = json!({
        "args": args,
        "exit_code": output.status.code().unwrap_or(1),
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "last_used": last_used,
    });
    scrub_last_used(&mut payload);
    payload
}

/// Narrows parity assertions to the contract fields that must match across
/// oracle and Rust runs, so snapshots cannot bless unrelated drift. (ref: DL-001)
fn parity_payload(value: &Value) -> Value {
    json!({
        "args": value.get("args").cloned().unwrap_or(Value::Null),
        "exit_code": value.get("exit_code").cloned().unwrap_or(Value::Null),
        "stdout": normalized_stdout(value),
        "stderr": value.get("stderr").cloned().unwrap_or(Value::Null),
        "last_used": normalized_last_used(value),
    })
}

/// Ignores raw alternate-screen teardown bytes so M-001 can verify executable
/// CLI parity without conflating the separate terminal-lifetime work in M-004.
fn normalized_stdout(value: &Value) -> Value {
    const TEARDOWN_SUFFIX: &str = "\u{1b}[?25h\u{1b}[?1049l";

    match value.get("stdout") {
        Some(Value::String(stdout)) => {
            Value::String(stdout.trim_end_matches(TEARDOWN_SUFFIX).to_owned())
        }
        Some(other) => other.clone(),
        None => Value::Null,
    }
}

/// Removes ambient-locale time-format drift from last_used parity so M-001 stays
/// scoped to the reviewed executable-oracle regression. (ref: DL-005)
fn normalized_last_used(value: &Value) -> Value {
    match value.get("last_used") {
        Some(Value::Object(object)) => {
            let mut normalized = object.clone();
            normalized.remove("time_format");
            Value::Object(normalized)
        }
        Some(other) => other.clone(),
        None => Value::Null,
    }
}

/// Keeps banner parsing shared so both executables are held to the same shape
/// contract prior to snapshot review. (ref: DL-001)
fn version_stdout(value: &Value) -> &str {
    value["stdout"]
        .as_str()
        .expect("version payload should include stdout text")
}

/// Verifies the upstream binary name and tokenized version format, which are
/// part of the CLI contract independent of exact version text. (ref: DL-001)
fn assert_version_banner_shape(label: &str, value: &Value) {
    let trimmed = version_stdout(value)
        .strip_suffix('\n')
        .expect("version banner should end with a newline");
    let mut parts = trimmed.split_whitespace();
    assert_eq!(
        parts.next(),
        Some("claude-monitor"),
        "{label} banner should keep the upstream binary name"
    );
    let version = parts
        .next()
        .unwrap_or_else(|| panic!("{label} banner should include a version token"));
    assert!(
        !version.is_empty(),
        "{label} banner version token should not be empty"
    );
    assert!(
        parts.next().is_none(),
        "{label} banner should contain only the binary name and version"
    );
}

/// Reuses the same oracle and Rust harness for each scenario so every CLI
/// regression test measures direct contract equality. (ref: DL-001, DL-005)
fn assert_cli_parity(snapshot: &str, scenario: &str, args: &[&str], seed: Option<&str>) {
    let oracle_home = unique_home(&format!("{scenario}-oracle"));
    let rust_home = unique_home(&format!("{scenario}-rust"));
    seed_home(&oracle_home, seed);
    seed_home(&rust_home, seed);

    let oracle = run_oracle(scenario, &oracle_home);
    let rust = run_rust(args, &rust_home);

    assert_eq!(
        parity_payload(&rust),
        parity_payload(&oracle),
        "rust output diverged from oracle for {scenario}"
    );
    insta::assert_json_snapshot!(snapshot, json!({ "oracle": oracle, "rust": rust }));
}

#[test]
fn version_banner_matches_oracle_snapshot() {
    let oracle_home = unique_home("cli-version-oracle");
    let rust_home = unique_home("cli-version-rust");
    seed_home(&oracle_home, None);
    seed_home(&rust_home, None);

    let oracle = run_oracle("cli-version", &oracle_home);
    let rust = run_rust(&["--version"], &rust_home);

    assert_eq!(oracle["exit_code"], json!(0));
    assert_eq!(rust["exit_code"], json!(0));
    assert_eq!(oracle["stderr"], json!(""));
    assert_eq!(rust["stderr"], json!(""));
    assert_version_banner_shape("oracle", &oracle);
    assert_version_banner_shape("rust", &rust);
    insta::assert_json_snapshot!("cli-version", json!({ "oracle": oracle, "rust": rust }));
}

#[test]
fn defaults_and_last_used_match_oracle_snapshot() {
    assert_cli_parity("cli-defaults", "cli-defaults", &[], None);
}

#[test]
fn overrides_match_oracle_snapshot() {
    let args = [
        "--plan",
        "max20",
        "--view",
        "daily",
        "--theme",
        "dark",
        "--timezone",
        "UTC",
        "--refresh-rate",
        "5",
        "--refresh-per-second",
        "1.5",
    ];

    assert_cli_parity("cli-overrides", "cli-overrides", &args, None);
}

#[test]
fn clear_flow_removes_persisted_last_used_file() {
    let seed = r#"{"view":"daily"}"#;
    let oracle_home = unique_home("cli-clear-oracle");
    let rust_home = unique_home("cli-clear-rust");
    seed_home(&oracle_home, Some(seed));
    seed_home(&rust_home, Some(seed));

    let oracle = run_oracle("cli-clear", &oracle_home);
    let rust = run_rust(&["--clear"], &rust_home);

    assert_eq!(parity_payload(&rust), parity_payload(&oracle));
    assert!(
        !rust_home.join(".claude-monitor/last_used.json").exists(),
        "clear should remove last_used.json"
    );
    insta::assert_json_snapshot!("cli-clear", json!({ "oracle": oracle, "rust": rust }));
}
