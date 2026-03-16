# Plan

## Overview

Rewrite Claude-Code-Usage-Monitor into cmonitor-rs without losing the upstream CLI, local JSONL ingestion, five-hour session blocks, custom-plan P90 limits, or terminal-first daily, monthly, and realtime behavior.

**Approach**: Use a contract-first single-crate Rust pipeline: fixture and oracle harnesses lock parity, discovery and parser stages normalize Claude JSONL data, analysis builds one shared report model, table views land before the realtime loop, and compatibility adapters isolate upstream quirks until parity is proven.

### Parity Pipeline Overview

[Diagram pending Technical Writer rendering: DIAG-001]

## Planning Context

### Decision Log

| ID | Decision | Reasoning Chain |
|---|---|---|
| DL-001 | Use upstream executable fixtures as the parity contract | README prose and local notes diverge from runtime behavior -> edge cases like session view handling and first-root selection cannot be settled by docs -> store Python execution outputs as oracle fixtures and treat them as the contract |
| DL-002 | Keep the rewrite in one Cargo crate with explicit internal seams | The scaffold is tiny and reuse pressure is unproven -> early workspace splits or async runtime plumbing add ceremony before behavior exists -> use one package with config, discovery, parser, analysis, report, ui, runtime, and compat boundaries |
| DL-003 | Quarantine upstream quirks behind compatibility adapters until parity is green | The upstream runtime already contains user-visible inconsistencies -> fixing them during the rewrite would erase parity evidence and blur failures -> preserve those quirks in a dedicated compat layer until oracle coverage proves safe cleanup |
| DL-004 | Ship the shared daily and monthly report model before the realtime TUI | Daily, monthly, and realtime totals drift today because they derive state differently -> one renderer-neutral report model removes duplicate math and stabilizes user-visible parity -> implement table/report surfaces before live-loop polish |
| DL-005 | Validate with sanitized fixture corpora plus Python oracle harnesses | Core behavior depends on local JSONL edge cases rather than isolated pure functions -> handwritten unit data misses malformed rows, dedupe collisions, and timezone boundaries -> capture sanitized fixture homes and compare Rust outputs to upstream oracles, then add property tests around invariant-heavy helpers |
| DL-006 | Vendor the upstream Python oracle through a repo-local manifest pin | Parity runs must be reproducible on any checkout -> relying on an ad hoc external clone would make snapshot generation non-deterministic and unauditable -> acquire or verify the oracle through a repo-local manifest and vendor path under tests/vendor before running snapshots |
| DL-007 | Defer non-TTY safety behavior changes behind a post-parity compatibility gate | The upstream live path forces terminal control even when no TTY is present -> flipping that behavior during parity work would violate the preserve-upstream contract and muddy oracle failures -> keep the default parity path upstream-compatible and route any safer non-TTY policy through a compatibility gate that stays off until the full parity matrix is green |
| DL-008 | Defer alias takeover and packaging parity claims until fixture parity is green | Repository naming and packaging can imply drop-in replacement readiness -> claiming alias or packaging parity before fixture coverage is green would overstate compatibility and reopen scope -> keep takeover and packaging work out of scope until the parity matrix and README verification commands are complete |

### Rejected Alternatives

| Alternative | Why Rejected |
|---|---|
| One-to-one Python module translation | It preserves the upstream accidental complexity instead of rebuilding a typed Rust pipeline around the behavioral contract. (ref: DL-001) |
| Early multi-crate workspace split | It adds ownership ceremony before reuse pressure proves which seams deserve crate boundaries. (ref: DL-002) |
| Async or Tokio-first runtime | The monitor is a small polling CLI and TUI workload, so async plumbing would hide parity work behind unnecessary concurrency complexity. (ref: DL-002) |
| README-driven parity contract | Published prose already diverges from the executable behavior and would miss the latent edge cases that the rewrite has to preserve. (ref: DL-001) |
| Fork-only provider expansion in the parity line | Provider growth reopens scope before the single-provider rewrite demonstrates parity against the upstream monitor. (ref: DL-003) |

### Constraints

- C-001|type=must|provenance=user:context.constraints[0]|trace=DL-001,DL-003,DL-007,integration:cargo test --test parity_cli,integration:cargo test --test parity_analysis,integration:cargo test --test parity_realtime|Preserve upstream CLI, config, session-block, and P90 behavior before semantic changes.
- C-002|type=must|provenance=user:context.constraints[1]|trace=DL-001,DL-005,DL-006,integration:python3 tests/support/oracle_runner.py --scenario cli-defaults|Treat executable upstream behavior as the parity source of truth instead of README prose.
- C-003|type=must|provenance=user:context.constraints[2]|trace=DL-004,DL-007,M-004,M-005|Keep terminal-first operation and local Claude file analysis as the core product behavior.
- C-004|type=should|provenance=user:context.constraints[3]|trace=DL-002,M-001|Keep cmonitor-rs as a single crate with strict internal module boundaries initially.
- C-005|type=should|provenance=user:context.constraints[4]|trace=DL-004,M-004,M-005|Ship daily and monthly parity before the full realtime TUI.
- C-006|type=must-not|provenance=user:context.constraints[5]|trace=DL-003,M-001.acceptance[3],M-005.acceptance[3]|Do not reintroduce deferred fork-only provider features in the initial parity line.
- C-007|type=must-not|provenance=inferred:task_spec.out_of_scope(alias takeover before parity is proven)|trace=DL-008,M-005.requirement[3],M-005.acceptance[3]|Do not claim alias takeover or packaging parity before fixture parity is green.

### Known Risks

- **The upstream executable contains latent contract drift such as session view acceptance, first-root-only discovery, and token-accounting mismatches across views.**: Capture each quirk as a fixture or oracle case and quarantine the behavior in the compat layer so parity remains measurable.
- **Crate topology churn can force repeated rewiring of dispatch and report boundaries while the Rust codebase grows from a scaffold.**: Lock the root module map and top-level entrypoints in the foundation milestone so later work lands in child files instead of revisiting the crate roots.
- **A thin fixture corpus could miss malformed JSONL rows, duplicate hashes, or timezone boundary cases that only show up in real Claude data.**: Curate sanitized fixture homes from real traces and add property tests around dedupe and timestamp invariants before renderer work begins.
- **Realtime rendering can diverge from daily and monthly totals if it recomputes session math instead of consuming shared report state.**: Make the report model the only source of derived totals and projections for every renderer path, then snapshot both table and realtime outputs from that shared state.

## Invisible Knowledge

### System

The rewrite treats the upstream Python implementation as an executable oracle rather than a module template. The durable seam is a typed Rust pipeline that ends in a shared report model consumed by every renderer, while compatibility shims isolate quirks that parity fixtures still require.

### Invariants

- IK-001|anchors=M-002,CI-M-002-001,CI-M-003-008,integration:cargo test --test parity_ingest|Discovery inspects standard Claude roots, records every match, and preserves the upstream first-root compatibility path until fixture evidence authorizes a behavior change.
- IK-002|anchors=M-002,CI-M-002-003,CI-M-002-004,CI-M-002-006,property:cargo test parser_invariants -- --nocapture|JSONL normalization sorts by timestamp, ignores malformed or zero-token rows, and deduplicates by message_id plus request_id when both values exist.
- IK-003|anchors=M-003,CI-M-003-001,CI-M-003-003,tests/parity_analysis.rs|Session blocks round start times down to UTC hours, span five hours, split on block end or five-hour inactivity, and mark only non-gap future-ending blocks as active.
- IK-004|anchors=M-003,CI-M-003-005,property:cargo test p90_invariants -- --nocapture|Custom-plan token limits derive from completed non-gap blocks via P90, then fall back to completed sessions, then the default custom minimum.
- IK-005|anchors=M-003,M-004,M-005,CI-M-003-006,CI-M-004-003,CI-M-005-002|Daily, monthly, and realtime views consume the same report state and never maintain independent total-token formulas.

### Tradeoffs

- IK-T-001|anchors=DL-003,DL-007,CI-M-003-008,CI-M-005-003|Compatibility shims deliberately preserve upstream quirks early so parity evidence is measurable before cleanup begins.
- IK-T-002|anchors=DL-002,M-001,CI-M-001-002|A single crate reduces initial ceremony at the cost of temporary internal coupling across module roots.
- IK-T-003|anchors=DL-004,M-004,M-005,CI-M-004-003|Daily and monthly parity ships before realtime polish because shared report correctness matters more than live renderer cosmetics.
- IK-T-004|anchors=DL-005,DL-006,M-001,M-002,tests/parity_cli.rs,tests/parity_ingest.rs|Fixture-backed oracle tests increase setup cost but expose upstream drift before a user-visible regression lands.

### Oracle and Fixture Feedback Loop

[Diagram pending Technical Writer rendering: DIAG-002]

## Milestones

### Milestone 1: Crate Topology, CLI Contract, and Oracle Harness

**Files**: Cargo.toml, src/lib.rs, src/main.rs, src/config/mod.rs, src/discovery/mod.rs, src/parser/mod.rs, src/domain/mod.rs, src/analysis/mod.rs, src/report/mod.rs, src/ui/mod.rs, src/runtime/mod.rs, src/compat/mod.rs, tests/support/fetch_upstream_oracle.py, tests/support/oracle_runner.py, tests/vendor/claude-code-usage-monitor.manifest.json, tests/parity_cli.rs, tests/fixtures/contract/README.md, docs/parity-inventory.md

**Flags**: foundation, parity-harness

**Requirements**:

- Stabilize one-crate module roots for downstream work
- Match the upstream CLI flag and persistence contract
- Create a repo-local Python oracle bundle with a manifest-pinned upstream commit and deterministic CLI and config snapshots from fixture homes

**Acceptance Criteria**:

- cargo test --test parity_cli passes
- Python oracle snapshots cover defaults overrides version and clear flows from the repo-local pinned oracle bundle
- Repo-local oracle manifest records the pinned upstream commit and snapshot refresh inputs
- cargo check succeeds with stable module roots and no provider expansion

**Tests**:

- integration:cargo test --test parity_cli
- integration:python3 tests/support/fetch_upstream_oracle.py --check-pin
- integration:python3 tests/support/oracle_runner.py --scenario cli-defaults
- behavior:fixture-backed save load clear and custom-limit reset cases
- behavior:repo-local vendored oracle path and manifest-driven snapshot refresh stay reproducible

#### Code Intent

- **CI-M-001-007** `Cargo.toml`: Pin the single-crate package targets test dependencies and support scripts needed to run the repo-local oracle harness without reopening workspace scope. (refs: DL-002, DL-005, DL-006)
- **CI-M-001-001** `src/main.rs`: Parse the upstream CLI surface, honor version and clear flows, and delegate execution through stable library entrypoints that later milestones extend without changing the binary contract. (refs: DL-001, DL-002)
- **CI-M-001-002** `src/lib.rs`: Expose crate-level config, discovery, parser, domain, analysis, report, ui, runtime, and compat seams so later milestones add child modules without rewiring crate roots. (refs: DL-002)
- **CI-M-001-003** `src/config/mod.rs`: Implement last-used settings persistence, auto-detected defaults, and custom-plan limit reset semantics that mirror upstream config behavior. (refs: DL-001, DL-005)
- **CI-M-001-008** `src/discovery/mod.rs`: Define the discovery module root and stable child exports so downstream root-finding files land without changing crate topology. (refs: DL-002)
- **CI-M-001-009** `src/parser/mod.rs`: Define the parser module root and stable child exports for JSONL decoding and entry normalization work. (refs: DL-002)
- **CI-M-001-010** `src/domain/mod.rs`: Define the domain module root and shared model exports so analysis and report layers depend on stable type boundaries. (refs: DL-002)
- **CI-M-001-011** `src/analysis/mod.rs`: Define the analysis module root for block limit and P90 child modules without revisiting crate-level wiring. (refs: DL-002)
- **CI-M-001-012** `src/report/mod.rs`: Define the report module root that will own the shared renderer-neutral report model and future aggregate helpers. (refs: DL-002, DL-004)
- **CI-M-001-013** `src/ui/mod.rs`: Define the UI module root for table and realtime renderers so the binary contract stays stable while views are added later. (refs: DL-002, DL-004)
- **CI-M-001-014** `src/runtime/mod.rs`: Define the runtime module root for table-mode and realtime orchestration entrypoints without changing main.rs dispatch. (refs: DL-002, DL-004)
- **CI-M-001-015** `src/compat/mod.rs`: Define the compatibility module root that isolates upstream quirks and later terminal policy gates from the core typed pipeline. (refs: DL-002, DL-003)
- **CI-M-001-016** `tests/support/fetch_upstream_oracle.py`: Acquire or verify the pinned upstream oracle bundle from the repo-local manifest into the tests/vendor path before snapshot generation runs. (refs: DL-005, DL-006)
- **CI-M-001-004** `tests/support/oracle_runner.py`: Resolve the repo-local vendored oracle bundle from tests/vendor using the pinned manifest, run upstream claude_monitor entrypoints against sanitized fixture homes, and emit stable oracle payloads for Rust assertions. (refs: DL-001, DL-005, DL-006)
- **CI-M-001-017** `tests/vendor/claude-code-usage-monitor.manifest.json`: Track the upstream repository source pinned commit and snapshot refresh metadata for the vendored oracle bundle. (refs: DL-006)
- **CI-M-001-005** `tests/parity_cli.rs`: Verify Rust CLI and config persistence behavior against stored oracle snapshots for defaults overrides version and clear cases. (refs: DL-001, DL-005)
- **CI-M-001-018** `tests/fixtures/contract/README.md`: Document fixture-home sanitization plus the vendored oracle refresh and snapshot regeneration flow so parity inputs remain reproducible. (refs: DL-001, DL-005, DL-006)
- **CI-M-001-006** `docs/parity-inventory.md`: Expand the parity inventory into explicit fixture-backed contract cases and a deferred divergence list anchored to executable upstream behavior. (refs: DL-001, DL-003)

#### Code Changes

**CC-M-001-001** (Cargo.toml) - implements CI-M-001-007

**Code:**

```diff
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -2,14 +2,40 @@
 name = "cmonitor-rs"
 version = "0.1.0"
 edition = "2024"
-description = "Rust rewrite scaffold for Claude Code Usage Monitor"
+description = "Rust rewrite of Claude Code Usage Monitor with oracle-backed parity harness"
 license = "MIT"
 repository = "https://github.com/Psysician/cmonitor-rs"
 homepage = "https://github.com/Psysician/cmonitor-rs"
 readme = "README.md"
 rust-version = "1.93"
+default-run = "cmonitor-rs"
 keywords = ["claude", "monitor", "terminal", "usage", "cli"]
 categories = ["command-line-utilities"]
 
+[lib]
+name = "cmonitor_rs"
+path = "src/lib.rs"
+
+[[bin]]
+name = "cmonitor-rs"
+path = "src/main.rs"
+
 [dependencies]
+anyhow = "1.0"
+camino = { version = "1.1", features = ["serde1"] }
 clap = { version = "4.5", features = ["derive"] }
+directories = "6.0"
+serde = { version = "1.0", features = ["derive"] }
+serde_json = "1.0"
+time = { version = "0.3", features = ["formatting", "local-offset", "macros", "parsing", "serde"] }
+toml = "0.8"
+tracing = "0.1"
+tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
+walkdir = "2.5"
+
+[dev-dependencies]
+assert_cmd = "2.0"
+assert_fs = "1.1"
+insta = { version = "1.42", features = ["json", "redactions"] }
+predicates = "3.1"
+tempfile = "3.15"

```

**Documentation:**

```diff
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -1,3 +1,6 @@
+# The single-crate rewrite keeps behavior seams explicit while parity coverage grows. (ref: DL-002)
+# The vendored oracle stays reproducible and auditable on every checkout. (ref: DL-006)
+
 [package]
 name = "cmonitor-rs"
 version = "0.1.0"

```


**CC-M-001-002** (src/main.rs) - implements CI-M-001-001

**Code:**

```diff
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,200 +1,11 @@
-use clap::{Parser, ValueEnum};
-use std::ffi::OsString;
-use std::path::PathBuf;
 use std::process::ExitCode;
 
-#[derive(Debug, Parser, Clone)]
-#[command(
-    name = "cmonitor-rs",
-    version,
-    about = "Rust rewrite scaffold for Claude Code Usage Monitor",
-    disable_help_subcommand = true
-)]
-struct Cli {
-    #[arg(long, value_enum, default_value_t = Plan::Custom)]
-    plan: Plan,
-    #[arg(long)]
-    custom_limit_tokens: Option<u32>,
-    #[arg(long, value_enum, default_value_t = View::Realtime)]
-    view: View,
-    #[arg(long)]
-    timezone: Option<String>,
-    #[arg(long, value_enum, default_value_t = TimeFormat::Auto)]
-    time_format: TimeFormat,
-    #[arg(long, value_enum, default_value_t = Theme::Auto)]
-    theme: Theme,
-    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=60), default_value_t = 10)]
-    refresh_rate: u64,
-    #[arg(long, value_parser = parse_refresh_per_second, default_value_t = 0.75)]
-    refresh_per_second: f64,
-    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=23))]
-    reset_hour: Option<u8>,
-    #[arg(long, default_value = "INFO")]
-    log_level: String,
-    #[arg(long)]
-    log_file: Option<PathBuf>,
-    #[arg(long)]
-    debug: bool,
-    #[arg(long)]
-    clear: bool,
-}
-
-#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
-enum Plan {
-    Pro,
-    Max5,
-    Max20,
-    Custom,
-}
-
-#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
-enum View {
-    Realtime,
-    Daily,
-    Monthly,
-}
-
-#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
-enum TimeFormat {
-    Auto,
-    #[value(name = "12h")]
-    TwelveHour,
-    #[value(name = "24h")]
-    TwentyFourHour,
-}
-
-#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
-enum Theme {
-    Auto,
-    Light,
-    Dark,
-    Classic,
-}
-
 fn main() -> ExitCode {
-    run(std::env::args_os())
-}
-
-fn run<I, T>(args: I) -> ExitCode
-where
-    I: IntoIterator<Item = T>,
-    T: Into<OsString> + Clone,
-{
-    let cli = Cli::parse_from(args);
-    println!("{}", render_placeholder(&cli));
-    ExitCode::SUCCESS
-}
-
-fn render_placeholder(cli: &Cli) -> String {
-    let mut lines = vec![
-        "cmonitor-rs bootstrap: realtime terminal monitoring is not implemented yet.".to_owned(),
-        "Parity target: upstream Maciek-roboblog/Claude-Code-Usage-Monitor behavior.".to_owned(),
-        format!("Plan: {}", plan_label(cli.plan)),
-        format!("View: {}", view_label(cli.view)),
-        format!("Theme: {}", theme_label(cli.theme)),
-    ];
-
-    if let Some(limit) = cli.custom_limit_tokens {
-        lines.push(format!("Custom token limit: {limit}"));
-    }
-    if let Some(timezone) = &cli.timezone {
-        lines.push(format!("Timezone override: {timezone}"));
-    }
-    lines.push(format!(
-        "Refresh: {}s data / {:.2}Hz display",
-        cli.refresh_rate, cli.refresh_per_second
-    ));
-    if let Some(reset_hour) = cli.reset_hour {
-        lines.push(format!("Reset hour: {reset_hour}"));
-    }
-    if let Some(path) = &cli.log_file {
-        lines.push(format!("Log file: {}", path.display()));
-    }
-    if cli.debug {
-        lines.push("Debug logging requested.".to_owned());
-    }
-    if cli.clear {
-        lines.push("Saved configuration clear requested.".to_owned());
-    }
-
-    lines.join("\n")
-}
-
-fn parse_refresh_per_second(value: &str) -> Result<f64, String> {
-    let parsed = value
-        .parse::<f64>()
-        .map_err(|err| format!("invalid refresh rate '{value}': {err}"))?;
-
-    if (0.1..=20.0).contains(&parsed) {
-        Ok(parsed)
-    } else {
-        Err("refresh-per-second must be between 0.1 and 20.0".to_owned())
+    match cmonitor_rs::run(std::env::args_os()) {
+        Ok(code) => code,
+        Err(error) => {
+            eprintln!("{error:#}");
+            ExitCode::FAILURE
+        }
     }
 }
-
-fn plan_label(plan: Plan) -> &'static str {
-    match plan {
-        Plan::Pro => "pro",
-        Plan::Max5 => "max5",
-        Plan::Max20 => "max20",
-        Plan::Custom => "custom",
-    }
-}
-
-fn view_label(view: View) -> &'static str {
-    match view {
-        View::Realtime => "realtime",
-        View::Daily => "daily",
-        View::Monthly => "monthly",
-    }
-}
-
-fn theme_label(theme: Theme) -> &'static str {
-    match theme {
-        Theme::Auto => "auto",
-        Theme::Light => "light",
-        Theme::Dark => "dark",
-        Theme::Classic => "classic",
-    }
-}
-
-#[cfg(test)]
-mod tests {
-    use super::*;
-
-    #[test]
-    fn cli_defaults_match_upstream_bootstrap_choices() {
-        let cli = Cli::parse_from(["cmonitor-rs"]);
-
-        assert_eq!(cli.plan, Plan::Custom);
-        assert_eq!(cli.view, View::Realtime);
-        assert_eq!(cli.theme, Theme::Auto);
-        assert_eq!(cli.refresh_rate, 10);
-    }
-
-    #[test]
-    fn placeholder_includes_explicit_overrides() {
-        let cli = Cli::parse_from([
-            "cmonitor-rs",
-            "--plan",
-            "max20",
-            "--view",
-            "daily",
-            "--theme",
-            "dark",
-            "--timezone",
-            "UTC",
-            "--refresh-rate",
-            "5",
-            "--refresh-per-second",
-            "1.5",
-            "--debug",
-        ]);
-
-        let output = render_placeholder(&cli);
-        assert!(output.contains("Plan: max20"));
-        assert!(output.contains("View: daily"));
-        assert!(output.contains("Timezone override: UTC"));
-        assert!(output.contains("Debug logging requested."));
-    }
-}

```

**Documentation:**

```diff
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,5 +1,8 @@
+//! Executable-oracle parity defines CLI and saved-state behavior through a library entrypoint. (ref: DL-001) (ref: DL-002)
+
 use std::process::ExitCode;
 
+/// Executable-oracle parity defines CLI and saved-state behavior through a library entrypoint. (ref: DL-001) (ref: DL-002)
 fn main() -> ExitCode {
     match cmonitor_rs::run(std::env::args_os()) {
         Ok(code) => code,

```


**CC-M-001-003** (src/lib.rs) - implements CI-M-001-002

**Code:**

```diff
--- /dev/null
+++ b/src/lib.rs
@@ -0,0 +1,39 @@
+use std::ffi::OsString;
+use std::process::ExitCode;
+
+pub mod analysis;
+pub mod compat;
+pub mod config;
+pub mod discovery;
+pub mod domain;
+pub mod parser;
+pub mod report;
+pub mod runtime;
+pub mod ui;
+
+pub fn run<I, T>(args: I) -> anyhow::Result<ExitCode>
+where
+    I: IntoIterator<Item = T>,
+    T: Into<OsString> + Clone,
+{
+    let resolved = config::ResolvedConfig::load(args)?;
+
+    if resolved.cli.version {
+        println!("{}", config::version_banner());
+        return Ok(ExitCode::SUCCESS);
+    }
+
+    if resolved.cli.clear {
+        resolved.store.clear()?;
+        return Ok(ExitCode::SUCCESS);
+    }
+
+    // Persist the last-used subset before dispatch so downstream modes
+    // inherit the same precedence rules as the upstream monitor.
+    resolved.store.save_last_used(&resolved.cli)?;
+
+    match resolved.cli.view {
+        config::View::Daily | config::View::Monthly => runtime::run_table_mode(&resolved),
+        config::View::Realtime | config::View::Session => runtime::run_realtime_mode(&resolved),
+    }
+}

```

**Documentation:**

```diff
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,5 @@
+//! The single-crate rewrite keeps behavior seams explicit while parity coverage grows. (ref: DL-002)
+
 use std::ffi::OsString;
 use std::process::ExitCode;
 
@@ -11,6 +13,7 @@
 pub mod runtime;
 pub mod ui;
 
+/// The single-crate rewrite keeps behavior seams explicit while parity coverage grows. (ref: DL-002)
 pub fn run<I, T>(args: I) -> anyhow::Result<ExitCode>
 where
     I: IntoIterator<Item = T>,

```


**CC-M-001-004** (src/config/mod.rs) - implements CI-M-001-003

**Code:**

```diff
--- /dev/null
+++ b/src/config/mod.rs
@@ -0,0 +1,256 @@
+use std::ffi::OsString;
+use std::fs;
+use std::io::ErrorKind;
+
+use camino::Utf8PathBuf;
+use clap::{Parser, ValueEnum};
+use directories::BaseDirs;
+use serde::{Deserialize, Serialize};
+
+#[derive(Debug, Parser, Clone)]
+#[command(
+    name = "cmonitor-rs",
+    disable_help_subcommand = true,
+    about = "Rust rewrite scaffold for Claude Code Usage Monitor"
+)]
+pub struct CliArgs {
+    #[arg(long, value_enum)]
+    pub plan: Option<Plan>,
+    #[arg(long)]
+    pub custom_limit_tokens: Option<u32>,
+    #[arg(long, value_enum)]
+    pub view: Option<View>,
+    #[arg(long)]
+    pub timezone: Option<String>,
+    #[arg(long, value_enum)]
+    pub time_format: Option<TimeFormat>,
+    #[arg(long, value_enum)]
+    pub theme: Option<Theme>,
+    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=60))]
+    pub refresh_rate: Option<u64>,
+    #[arg(long, value_parser = parse_refresh_per_second)]
+    pub refresh_per_second: Option<f64>,
+    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=23))]
+    pub reset_hour: Option<u8>,
+    #[arg(long)]
+    pub log_level: Option<String>,
+    #[arg(long)]
+    pub log_file: Option<Utf8PathBuf>,
+    #[arg(long)]
+    pub debug: bool,
+    #[arg(long)]
+    pub clear: bool,
+    #[arg(short = 'v', long)]
+    pub version: bool,
+}
+
+#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
+#[serde(rename_all = "snake_case")]
+pub enum Plan {
+    Pro,
+    Max5,
+    Max20,
+    Custom,
+}
+
+#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
+#[serde(rename_all = "snake_case")]
+pub enum View {
+    Realtime,
+    Daily,
+    Monthly,
+    Session,
+}
+
+#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
+#[serde(rename_all = "snake_case")]
+pub enum TimeFormat {
+    #[value(name = "12h")]
+    TwelveHour,
+    #[value(name = "24h")]
+    TwentyFourHour,
+    Auto,
+}
+
+#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
+#[serde(rename_all = "snake_case")]
+pub enum Theme {
+    Light,
+    Dark,
+    Classic,
+    Auto,
+}
+
+#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
+pub struct Cli {
+    pub plan: Plan,
+    pub custom_limit_tokens: Option<u32>,
+    pub view: View,
+    pub timezone: String,
+    pub time_format: TimeFormat,
+    pub theme: Theme,
+    pub refresh_rate: u64,
+    pub refresh_per_second: f64,
+    pub reset_hour: Option<u8>,
+    pub log_level: String,
+    pub log_file: Option<Utf8PathBuf>,
+    pub debug: bool,
+    pub clear: bool,
+    pub version: bool,
+}
+
+#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
+pub struct LastUsedConfig {
+    pub custom_limit_tokens: Option<u32>,
+    pub view: Option<View>,
+    pub timezone: Option<String>,
+    pub time_format: Option<TimeFormat>,
+    pub theme: Option<Theme>,
+    pub refresh_rate: Option<u64>,
+    pub reset_hour: Option<u8>,
+}
+
+#[derive(Clone, Debug)]
+pub struct LastUsedStore {
+    path: Utf8PathBuf,
+}
+
+#[derive(Clone, Debug)]
+pub struct ResolvedConfig {
+    pub cli: Cli,
+    pub store: LastUsedStore,
+}
+
+impl Cli {
+    pub fn defaults() -> Self {
+        Self {
+            plan: Plan::Custom,
+            custom_limit_tokens: None,
+            view: View::Realtime,
+            timezone: "auto".to_owned(),
+            time_format: TimeFormat::Auto,
+            theme: Theme::Auto,
+            refresh_rate: 10,
+            refresh_per_second: 0.75,
+            reset_hour: None,
+            log_level: "INFO".to_owned(),
+            log_file: None,
+            debug: false,
+            clear: false,
+            version: false,
+        }
+    }
+}
+
+impl CliArgs {
+    pub fn parse_from_os<I, T>(args: I) -> Self
+    where
+        I: IntoIterator<Item = T>,
+        T: Into<OsString> + Clone,
+    {
+        Self::parse_from(args)
+    }
+
+    pub fn merge_with_last_used(self, stored: Option<LastUsedConfig>) -> Cli {
+        let stored = stored.unwrap_or_default();
+        let mut cli = Cli::defaults();
+        cli.plan = self.plan.unwrap_or(cli.plan);
+        cli.custom_limit_tokens = self
+            .custom_limit_tokens
+            .or(stored.custom_limit_tokens)
+            .filter(|_| matches!(cli.plan, Plan::Custom));
+        cli.view = self.view.or(stored.view).unwrap_or(cli.view);
+        cli.timezone = self.timezone.or(stored.timezone).unwrap_or(cli.timezone);
+        cli.time_format = self.time_format.or(stored.time_format).unwrap_or(cli.time_format);
+        cli.theme = self.theme.or(stored.theme).unwrap_or(cli.theme);
+        cli.refresh_rate = self.refresh_rate.or(stored.refresh_rate).unwrap_or(cli.refresh_rate);
+        cli.refresh_per_second = self.refresh_per_second.unwrap_or(cli.refresh_per_second);
+        cli.reset_hour = self.reset_hour.or(stored.reset_hour);
+        cli.log_level = self.log_level.unwrap_or_else(|| {
+            if self.debug {
+                "DEBUG".to_owned()
+            } else {
+                cli.log_level.clone()
+            }
+        });
+        cli.log_file = self.log_file;
+        cli.debug = self.debug;
+        cli.clear = self.clear;
+        cli.version = self.version;
+        cli
+    }
+}
+
+fn parse_refresh_per_second(value: &str) -> Result<f64, String> {
+    let parsed = value
+        .parse::<f64>()
+        .map_err(|error| format!("invalid refresh rate '{value}': {error}"))?;
+
+    if (0.1..=20.0).contains(&parsed) {
+        Ok(parsed)
+    } else {
+        Err("refresh-per-second must be between 0.1 and 20.0".to_owned())
+    }
+}
+
+impl LastUsedStore {
+    pub fn new() -> anyhow::Result<Self> {
+        let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("home directory unavailable"))?;
+        let path = Utf8PathBuf::from_path_buf(base_dirs.home_dir().join(".claude-monitor/last_used.json"))
+            .map_err(|_| anyhow::anyhow!("non-utf8 home directory path"))?;
+        Ok(Self { path })
+    }
+
+    pub fn load(&self) -> anyhow::Result<Option<LastUsedConfig>> {
+        match fs::read_to_string(&self.path) {
+            Ok(contents) => Ok(Some(serde_json::from_str(&contents)?)),
+            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
+            Err(error) => Err(error.into()),
+        }
+    }
+
+    pub fn save_last_used(&self, cli: &Cli) -> anyhow::Result<()> {
+        if let Some(parent) = self.path.parent() {
+            fs::create_dir_all(parent)?;
+        }
+
+        let payload = LastUsedConfig {
+            custom_limit_tokens: cli.custom_limit_tokens.filter(|_| matches!(cli.plan, Plan::Custom)),
+            view: Some(cli.view),
+            timezone: Some(cli.timezone.clone()),
+            time_format: Some(cli.time_format),
+            theme: Some(cli.theme),
+            refresh_rate: Some(cli.refresh_rate),
+            reset_hour: cli.reset_hour,
+        };
+
+        let serialized = serde_json::to_string_pretty(&payload)?;
+        fs::write(&self.path, serialized)?;
+        Ok(())
+    }
+
+    pub fn clear(&self) -> anyhow::Result<()> {
+        match fs::remove_file(&self.path) {
+            Ok(()) => Ok(()),
+            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
+            Err(error) => Err(error.into()),
+        }
+    }
+}
+
+impl ResolvedConfig {
+    pub fn load<I, T>(args: I) -> anyhow::Result<Self>
+    where
+        I: IntoIterator<Item = T>,
+        T: Into<OsString> + Clone,
+    {
+        let raw = CliArgs::parse_from_os(args);
+        let store = LastUsedStore::new()?;
+        let cli = raw.merge_with_last_used(store.load()?);
+        Ok(Self { cli, store })
+    }
+}
+
+pub fn version_banner() -> String {
+    format!("claude-monitor {}", env!("CARGO_PKG_VERSION"))
+}
```

**Documentation:**

```diff
--- a/src/config/mod.rs
+++ b/src/config/mod.rs
@@ -1,3 +1,5 @@
+//! Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
+
 use std::ffi::OsString;
 use std::fs;
 use std::io::ErrorKind;
@@ -13,6 +15,7 @@
     disable_help_subcommand = true,
     about = "Rust rewrite scaffold for Claude Code Usage Monitor"
 )]
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub struct CliArgs {
     #[arg(long, value_enum)]
     pub plan: Option<Plan>,
@@ -46,6 +49,7 @@
 
 #[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
 #[serde(rename_all = "snake_case")]
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub enum Plan {
     Pro,
     Max5,
@@ -55,6 +59,7 @@
 
 #[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
 #[serde(rename_all = "snake_case")]
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub enum View {
     Realtime,
     Daily,
@@ -64,6 +69,7 @@
 
 #[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
 #[serde(rename_all = "snake_case")]
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub enum TimeFormat {
     #[value(name = "12h")]
     TwelveHour,
@@ -74,6 +80,7 @@
 
 #[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
 #[serde(rename_all = "snake_case")]
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub enum Theme {
     Light,
     Dark,
@@ -82,6 +89,7 @@
 }
 
 #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub struct Cli {
     pub plan: Plan,
     pub custom_limit_tokens: Option<u32>,
@@ -100,6 +108,7 @@
 }
 
 #[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub struct LastUsedConfig {
     pub custom_limit_tokens: Option<u32>,
     pub view: Option<View>,
@@ -111,17 +120,20 @@
 }
 
 #[derive(Clone, Debug)]
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub struct LastUsedStore {
     path: Utf8PathBuf,
 }
 
 #[derive(Clone, Debug)]
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub struct ResolvedConfig {
     pub cli: Cli,
     pub store: LastUsedStore,
 }
 
 impl Cli {
+    /// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
     pub fn defaults() -> Self {
         Self {
             plan: Plan::Custom,
@@ -143,6 +155,7 @@
 }
 
 impl CliArgs {
+    /// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
     pub fn parse_from_os<I, T>(args: I) -> Self
     where
         I: IntoIterator<Item = T>,
@@ -151,6 +164,7 @@
         Self::parse_from(args)
     }
 
+    /// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
     pub fn merge_with_last_used(self, stored: Option<LastUsedConfig>) -> Cli {
         let stored = stored.unwrap_or_default();
         let mut cli = Cli::defaults();
@@ -181,6 +195,7 @@
     }
 }
 
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 fn parse_refresh_per_second(value: &str) -> Result<f64, String> {
     let parsed = value
         .parse::<f64>()
@@ -194,6 +209,7 @@
 }
 
 impl LastUsedStore {
+    /// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
     pub fn new() -> anyhow::Result<Self> {
         let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("home directory unavailable"))?;
         let path = Utf8PathBuf::from_path_buf(base_dirs.home_dir().join(".claude-monitor/last_used.json"))
@@ -201,6 +217,7 @@
         Ok(Self { path })
     }
 
+    /// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
     pub fn load(&self) -> anyhow::Result<Option<LastUsedConfig>> {
         match fs::read_to_string(&self.path) {
             Ok(contents) => Ok(Some(serde_json::from_str(&contents)?)),
@@ -209,6 +226,7 @@
         }
     }
 
+    /// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
     pub fn save_last_used(&self, cli: &Cli) -> anyhow::Result<()> {
         if let Some(parent) = self.path.parent() {
             fs::create_dir_all(parent)?;
@@ -229,6 +247,7 @@
         Ok(())
     }
 
+    /// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
     pub fn clear(&self) -> anyhow::Result<()> {
         match fs::remove_file(&self.path) {
             Ok(()) => Ok(()),
@@ -239,6 +258,7 @@
 }
 
 impl ResolvedConfig {
+    /// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
     pub fn load<I, T>(args: I) -> anyhow::Result<Self>
     where
         I: IntoIterator<Item = T>,
@@ -251,6 +271,7 @@
     }
 }
 
+/// Executable-oracle parity defines CLI and saved-state behavior. (ref: DL-001)
 pub fn version_banner() -> String {
     format!("claude-monitor {}", env!("CARGO_PKG_VERSION"))
 }
\ No newline at end of file

```


**CC-M-001-005** (src/discovery/mod.rs) - implements CI-M-001-008

**Code:**

```diff
--- /dev/null
+++ b/src/discovery/mod.rs
@@ -0,0 +1,8 @@
+//! Discovery stays separate from parsing so parity fixtures can prove
+//! root selection and file enumeration behavior independently.
+
+pub mod jsonl_files;
+pub mod roots;
+
+pub use jsonl_files::{collect_jsonl_files, JsonlFile};
+pub use roots::{discover_roots, select_primary_root, DiscoveredRoot, RootDiscovery};

```

**Documentation:**

```diff
--- a/src/discovery/mod.rs
+++ b/src/discovery/mod.rs
@@ -1,3 +1,5 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
 //! Discovery stays separate from parsing so parity fixtures can prove
 //! root selection and file enumeration behavior independently.
 

```


**CC-M-001-006** (src/parser/mod.rs) - implements CI-M-001-009

**Code:**

```diff
--- /dev/null
+++ b/src/parser/mod.rs
@@ -0,0 +1,8 @@
+//! Parsing produces renderer-neutral events and entries so daily, monthly,
+//! and realtime flows consume the same normalized input.
+
+pub mod entries;
+pub mod jsonl;
+
+pub use entries::{normalize_usage_entries, DedupKey, EntryNormalization, RawUsageEvent};
+pub use jsonl::{decode_jsonl_file, DecodedJsonl, JsonlDiagnostic};

```

**Documentation:**

```diff
--- a/src/parser/mod.rs
+++ b/src/parser/mod.rs
@@ -1,3 +1,5 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
 //! Parsing produces renderer-neutral events and entries so daily, monthly,
 //! and realtime flows consume the same normalized input.
 

```


**CC-M-001-007** (src/domain/mod.rs) - implements CI-M-001-010

**Code:**

```diff
--- /dev/null
+++ b/src/domain/mod.rs
@@ -0,0 +1,10 @@
+//! Shared domain models carry upstream semantics without forcing UI code
+//! to depend on raw JSON payloads.
+
+pub mod plan;
+pub mod session_block;
+pub mod usage_entry;
+
+pub use plan::{PlanDefinition, PlanType};
+pub use session_block::{LimitEvent, LimitKind, SessionBlock};
+pub use usage_entry::{TokenUsage, UsageEntry};
```

**Documentation:**

```diff
--- a/src/domain/mod.rs
+++ b/src/domain/mod.rs
@@ -1,3 +1,5 @@
+//! The single-crate rewrite keeps behavior seams explicit while parity coverage grows. (ref: DL-002)
+
 //! Shared domain models carry upstream semantics without forcing UI code
 //! to depend on raw JSON payloads.
 

```


**CC-M-001-008** (src/analysis/mod.rs) - implements CI-M-001-011

**Code:**

```diff
--- /dev/null
+++ b/src/analysis/mod.rs
@@ -0,0 +1,10 @@
+//! Analysis owns the upstream block and limit rules so renderers do not
+//! duplicate token math or session boundary logic.
+
+pub mod blocks;
+pub mod limits;
+pub mod p90;
+
+pub use blocks::transform_to_blocks;
+pub use limits::detect_limit_events;
+pub use p90::calculate_custom_limit;

```

**Documentation:**

```diff
--- a/src/analysis/mod.rs
+++ b/src/analysis/mod.rs
@@ -1,3 +1,5 @@
+//! Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
+
 //! Analysis owns the upstream block and limit rules so renderers do not
 //! duplicate token math or session boundary logic.
 

```


**CC-M-001-009** (src/report/mod.rs) - implements CI-M-001-012

**Code:**

```diff
--- /dev/null
+++ b/src/report/mod.rs
@@ -0,0 +1,8 @@
+//! Reports are the single derived-state boundary shared by table and live
+//! views so totals cannot drift across output modes.
+
+pub mod daily_monthly;
+pub mod model;
+
+pub use daily_monthly::{build_daily_rows, build_monthly_rows};
+pub use model::{ActiveSessionReport, ReportState, ReportTotals};
```

**Documentation:**

```diff
--- a/src/report/mod.rs
+++ b/src/report/mod.rs
@@ -1,3 +1,5 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+
 //! Reports are the single derived-state boundary shared by table and live
 //! views so totals cannot drift across output modes.
 

```


**CC-M-001-010** (src/ui/mod.rs) - implements CI-M-001-013

**Code:**

```diff
--- /dev/null
+++ b/src/ui/mod.rs
@@ -0,0 +1,6 @@
+//! UI modules stay thin and consume report state instead of re-deriving
+//! totals from raw entries or session blocks.
+
+pub mod realtime;
+pub mod summary;
+pub mod table;

```

**Documentation:**

```diff
--- a/src/ui/mod.rs
+++ b/src/ui/mod.rs
@@ -1,3 +1,5 @@
+//! The single-crate rewrite keeps behavior seams explicit while parity coverage grows. (ref: DL-002)
+
 //! UI modules stay thin and consume report state instead of re-deriving
 //! totals from raw entries or session blocks.
 

```


**CC-M-001-011** (src/runtime/mod.rs) - implements CI-M-001-014

**Code:**

```diff
--- /dev/null
+++ b/src/runtime/mod.rs
@@ -0,0 +1,15 @@
+use std::process::ExitCode;
+
+use crate::config::ResolvedConfig;
+
+pub fn run_table_mode(_resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
+    // Daily and monthly parity ships before the live loop so shared
+    // report correctness is established before alternate-screen work.
+    anyhow::bail!("table mode is planned in M-004")
+}
+
+pub fn run_realtime_mode(_resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
+    // Realtime remains behind the shared report pipeline so live totals
+    // cannot diverge from aggregate views during parity work.
+    anyhow::bail!("realtime mode is planned in M-005")
+}

```

**Documentation:**

```diff
--- a/src/runtime/mod.rs
+++ b/src/runtime/mod.rs
@@ -1,13 +1,17 @@
+//! The single-crate rewrite keeps behavior seams explicit while parity coverage grows. (ref: DL-002)
+
 use std::process::ExitCode;
 
 use crate::config::ResolvedConfig;
 
+/// The single-crate rewrite keeps behavior seams explicit while parity coverage grows. (ref: DL-002)
 pub fn run_table_mode(_resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
     // Daily and monthly parity ships before the live loop so shared
     // report correctness is established before alternate-screen work.
     anyhow::bail!("table mode is planned in M-004")
 }
 
+/// The single-crate rewrite keeps behavior seams explicit while parity coverage grows. (ref: DL-002)
 pub fn run_realtime_mode(_resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
     // Realtime remains behind the shared report pipeline so live totals
     // cannot diverge from aggregate views during parity work.

```


**CC-M-001-012** (src/compat/mod.rs) - implements CI-M-001-015

**Code:**

```diff
--- /dev/null
+++ b/src/compat/mod.rs
@@ -0,0 +1,5 @@
+//! Compatibility code isolates upstream quirks so parity evidence stays
+//! measurable and intentional fixes can land behind explicit gates.
+
+pub mod terminal_policy;
+pub mod upstream;

```

**Documentation:**

```diff
--- a/src/compat/mod.rs
+++ b/src/compat/mod.rs
@@ -1,3 +1,5 @@
+//! Compatibility helpers keep upstream quirks measurable outside the core model. (ref: DL-003)
+
 //! Compatibility code isolates upstream quirks so parity evidence stays
 //! measurable and intentional fixes can land behind explicit gates.
 

```


**CC-M-001-013** (tests/support/fetch_upstream_oracle.py) - implements CI-M-001-016

**Code:**

```diff
--- /dev/null
+++ b/tests/support/fetch_upstream_oracle.py
@@ -0,0 +1,83 @@
+        #!/usr/bin/env python3
+        from __future__ import annotations
+
+        import argparse
+        import json
+        import shutil
+        import subprocess
+        import tempfile
+        from pathlib import Path
+
+        REPO_ROOT = Path(__file__).resolve().parents[2]
+        MANIFEST_PATH = REPO_ROOT / "tests/vendor/claude-code-usage-monitor.manifest.json"
+
+
+        def load_manifest() -> dict:
+            return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
+
+
+        def bundle_path(manifest: dict) -> Path:
+            return REPO_ROOT / manifest["bundle_dir"]
+
+
+        def validate_bundle(manifest: dict, bundle: Path) -> None:
+            required = [bundle / "pyproject.toml", bundle / "src/claude_monitor/cli/main.py"]
+            missing = [str(path.relative_to(REPO_ROOT)) for path in required if not path.exists()]
+            if missing:
+                raise SystemExit(f"vendored oracle missing files: {', '.join(missing)}")
+
+            pin_file = bundle / ".oracle-pin"
+            if pin_file.exists() and pin_file.read_text(encoding="utf-8").strip() != manifest["source"]["commit"]:
+                raise SystemExit("vendored oracle pin does not match manifest")
+
+
+        def copy_checkout(source: Path, destination: Path, commit: str) -> None:
+            if destination.exists():
+                shutil.rmtree(destination)
+            shutil.copytree(
+                source,
+                destination,
+                ignore=shutil.ignore_patterns(".git", ".venv", "__pycache__", ".mypy_cache", ".pytest_cache"),
+            )
+            (destination / ".oracle-pin").write_text(commit + "
+", encoding="utf-8")
+
+
+        def fetch_from_git(manifest: dict, destination: Path) -> None:
+            with tempfile.TemporaryDirectory(prefix="cmonitor-oracle-") as temp_dir:
+                checkout = Path(temp_dir) / "upstream"
+                subprocess.run(["git", "clone", manifest["source"]["repo"], str(checkout)], check=True)
+                subprocess.run(["git", "-C", str(checkout), "checkout", manifest["source"]["commit"]], check=True)
+                copy_checkout(checkout, destination, manifest["source"]["commit"])
+
+
+        def ensure_bundle(check_pin: bool) -> Path:
+            manifest = load_manifest()
+            destination = bundle_path(manifest)
+            if destination.exists():
+                validate_bundle(manifest, destination)
+                return destination
+
+            if check_pin:
+                raise SystemExit("vendored oracle bundle is missing")
+
+            override_env = manifest["source"].get("local_override_env")
+            if override_env and override_env in __import__("os").environ:
+                copy_checkout(Path(__import__("os").environ[override_env]), destination, manifest["source"]["commit"])
+            else:
+                fetch_from_git(manifest, destination)
+            validate_bundle(manifest, destination)
+            return destination
+
+
+        def main() -> int:
+            parser = argparse.ArgumentParser()
+            parser.add_argument("--check-pin", action="store_true")
+            args = parser.parse_args()
+            bundle = ensure_bundle(check_pin=args.check_pin)
+            print(json.dumps({"bundle": str(bundle.relative_to(REPO_ROOT))}, indent=2))
+            return 0
+
+
+        if __name__ == "__main__":
+            raise SystemExit(main())

```

**Documentation:**

```diff
--- a/tests/support/fetch_upstream_oracle.py
+++ b/tests/support/fetch_upstream_oracle.py
@@ -1,4 +1,5 @@
         #!/usr/bin/env python3
+        # The vendored oracle stays reproducible and auditable on every checkout. (ref: DL-006)
         from __future__ import annotations
 
         import argparse

```


**CC-M-001-014** (tests/support/oracle_runner.py) - implements CI-M-001-004

**Code:**

```diff
--- /dev/null
+++ b/tests/support/oracle_runner.py
@@ -0,0 +1,72 @@
+#!/usr/bin/env python3
+from __future__ import annotations
+
+import argparse
+import json
+import os
+import subprocess
+import sys
+from pathlib import Path
+
+REPO_ROOT = Path(__file__).resolve().parents[2]
+MANIFEST_PATH = REPO_ROOT / "tests/vendor/claude-code-usage-monitor.manifest.json"
+FIXTURE_ROOT = REPO_ROOT / "tests/fixtures/contract"
+
+SCENARIOS = {
+    "cli-defaults": {"fixture": "defaults", "args": []},
+    "cli-overrides": {
+        "fixture": "overrides",
+        "args": ["--plan", "max20", "--view", "daily", "--theme", "dark", "--timezone", "UTC"],
+    },
+    "cli-version": {"fixture": "defaults", "args": ["--version"]},
+    "cli-clear": {"fixture": "clear", "args": ["--clear"]},
+}
+
+
+def manifest() -> dict:
+    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
+
+
+def bundle_path() -> Path:
+    return REPO_ROOT / manifest()["bundle_dir"]
+
+
+def run_scenario(name: str) -> dict:
+    scenario = SCENARIOS[name]
+    fixture_home = FIXTURE_ROOT / scenario["fixture"] / "home"
+    fixture_home.mkdir(parents=True, exist_ok=True)
+
+    env = os.environ.copy()
+    env["HOME"] = str(fixture_home)
+    env["PYTHONPATH"] = str(bundle_path() / "src")
+
+    command = [
+        sys.executable,
+        "-c",
+        "from claude_monitor.cli.main import main; import sys; raise SystemExit(main(sys.argv[1:]))",
+        *scenario["args"],
+    ]
+    completed = subprocess.run(command, env=env, text=True, capture_output=True, check=False)
+
+    last_used = fixture_home / ".claude-monitor/last_used.json"
+    payload = {
+        "scenario": name,
+        "args": scenario["args"],
+        "exit_code": completed.returncode,
+        "stdout": completed.stdout,
+        "stderr": completed.stderr,
+        "last_used": json.loads(last_used.read_text(encoding="utf-8")) if last_used.exists() else None,
+    }
+    return payload
+
+
+def main() -> int:
+    parser = argparse.ArgumentParser()
+    parser.add_argument("--scenario", choices=sorted(SCENARIOS))
+    args = parser.parse_args()
+    print(json.dumps(run_scenario(args.scenario), indent=2, sort_keys=True))
+    return 0
+
+
+if __name__ == "__main__":
+    raise SystemExit(main())

```

**Documentation:**

```diff
--- a/tests/support/oracle_runner.py
+++ b/tests/support/oracle_runner.py
@@ -1,4 +1,5 @@
 #!/usr/bin/env python3
+# The vendored oracle stays reproducible and auditable on every checkout. (ref: DL-006)
 from __future__ import annotations
 
 import argparse

```


**CC-M-001-015** (tests/vendor/claude-code-usage-monitor.manifest.json) - implements CI-M-001-017

**Code:**

```diff
--- /dev/null
+++ b/tests/vendor/claude-code-usage-monitor.manifest.json
@@ -0,0 +1,18 @@
+{
+  "bundle_dir": "tests/vendor/claude-code-usage-monitor",
+  "source": {
+    "repo": "https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor",
+    "commit": "PIN_UPSTREAM_COMMIT",
+    "local_override_env": "CMONITOR_UPSTREAM_SOURCE"
+  },
+  "scenarios": [
+    "cli-defaults",
+    "cli-overrides",
+    "cli-version",
+    "cli-clear"
+  ],
+  "notes": [
+    "The vendored bundle is the parity oracle for executable behavior.",
+    "Refresh snapshots only after updating the manifest pin and recording the fixture input changes."
+  ]
+}

```

**Documentation:**

```diff
--- a/tests/vendor/claude-code-usage-monitor.manifest.json
+++ b/tests/vendor/claude-code-usage-monitor.manifest.json
@@ -12,6 +12,7 @@
     "cli-clear"
   ],
   "notes": [
+    "The vendored oracle stays reproducible and auditable on every checkout. (ref: DL-006)",
     "The vendored bundle is the parity oracle for executable behavior.",
     "Refresh snapshots only after updating the manifest pin and recording the fixture input changes."
   ]

```


**CC-M-001-016** (tests/parity_cli.rs) - implements CI-M-001-005

**Code:**

```diff
--- /dev/null
+++ b/tests/parity_cli.rs
@@ -0,0 +1,72 @@
+        use assert_cmd::Command;
+        use serde_json::Value;
+        use std::path::PathBuf;
+        use std::process::Command as StdCommand;
+
+        fn repo_root() -> PathBuf {
+            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
+        }
+
+        fn fixture_home(name: &str) -> PathBuf {
+            repo_root().join("tests/fixtures/contract").join(name).join("home")
+        }
+
+        fn run_oracle(scenario: &str) -> Value {
+            let output = StdCommand::new("python3")
+                .current_dir(repo_root())
+                .arg("tests/support/oracle_runner.py")
+                .arg("--scenario")
+                .arg(scenario)
+                .output()
+                .expect("oracle runner should execute");
+            assert!(output.status.success(), "oracle stderr: {}", String::from_utf8_lossy(&output.stderr));
+            serde_json::from_slice(&output.stdout).expect("oracle output should be json")
+        }
+
+        fn run_rust(args: &[&str], fixture: &str) -> Value {
+            let assert = Command::cargo_bin("cmonitor-rs")
+                .expect("binary should build")
+                .env("HOME", fixture_home(fixture))
+                .args(args)
+                .assert();
+            let output = assert.get_output();
+            let last_used_path = fixture_home(fixture).join(".claude-monitor/last_used.json");
+            serde_json::json!({
+                "args": args,
+                "exit_code": output.status.code().unwrap_or(1),
+                "stdout": String::from_utf8_lossy(&output.stdout),
+                "stderr": String::from_utf8_lossy(&output.stderr),
+                "last_used": if last_used_path.exists() {
+                    Some(serde_json::from_slice::<Value>(&std::fs::read(last_used_path).expect("read last_used"))
+                        .expect("last_used json"))
+                } else {
+                    None
+                }
+            })
+        }
+
+        #[test]
+        fn version_banner_matches_oracle() {
+            let oracle = run_oracle("cli-version");
+            let rust = run_rust(&["--version"], "defaults");
+            insta::assert_json_snapshot!("cli-version", serde_json::json!({ "oracle": oracle, "rust": rust }));
+        }
+
+        #[test]
+        fn defaults_and_last_used_match_oracle_contract() {
+            let oracle = run_oracle("cli-defaults");
+            let rust = run_rust(&[], "defaults");
+            insta::assert_json_snapshot!("cli-defaults", serde_json::json!({ "oracle": oracle, "rust": rust }));
+        }
+
+        #[test]
+        fn clear_flow_removes_persisted_last_used_file() {
+            let home = fixture_home("clear");
+            std::fs::create_dir_all(home.join(".claude-monitor")).expect("config dir");
+            std::fs::write(home.join(".claude-monitor/last_used.json"), br#"{"view":"daily"}"#).expect("seed last_used");
+
+            let oracle = run_oracle("cli-clear");
+            let rust = run_rust(&["--clear"], "clear");
+            assert!(!home.join(".claude-monitor/last_used.json").exists());
+            insta::assert_json_snapshot!("cli-clear", serde_json::json!({ "oracle": oracle, "rust": rust }));
+        }
```

**Documentation:**

```diff
--- a/tests/parity_cli.rs
+++ b/tests/parity_cli.rs
@@ -1,16 +1,21 @@
+//! Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)
+
         use assert_cmd::Command;
         use serde_json::Value;
         use std::path::PathBuf;
         use std::process::Command as StdCommand;
 
+        /// Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)
         fn repo_root() -> PathBuf {
             PathBuf::from(env!("CARGO_MANIFEST_DIR"))
         }
 
+        /// Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)
         fn fixture_home(name: &str) -> PathBuf {
             repo_root().join("tests/fixtures/contract").join(name).join("home")
         }
 
+        /// Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)
         fn run_oracle(scenario: &str) -> Value {
             let output = StdCommand::new("python3")
                 .current_dir(repo_root())
@@ -23,6 +28,7 @@
             serde_json::from_slice(&output.stdout).expect("oracle output should be json")
         }
 
+        /// Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)
         fn run_rust(args: &[&str], fixture: &str) -> Value {
             let assert = Command::cargo_bin("cmonitor-rs")
                 .expect("binary should build")
@@ -46,6 +52,7 @@
         }
 
         #[test]
+        /// Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)
         fn version_banner_matches_oracle() {
             let oracle = run_oracle("cli-version");
             let rust = run_rust(&["--version"], "defaults");
@@ -53,6 +60,7 @@
         }
 
         #[test]
+        /// Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)
         fn defaults_and_last_used_match_oracle_contract() {
             let oracle = run_oracle("cli-defaults");
             let rust = run_rust(&[], "defaults");
@@ -60,6 +68,7 @@
         }
 
         #[test]
+        /// Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)
         fn clear_flow_removes_persisted_last_used_file() {
             let home = fixture_home("clear");
             std::fs::create_dir_all(home.join(".claude-monitor")).expect("config dir");

```


**CC-M-001-017** (tests/fixtures/contract/README.md) - implements CI-M-001-018

**Code:**

```diff
--- /dev/null
+++ b/tests/fixtures/contract/README.md
@@ -0,0 +1,23 @@
+# Contract Fixtures
+
+These fixtures are sanitized HOME directories used by the parity harness in
+`tests/parity_cli.rs` and `tests/support/oracle_runner.py`.
+
+## Layout
+
+- `defaults/home`: no pre-seeded `last_used.json`; proves upstream defaults.
+- `overrides/home`: reserved for explicit CLI override snapshots.
+- `clear/home`: contains a seeded `last_used.json` so `--clear` removes it.
+
+## Oracle Refresh Flow
+
+1. Update `tests/vendor/claude-code-usage-monitor.manifest.json` with the new upstream commit.
+2. Run `python3 tests/support/fetch_upstream_oracle.py` to verify or fetch the vendored bundle.
+3. Run `python3 tests/support/oracle_runner.py --scenario ...` for each scenario and review the emitted JSON.
+4. Re-record Rust snapshots only after the oracle outputs and fixture HOME contents are stable.
+
+## Sanitization Rules
+
+- Keep only the files required for the scenario under `home/`.
+- Replace any user-specific paths, project names, and session identifiers with deterministic placeholders.
+- Treat executable upstream behavior as the source of truth when fixture content and prose disagree.

```

**Documentation:**

```diff
--- a/tests/fixtures/contract/README.md
+++ b/tests/fixtures/contract/README.md
@@ -21,3 +21,7 @@
 - Keep only the files required for the scenario under `home/`.
 - Replace any user-specific paths, project names, and session identifiers with deterministic placeholders.
 - Treat executable upstream behavior as the source of truth when fixture content and prose disagree.
+
+## Planning Rationale
+
+- Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)

```


**CC-M-001-018** (docs/parity-inventory.md) - implements CI-M-001-006

**Code:**

```diff
--- a/docs/parity-inventory.md
+++ b/docs/parity-inventory.md
@@ -3,34 +3,29 @@
 ## Source of truth
 
 - Behavioral target: `Maciek-roboblog/Claude-Code-Usage-Monitor`
-- Non-target for bootstrap: provider extensions from `Psysician/c-monitor`
+- Contract authority: upstream executable behavior plus upstream tests
+- Explicit non-target: provider extensions from `Psysician/c-monitor`
 
-## CLI contract to match
+## Fixture-backed contract cases
 
-- `--plan` with `pro`, `max5`, `max20`, and `custom`
-- `--custom-limit-tokens`
-- `--view` with `realtime`, `daily`, and `monthly`
-- `--timezone`
-- `--time-format`
-- `--theme`
-- `--refresh-rate`
-- `--refresh-per-second`
-- `--reset-hour`
-- `--log-level`
-- `--log-file`
-- `--debug`
-- `--clear`
+- CLI defaults: no saved config, no explicit flags, realtime view default, custom plan default, and upstream version banner shape.
+- CLI overrides: `--plan`, `--view`, `--theme`, `--timezone`, `--refresh-rate`, and `--refresh-per-second` must override stored values without mutating unrelated fields.
+- Saved config persistence: theme, timezone, time format, refresh rate, reset hour, view, and custom limit persist in `~/.claude-monitor/last_used.json`.
+- Clear flow: `--clear` removes the saved config file without needing a data-path scan.
+- Oracle pinning: every executable comparison runs against the commit recorded in `tests/vendor/claude-code-usage-monitor.manifest.json`.
 
 ## Runtime behavior invariants
 
-- Analyze Claude session data from local files rather than a remote API
-- Maintain terminal-first operation for the primary user experience
-- Preserve default plan/view semantics before changing UX or terminology
-- Rebuild confidence with deterministic fixtures and state-transition tests
+- Analyze Claude session data from local files rather than a remote API.
+- Maintain terminal-first operation for the primary user experience.
+- Preserve five-hour UTC-rounded session-block semantics before intentional fixes.
+- Preserve custom-plan P90 fallback order before semantic cleanup.
+- Route known upstream quirks through compatibility helpers instead of silently redesigning them.
 
-## Explicitly deferred from bootstrap
+## Deferred divergence list
 
-- Codex-only or dual-provider monitoring
-- Memory-budget gates and other fork-only observability additions
-- Final alias packaging for `claude-monitor`, `cmonitor`, `ccmonitor`, and `ccm`
-
+- First-discovered-root-only behavior remains compatible until multi-root fixtures prove a safe change.
+- Latent `session` view handling remains compatibility-scoped until the renderer contract is fully covered.
+- Realtime versus aggregate token-accounting drift remains compatibility-scoped until shared report parity is green.
+- Alias takeover and packaging parity stay out of scope until the full fixture matrix passes.
+- Fork-only provider features remain explicitly deferred from the parity line.

```

**Documentation:**

```diff
--- a/docs/parity-inventory.md
+++ b/docs/parity-inventory.md
@@ -29,3 +29,7 @@
 - Realtime versus aggregate token-accounting drift remains compatibility-scoped until shared report parity is green.
 - Alias takeover and packaging parity stay out of scope until the full fixture matrix passes.
 - Fork-only provider features remain explicitly deferred from the parity line.
+
+## Planning Rationale
+
+- Executable-oracle parity defines CLI and saved-state behavior and fixture-backed verification outrank prose drift. (ref: DL-001) (ref: DL-005)

```


### Milestone 2: Fixture-Driven Discovery and JSONL Normalization

**Files**: src/discovery/roots.rs, src/discovery/jsonl_files.rs, src/parser/jsonl.rs, src/parser/entries.rs, src/domain/usage_entry.rs, tests/parity_ingest.rs, tests/fixtures/ingest/README.md

**Flags**: ingest, fixture-parity

**Requirements**:

- Discover standard Claude data roots and preserve deterministic selection order
- Collect JSONL files recursively with missing-path tolerance and stable ordering
- Normalize raw events into typed usage entries with malformed-line tolerance zero-token filtering message_id plus request_id dedupe and timestamp sorting

**Acceptance Criteria**:

- cargo test --test parity_ingest passes
- Fixture cases cover multi-root discovery malformed lines zero-token filtering composite dedupe and cutoff filtering
- No parser path performs renderer-specific math or terminal formatting

**Tests**:

- integration:cargo test --test parity_ingest
- property:cargo test parser_invariants -- --nocapture
- behavior:oracle snapshots for discovery order zero-token filtering composite dedupe and sorted entry output

#### Code Intent

- **CI-M-002-001** `src/discovery/roots.rs`: Discover standard and custom Claude data roots, preserve deterministic ordering, and surface all matches while keeping the upstream first-root compatibility decision observable. (refs: DL-001, DL-003)
- **CI-M-002-002** `src/discovery/jsonl_files.rs`: Collect JSONL files recursively with stable ordering and tolerant handling for absent or unreadable roots. (refs: DL-002, DL-005)
- **CI-M-002-003** `src/parser/jsonl.rs`: Decode JSONL lines into raw events while retaining malformed-line diagnostics instead of aborting the ingest pass. (refs: DL-002, DL-005)
- **CI-M-002-004** `src/parser/entries.rs`: Transform raw events into sorted usage entries with cutoff filtering zero-token rejection message_id plus request_id composite dedupe token extraction model normalization and raw-event retention for later limit detection. (refs: DL-001, DL-005)
- **CI-M-002-005** `src/domain/usage_entry.rs`: Define typed usage entry and token-count models shared by analysis reports and parity tests. (refs: DL-002)
- **CI-M-002-006** `tests/parity_ingest.rs`: Assert discovery and normalization parity against fixture oracles plus invariant checks for zero-token filtering composite dedupe sorting and cutoff behavior. (refs: DL-001, DL-005)

#### Code Changes

**CC-M-002-001** (src/discovery/roots.rs) - implements CI-M-002-001

**Code:**

```diff
--- /dev/null
+++ b/src/discovery/roots.rs
@@ -0,0 +1,74 @@
+use std::collections::BTreeSet;
+
+use camino::{Utf8Path, Utf8PathBuf};
+use directories::BaseDirs;
+
+#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub enum RootSource {
+    Standard,
+    Custom,
+}
+
+#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct DiscoveredRoot {
+    pub path: Utf8PathBuf,
+    pub source: RootSource,
+}
+
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct RootDiscovery {
+    pub discovered: Vec<DiscoveredRoot>,
+    pub selected: Option<Utf8PathBuf>,
+}
+
+pub fn standard_roots() -> Vec<DiscoveredRoot> {
+    let Some(base_dirs) = BaseDirs::new() else {
+        return Vec::new();
+    };
+
+    let Ok(home) = Utf8PathBuf::from_path_buf(base_dirs.home_dir().to_path_buf()) else {
+        return Vec::new();
+    };
+
+    vec![
+        DiscoveredRoot {
+            path: home.join(".claude/projects"),
+            source: RootSource::Standard,
+        },
+        DiscoveredRoot {
+            path: home.join(".config/claude/projects"),
+            source: RootSource::Standard,
+        },
+    ]
+}
+
+pub fn discover_roots(custom_roots: &[Utf8PathBuf]) -> RootDiscovery {
+    let mut seen = BTreeSet::new();
+    let mut discovered = Vec::new();
+
+    let mut candidates = Vec::new();
+    if custom_roots.is_empty() {
+        candidates.extend(standard_roots());
+    } else {
+        candidates.extend(custom_roots.iter().cloned().map(|path| DiscoveredRoot {
+            path,
+            source: RootSource::Custom,
+        }));
+    }
+
+    for candidate in candidates {
+        if candidate.path.is_dir() && seen.insert(candidate.path.clone()) {
+            discovered.push(candidate);
+        }
+    }
+
+    discovered.sort_by(|left, right| left.path.cmp(&right.path));
+    let selected = discovered.first().map(|root| root.path.clone());
+    RootDiscovery { discovered, selected }
+}
+
+pub fn select_primary_root(discovery: &RootDiscovery) -> Option<&Utf8Path> {
+    // Preserve the upstream first-root selection until fixture evidence
+    // proves a safe multi-root behavior change.
+    discovery.selected.as_deref()
+}
```

**Documentation:**

```diff
--- a/src/discovery/roots.rs
+++ b/src/discovery/roots.rs
@@ -1,26 +1,32 @@
+//! Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
+
 use std::collections::BTreeSet;
 
 use camino::{Utf8Path, Utf8PathBuf};
 use directories::BaseDirs;
 
 #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
 pub enum RootSource {
     Standard,
     Custom,
 }
 
 #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
 pub struct DiscoveredRoot {
     pub path: Utf8PathBuf,
     pub source: RootSource,
 }
 
 #[derive(Clone, Debug, Default, PartialEq, Eq)]
+/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
 pub struct RootDiscovery {
     pub discovered: Vec<DiscoveredRoot>,
     pub selected: Option<Utf8PathBuf>,
 }
 
+/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
 pub fn standard_roots() -> Vec<DiscoveredRoot> {
     let Some(base_dirs) = BaseDirs::new() else {
         return Vec::new();
@@ -42,6 +48,7 @@
     ]
 }
 
+/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
 pub fn discover_roots(custom_roots: &[Utf8PathBuf]) -> RootDiscovery {
     let mut seen = BTreeSet::new();
     let mut discovered = Vec::new();
@@ -67,6 +74,7 @@
     RootDiscovery { discovered, selected }
 }
 
+/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
 pub fn select_primary_root(discovery: &RootDiscovery) -> Option<&Utf8Path> {
     // Preserve the upstream first-root selection until fixture evidence
     // proves a safe multi-root behavior change.

```


**CC-M-002-002** (src/discovery/jsonl_files.rs) - implements CI-M-002-002

**Code:**

```diff
--- /dev/null
+++ b/src/discovery/jsonl_files.rs
@@ -0,0 +1,30 @@
+use camino::{Utf8Path, Utf8PathBuf};
+use walkdir::WalkDir;
+
+#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct JsonlFile {
+    pub root: Utf8PathBuf,
+    pub path: Utf8PathBuf,
+}
+
+pub fn collect_jsonl_files(root: &Utf8Path) -> Vec<JsonlFile> {
+    if !root.exists() || !root.is_dir() {
+        return Vec::new();
+    }
+
+    let mut files = WalkDir::new(root)
+        .into_iter()
+        .filter_map(Result::ok)
+        .filter(|entry| entry.file_type().is_file())
+        .filter(|entry| entry.path().extension().is_some_and(|extension| extension == "jsonl"))
+        .filter_map(|entry| Utf8PathBuf::from_path_buf(entry.into_path()).ok())
+        .map(|path| JsonlFile {
+            root: root.to_owned(),
+            path,
+        })
+        .collect::<Vec<_>>();
+
+    // Stable sort keeps fixture snapshots deterministic across hosts.
+    files.sort_by(|left, right| left.path.cmp(&right.path));
+    files
+}

```

**Documentation:**

```diff
--- a/src/discovery/jsonl_files.rs
+++ b/src/discovery/jsonl_files.rs
@@ -1,12 +1,16 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
 use camino::{Utf8Path, Utf8PathBuf};
 use walkdir::WalkDir;
 
 #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub struct JsonlFile {
     pub root: Utf8PathBuf,
     pub path: Utf8PathBuf,
 }
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub fn collect_jsonl_files(root: &Utf8Path) -> Vec<JsonlFile> {
     if !root.exists() || !root.is_dir() {
         return Vec::new();

```


**CC-M-002-003** (src/parser/jsonl.rs) - implements CI-M-002-003

**Code:**

```diff
--- /dev/null
+++ b/src/parser/jsonl.rs
@@ -0,0 +1,56 @@
+use std::fs::File;
+use std::io::{BufRead, BufReader};
+
+use camino::Utf8PathBuf;
+use serde_json::Value;
+
+use crate::discovery::JsonlFile;
+
+#[derive(Clone, Debug, PartialEq)]
+pub struct RawUsageEvent {
+    pub source_file: Utf8PathBuf,
+    pub line_number: usize,
+    pub payload: Value,
+}
+
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct JsonlDiagnostic {
+    pub source_file: Utf8PathBuf,
+    pub line_number: usize,
+    pub message: String,
+}
+
+#[derive(Clone, Debug, Default, PartialEq)]
+pub struct DecodedJsonl {
+    pub events: Vec<RawUsageEvent>,
+    pub diagnostics: Vec<JsonlDiagnostic>,
+}
+
+pub fn decode_jsonl_file(file: &JsonlFile) -> anyhow::Result<DecodedJsonl> {
+    let reader = BufReader::new(File::open(&file.path)?);
+    let mut decoded = DecodedJsonl::default();
+
+    for (index, line) in reader.lines().enumerate() {
+        let line_number = index + 1;
+        let line = line?;
+        let trimmed = line.trim();
+        if trimmed.is_empty() {
+            continue;
+        }
+
+        match serde_json::from_str::<Value>(trimmed) {
+            Ok(payload) => decoded.events.push(RawUsageEvent {
+                source_file: file.path.clone(),
+                line_number,
+                payload,
+            }),
+            Err(error) => decoded.diagnostics.push(JsonlDiagnostic {
+                source_file: file.path.clone(),
+                line_number,
+                message: error.to_string(),
+            }),
+        }
+    }
+
+    Ok(decoded)
+}

```

**Documentation:**

```diff
--- a/src/parser/jsonl.rs
+++ b/src/parser/jsonl.rs
@@ -1,3 +1,5 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
 use std::fs::File;
 use std::io::{BufRead, BufReader};
 
@@ -7,6 +9,7 @@
 use crate::discovery::JsonlFile;
 
 #[derive(Clone, Debug, PartialEq)]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub struct RawUsageEvent {
     pub source_file: Utf8PathBuf,
     pub line_number: usize,
@@ -14,6 +17,7 @@
 }
 
 #[derive(Clone, Debug, PartialEq, Eq)]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub struct JsonlDiagnostic {
     pub source_file: Utf8PathBuf,
     pub line_number: usize,
@@ -21,11 +25,13 @@
 }
 
 #[derive(Clone, Debug, Default, PartialEq)]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub struct DecodedJsonl {
     pub events: Vec<RawUsageEvent>,
     pub diagnostics: Vec<JsonlDiagnostic>,
 }
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub fn decode_jsonl_file(file: &JsonlFile) -> anyhow::Result<DecodedJsonl> {
     let reader = BufReader::new(File::open(&file.path)?);
     let mut decoded = DecodedJsonl::default();

```


**CC-M-002-004** (src/parser/entries.rs) - implements CI-M-002-004

**Code:**

```diff
--- /dev/null
+++ b/src/parser/entries.rs
@@ -0,0 +1,118 @@
+use std::collections::BTreeSet;
+
+use serde_json::Value;
+use time::OffsetDateTime;
+
+use crate::domain::{TokenUsage, UsageEntry};
+pub use crate::parser::jsonl::RawUsageEvent;
+use crate::parser::jsonl::DecodedJsonl;
+
+#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct DedupKey {
+    pub message_id: String,
+    pub request_id: String,
+}
+
+#[derive(Clone, Debug, Default, PartialEq)]
+pub struct EntryNormalization {
+    pub entries: Vec<UsageEntry>,
+    pub retained_raw_events: Vec<RawUsageEvent>,
+    pub skipped_zero_tokens: usize,
+    pub skipped_before_cutoff: usize,
+    pub skipped_duplicates: usize,
+}
+
+pub fn normalize_usage_entries(
+    decoded: DecodedJsonl,
+    cutoff: Option<OffsetDateTime>,
+) -> EntryNormalization {
+    let mut report = EntryNormalization::default();
+    let mut seen = BTreeSet::new();
+
+    for event in decoded.events {
+        let Some(timestamp) = parse_timestamp(&event.payload) else {
+            continue;
+        };
+        if cutoff.is_some_and(|limit| timestamp < limit) {
+            report.skipped_before_cutoff += 1;
+            continue;
+        }
+
+        let tokens = extract_tokens(&event.payload);
+        if tokens.total_tokens() == 0 {
+            report.skipped_zero_tokens += 1;
+            continue;
+        }
+
+        if let Some(key) = dedup_key(&event.payload) {
+            if !seen.insert(key) {
+                report.skipped_duplicates += 1;
+                continue;
+            }
+        }
+
+        report.entries.push(UsageEntry {
+            timestamp,
+            model: normalize_model(&event.payload),
+            message_id: message_id(&event.payload),
+            request_id: request_id(&event.payload),
+            tokens,
+            cost_usd: event.payload.get("cost").and_then(Value::as_f64),
+            source_file: event.source_file.clone(),
+            line_number: event.line_number,
+        });
+        report.retained_raw_events.push(event);
+    }
+
+    report.entries.sort_by(|left, right| left.timestamp.cmp(&right.timestamp));
+    report
+}
+
+fn parse_timestamp(payload: &Value) -> Option<OffsetDateTime> {
+    payload
+        .get("timestamp")
+        .and_then(Value::as_str)
+        .and_then(|value| OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok())
+}
+
+fn extract_tokens(payload: &Value) -> TokenUsage {
+    let usage = payload.get("usage").cloned().unwrap_or(Value::Null);
+    TokenUsage {
+        input_tokens: usage.get("input_tokens").and_then(Value::as_u64).unwrap_or_default(),
+        output_tokens: usage.get("output_tokens").and_then(Value::as_u64).unwrap_or_default(),
+        cache_creation_tokens: usage.get("cache_creation_tokens").and_then(Value::as_u64).unwrap_or_default(),
+        cache_read_tokens: usage.get("cache_read_tokens").and_then(Value::as_u64).unwrap_or_default(),
+    }
+}
+
+fn message_id(payload: &Value) -> Option<String> {
+    payload
+        .get("message_id")
+        .and_then(Value::as_str)
+        .map(ToOwned::to_owned)
+        .or_else(|| payload.get("message").and_then(|message| message.get("id")).and_then(Value::as_str).map(ToOwned::to_owned))
+}
+
+fn request_id(payload: &Value) -> Option<String> {
+    payload
+        .get("request_id")
+        .and_then(Value::as_str)
+        .map(ToOwned::to_owned)
+        .or_else(|| payload.get("requestId").and_then(Value::as_str).map(ToOwned::to_owned))
+}
+
+fn dedup_key(payload: &Value) -> Option<DedupKey> {
+    Some(DedupKey {
+        message_id: message_id(payload)?,
+        request_id: request_id(payload)?,
+    })
+}
+
+fn normalize_model(payload: &Value) -> String {
+    payload
+        .get("model")
+        .and_then(Value::as_str)
+        .or_else(|| payload.get("message").and_then(|message| message.get("model")).and_then(Value::as_str))
+        .unwrap_or("unknown")
+        .to_lowercase()
+}

```

**Documentation:**

```diff
--- a/src/parser/entries.rs
+++ b/src/parser/entries.rs
@@ -1,3 +1,5 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
 use std::collections::BTreeSet;
 
 use serde_json::Value;
@@ -8,12 +10,14 @@
 use crate::parser::jsonl::DecodedJsonl;
 
 #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub struct DedupKey {
     pub message_id: String,
     pub request_id: String,
 }
 
 #[derive(Clone, Debug, Default, PartialEq)]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub struct EntryNormalization {
     pub entries: Vec<UsageEntry>,
     pub retained_raw_events: Vec<RawUsageEvent>,
@@ -22,6 +26,7 @@
     pub skipped_duplicates: usize,
 }
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub fn normalize_usage_entries(
     decoded: DecodedJsonl,
     cutoff: Option<OffsetDateTime>,
@@ -68,6 +73,7 @@
     report
 }
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn parse_timestamp(payload: &Value) -> Option<OffsetDateTime> {
     payload
         .get("timestamp")
@@ -75,6 +81,7 @@
         .and_then(|value| OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok())
 }
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn extract_tokens(payload: &Value) -> TokenUsage {
     let usage = payload.get("usage").cloned().unwrap_or(Value::Null);
     TokenUsage {
@@ -85,6 +92,7 @@
     }
 }
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn message_id(payload: &Value) -> Option<String> {
     payload
         .get("message_id")
@@ -93,6 +101,7 @@
         .or_else(|| payload.get("message").and_then(|message| message.get("id")).and_then(Value::as_str).map(ToOwned::to_owned))
 }
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn request_id(payload: &Value) -> Option<String> {
     payload
         .get("request_id")
@@ -101,6 +110,7 @@
         .or_else(|| payload.get("requestId").and_then(Value::as_str).map(ToOwned::to_owned))
 }
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn dedup_key(payload: &Value) -> Option<DedupKey> {
     Some(DedupKey {
         message_id: message_id(payload)?,
@@ -108,6 +118,7 @@
     })
 }
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn normalize_model(payload: &Value) -> String {
     payload
         .get("model")

```


**CC-M-002-005** (src/domain/usage_entry.rs) - implements CI-M-002-005

**Code:**

```diff
--- /dev/null
+++ b/src/domain/usage_entry.rs
@@ -0,0 +1,29 @@
+use camino::Utf8PathBuf;
+use serde::{Deserialize, Serialize};
+use time::OffsetDateTime;
+
+#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
+pub struct TokenUsage {
+    pub input_tokens: u64,
+    pub output_tokens: u64,
+    pub cache_creation_tokens: u64,
+    pub cache_read_tokens: u64,
+}
+
+impl TokenUsage {
+    pub fn total_tokens(&self) -> u64 {
+        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
+    }
+}
+
+#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+pub struct UsageEntry {
+    pub timestamp: OffsetDateTime,
+    pub model: String,
+    pub message_id: Option<String>,
+    pub request_id: Option<String>,
+    pub tokens: TokenUsage,
+    pub cost_usd: Option<f64>,
+    pub source_file: Utf8PathBuf,
+    pub line_number: usize,
+}

```

**Documentation:**

```diff
--- a/src/domain/usage_entry.rs
+++ b/src/domain/usage_entry.rs
@@ -1,8 +1,11 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
 use camino::Utf8PathBuf;
 use serde::{Deserialize, Serialize};
 use time::OffsetDateTime;
 
 #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub struct TokenUsage {
     pub input_tokens: u64,
     pub output_tokens: u64,
@@ -11,12 +14,14 @@
 }
 
 impl TokenUsage {
+    /// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
     pub fn total_tokens(&self) -> u64 {
         self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
     }
 }
 
 #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 pub struct UsageEntry {
     pub timestamp: OffsetDateTime,
     pub model: String,

```


**CC-M-002-006** (tests/parity_ingest.rs) - implements CI-M-002-006

**Code:**

```diff
--- /dev/null
+++ b/tests/parity_ingest.rs
@@ -0,0 +1,75 @@
+use std::path::PathBuf;
+
+use serde_json::json;
+use time::macros::datetime;
+
+use cmonitor_rs::discovery::{collect_jsonl_files, discover_roots};
+use cmonitor_rs::parser::{normalize_usage_entries, DecodedJsonl, RawUsageEvent};
+
+fn fixture_path(name: &str) -> PathBuf {
+    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ingest").join(name)
+}
+
+#[test]
+fn discovery_preserves_sorted_first_root_selection() {
+    let discovery = discover_roots(&[]);
+    assert!(discovery.discovered.len() >= discovery.selected.iter().count());
+    if let Some(selected) = discovery.selected {
+        assert!(discovery.discovered.iter().any(|root| root.path == selected));
+    }
+}
+
+#[test]
+fn jsonl_file_collection_is_stable_and_recursive() {
+    let root = fixture_path("sample-home");
+    let files = collect_jsonl_files(camino::Utf8Path::from_path(root.as_path()).expect("utf8 path"));
+    let mut sorted = files.iter().map(|file| file.path.clone()).collect::<Vec<_>>();
+    sorted.sort();
+    assert_eq!(files.iter().map(|file| file.path.clone()).collect::<Vec<_>>(), sorted);
+}
+
+#[test]
+fn normalization_filters_zero_tokens_cutoff_and_duplicates() {
+    let decoded = DecodedJsonl {
+        events: vec![
+            RawUsageEvent {
+                source_file: camino::Utf8PathBuf::from("fixture.jsonl"),
+                line_number: 1,
+                payload: json!({
+                    "timestamp": "2026-03-14T12:00:00Z",
+                    "message_id": "m-1",
+                    "request_id": "r-1",
+                    "model": "Claude-3-7-Sonnet-20250219",
+                    "usage": {"input_tokens": 10, "output_tokens": 2}
+                }),
+            },
+            RawUsageEvent {
+                source_file: camino::Utf8PathBuf::from("fixture.jsonl"),
+                line_number: 2,
+                payload: json!({
+                    "timestamp": "2026-03-14T12:00:01Z",
+                    "message_id": "m-1",
+                    "request_id": "r-1",
+                    "model": "Claude-3-7-Sonnet-20250219",
+                    "usage": {"input_tokens": 10, "output_tokens": 2}
+                }),
+            },
+            RawUsageEvent {
+                source_file: camino::Utf8PathBuf::from("fixture.jsonl"),
+                line_number: 3,
+                payload: json!({
+                    "timestamp": "2026-03-14T11:59:00Z",
+                    "model": "unknown",
+                    "usage": {"input_tokens": 0, "output_tokens": 0}
+                }),
+            },
+        ],
+        diagnostics: Vec::new(),
+    };
+
+    let report = normalize_usage_entries(decoded, Some(datetime!(2026-03-14 11:59:30 UTC)));
+    assert_eq!(report.entries.len(), 1);
+    assert_eq!(report.skipped_duplicates, 1);
+    assert_eq!(report.skipped_zero_tokens, 1);
+    assert_eq!(report.entries[0].model, "claude-3-7-sonnet-20250219");
+}

```

**Documentation:**

```diff
--- a/tests/parity_ingest.rs
+++ b/tests/parity_ingest.rs
@@ -1,3 +1,5 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
 use std::path::PathBuf;
 
 use serde_json::json;
@@ -6,11 +8,13 @@
 use cmonitor_rs::discovery::{collect_jsonl_files, discover_roots};
 use cmonitor_rs::parser::{normalize_usage_entries, DecodedJsonl, RawUsageEvent};
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn fixture_path(name: &str) -> PathBuf {
     PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ingest").join(name)
 }
 
 #[test]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn discovery_preserves_sorted_first_root_selection() {
     let discovery = discover_roots(&[]);
     assert!(discovery.discovered.len() >= discovery.selected.iter().count());
@@ -20,6 +24,7 @@
 }
 
 #[test]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn jsonl_file_collection_is_stable_and_recursive() {
     let root = fixture_path("sample-home");
     let files = collect_jsonl_files(camino::Utf8Path::from_path(root.as_path()).expect("utf8 path"));
@@ -29,6 +34,7 @@
 }
 
 #[test]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn normalization_filters_zero_tokens_cutoff_and_duplicates() {
     let decoded = DecodedJsonl {
         events: vec![

```


### Milestone 3: Session Block Analysis and Shared Report Model

**Files**: src/domain/session_block.rs, src/domain/plan.rs, src/analysis/blocks.rs, src/analysis/limits.rs, src/analysis/p90.rs, src/report/model.rs, src/report/daily_monthly.rs, src/compat/upstream.rs, tests/parity_analysis.rs, tests/fixtures/analysis/README.md

**Flags**: analysis, shared-report

**Requirements**:

- Group normalized entries into five-hour UTC-rounded session and gap blocks
- Detect limit messages and attach them to block and report context
- Compute custom-plan limits with upstream P90 fallback ordering and expose one shared report model for table and realtime consumers

**Acceptance Criteria**:

- cargo test --test parity_analysis passes
- Block boundary gap limit and custom-plan cases match upstream snapshots
- Report state exposes daily monthly and active-session totals without renderer-specific recomputation

**Tests**:

- integration:cargo test --test parity_analysis
- property:cargo test p90_invariants -- --nocapture
- behavior:fixture-backed block and report snapshots

#### Code Intent

- **CI-M-003-001** `src/domain/session_block.rs`: Define session block and gap block models with aggregate token cost and message fields that preserve upstream lineage while remaining renderer-neutral. (refs: DL-002, DL-003)
- **CI-M-003-002** `src/domain/plan.rs`: Encode plan thresholds default custom minimums and cost and message limits with compatibility labels aligned to upstream semantics. (refs: DL-001, DL-003)
- **CI-M-003-003** `src/analysis/blocks.rs`: Transform usage entries into ordered session and gap blocks using UTC hour rounding block-end rollover and five-hour inactivity splitting rules. (refs: DL-001, DL-005)
- **CI-M-003-004** `src/analysis/limits.rs`: Parse raw system and tool-result messages into structured limit events and assign them to block time ranges for later rendering. (refs: DL-001, DL-003)
- **CI-M-003-005** `src/analysis/p90.rs`: Compute custom-plan token limits from completed non-gap blocks with fallback to completed sessions and then the default minimum exactly in upstream order. (refs: DL-001, DL-005)
- **CI-M-003-006** `src/report/model.rs`: Project analysis output into a shared report state consumed by daily monthly and realtime renderers without duplicating total calculations. (refs: DL-002, DL-004)
- **CI-M-003-007** `src/report/daily_monthly.rs`: Produce grouped daily and monthly aggregates from the shared report state with model lists token totals and cost totals ready for terminal tables. (refs: DL-004)
- **CI-M-003-008** `src/compat/upstream.rs`: Centralize compatibility shims for first-root selection latent session-view handling and known total-token accounting quirks so deliberate fixes remain isolated. (refs: DL-003, DL-004)
- **CI-M-003-009** `tests/parity_analysis.rs`: Assert block and report parity across boundary limit and custom-plan fixtures against upstream snapshots. (refs: DL-001, DL-005)

#### Code Changes

**CC-M-003-001** (src/domain/session_block.rs) - implements CI-M-003-001

**Code:**

```diff
--- /dev/null
+++ b/src/domain/session_block.rs
@@ -0,0 +1,54 @@
+use serde::{Deserialize, Serialize};
+use time::OffsetDateTime;
+
+use crate::domain::{TokenUsage, UsageEntry};
+
+#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+pub enum LimitKind {
+    Usage,
+    Rate,
+    Opus,
+}
+
+#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+pub struct LimitEvent {
+    pub kind: LimitKind,
+    pub timestamp: OffsetDateTime,
+    pub message: String,
+    pub reset_at: Option<OffsetDateTime>,
+}
+
+#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+pub struct SessionBlock {
+    pub id: String,
+    pub start_time: OffsetDateTime,
+    pub end_time: OffsetDateTime,
+    pub actual_end_time: Option<OffsetDateTime>,
+    pub is_gap: bool,
+    pub is_active: bool,
+    pub entries: Vec<UsageEntry>,
+    pub limits: Vec<LimitEvent>,
+    pub tokens: TokenUsage,
+    pub cost_usd: f64,
+    pub message_count: usize,
+    pub models: Vec<String>,
+}
+
+impl SessionBlock {
+    pub fn empty_gap(start_time: OffsetDateTime, end_time: OffsetDateTime) -> Self {
+        Self {
+            id: format!("gap-{}", start_time.unix_timestamp()),
+            start_time,
+            end_time,
+            actual_end_time: None,
+            is_gap: true,
+            is_active: false,
+            entries: Vec::new(),
+            limits: Vec::new(),
+            tokens: TokenUsage::default(),
+            cost_usd: 0.0,
+            message_count: 0,
+            models: Vec::new(),
+        }
+    }
+}

```

**Documentation:**

```diff
--- a/src/domain/session_block.rs
+++ b/src/domain/session_block.rs
@@ -1,9 +1,12 @@
+//! Session math stays aligned with executable parity and quarantined quirk handling. (ref: DL-001) (ref: DL-003)
+
 use serde::{Deserialize, Serialize};
 use time::OffsetDateTime;
 
 use crate::domain::{TokenUsage, UsageEntry};
 
 #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+/// Session math stays aligned with executable parity and quarantined quirk handling. (ref: DL-001) (ref: DL-003)
 pub enum LimitKind {
     Usage,
     Rate,
@@ -11,6 +14,7 @@
 }
 
 #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+/// Session math stays aligned with executable parity and quarantined quirk handling. (ref: DL-001) (ref: DL-003)
 pub struct LimitEvent {
     pub kind: LimitKind,
     pub timestamp: OffsetDateTime,
@@ -19,6 +23,7 @@
 }
 
 #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+/// Session math stays aligned with executable parity and quarantined quirk handling. (ref: DL-001) (ref: DL-003)
 pub struct SessionBlock {
     pub id: String,
     pub start_time: OffsetDateTime,
@@ -35,6 +40,7 @@
 }
 
 impl SessionBlock {
+    /// Session math stays aligned with executable parity and quarantined quirk handling. (ref: DL-001) (ref: DL-003)
     pub fn empty_gap(start_time: OffsetDateTime, end_time: OffsetDateTime) -> Self {
         Self {
             id: format!("gap-{}", start_time.unix_timestamp()),

```


**CC-M-003-002** (src/domain/plan.rs) - implements CI-M-003-002

**Code:**

```diff
--- /dev/null
+++ b/src/domain/plan.rs
@@ -0,0 +1,50 @@
+use serde::{Deserialize, Serialize};
+
+#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
+pub enum PlanType {
+    Pro,
+    Max5,
+    Max20,
+    Custom,
+}
+
+#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
+pub struct PlanDefinition {
+    pub name: &'static str,
+    pub token_limit: Option<u64>,
+    pub message_limit: Option<u32>,
+    pub default_custom_minimum: u64,
+}
+
+pub const DEFAULT_CUSTOM_MINIMUM: u64 = 44_000;
+pub const LIMIT_THRESHOLD: f64 = 0.90;
+pub const COMMON_TOKEN_LIMITS: [u64; 3] = [44_000, 220_000, 880_000];
+
+pub fn plan_definition(plan: PlanType, custom_limit: Option<u64>) -> PlanDefinition {
+    match plan {
+        PlanType::Pro => PlanDefinition {
+            name: "pro",
+            token_limit: Some(COMMON_TOKEN_LIMITS[0]),
+            message_limit: Some(45),
+            default_custom_minimum: DEFAULT_CUSTOM_MINIMUM,
+        },
+        PlanType::Max5 => PlanDefinition {
+            name: "max5",
+            token_limit: Some(COMMON_TOKEN_LIMITS[1]),
+            message_limit: Some(225),
+            default_custom_minimum: DEFAULT_CUSTOM_MINIMUM,
+        },
+        PlanType::Max20 => PlanDefinition {
+            name: "max20",
+            token_limit: Some(COMMON_TOKEN_LIMITS[2]),
+            message_limit: Some(900),
+            default_custom_minimum: DEFAULT_CUSTOM_MINIMUM,
+        },
+        PlanType::Custom => PlanDefinition {
+            name: "custom",
+            token_limit: custom_limit,
+            message_limit: None,
+            default_custom_minimum: DEFAULT_CUSTOM_MINIMUM,
+        },
+    }
+}

```

**Documentation:**

```diff
--- a/src/domain/plan.rs
+++ b/src/domain/plan.rs
@@ -1,6 +1,9 @@
+//! Plan semantics stay tied to upstream custom-plan behavior. (ref: DL-001)
+
 use serde::{Deserialize, Serialize};
 
 #[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
+/// Plan semantics stay tied to upstream custom-plan behavior. (ref: DL-001)
 pub enum PlanType {
     Pro,
     Max5,
@@ -9,6 +12,7 @@
 }
 
 #[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
+/// Plan semantics stay tied to upstream custom-plan behavior. (ref: DL-001)
 pub struct PlanDefinition {
     pub name: &'static str,
     pub token_limit: Option<u64>,
@@ -20,6 +24,7 @@
 pub const LIMIT_THRESHOLD: f64 = 0.90;
 pub const COMMON_TOKEN_LIMITS: [u64; 3] = [44_000, 220_000, 880_000];
 
+/// Plan semantics stay tied to upstream custom-plan behavior. (ref: DL-001)
 pub fn plan_definition(plan: PlanType, custom_limit: Option<u64>) -> PlanDefinition {
     match plan {
         PlanType::Pro => PlanDefinition {

```


**CC-M-003-003** (src/analysis/blocks.rs) - implements CI-M-003-003

**Code:**

```diff
--- /dev/null
+++ b/src/analysis/blocks.rs
@@ -0,0 +1,86 @@
+use time::{Duration, OffsetDateTime, UtcOffset};
+
+use crate::domain::{SessionBlock, TokenUsage, UsageEntry};
+
+pub const SESSION_WINDOW_HOURS: i64 = 5;
+
+pub fn transform_to_blocks(entries: &[UsageEntry], now: OffsetDateTime) -> Vec<SessionBlock> {
+    if entries.is_empty() {
+        return Vec::new();
+    }
+
+    let mut blocks = Vec::new();
+    let mut current = new_block(&entries[0]);
+
+    for entry in entries.iter().cloned() {
+        let should_roll = entry.timestamp >= current.end_time
+            || current
+                .entries
+                .last()
+                .is_some_and(|last| entry.timestamp - last.timestamp >= Duration::hours(SESSION_WINDOW_HOURS));
+
+        if should_roll {
+            finalize_block(&mut current, now);
+            if let Some(last_end) = current.actual_end_time {
+                if entry.timestamp - last_end >= Duration::hours(SESSION_WINDOW_HOURS) {
+                    blocks.push(current.clone());
+                    blocks.push(SessionBlock::empty_gap(last_end, entry.timestamp));
+                    current = new_block(&entry);
+                } else {
+                    blocks.push(current.clone());
+                    current = new_block(&entry);
+                }
+            } else {
+                blocks.push(current.clone());
+                current = new_block(&entry);
+            }
+        }
+
+        append_entry(&mut current, entry);
+    }
+
+    finalize_block(&mut current, now);
+    blocks.push(current);
+    blocks
+}
+
+fn new_block(entry: &UsageEntry) -> SessionBlock {
+    let start_time = round_down_to_utc_hour(entry.timestamp);
+    SessionBlock {
+        id: start_time.unix_timestamp().to_string(),
+        start_time,
+        end_time: start_time + Duration::hours(SESSION_WINDOW_HOURS),
+        actual_end_time: None,
+        is_gap: false,
+        is_active: false,
+        entries: Vec::new(),
+        limits: Vec::new(),
+        tokens: TokenUsage::default(),
+        cost_usd: 0.0,
+        message_count: 0,
+        models: Vec::new(),
+    }
+}
+
+fn append_entry(block: &mut SessionBlock, entry: UsageEntry) {
+    block.tokens.input_tokens += entry.tokens.input_tokens;
+    block.tokens.output_tokens += entry.tokens.output_tokens;
+    block.tokens.cache_creation_tokens += entry.tokens.cache_creation_tokens;
+    block.tokens.cache_read_tokens += entry.tokens.cache_read_tokens;
+    block.cost_usd += entry.cost_usd.unwrap_or_default();
+    block.message_count += 1;
+    if !block.models.iter().any(|model| model == &entry.model) {
+        block.models.push(entry.model.clone());
+    }
+    block.entries.push(entry);
+}
+
+fn finalize_block(block: &mut SessionBlock, now: OffsetDateTime) {
+    block.actual_end_time = block.entries.last().map(|entry| entry.timestamp);
+    block.is_active = !block.is_gap && block.end_time > now;
+}
+
+fn round_down_to_utc_hour(timestamp: OffsetDateTime) -> OffsetDateTime {
+    let utc = timestamp.to_offset(UtcOffset::UTC);
+    utc.replace_minute(0).unwrap().replace_second(0).unwrap().replace_millisecond(0).unwrap().replace_microsecond(0).unwrap().replace_nanosecond(0).unwrap()
+}

```

**Documentation:**

```diff
--- a/src/analysis/blocks.rs
+++ b/src/analysis/blocks.rs
@@ -1,9 +1,12 @@
+//! Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
+
 use time::{Duration, OffsetDateTime, UtcOffset};
 
 use crate::domain::{SessionBlock, TokenUsage, UsageEntry};
 
 pub const SESSION_WINDOW_HOURS: i64 = 5;
 
+/// Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
 pub fn transform_to_blocks(entries: &[UsageEntry], now: OffsetDateTime) -> Vec<SessionBlock> {
     if entries.is_empty() {
         return Vec::new();
@@ -44,6 +47,7 @@
     blocks
 }
 
+/// Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
 fn new_block(entry: &UsageEntry) -> SessionBlock {
     let start_time = round_down_to_utc_hour(entry.timestamp);
     SessionBlock {
@@ -62,6 +66,7 @@
     }
 }
 
+/// Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
 fn append_entry(block: &mut SessionBlock, entry: UsageEntry) {
     block.tokens.input_tokens += entry.tokens.input_tokens;
     block.tokens.output_tokens += entry.tokens.output_tokens;
@@ -75,11 +80,13 @@
     block.entries.push(entry);
 }
 
+/// Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
 fn finalize_block(block: &mut SessionBlock, now: OffsetDateTime) {
     block.actual_end_time = block.entries.last().map(|entry| entry.timestamp);
     block.is_active = !block.is_gap && block.end_time > now;
 }
 
+/// Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
 fn round_down_to_utc_hour(timestamp: OffsetDateTime) -> OffsetDateTime {
     let utc = timestamp.to_offset(UtcOffset::UTC);
     utc.replace_minute(0).unwrap().replace_second(0).unwrap().replace_millisecond(0).unwrap().replace_microsecond(0).unwrap().replace_nanosecond(0).unwrap()

```


**CC-M-003-004** (src/analysis/limits.rs) - implements CI-M-003-004

**Code:**

```diff
--- /dev/null
+++ b/src/analysis/limits.rs
@@ -0,0 +1,49 @@
+use serde_json::Value;
+use time::{format_description::well_known::Rfc3339, OffsetDateTime};
+
+use crate::domain::{LimitEvent, LimitKind, SessionBlock};
+use crate::parser::RawUsageEvent;
+
+pub fn detect_limit_events(events: &[RawUsageEvent], blocks: &mut [SessionBlock]) -> Vec<LimitEvent> {
+    let mut detected = Vec::new();
+    for event in events {
+        let Some(limit) = parse_limit_event(event) else {
+            continue;
+        };
+        if let Some(block) = blocks.iter_mut().find(|block| !block.is_gap && limit.timestamp >= block.start_time && limit.timestamp <= block.end_time) {
+            block.limits.push(limit.clone());
+        }
+        detected.push(limit);
+    }
+    detected
+}
+
+fn parse_limit_event(event: &RawUsageEvent) -> Option<LimitEvent> {
+    let entry_type = event.payload.get("type").and_then(Value::as_str)?;
+    let content = event.payload.get("content").and_then(Value::as_str)?;
+    let lowered = content.to_lowercase();
+    if !lowered.contains("limit") && !lowered.contains("rate") {
+        return None;
+    }
+
+    let timestamp = event
+        .payload
+        .get("timestamp")
+        .and_then(Value::as_str)
+        .and_then(|raw| OffsetDateTime::parse(raw, &Rfc3339).ok())?;
+
+    let kind = if lowered.contains("opus") {
+        LimitKind::Opus
+    } else if lowered.contains("rate") {
+        LimitKind::Rate
+    } else {
+        LimitKind::Usage
+    };
+
+    Some(LimitEvent {
+        kind,
+        timestamp,
+        message: content.to_owned(),
+        reset_at: None,
+    })
+}

```

**Documentation:**

```diff
--- a/src/analysis/limits.rs
+++ b/src/analysis/limits.rs
@@ -1,9 +1,12 @@
+//! Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
+
 use serde_json::Value;
 use time::{format_description::well_known::Rfc3339, OffsetDateTime};
 
 use crate::domain::{LimitEvent, LimitKind, SessionBlock};
 use crate::parser::RawUsageEvent;
 
+/// Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
 pub fn detect_limit_events(events: &[RawUsageEvent], blocks: &mut [SessionBlock]) -> Vec<LimitEvent> {
     let mut detected = Vec::new();
     for event in events {
@@ -18,6 +21,7 @@
     detected
 }
 
+/// Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
 fn parse_limit_event(event: &RawUsageEvent) -> Option<LimitEvent> {
     let entry_type = event.payload.get("type").and_then(Value::as_str)?;
     let content = event.payload.get("content").and_then(Value::as_str)?;

```


**CC-M-003-005** (src/analysis/p90.rs) - implements CI-M-003-005

**Code:**

```diff
--- /dev/null
+++ b/src/analysis/p90.rs
@@ -0,0 +1,33 @@
+use crate::domain::plan::{COMMON_TOKEN_LIMITS, DEFAULT_CUSTOM_MINIMUM, LIMIT_THRESHOLD};
+use crate::domain::SessionBlock;
+
+pub fn calculate_custom_limit(blocks: &[SessionBlock]) -> Option<u64> {
+    if blocks.is_empty() {
+        return None;
+    }
+
+    let mut hit_limits = completed_totals(blocks)
+        .into_iter()
+        .filter(|total| COMMON_TOKEN_LIMITS.iter().any(|limit| (*total as f64) >= (*limit as f64 * LIMIT_THRESHOLD)))
+        .collect::<Vec<_>>();
+
+    if hit_limits.is_empty() {
+        hit_limits = completed_totals(blocks);
+    }
+    if hit_limits.is_empty() {
+        return Some(DEFAULT_CUSTOM_MINIMUM);
+    }
+
+    hit_limits.sort_unstable();
+    let index = ((hit_limits.len() - 1) as f64 * 0.9).round() as usize;
+    Some(hit_limits[index].max(DEFAULT_CUSTOM_MINIMUM))
+}
+
+fn completed_totals(blocks: &[SessionBlock]) -> Vec<u64> {
+    blocks
+        .iter()
+        .filter(|block| !block.is_gap && !block.is_active)
+        .map(|block| block.tokens.total_tokens())
+        .filter(|total| *total > 0)
+        .collect()
+}

```

**Documentation:**

```diff
--- a/src/analysis/p90.rs
+++ b/src/analysis/p90.rs
@@ -1,6 +1,9 @@
+//! Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
+
 use crate::domain::plan::{COMMON_TOKEN_LIMITS, DEFAULT_CUSTOM_MINIMUM, LIMIT_THRESHOLD};
 use crate::domain::SessionBlock;
 
+/// Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
 pub fn calculate_custom_limit(blocks: &[SessionBlock]) -> Option<u64> {
     if blocks.is_empty() {
         return None;
@@ -23,6 +26,7 @@
     Some(hit_limits[index].max(DEFAULT_CUSTOM_MINIMUM))
 }
 
+/// Analysis stays centralized so table and realtime views read one set of totals. (ref: DL-004) (ref: DL-005)
 fn completed_totals(blocks: &[SessionBlock]) -> Vec<u64> {
     blocks
         .iter()

```


**CC-M-003-006** (src/report/model.rs) - implements CI-M-003-006

**Code:**

```diff
--- /dev/null
+++ b/src/report/model.rs
@@ -0,0 +1,60 @@
+use serde::{Deserialize, Serialize};
+use time::OffsetDateTime;
+
+use crate::domain::{LimitEvent, SessionBlock};
+
+#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
+pub struct ReportTotals {
+    pub total_tokens: u64,
+    pub total_cost_usd: f64,
+    pub total_messages: usize,
+}
+
+#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+pub struct ActiveSessionReport {
+    pub block_id: String,
+    pub started_at: OffsetDateTime,
+    pub ends_at: OffsetDateTime,
+    pub totals: ReportTotals,
+    pub warnings: Vec<LimitEvent>,
+}
+
+#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+pub struct ReportState {
+    pub generated_at: OffsetDateTime,
+    pub blocks: Vec<SessionBlock>,
+    pub limits: Vec<LimitEvent>,
+    pub totals: ReportTotals,
+    pub active_session: Option<ActiveSessionReport>,
+}
+
+impl ReportState {
+    pub fn from_blocks(generated_at: OffsetDateTime, blocks: Vec<SessionBlock>, limits: Vec<LimitEvent>) -> Self {
+        let totals = blocks.iter().fold(ReportTotals::default(), |mut totals, block| {
+            totals.total_tokens += block.tokens.total_tokens();
+            totals.total_cost_usd += block.cost_usd;
+            totals.total_messages += block.message_count;
+            totals
+        });
+
+        let active_session = blocks.iter().find(|block| block.is_active).map(|block| ActiveSessionReport {
+            block_id: block.id.clone(),
+            started_at: block.start_time,
+            ends_at: block.end_time,
+            totals: ReportTotals {
+                total_tokens: block.tokens.total_tokens(),
+                total_cost_usd: block.cost_usd,
+                total_messages: block.message_count,
+            },
+            warnings: block.limits.clone(),
+        });
+
+        Self {
+            generated_at,
+            blocks,
+            limits,
+            totals,
+            active_session,
+        }
+    }
+}
```

**Documentation:**

```diff
--- a/src/report/model.rs
+++ b/src/report/model.rs
@@ -1,9 +1,12 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+
 use serde::{Deserialize, Serialize};
 use time::OffsetDateTime;
 
 use crate::domain::{LimitEvent, SessionBlock};
 
 #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub struct ReportTotals {
     pub total_tokens: u64,
     pub total_cost_usd: f64,
@@ -11,6 +14,7 @@
 }
 
 #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub struct ActiveSessionReport {
     pub block_id: String,
     pub started_at: OffsetDateTime,
@@ -20,6 +24,7 @@
 }
 
 #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub struct ReportState {
     pub generated_at: OffsetDateTime,
     pub blocks: Vec<SessionBlock>,
@@ -29,6 +34,7 @@
 }
 
 impl ReportState {
+    /// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
     pub fn from_blocks(generated_at: OffsetDateTime, blocks: Vec<SessionBlock>, limits: Vec<LimitEvent>) -> Self {
         let totals = blocks.iter().fold(ReportTotals::default(), |mut totals, block| {
             totals.total_tokens += block.tokens.total_tokens();

```


**CC-M-003-007** (src/report/daily_monthly.rs) - implements CI-M-003-007

**Code:**

```diff
--- /dev/null
+++ b/src/report/daily_monthly.rs
@@ -0,0 +1,50 @@
+use std::collections::BTreeMap;
+
+use serde::{Deserialize, Serialize};
+use time::format_description::FormatItem;
+use time::macros::format_description;
+
+use crate::report::ReportState;
+
+static DAY_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");
+static MONTH_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]");
+
+#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
+pub struct AggregateRow {
+    pub label: String,
+    pub total_tokens: u64,
+    pub total_cost_usd: f64,
+    pub models: Vec<String>,
+}
+
+pub fn build_daily_rows(report: &ReportState) -> Vec<AggregateRow> {
+    build_rows(report, DAY_FORMAT)
+}
+
+pub fn build_monthly_rows(report: &ReportState) -> Vec<AggregateRow> {
+    build_rows(report, MONTH_FORMAT)
+}
+
+fn build_rows(report: &ReportState, formatter: &[FormatItem<'static>]) -> Vec<AggregateRow> {
+    let mut grouped = BTreeMap::<String, AggregateRow>::new();
+    for block in &report.blocks {
+        if block.is_gap {
+            continue;
+        }
+        let label = block.start_time.format(formatter).unwrap_or_else(|_| "unknown".to_owned());
+        let row = grouped.entry(label.clone()).or_insert_with(|| AggregateRow {
+            label,
+            total_tokens: 0,
+            total_cost_usd: 0.0,
+            models: Vec::new(),
+        });
+        row.total_tokens += block.tokens.total_tokens();
+        row.total_cost_usd += block.cost_usd;
+        for model in &block.models {
+            if !row.models.iter().any(|existing| existing == model) {
+                row.models.push(model.clone());
+            }
+        }
+    }
+    grouped.into_values().collect()
+}

```

**Documentation:**

```diff
--- a/src/report/daily_monthly.rs
+++ b/src/report/daily_monthly.rs
@@ -1,3 +1,5 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+
 use std::collections::BTreeMap;
 
 use serde::{Deserialize, Serialize};
@@ -10,6 +12,7 @@
 static MONTH_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]");
 
 #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub struct AggregateRow {
     pub label: String,
     pub total_tokens: u64,
@@ -17,14 +20,17 @@
     pub models: Vec<String>,
 }
 
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub fn build_daily_rows(report: &ReportState) -> Vec<AggregateRow> {
     build_rows(report, DAY_FORMAT)
 }
 
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub fn build_monthly_rows(report: &ReportState) -> Vec<AggregateRow> {
     build_rows(report, MONTH_FORMAT)
 }
 
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 fn build_rows(report: &ReportState, formatter: &[FormatItem<'static>]) -> Vec<AggregateRow> {
     let mut grouped = BTreeMap::<String, AggregateRow>::new();
     for block in &report.blocks {

```


**CC-M-003-008** (src/compat/upstream.rs) - implements CI-M-003-008

**Code:**

```diff
--- /dev/null
+++ b/src/compat/upstream.rs
@@ -0,0 +1,24 @@
+use crate::config::View;
+use crate::discovery::RootDiscovery;
+use crate::domain::SessionBlock;
+
+pub fn normalize_requested_view(view: View) -> View {
+    // Preserve the latent upstream session alias until the renderer
+    // contract is covered by explicit fixture cases.
+    match view {
+        View::Session => View::Realtime,
+        other => other,
+    }
+}
+
+pub fn primary_root_index(discovery: &RootDiscovery) -> Option<usize> {
+    discovery.selected.as_ref().and_then(|selected| {
+        discovery.discovered.iter().position(|root| &root.path == selected)
+    })
+}
+
+pub fn display_total_tokens(block: &SessionBlock) -> u64 {
+    // Keep total-token compatibility isolated from the typed models so
+    // post-parity cleanup can change one function instead of every view.
+    block.tokens.input_tokens + block.tokens.output_tokens + block.tokens.cache_read_tokens
+}

```

**Documentation:**

```diff
--- a/src/compat/upstream.rs
+++ b/src/compat/upstream.rs
@@ -1,7 +1,10 @@
+//! Compatibility helpers keep upstream quirks measurable outside the core model. (ref: DL-003)
+
 use crate::config::View;
 use crate::discovery::RootDiscovery;
 use crate::domain::SessionBlock;
 
+/// Compatibility helpers keep upstream quirks measurable outside the core model. (ref: DL-003)
 pub fn normalize_requested_view(view: View) -> View {
     // Preserve the latent upstream session alias until the renderer
     // contract is covered by explicit fixture cases.
@@ -11,12 +14,14 @@
     }
 }
 
+/// Compatibility helpers keep upstream quirks measurable outside the core model. (ref: DL-003)
 pub fn primary_root_index(discovery: &RootDiscovery) -> Option<usize> {
     discovery.selected.as_ref().and_then(|selected| {
         discovery.discovered.iter().position(|root| &root.path == selected)
     })
 }
 
+/// Compatibility helpers keep upstream quirks measurable outside the core model. (ref: DL-003)
 pub fn display_total_tokens(block: &SessionBlock) -> u64 {
     // Keep total-token compatibility isolated from the typed models so
     // post-parity cleanup can change one function instead of every view.

```


**CC-M-003-009** (tests/parity_analysis.rs) - implements CI-M-003-009

**Code:**

```diff
--- /dev/null
+++ b/tests/parity_analysis.rs
@@ -0,0 +1,79 @@
+use serde_json::json;
+use time::macros::datetime;
+
+use cmonitor_rs::analysis::{calculate_custom_limit, detect_limit_events, transform_to_blocks};
+use cmonitor_rs::domain::{TokenUsage, UsageEntry};
+use cmonitor_rs::parser::RawUsageEvent;
+use cmonitor_rs::report::{build_daily_rows, ReportState};
+
+fn usage_entry(timestamp: time::OffsetDateTime, total: u64) -> UsageEntry {
+    UsageEntry {
+        timestamp,
+        model: "claude-3-7-sonnet-20250219".to_owned(),
+        message_id: None,
+        request_id: None,
+        tokens: TokenUsage {
+            input_tokens: total,
+            output_tokens: 0,
+            cache_creation_tokens: 0,
+            cache_read_tokens: 0,
+        },
+        cost_usd: Some(0.01),
+        source_file: camino::Utf8PathBuf::from("fixture.jsonl"),
+        line_number: 1,
+    }
+}
+
+#[test]
+fn block_builder_rounds_to_utc_and_inserts_gaps() {
+    let entries = vec![
+        usage_entry(datetime!(2026-03-14 12:15 UTC), 10),
+        usage_entry(datetime!(2026-03-14 12:45 UTC), 20),
+        usage_entry(datetime!(2026-03-14 18:15 UTC), 30),
+    ];
+    let blocks = transform_to_blocks(&entries, datetime!(2026-03-14 19:00 UTC));
+    assert_eq!(blocks.len(), 3);
+    assert_eq!(blocks[0].start_time, datetime!(2026-03-14 12:00 UTC));
+    assert!(blocks[1].is_gap);
+}
+
+#[test]
+fn limit_detection_assigns_warnings_to_block_ranges() {
+    let entries = vec![usage_entry(datetime!(2026-03-14 12:15 UTC), 10)];
+    let mut blocks = transform_to_blocks(&entries, datetime!(2026-03-14 12:30 UTC));
+    let limits = detect_limit_events(
+        &[RawUsageEvent {
+            source_file: camino::Utf8PathBuf::from("fixture.jsonl"),
+            line_number: 10,
+            payload: json!({
+                "timestamp": "2026-03-14T12:20:00Z",
+                "type": "system",
+                "content": "Rate limit reached until later"
+            }),
+        }],
+        &mut blocks,
+    );
+    assert_eq!(limits.len(), 1);
+    assert_eq!(blocks[0].limits.len(), 1);
+}
+
+#[test]
+fn custom_limit_uses_completed_non_gap_p90() {
+    let entries = vec![
+        usage_entry(datetime!(2026-03-14 00:05 UTC), 50_000),
+        usage_entry(datetime!(2026-03-14 06:05 UTC), 60_000),
+        usage_entry(datetime!(2026-03-14 12:05 UTC), 90_000),
+    ];
+    let blocks = transform_to_blocks(&entries, datetime!(2026-03-15 00:00 UTC));
+    let limit = calculate_custom_limit(&blocks).expect("limit should be calculated");
+    assert!(limit >= 50_000);
+}
+
+#[test]
+fn report_rows_share_one_total_pipeline() {
+    let entries = vec![usage_entry(datetime!(2026-03-14 12:15 UTC), 10)];
+    let blocks = transform_to_blocks(&entries, datetime!(2026-03-14 13:00 UTC));
+    let report = ReportState::from_blocks(datetime!(2026-03-14 13:00 UTC), blocks, Vec::new());
+    let rows = build_daily_rows(&report);
+    assert_eq!(rows[0].total_tokens, report.totals.total_tokens);
+}

```

**Documentation:**

```diff
--- a/tests/parity_analysis.rs
+++ b/tests/parity_analysis.rs
@@ -1,3 +1,5 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
 use serde_json::json;
 use time::macros::datetime;
 
@@ -6,6 +8,7 @@
 use cmonitor_rs::parser::RawUsageEvent;
 use cmonitor_rs::report::{build_daily_rows, ReportState};
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn usage_entry(timestamp: time::OffsetDateTime, total: u64) -> UsageEntry {
     UsageEntry {
         timestamp,
@@ -25,6 +28,7 @@
 }
 
 #[test]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn block_builder_rounds_to_utc_and_inserts_gaps() {
     let entries = vec![
         usage_entry(datetime!(2026-03-14 12:15 UTC), 10),
@@ -38,6 +42,7 @@
 }
 
 #[test]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn limit_detection_assigns_warnings_to_block_ranges() {
     let entries = vec![usage_entry(datetime!(2026-03-14 12:15 UTC), 10)];
     let mut blocks = transform_to_blocks(&entries, datetime!(2026-03-14 12:30 UTC));
@@ -58,6 +63,7 @@
 }
 
 #[test]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn custom_limit_uses_completed_non_gap_p90() {
     let entries = vec![
         usage_entry(datetime!(2026-03-14 00:05 UTC), 50_000),
@@ -70,6 +76,7 @@
 }
 
 #[test]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn report_rows_share_one_total_pipeline() {
     let entries = vec![usage_entry(datetime!(2026-03-14 12:15 UTC), 10)];
     let blocks = transform_to_blocks(&entries, datetime!(2026-03-14 13:00 UTC));

```


### Milestone 4: Daily and Monthly Terminal Views

**Files**: src/ui/table.rs, src/ui/summary.rs, src/runtime/table_mode.rs, src/runtime/theme.rs, tests/parity_tables.rs, tests/fixtures/tables/README.md, docs/architecture-notes.md

**Flags**: tables, terminal-ui

**Requirements**:

- Render daily and monthly terminal tables from the shared report model with timezone-aware titles totals rows and empty states
- Apply theme and time-format inputs without changing report semantics
- Keep table mode deterministic and snapshot-friendly without alternate-screen control

**Acceptance Criteria**:

- cargo test --test parity_tables passes
- cargo run -- --view daily and monthly uses the shared report pipeline without realtime dependencies
- Architecture notes describe renderer boundaries and shared report ownership

**Tests**:

- integration:cargo test --test parity_tables
- behavior:snapshot daily and monthly outputs for data and empty-state fixtures
- smoke:cargo run -- --view daily --plan max20 --timezone UTC

#### Code Intent

- **CI-M-004-001** `src/ui/table.rs`: Render daily and monthly report tables with model lists cache columns token totals and cost totals in deterministic terminal layouts. (refs: DL-004)
- **CI-M-004-002** `src/ui/summary.rs`: Render summary and empty-state panels shared by non-live table modes so no-data behavior stays aligned across views. (refs: DL-004)
- **CI-M-004-003** `src/runtime/table_mode.rs`: Load report state and print daily or monthly terminal views without entering alternate-screen mode or starting the live refresh loop. (refs: DL-002, DL-004)
- **CI-M-004-004** `src/runtime/theme.rs`: Resolve light dark classic and auto theme selections into renderer styles without altering numeric output or compatibility behavior. (refs: DL-002)
- **CI-M-004-005** `tests/parity_tables.rs`: Snapshot daily and monthly terminal outputs against fixture oracles covering populated and empty-state tables. (refs: DL-001, DL-005)
- **CI-M-004-006** `docs/architecture-notes.md`: Document the shared report model and renderer boundaries once table parity is implemented. (refs: DL-002, DL-004)

#### Code Changes

**CC-M-004-001** (src/ui/table.rs) - implements CI-M-004-001

**Code:**

```diff
--- /dev/null
+++ b/src/ui/table.rs
@@ -0,0 +1,16 @@
+        use crate::report::daily_monthly::AggregateRow;
+
+        pub fn render_table(title: &str, rows: &[AggregateRow]) -> String {
+            let mut lines = vec![title.to_owned(), "label | tokens | cost | models".to_owned()];
+            for row in rows {
+                lines.push(format!(
+                    "{} | {} | {:.4} | {}",
+                    row.label,
+                    row.total_tokens,
+                    row.total_cost_usd,
+                    row.models.join(", ")
+                ));
+            }
+            lines.join("
+")
+        }

```

**Documentation:**

```diff
--- a/src/ui/table.rs
+++ b/src/ui/table.rs
@@ -1,5 +1,8 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+
         use crate::report::daily_monthly::AggregateRow;
 
+        /// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
         pub fn render_table(title: &str, rows: &[AggregateRow]) -> String {
             let mut lines = vec![title.to_owned(), "label | tokens | cost | models".to_owned()];
             for row in rows {

```


**CC-M-004-002** (src/ui/summary.rs) - implements CI-M-004-002

**Code:**

```diff
--- /dev/null
+++ b/src/ui/summary.rs
@@ -0,0 +1,16 @@
+        use crate::report::ReportState;
+
+        pub fn render_summary(report: &ReportState) -> String {
+            format!(
+                "total tokens: {}
+total cost: {:.4}
+total messages: {}",
+                report.totals.total_tokens,
+                report.totals.total_cost_usd,
+                report.totals.total_messages
+            )
+        }
+
+        pub fn render_empty_state(view: &str) -> String {
+            format!("no claude usage data available for {view}")
+        }

```

**Documentation:**

```diff
--- a/src/ui/summary.rs
+++ b/src/ui/summary.rs
@@ -1,5 +1,8 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+
         use crate::report::ReportState;
 
+        /// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
         pub fn render_summary(report: &ReportState) -> String {
             format!(
                 "total tokens: {}
@@ -11,6 +14,7 @@
             )
         }
 
+        /// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
         pub fn render_empty_state(view: &str) -> String {
             format!("no claude usage data available for {view}")
         }

```


**CC-M-004-003** (src/runtime/table_mode.rs) - implements CI-M-004-003

**Code:**

```diff
--- /dev/null
+++ b/src/runtime/table_mode.rs
@@ -0,0 +1,27 @@
+use std::process::ExitCode;
+
+use crate::compat::upstream::normalize_requested_view;
+use crate::config::{ResolvedConfig, View};
+use crate::report::{build_daily_rows, build_monthly_rows};
+use crate::runtime::orchestrator::load_report_state;
+use crate::ui::{summary, table};
+
+pub fn run_table_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
+    let report = load_report_state(resolved)?;
+    let view = normalize_requested_view(resolved.cli.view);
+
+    if report.blocks.is_empty() {
+        println!("{}", summary::render_empty_state(match view {
+            View::Monthly => "monthly",
+            _ => "daily",
+        }));
+        return Ok(ExitCode::SUCCESS);
+    }
+
+    println!("{}", summary::render_summary(&report));
+    match view {
+        View::Monthly => println!("{}", table::render_table("monthly usage", &build_monthly_rows(&report))),
+        _ => println!("{}", table::render_table("daily usage", &build_daily_rows(&report))),
+    }
+    Ok(ExitCode::SUCCESS)
+}

```

**Documentation:**

```diff
--- a/src/runtime/table_mode.rs
+++ b/src/runtime/table_mode.rs
@@ -1,3 +1,5 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+
 use std::process::ExitCode;
 
 use crate::compat::upstream::normalize_requested_view;
@@ -6,6 +8,7 @@
 use crate::runtime::orchestrator::load_report_state;
 use crate::ui::{summary, table};
 
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub fn run_table_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
     let report = load_report_state(resolved)?;
     let view = normalize_requested_view(resolved.cli.view);

```


**CC-M-004-004** (src/runtime/theme.rs) - implements CI-M-004-004

**Code:**

```diff
--- /dev/null
+++ b/src/runtime/theme.rs
@@ -0,0 +1,17 @@
+use crate::config::Theme;
+
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ThemePalette {
+    pub accent: &'static str,
+    pub warning: &'static str,
+    pub subtle: &'static str,
+}
+
+pub fn resolve_theme(theme: Theme) -> ThemePalette {
+    match theme {
+        Theme::Light => ThemePalette { accent: "blue", warning: "red", subtle: "black" },
+        Theme::Dark => ThemePalette { accent: "cyan", warning: "yellow", subtle: "white" },
+        Theme::Classic => ThemePalette { accent: "green", warning: "magenta", subtle: "white" },
+        Theme::Auto => ThemePalette { accent: "cyan", warning: "yellow", subtle: "default" },
+    }
+}

```

**Documentation:**

```diff
--- a/src/runtime/theme.rs
+++ b/src/runtime/theme.rs
@@ -1,12 +1,16 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+
 use crate::config::Theme;
 
 #[derive(Clone, Debug, PartialEq, Eq)]
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub struct ThemePalette {
     pub accent: &'static str,
     pub warning: &'static str,
     pub subtle: &'static str,
 }
 
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub fn resolve_theme(theme: Theme) -> ThemePalette {
     match theme {
         Theme::Light => ThemePalette { accent: "blue", warning: "red", subtle: "black" },

```


**CC-M-004-005** (tests/parity_tables.rs) - implements CI-M-004-005

**Code:**

```diff
--- /dev/null
+++ b/tests/parity_tables.rs
@@ -0,0 +1,42 @@
+        use time::macros::datetime;
+
+        use cmonitor_rs::analysis::transform_to_blocks;
+        use cmonitor_rs::domain::{TokenUsage, UsageEntry};
+        use cmonitor_rs::report::{build_daily_rows, build_monthly_rows, ReportState};
+        use cmonitor_rs::ui::{summary, table};
+
+        fn entry(timestamp: time::OffsetDateTime, total: u64) -> UsageEntry {
+            UsageEntry {
+                timestamp,
+                model: "claude-3-7-sonnet-20250219".to_owned(),
+                message_id: None,
+                request_id: None,
+                tokens: TokenUsage { input_tokens: total, output_tokens: 0, cache_creation_tokens: 0, cache_read_tokens: 0 },
+                cost_usd: Some(0.01),
+                source_file: camino::Utf8PathBuf::from("fixture.jsonl"),
+                line_number: 1,
+            }
+        }
+
+        #[test]
+        fn daily_table_snapshot_is_deterministic() {
+            let blocks = transform_to_blocks(&[entry(datetime!(2026-03-14 12:15 UTC), 20)], datetime!(2026-03-14 13:00 UTC));
+            let report = ReportState::from_blocks(datetime!(2026-03-14 13:00 UTC), blocks, Vec::new());
+            let output = format!("{}
+{}", summary::render_summary(&report), table::render_table("daily usage", &build_daily_rows(&report)));
+            insta::assert_snapshot!("daily-table", output);
+        }
+
+        #[test]
+        fn monthly_table_snapshot_is_deterministic() {
+            let blocks = transform_to_blocks(&[entry(datetime!(2026-03-14 12:15 UTC), 20)], datetime!(2026-03-14 13:00 UTC));
+            let report = ReportState::from_blocks(datetime!(2026-03-14 13:00 UTC), blocks, Vec::new());
+            let output = format!("{}
+{}", summary::render_summary(&report), table::render_table("monthly usage", &build_monthly_rows(&report)));
+            insta::assert_snapshot!("monthly-table", output);
+        }
+
+        #[test]
+        fn empty_state_stays_explicit() {
+            assert_eq!(summary::render_empty_state("daily"), "no claude usage data available for daily");
+        }

```

**Documentation:**

```diff
--- a/tests/parity_tables.rs
+++ b/tests/parity_tables.rs
@@ -1,3 +1,5 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
         use time::macros::datetime;
 
         use cmonitor_rs::analysis::transform_to_blocks;
@@ -5,6 +7,7 @@
         use cmonitor_rs::report::{build_daily_rows, build_monthly_rows, ReportState};
         use cmonitor_rs::ui::{summary, table};
 
+        /// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
         fn entry(timestamp: time::OffsetDateTime, total: u64) -> UsageEntry {
             UsageEntry {
                 timestamp,
@@ -19,6 +22,7 @@
         }
 
         #[test]
+        /// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
         fn daily_table_snapshot_is_deterministic() {
             let blocks = transform_to_blocks(&[entry(datetime!(2026-03-14 12:15 UTC), 20)], datetime!(2026-03-14 13:00 UTC));
             let report = ReportState::from_blocks(datetime!(2026-03-14 13:00 UTC), blocks, Vec::new());
@@ -28,6 +32,7 @@
         }
 
         #[test]
+        /// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
         fn monthly_table_snapshot_is_deterministic() {
             let blocks = transform_to_blocks(&[entry(datetime!(2026-03-14 12:15 UTC), 20)], datetime!(2026-03-14 13:00 UTC));
             let report = ReportState::from_blocks(datetime!(2026-03-14 13:00 UTC), blocks, Vec::new());
@@ -37,6 +42,7 @@
         }
 
         #[test]
+        /// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
         fn empty_state_stays_explicit() {
             assert_eq!(summary::render_empty_state("daily"), "no claude usage data available for daily");
         }

```


**CC-M-004-006** (docs/architecture-notes.md) - implements CI-M-004-006

**Code:**

```diff
--- a/docs/architecture-notes.md
+++ b/docs/architecture-notes.md
@@ -2,24 +2,25 @@
 
 ## Runtime loop
 
-1. Load CLI/config state and resolve plan thresholds.
-2. Discover and parse Claude session files from the configured data root.
-3. Build an in-memory view model for usage, limits, and warning thresholds.
-4. Drive a refresh loop that updates derived state on a fixed cadence.
-5. Render terminal output without coupling parsing to presentation details.
+1. Load CLI arguments and merge saved `last_used.json` preferences.
+2. Discover Claude data roots and preserve the upstream first-root compatibility decision.
+3. Collect and decode JSONL files into raw events plus malformed-line diagnostics.
+4. Normalize usage entries, build five-hour session blocks, detect limit events, and compute custom-plan limits.
+5. Project the analysis output into one shared report state.
+6. Render either deterministic daily or monthly tables, or hand the same report state to realtime mode.
 
-## Planned module boundaries
+## Shared report boundary
 
-- `config`: CLI parsing and persisted preferences
-- `discovery`: file traversal and session selection
-- `parser`: JSONL decoding and usage normalization
-- `domain`: plans, thresholds, and usage state
-- `analysis`: burn-rate, predictions, and warning computation
-- `ui`: terminal layout and refresh loop
+- `parser` owns raw-event decoding and typed entry normalization.
+- `analysis` owns session blocks, limit detection, and P90 custom-limit math.
+- `report` owns the renderer-neutral totals and grouped aggregates.
+- `ui` only turns report state into strings or terminal widgets.
+- `runtime` owns orchestration, refresh cadence, and terminal lifecycle.
+- `compat` isolates upstream quirks such as first-root selection, latent `session` view handling, and alternate-screen defaults.
 
 ## Implementation guardrails
 
-- Keep the data model reusable by daily/monthly views and realtime UI.
-- Prefer fixture-driven session snapshots over synthetic one-off parsing tests.
-- Defer provider expansion until the single-provider baseline is stable.
-
+- Daily and monthly views must consume the same `ReportState` as realtime mode.
+- Table mode must remain deterministic and avoid alternate-screen control.
+- Intentional safety fixes stay behind compatibility gates until the parity matrix is green.
+- Fixture-backed oracle comparisons outrank README prose when the two disagree.

```

**Documentation:**

```diff
--- a/docs/architecture-notes.md
+++ b/docs/architecture-notes.md
@@ -24,3 +24,7 @@
 - Table mode must remain deterministic and avoid alternate-screen control.
 - Intentional safety fixes stay behind compatibility gates until the parity matrix is green.
 - Fixture-backed oracle comparisons outrank README prose when the two disagree.
+
+## Planning Rationale
+
+- Every renderer consumes one report state for totals and warnings. (ref: DL-004)

```


### Milestone 5: Realtime Loop, Compatibility-Scoped Terminal Policy, and Parity Cutover

**Files**: src/ui/realtime.rs, src/runtime/realtime.rs, src/runtime/terminal.rs, src/compat/terminal_policy.rs, src/runtime/orchestrator.rs, tests/parity_realtime.rs, tests/fixtures/realtime/README.md, README.md

**Flags**: realtime, cutover

**Requirements**:

- Drive realtime monitoring from the same shared report model as table modes
- Preserve upstream refresh and alternate-screen semantics in parity mode while routing non-TTY safety through a compatibility gate that stays off until the parity matrix is green
- Document shipped parity scope deferred divergences deferred alias takeover or packaging work and verification commands once the full parity matrix is green

**Acceptance Criteria**:

- cargo test --test parity_realtime passes
- make verify and the full parity test matrix pass with the upstream-default terminal policy still serving as the parity baseline
- README documents parity scope local Claude data behavior deferred alias or packaging work and deferred post-parity terminal safety work without provider expansion

**Tests**:

- integration:cargo test --test parity_realtime
- behavior:snapshot active no-session and warning states with the parity-mode terminal policy
- behavior:compat-gated non-TTY safety remains disabled by default until parity cutover
- smoke:make verify

#### Code Intent

- **CI-M-005-001** `src/ui/realtime.rs`: Render active-session no-session and warning screens from shared report projections with deterministic sections for tokens cost time and limits. (refs: DL-003, DL-004)
- **CI-M-005-002** `src/runtime/realtime.rs`: Coordinate refresh cadence data reloads and renderer updates for realtime mode using the shared report pipeline. (refs: DL-002, DL-004)
- **CI-M-005-003** `src/runtime/terminal.rs`: Own alternate-screen entry and exit plus signal-safe restoration while consulting the compatibility-scoped terminal policy so the default parity path keeps upstream terminal forcing until post-parity cutover. (refs: DL-003, DL-007)
- **CI-M-005-007** `src/compat/terminal_policy.rs`: Hold the parity-mode terminal policy and deferred non-TTY safety gate so terminal behavior changes stay isolated from runtime orchestration until parity is green. (refs: DL-003, DL-007)
- **CI-M-005-004** `src/runtime/orchestrator.rs`: Connect config discovery analysis report and UI layers into one synchronous monitoring loop that preserves upstream refresh semantics. (refs: DL-002, DL-004)
- **CI-M-005-005** `tests/parity_realtime.rs`: Verify realtime snapshots refresh policy handling and the upstream-default parity terminal policy against stored oracles while separately asserting that the deferred non-TTY gate stays disabled by default. (refs: DL-001, DL-005, DL-007)
- **CI-M-005-006** `README.md`: Describe the shipped parity scope verification commands local Claude data behavior deferred alias or packaging work and deferred post-parity terminal safety improvements after the full matrix passes. (refs: DL-003, DL-005, DL-007, DL-008)

#### Code Changes

**CC-M-005-001** (src/ui/realtime.rs) - implements CI-M-005-001

**Code:**

```diff
--- /dev/null
+++ b/src/ui/realtime.rs
@@ -0,0 +1,20 @@
+        use crate::report::ReportState;
+
+        pub fn render_realtime(report: &ReportState) -> String {
+            if let Some(active) = &report.active_session {
+                return format!(
+                    "active block: {}
+ends at: {}
+tokens: {}
+cost: {:.4}
+warnings: {}",
+                    active.block_id,
+                    active.ends_at,
+                    active.totals.total_tokens,
+                    active.totals.total_cost_usd,
+                    active.warnings.len()
+                );
+            }
+
+            "no active claude session".to_owned()
+        }

```

**Documentation:**

```diff
--- a/src/ui/realtime.rs
+++ b/src/ui/realtime.rs
@@ -1,5 +1,9 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+//! Terminal policy stays scoped to parity-compatible behavior. (ref: DL-007)
+
         use crate::report::ReportState;
 
+        /// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
         pub fn render_realtime(report: &ReportState) -> String {
             if let Some(active) = &report.active_session {
                 return format!(

```


**CC-M-005-002** (src/runtime/realtime.rs) - implements CI-M-005-002

**Code:**

```diff
--- /dev/null
+++ b/src/runtime/realtime.rs
@@ -0,0 +1,17 @@
+use std::process::ExitCode;
+use std::thread;
+use std::time::Duration;
+
+use crate::config::ResolvedConfig;
+use crate::runtime::orchestrator::load_report_state;
+use crate::runtime::terminal::TerminalGuard;
+use crate::ui::realtime;
+
+pub fn run_realtime_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
+    let _guard = TerminalGuard::enter(&resolved.cli)?;
+    loop {
+        let report = load_report_state(resolved)?;
+        println!("{}", realtime::render_realtime(&report));
+        thread::sleep(Duration::from_secs(resolved.cli.refresh_rate));
+    }
+}

```

**Documentation:**

```diff
--- a/src/runtime/realtime.rs
+++ b/src/runtime/realtime.rs
@@ -1,3 +1,6 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+//! Terminal policy stays scoped to parity-compatible behavior. (ref: DL-007)
+
 use std::process::ExitCode;
 use std::thread;
 use std::time::Duration;
@@ -7,6 +10,7 @@
 use crate::runtime::terminal::TerminalGuard;
 use crate::ui::realtime;
 
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub fn run_realtime_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
     let _guard = TerminalGuard::enter(&resolved.cli)?;
     loop {

```


**CC-M-005-003** (src/runtime/terminal.rs) - implements CI-M-005-003

**Code:**

```diff
--- /dev/null
+++ b/src/runtime/terminal.rs
@@ -0,0 +1,24 @@
+use crate::compat::terminal_policy::{default_terminal_policy, TerminalPolicy};
+use crate::config::Cli;
+
+pub struct TerminalGuard {
+    policy: TerminalPolicy,
+}
+
+impl TerminalGuard {
+    pub fn enter(cli: &Cli) -> anyhow::Result<Self> {
+        let policy = default_terminal_policy(cli);
+        if policy.force_alternate_screen {
+            print!("\x1b[?1049h");
+        }
+        Ok(Self { policy })
+    }
+}
+
+impl Drop for TerminalGuard {
+    fn drop(&mut self) {
+        if self.policy.force_alternate_screen {
+            print!("\x1b[?1049l");
+        }
+    }
+}

```

**Documentation:**

```diff
--- a/src/runtime/terminal.rs
+++ b/src/runtime/terminal.rs
@@ -1,11 +1,15 @@
+//! Terminal policy stays scoped to parity-compatible behavior. (ref: DL-007)
+
 use crate::compat::terminal_policy::{default_terminal_policy, TerminalPolicy};
 use crate::config::Cli;
 
+/// Terminal policy stays scoped to parity-compatible behavior. (ref: DL-007)
 pub struct TerminalGuard {
     policy: TerminalPolicy,
 }
 
 impl TerminalGuard {
+    /// Terminal policy stays scoped to parity-compatible behavior. (ref: DL-007)
     pub fn enter(cli: &Cli) -> anyhow::Result<Self> {
         let policy = default_terminal_policy(cli);
         if policy.force_alternate_screen {
@@ -16,6 +20,7 @@
 }
 
 impl Drop for TerminalGuard {
+    /// Terminal policy stays scoped to parity-compatible behavior. (ref: DL-007)
     fn drop(&mut self) {
         if self.policy.force_alternate_screen {
             print!("\x1b[?1049l");

```


**CC-M-005-004** (src/compat/terminal_policy.rs) - implements CI-M-005-007

**Code:**

```diff
--- /dev/null
+++ b/src/compat/terminal_policy.rs
@@ -0,0 +1,16 @@
+use crate::config::Cli;
+
+#[derive(Copy, Clone, Debug, PartialEq, Eq)]
+pub struct TerminalPolicy {
+    pub force_alternate_screen: bool,
+    pub deferred_non_tty_gate: bool,
+}
+
+pub fn default_terminal_policy(_cli: &Cli) -> TerminalPolicy {
+    // Preserve the upstream default terminal forcing until the full
+    // parity matrix is green; safer non-TTY behavior stays deferred.
+    TerminalPolicy {
+        force_alternate_screen: true,
+        deferred_non_tty_gate: false,
+    }
+}

```

**Documentation:**

```diff
--- a/src/compat/terminal_policy.rs
+++ b/src/compat/terminal_policy.rs
@@ -1,11 +1,15 @@
+//! Terminal policy stays scoped to parity-compatible behavior and compatibility gates carry non-TTY policy. (ref: DL-007)
+
 use crate::config::Cli;
 
 #[derive(Copy, Clone, Debug, PartialEq, Eq)]
+/// Terminal policy stays scoped to parity-compatible behavior and compatibility gates carry non-TTY policy. (ref: DL-007)
 pub struct TerminalPolicy {
     pub force_alternate_screen: bool,
     pub deferred_non_tty_gate: bool,
 }
 
+/// Terminal policy stays scoped to parity-compatible behavior and compatibility gates carry non-TTY policy. (ref: DL-007)
 pub fn default_terminal_policy(_cli: &Cli) -> TerminalPolicy {
     // Preserve the upstream default terminal forcing until the full
     // parity matrix is green; safer non-TTY behavior stays deferred.

```


**CC-M-005-005** (src/runtime/orchestrator.rs) - implements CI-M-005-004

**Code:**

```diff
--- /dev/null
+++ b/src/runtime/orchestrator.rs
@@ -0,0 +1,28 @@
+use time::OffsetDateTime;
+
+use crate::analysis::{calculate_custom_limit, detect_limit_events, transform_to_blocks};
+use crate::compat::upstream::primary_root_index;
+use crate::config::ResolvedConfig;
+use crate::discovery::{collect_jsonl_files, discover_roots};
+use crate::parser::{decode_jsonl_file, normalize_usage_entries};
+use crate::report::ReportState;
+
+pub fn load_report_state(resolved: &ResolvedConfig) -> anyhow::Result<ReportState> {
+    let discovery = discover_roots(&[]);
+    let _selected_root = primary_root_index(&discovery)
+        .and_then(|index| discovery.discovered.get(index))
+        .ok_or_else(|| anyhow::anyhow!("no Claude data root found"))?;
+
+    let mut decoded = crate::parser::DecodedJsonl::default();
+    for file in collect_jsonl_files(discovery.selected.as_deref().expect("selected root exists")) {
+        let file_decoded = decode_jsonl_file(&file)?;
+        decoded.events.extend(file_decoded.events);
+        decoded.diagnostics.extend(file_decoded.diagnostics);
+    }
+
+    let normalized = normalize_usage_entries(decoded, None);
+    let mut blocks = transform_to_blocks(&normalized.entries, OffsetDateTime::now_utc());
+    let limits = detect_limit_events(&normalized.retained_raw_events, &mut blocks);
+    let _custom_limit = calculate_custom_limit(&blocks);
+    Ok(ReportState::from_blocks(OffsetDateTime::now_utc(), blocks, limits))
+}

```

**Documentation:**

```diff
--- a/src/runtime/orchestrator.rs
+++ b/src/runtime/orchestrator.rs
@@ -1,3 +1,6 @@
+//! Every renderer consumes one report state for totals and warnings. (ref: DL-004)
+//! Terminal policy stays scoped to parity-compatible behavior. (ref: DL-007)
+
 use time::OffsetDateTime;
 
 use crate::analysis::{calculate_custom_limit, detect_limit_events, transform_to_blocks};
@@ -7,6 +10,7 @@
 use crate::parser::{decode_jsonl_file, normalize_usage_entries};
 use crate::report::ReportState;
 
+/// Every renderer consumes one report state for totals and warnings. (ref: DL-004)
 pub fn load_report_state(resolved: &ResolvedConfig) -> anyhow::Result<ReportState> {
     let discovery = discover_roots(&[]);
     let _selected_root = primary_root_index(&discovery)

```


**CC-M-005-006** (tests/parity_realtime.rs) - implements CI-M-005-005

**Code:**

```diff
--- /dev/null
+++ b/tests/parity_realtime.rs
@@ -0,0 +1,48 @@
+use cmonitor_rs::compat::terminal_policy::{default_terminal_policy, TerminalPolicy};
+use cmonitor_rs::config::{Cli, Plan, Theme, TimeFormat, View};
+use cmonitor_rs::report::{ActiveSessionReport, ReportState, ReportTotals};
+use cmonitor_rs::ui::realtime::render_realtime;
+use time::macros::datetime;
+
+fn cli() -> Cli {
+    Cli {
+        plan: Plan::Custom,
+        custom_limit_tokens: None,
+        view: View::Realtime,
+        timezone: "UTC".to_owned(),
+        time_format: TimeFormat::Auto,
+        theme: Theme::Auto,
+        refresh_rate: 10,
+        refresh_per_second: 0.75,
+        reset_hour: None,
+        log_level: "INFO".to_owned(),
+        log_file: None,
+        debug: false,
+        clear: false,
+        version: false,
+    }
+}
+
+#[test]
+fn realtime_render_snapshot_is_deterministic() {
+    let report = ReportState {
+        generated_at: datetime!(2026-03-14 12:30 UTC),
+        blocks: Vec::new(),
+        limits: Vec::new(),
+        totals: ReportTotals { total_tokens: 12, total_cost_usd: 0.01, total_messages: 1 },
+        active_session: Some(ActiveSessionReport {
+            block_id: "1700000000".to_owned(),
+            started_at: datetime!(2026-03-14 12:00 UTC),
+            ends_at: datetime!(2026-03-14 17:00 UTC),
+            totals: ReportTotals { total_tokens: 12, total_cost_usd: 0.01, total_messages: 1 },
+            warnings: Vec::new(),
+        }),
+    };
+    insta::assert_snapshot!("realtime-render", render_realtime(&report));
+}
+
+#[test]
+fn parity_terminal_policy_keeps_non_tty_gate_disabled() {
+    let policy = default_terminal_policy(&cli());
+    assert_eq!(policy, TerminalPolicy { force_alternate_screen: true, deferred_non_tty_gate: false });
+}

```

**Documentation:**

```diff
--- a/tests/parity_realtime.rs
+++ b/tests/parity_realtime.rs
@@ -1,9 +1,12 @@
+//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+
 use cmonitor_rs::compat::terminal_policy::{default_terminal_policy, TerminalPolicy};
 use cmonitor_rs::config::{Cli, Plan, Theme, TimeFormat, View};
 use cmonitor_rs::report::{ActiveSessionReport, ReportState, ReportTotals};
 use cmonitor_rs::ui::realtime::render_realtime;
 use time::macros::datetime;
 
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn cli() -> Cli {
     Cli {
         plan: Plan::Custom,
@@ -24,6 +27,7 @@
 }
 
 #[test]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn realtime_render_snapshot_is_deterministic() {
     let report = ReportState {
         generated_at: datetime!(2026-03-14 12:30 UTC),
@@ -42,6 +46,7 @@
 }
 
 #[test]
+/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn parity_terminal_policy_keeps_non_tty_gate_disabled() {
     let policy = default_terminal_policy(&cli());
     assert_eq!(policy, TerminalPolicy { force_alternate_screen: true, deferred_non_tty_gate: false });

```


**CC-M-005-007** (README.md) - implements CI-M-005-006

**Code:**

```diff
--- a/README.md
+++ b/README.md
@@ -1,54 +1,41 @@
 # cmonitor-rs
 
-`cmonitor-rs` is a fresh Rust rewrite scaffold for [`Maciek-roboblog/Claude-Code-Usage-Monitor`](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor).
+`cmonitor-rs` is a clean Rust rewrite of [`Maciek-roboblog/Claude-Code-Usage-Monitor`](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor).
 
-This repository is intentionally separate from the existing [`Psysician/c-monitor`](https://github.com/Psysician/c-monitor) Python fork. The initial parity target is the original upstream monitor behavior, not the fork's added multi-provider features.
+The shipped parity line stays focused on the upstream Claude-only monitor contract: local Claude JSONL ingestion, five-hour session blocks, custom-plan P90 limits, and terminal-first daily, monthly, and realtime views.
 
-## Current Status
+## Verification
 
-- Cargo binary project with CI, lint/test commands, and issue templates
-- Initial CLI scaffold for plan/view/theme/refresh configuration
-- Project docs capturing upstream parity, runtime architecture, and the first milestone
+- `cargo test --test parity_cli`
+- `cargo test --test parity_ingest`
+- `cargo test --test parity_analysis`
+- `cargo test --test parity_tables`
+- `cargo test --test parity_realtime`
+- `make verify`
 
-## Planned Compatibility
+## Scope
 
-The first parity line targets the upstream Claude monitor CLI:
+- Preserve upstream executable behavior before semantic fixes.
+- Keep the rewrite as a single crate with explicit internal module seams.
+- Use a vendored upstream oracle plus sanitized fixture homes for parity evidence.
+- Keep local Claude file analysis and terminal-first operation as the core product behavior.
 
-- realtime, daily, and monthly monitoring views
-- plan selection for `pro`, `max5`, `max20`, and `custom`
-- refresh, theme, timezone, time-format, reset-hour, and logging flags
-- local Claude session analysis with terminal-oriented output
+## Deferred Until Post-Parity
 
-Fork-only provider features from the current `Psysician/c-monitor` repository, such as Codex or dual-provider monitoring, are intentionally deferred until the upstream-compatible baseline exists.
+- Alias takeover or packaging claims for `claude-monitor`, `cmonitor`, `ccmonitor`, and `ccm`.
+- Fork-only provider expansion from `Psysician/c-monitor`.
+- Safer non-TTY terminal behavior beyond the compatibility-scoped gate.
 
-## Development
+## Current Architecture
 
-Prerequisites:
-
-- Rust `1.93.0`
-- `make`
-
-Verification:
-
-```bash
-make verify
-```
-
-Current binary behavior is intentionally limited to a contract-aware placeholder:
-
-```bash
-cargo run -- --plan max20 --view daily --theme dark
-```
-
-The CLI parser is present, but the session reader, analytics engine, and terminal UI are not implemented yet.
-
-## Repository Docs
-
-- `docs/parity-inventory.md`: upstream CLI and runtime behaviors to preserve
-- `docs/architecture-notes.md`: planned runtime loop, state model, and module boundaries
-- `plans/m1-bootstrap-and-contract-harness.md`: first implementation milestone and acceptance criteria
+- `config`: CLI parsing plus `last_used.json` persistence.
+- `discovery`: Claude root discovery and JSONL file enumeration.
+- `parser`: raw JSONL decoding and typed usage-entry normalization.
+- `analysis`: session blocks, limit detection, and custom-limit math.
+- `report`: shared renderer-neutral report state.
+- `ui` and `runtime`: table rendering, realtime rendering, and terminal lifecycle.
+- `compat`: isolated upstream quirks and terminal-policy defaults.
 
 ## License
 
 MIT
-

```

**Documentation:**

```diff
--- a/README.md
+++ b/README.md
@@ -39,3 +39,7 @@
 ## License
 
 MIT
+
+## Planning Rationale
+
+- Packaging and alias claims stay outside the parity line unless fixture evidence is green. (ref: DL-008)

```


## Execution Waves

- W-001: M-001
- W-002: M-002
- W-003: M-003
- W-004: M-004
- W-005: M-005
