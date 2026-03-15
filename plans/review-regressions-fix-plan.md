# Plan

## Overview

Fix the rewrite regressions that let cmonitor-rs claim parity while the vendored oracle falls back to emulation, limit warnings disappear before report assembly, daily and monthly labels ignore the resolved timezone, and realtime mode tears down the terminal immediately after the first frame.

**Approach**: Patch the reviewed seams in place: require executable bundle-mode oracle coverage, preserve raw warning events through normalization and orchestration, thread resolved timezone into row grouping, and keep realtime mode in a refresh loop backed by targeted regression tests and the existing integration suite.

## Planning Context

### Decision Log

| ID | Decision | Reasoning Chain |
|---|---|---|
| DL-001 | Use the vendored upstream Python executable as a hard parity gate for CLI coverage | Emulated fallback hides import breakage and behavioral drift -> parity evidence only counts when upstream entrypoints execute -> oracle acquisition and CLI tests require bundle mode instead of silently degrading |
| DL-002 | Preserve raw system and tool_result events alongside token-bearing usage entries until limit detection finishes | Limit warnings arrive in zero-token raw rows -> dropping those rows before analysis erases user-visible warnings -> normalization and orchestration carry a raw event stream for detect_limit_events while block math stays usage-entry based |
| DL-003 | Apply the resolved CLI timezone inside daily and monthly row grouping and label formatting | The runtime title already names the resolved timezone -> UTC bucket labels under that title misreport day and month boundaries -> row builders accept timezone context so grouping and presentation stay aligned |
| DL-004 | Keep realtime mode alive for a refresh loop that owns terminal guard lifetime | Alternate-screen state is restored on TerminalGuard drop -> single-render return erases the frame and leaves refresh settings unused -> realtime mode keeps the guard until an explicit exit path after repeated renders |
| DL-005 | Fix only the reviewed regressions with targeted regression coverage; do not reopen packaging work, alias takeover, or broad rewrite cleanup | The worktree is already mid-rewrite and dirty -> expanding scope into packaging alias takeover or broad cleanup increases merge risk without improving the reviewed regressions -> constrain the plan to in-place seam fixes plus focused tests that measure those regressions directly |
| DL-006 | Use chrono-tz for named-region DST support alongside the existing time crate instead of switching to a single datetime ecosystem | The time crate lacks IANA timezone database support -> named timezones like Europe/Berlin require DST-aware conversion at grouping boundaries -> chrono-tz provides the IANA database while time continues to own fixed-offset formatting and CLI timestamp surfaces -> dual-ecosystem cost is localized to format_label only |
| DL-007 | Use the ctrlc crate for SIGINT handling instead of raw FFI signal() calls | Raw signal() has undefined behavior in multi-threaded programs per POSIX -> the return type and calling convention vary across platforms -> ctrlc is a lightweight safe abstraction that handles platform differences and thread safety -> avoids unsafe blocks in application code |

### Rejected Alternatives

| Alternative | Why Rejected |
|---|---|
| Keep the emulated fallback as the main parity path | Import failures in the vendored bundle would keep passing while upstream behavior drifts out of coverage (ref: DL-001) |
| Rely on side-by-side CLI snapshots without equality assertions | Visible payload mismatches already pass when the suite only records both outputs (ref: DL-001) |
| Fix timezone display only in table titles | Aggregation buckets and rendered row labels would still stay in UTC under a non-UTC title (ref: DL-003) |
| Use raw FFI signal() for SIGINT handling | signal() has undefined behavior when called from multi-threaded programs per POSIX, return type usize is incorrect for the function pointer it returns, and no previous handler restoration on teardown (ref: DL-007) |
| Use time-tz instead of chrono-tz to stay in one datetime ecosystem | time-tz has a smaller IANA database surface and fewer downstream users than chrono-tz, and the dual-ecosystem cost is confined to one function (ref: DL-006) |

### Constraints

- MUST: keep the upstream Python executable as the parity oracle rather than replacing failures with emulation-only coverage
- MUST: respect the existing dirty worktree and avoid reverting unrelated rewrite files
- SHOULD: add targeted regression tests for each reported runtime and parity bug so fixes are measurable
- MUST-NOT: silently bless oracle-vs-Rust mismatches through snapshots that do not assert equality
- MUST-NOT: expand this regression pass into packaging work, alias takeover, or broad rewrite cleanup outside the reviewed seams

### Known Risks

- **The vendored oracle may need a larger upstream module closure than the review notes enumerate**: Drive the bundle list from the import graph used by claude_monitor.cli.main and verify bundle-mode execution before updating CLI snapshots
- **Realtime regression tests can hang if the refresh loop lacks a bounded test seam**: Factor the loop around an injected or bounded exit condition so tests can exercise multiple renders without waiting for manual interruption
- **Timezone assertions can become host-dependent if they rely on ambient local settings**: Use explicit fixed test timezones and cross-midnight fixtures rather than machine-local auto detection
- **Raw warning preservation can regress if normalization or orchestration paths re-filter the event stream down to token-bearing usage rows only**: Carry a dedicated raw-event collection from normalization through detect_limit_events and lock it with mixed-event ingest plus analysis fixtures before refactors land
- **A future upstream schema change could add token-bearing events with type system or tool_result**: The should_preserve_raw_event early-continue skips token extraction for matching events, so token-bearing warnings would be silently dropped from block totals. Add a diagnostic counter that flags preserved events carrying non-zero tokens so the assumption stays visible
- **Snapshot description frontmatter may not be supported by insta 1.42**: Verify that insta accepts description fields in snapshot YAML frontmatter before committing updated snapshots; omit the field if the version rejects unknown metadata keys

## Invisible Knowledge

### System

[DL-002][DL-003][DL-004] The runtime contract is a single shared pipeline: decoded JSONL events enter normalization, token-bearing entries feed session-block math, raw system and tool_result rows feed limit detection, ReportState carries both totals and warnings, and table plus realtime runtimes consume that same state.

### Invariants

- [DL-001][M-001] Executable CLI parity depends on the vendored upstream bundle running in bundle mode against fixture homes.
- [DL-002][M-002] Session blocks stay derived from typed usage entries even when raw warning events are preserved for limit detection.
- [DL-003][M-003][DL-004][M-004] Daily monthly and realtime surfaces consume the same ReportState instead of maintaining separate aggregation paths.
- [DL-004][M-004] Realtime mode keeps terminal ownership for the lifetime of the refresh loop and never enters alternate screen for the empty-data fast path.

### Tradeoffs

- [DL-005][M-001][M-002][M-003][M-004] The plan chooses focused seam fixes over packaging work, alias takeover, or broader architectural cleanup to avoid colliding with the dirty rewrite worktree.
- [DL-001][M-001] Bundle-mode parity raises vendor maintenance cost but prevents the harness from certifying behavior the upstream executable never ran.
- [DL-004][M-004] Realtime loop tests need an explicit bounded seam so the suite stays deterministic while still covering repeated renders.
- [DL-006][M-003] chrono-tz introduces a second datetime ecosystem alongside time; the conversion cost is localized to format_label and chrono_label.
- [DL-007][M-004] ctrlc crate adds a dependency but removes all unsafe signal handling and platform-specific FFI from application code.

### Report Pipeline

[Diagram pending Technical Writer rendering: DIAG-001]

## Milestones

### Milestone 1: Restore Executable Oracle CLI Parity

**Files**: tests/support/fetch_upstream_oracle.py, tests/support/oracle_runner.py, tests/parity_cli.rs, tests/vendor/claude-code-usage-monitor.manifest.json, tests/fixtures/contract/README.md

**Requirements**:

- Vendor the upstream bundle with every import needed by claude_monitor.cli.main
- Require oracle_runner scenarios to execute bundle mode instead of emulation
- Assert bundle-mode CLI coverage for version defaults overrides and clear flows; enforce exact payload parity for defaults overrides and clear; validate only the version banner shape so packaging scope stays unchanged

**Acceptance Criteria**:

- python3 tests/support/fetch_upstream_oracle.py --check-pin succeeds against the vendored bundle
- python3 tests/support/oracle_runner.py --scenario cli-version reports oracle_mode bundle
- cargo test --test parity_cli fails on emulated fallback on defaults/overrides/clear payload drift or on version-banner shape regressions

**Tests**:

- python3 tests/support/fetch_upstream_oracle.py --check-pin
- python3 tests/support/oracle_runner.py --scenario cli-version
- cargo test --test parity_cli

#### Code Intent

- **CI-M-001-001** `tests/support/fetch_upstream_oracle.py::materialize_checkout`: Copy the full upstream module subset required by claude_monitor.cli.main so the vendored oracle can import bootstrap core data monitoring terminal and UI dependencies from the pinned commit. (refs: DL-001, DL-005)
- **CI-M-001-002** `tests/support/oracle_runner.py::try_run_bundle`: Treat upstream import failures as parity harness failures for executable scenarios and report bundle-mode payloads only when the vendored oracle actually runs. (refs: DL-001, DL-005)
- **CI-M-001-003** `tests/parity_cli.rs`: Assert oracle_mode bundle for every CLI scenario; compare Rust and oracle payloads for defaults overrides and clear flows; validate the version banner shape without forcing package-version equality that would reopen packaging scope. (refs: DL-001, DL-005)
- **CI-M-001-004** `tests/vendor/claude-code-usage-monitor.manifest.json`: Record the manifest-backed vendored file inventory for the pinned upstream bundle so bundle-mode parity can verify every required module is present and intentional. (refs: DL-001, DL-005)
- **CI-M-001-005** `tests/fixtures/contract/README.md`: Document the fixture-home and vendored-oracle refresh workflow that bundle-mode CLI parity relies on so contributors can reproduce the oracle contract without reintroducing emulated fallback assumptions. (refs: DL-001, DL-005)

#### Code Changes

**CC-M-001-001** (tests/parity_cli.rs) - implements CI-M-001-003

**Code:**

```diff
--- a/tests/parity_cli.rs
+++ b/tests/parity_cli.rs
@@ -56,6 +56,11 @@
     let mut payload: Value =
         serde_json::from_slice(&output.stdout).expect("oracle output should be valid json");
     scrub_last_used(&mut payload);
+    assert_eq!(
+        payload["oracle_mode"],
+        json!("bundle"),
+        "oracle runner must execute the vendored upstream bundle"
+    );
     payload
 }
 

```

**Documentation:**

```diff
--- a/tests/parity_cli.rs
+++ b/tests/parity_cli.rs
@@ -56,6 +56,8 @@
     let mut payload: Value =
         serde_json::from_slice(&output.stdout).expect("oracle output should be valid json");
     scrub_last_used(&mut payload);
+    // Bundle mode keeps CLI parity anchored to executable upstream behavior so
+    // import drift fails the harness and emulation cannot mask it. (ref: DL-001)
     assert_eq!(
         payload["oracle_mode"],
         json!("bundle"),

```


**CC-M-001-002** (tests/support/fetch_upstream_oracle.py) - implements CI-M-001-001

**Code:**

```diff
--- a/tests/support/fetch_upstream_oracle.py
+++ b/tests/support/fetch_upstream_oracle.py
@@ -6,19 +6,13 @@
 import os
 import shutil
 import subprocess
+import sys
 import tempfile
 from pathlib import Path
 
 REPO_ROOT = Path(__file__).resolve().parents[2]
 MANIFEST_PATH = REPO_ROOT / "tests/vendor/claude-code-usage-monitor.manifest.json"
-REQUIRED_FILES = [
-    "pyproject.toml",
-    "src/claude_monitor/__init__.py",
-    "src/claude_monitor/__main__.py",
-    "src/claude_monitor/_version.py",
-    "src/claude_monitor/cli/__init__.py",
-    "src/claude_monitor/cli/main.py",
-]
+SKIP_DIRS = {"__pycache__"}
 
 
 def load_manifest() -> dict:
@@ -29,10 +23,39 @@
     return REPO_ROOT / manifest["bundle_dir"]
 
 
+def manifest_environment(manifest: dict, bundle: Path) -> dict[str, str]:
+    env = os.environ.copy()
+    env.update({key: str(value) for key, value in manifest.get("oracle_env", {}).items()})
+    env["PYTHONPATH"] = str(bundle / "src")
+    return env
+
+
+def expand_inventory(base: Path, manifest: dict) -> list[str]:
+    inventory: list[str] = []
+
+    for relative in manifest.get("required_files", []):
+        candidate = base / relative
+        if not candidate.exists():
+            raise SystemExit(f"vendored oracle missing manifest file: {relative}")
+        inventory.append(relative)
+
+    for relative in manifest.get("required_roots", []):
+        root = base / relative
+        if not root.exists():
+            raise SystemExit(f"vendored oracle missing manifest root: {relative}")
+        if root.is_file():
+            inventory.append(relative)
+            continue
+        for child in sorted(root.rglob("*.py")):
+            if any(part in SKIP_DIRS for part in child.parts):
+                continue
+            inventory.append(child.relative_to(base).as_posix())
+
+    return sorted(dict.fromkeys(inventory))
+
+
 def validate_bundle(manifest: dict, bundle: Path) -> None:
-    missing = [path for path in REQUIRED_FILES if not (bundle / path).exists()]
-    if missing:
-        raise SystemExit(f"vendored oracle missing files: {', '.join(missing)}")
+    expand_inventory(bundle, manifest)
 
     pin_file = bundle / ".oracle-pin"
     if not pin_file.exists():
@@ -40,17 +63,33 @@
     if pin_file.read_text(encoding="utf-8").strip() != manifest["source"]["commit"]:
         raise SystemExit("vendored oracle pin does not match manifest")
 
+    completed = subprocess.run(
+        [sys.executable, "-c", "import claude_monitor.cli.main"],
+        env=manifest_environment(manifest, bundle),
+        text=True,
+        capture_output=True,
+        check=False,
+    )
+    if completed.returncode != 0:
+        raise SystemExit(
+            "vendored oracle import check failed:\n" + completed.stderr.strip()
+        )
 
-def materialize_checkout(source: Path, destination: Path, commit: str) -> None:
+
+def materialize_checkout(source: Path, destination: Path, manifest: dict) -> None:
+    inventory = expand_inventory(source, manifest)
     if destination.exists():
         shutil.rmtree(destination)
     destination.mkdir(parents=True, exist_ok=True)
-    for relative in REQUIRED_FILES:
+    for relative in inventory:
         src = source / relative
         dest = destination / relative
         dest.parent.mkdir(parents=True, exist_ok=True)
         shutil.copy2(src, dest)
-    (destination / ".oracle-pin").write_text(commit + "\n", encoding="utf-8")
+    (destination / ".oracle-pin").write_text(
+        manifest["source"]["commit"] + "\n",
+        encoding="utf-8",
+    )
 
 
 def fetch_from_git(manifest: dict, destination: Path) -> None:
@@ -64,7 +103,7 @@
             ["git", "-C", str(checkout), "checkout", manifest["source"]["commit"]],
             check=True,
         )
-        materialize_checkout(checkout, destination, manifest["source"]["commit"])
+        materialize_checkout(checkout, destination, manifest)
 
 
 def ensure_bundle(check_pin: bool) -> Path:
@@ -82,7 +121,7 @@
         materialize_checkout(
             Path(os.environ[override_env]),
             destination,
-            manifest["source"]["commit"],
+            manifest,
         )
     else:
         fetch_from_git(manifest, destination)

```

**Documentation:**

```diff
--- a/tests/support/fetch_upstream_oracle.py
+++ b/tests/support/fetch_upstream_oracle.py
@@ -23,6 +23,9 @@

 def manifest_environment(manifest: dict, bundle: Path) -> dict[str, str]:
+    """Keeps bundle execution aligned with manifest-pinned defaults and import
+    roots so parity runs exercise the vendored oracle exactly. (ref: DL-001)"""
     env = os.environ.copy()
     env.update({key: str(value) for key, value in manifest.get("oracle_env", {}).items()})
     env["PYTHONPATH"] = str(bundle / "src")
@@ -29,6 +32,9 @@

 def expand_inventory(base: Path, manifest: dict) -> list[str]:
+    """Builds the manifest inventory that defines the executable oracle surface,
+    including import roots beyond the entrypoint file. (ref: DL-001)"""
     inventory: list[str] = []

     for relative in manifest.get("required_files", []):
@@ -72,6 +78,9 @@

 def materialize_checkout(source: Path, destination: Path, manifest: dict) -> None:
+    """Copies only the manifest inventory so the vendored bundle matches the
+    upstream modules required by bundle-mode parity. (ref: DL-001)"""
     inventory = expand_inventory(source, manifest)
     if destination.exists():
         shutil.rmtree(destination)

```


**CC-M-001-003** (tests/support/oracle_runner.py) - implements CI-M-001-002

**Code:**

```diff
--- a/tests/support/oracle_runner.py
+++ b/tests/support/oracle_runner.py
@@ -6,7 +6,6 @@
 import os
 import subprocess
 import sys
-import time
 from pathlib import Path
 
 REPO_ROOT = Path(__file__).resolve().parents[2]
@@ -45,17 +44,6 @@
     return REPO_ROOT / manifest()["bundle_dir"]
 
 
-def version_from_bundle() -> str:
-    pyproject_path = bundle_path() / "pyproject.toml"
-    try:
-        import tomllib  # py311+
-
-        data = tomllib.loads(pyproject_path.read_text(encoding="utf-8"))
-        return data.get("project", {}).get("version", "unknown")
-    except Exception:
-        return "unknown"
-
-
 def scenario_fixture_home(name: str, override: str | None) -> Path:
     if override:
         return Path(override)
@@ -72,144 +60,22 @@
     return payload
 
 
-def save_last_used(home: Path, payload: dict) -> None:
-    path = home / ".claude-monitor" / "last_used.json"
-    path.parent.mkdir(parents=True, exist_ok=True)
-    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
+def ensure_bundle() -> None:
+    subprocess.run(
+        [
+            sys.executable,
+            str(REPO_ROOT / "tests/support/fetch_upstream_oracle.py"),
+            "--check-pin",
+        ],
+        cwd=REPO_ROOT,
+        check=True,
+    )
 
 
-def default_settings() -> dict:
-    return {
-        "plan": "custom",
-        "custom_limit_tokens": None,
-        "view": "realtime",
-        "timezone": "auto",
-        "time_format": "auto",
-        "theme": "auto",
-        "refresh_rate": 10,
-        "refresh_per_second": 0.75,
-        "reset_hour": None,
-        "log_level": "INFO",
-        "log_file": None,
-        "debug": False,
-        "clear": False,
-        "version": False,
-    }
+def run_bundle(name: str, argv: list[str], fixture_home: Path) -> dict:
+    ensure_bundle()
+    fixture_home.mkdir(parents=True, exist_ok=True)
 
-
-def resolve_auto_defaults(settings: dict) -> dict:
-    resolved = dict(settings)
-    if resolved["timezone"] == "auto":
-        resolved["timezone"] = os.environ.get("CMONITOR_TEST_TIMEZONE", "UTC")
-    if resolved["time_format"] == "auto":
-        resolved["time_format"] = os.environ.get("CMONITOR_TEST_TIME_FORMAT", "24h")
-    if resolved["theme"] == "auto":
-        resolved["theme"] = os.environ.get("CMONITOR_TEST_THEME", "dark")
-    if resolved["debug"]:
-        resolved["log_level"] = "DEBUG"
-    return resolved
-
-
-def parse_cli(argv: list[str]) -> tuple[dict, set[str]]:
-    settings = default_settings()
-    provided = set()
-    index = 0
-    while index < len(argv):
-        arg = argv[index]
-        if arg in ("--version", "-v"):
-            settings["version"] = True
-            provided.add("version")
-            index += 1
-        elif arg == "--clear":
-            settings["clear"] = True
-            provided.add("clear")
-            index += 1
-        elif arg == "--debug":
-            settings["debug"] = True
-            provided.add("debug")
-            index += 1
-        elif arg.startswith("--"):
-            key = arg[2:].replace("-", "_")
-            provided.add(key)
-            value = argv[index + 1]
-            index += 2
-            if key in {"refresh_rate", "custom_limit_tokens", "reset_hour"}:
-                settings[key] = int(value)
-            elif key == "refresh_per_second":
-                settings[key] = float(value)
-            else:
-                settings[key] = value
-        else:
-            index += 1
-    return settings, provided
-
-
-def emulate_scenario(name: str, argv: list[str], fixture_home: Path) -> dict:
-    fixture_home.mkdir(parents=True, exist_ok=True)
-    raw_settings, provided = parse_cli(argv)
-    last_used = load_last_used(fixture_home) or {}
-
-    settings = dict(raw_settings)
-    if not settings["clear"]:
-        for key, value in last_used.items():
-            if key == "plan":
-                continue
-            if key not in provided:
-                settings[key] = value
-
-        if (
-            "plan" in provided
-            and settings["plan"] == "custom"
-            and "custom_limit_tokens" not in provided
-        ):
-            settings["custom_limit_tokens"] = None
-
-    settings = resolve_auto_defaults(settings)
-
-    if settings["version"]:
-        stdout = f"claude-monitor {version_from_bundle()}\n"
-        return {
-            "scenario": name,
-            "args": argv,
-            "exit_code": 0,
-            "stdout": stdout,
-            "stderr": "",
-            "last_used": None,
-            "source_commit": manifest()["source"]["commit"],
-            "oracle_mode": "emulated",
-        }
-
-    last_used_path = fixture_home / ".claude-monitor" / "last_used.json"
-    if settings["clear"]:
-        if last_used_path.exists():
-            last_used_path.unlink()
-        last_used_payload = None
-    else:
-        last_used_payload = {
-            "theme": settings["theme"],
-            "timezone": settings["timezone"],
-            "time_format": settings["time_format"],
-            "refresh_rate": settings["refresh_rate"],
-            "reset_hour": settings["reset_hour"],
-            "view": settings["view"],
-        }
-        if settings["custom_limit_tokens"]:
-            last_used_payload["custom_limit_tokens"] = settings["custom_limit_tokens"]
-        save_last_used(fixture_home, last_used_payload)
-
-    return {
-        "scenario": name,
-        "args": argv,
-        "exit_code": 0,
-        "stdout": "No Claude data directory found\n",
-        "stderr": "",
-        "last_used": last_used_payload,
-        "source_commit": manifest()["source"]["commit"],
-        "oracle_mode": "emulated",
-    }
-
-
-def try_run_bundle(name: str, argv: list[str], fixture_home: Path) -> dict | None:
     command = [
         sys.executable,
         "-c",
@@ -217,23 +83,16 @@
         *argv,
     ]
     env = os.environ.copy()
+    env.update({key: str(value) for key, value in manifest().get("oracle_env", {}).items()})
     env["HOME"] = str(fixture_home)
     env["PYTHONPATH"] = str(bundle_path() / "src")
     completed = subprocess.run(command, env=env, text=True, capture_output=True, check=False)
-    if completed.returncode == 0:
-        return {
-            "scenario": name,
-            "args": argv,
-            "exit_code": completed.returncode,
-            "stdout": completed.stdout,
-            "stderr": completed.stderr,
-            "last_used": load_last_used(fixture_home),
-            "source_commit": manifest()["source"]["commit"],
-            "oracle_mode": "bundle",
-        }
-
-    if "ModuleNotFoundError" in completed.stderr or "ImportError" in completed.stderr:
-        return None
+    if completed.returncode != 0 and (
+        "ModuleNotFoundError" in completed.stderr or "ImportError" in completed.stderr
+    ):
+        raise SystemExit(
+            f"vendored oracle import failed for {name}:\n{completed.stderr.strip()}"
+        )
 
     return {
         "scenario": name,
@@ -250,11 +109,7 @@
 def run_scenario(name: str, fixture_home_override: str | None) -> dict:
     scenario = SCENARIOS[name]
     fixture_home = scenario_fixture_home(name, fixture_home_override)
-    fixture_home.mkdir(parents=True, exist_ok=True)
-    bundle_result = try_run_bundle(name, scenario["args"], fixture_home)
-    if bundle_result is not None:
-        return bundle_result
-    return emulate_scenario(name, scenario["args"], fixture_home)
+    return run_bundle(name, scenario["args"], fixture_home)
 
 
 def main() -> int:

```

**Documentation:**

```diff
--- a/tests/support/oracle_runner.py
+++ b/tests/support/oracle_runner.py
@@ -60,6 +60,9 @@

 def ensure_bundle() -> None:
+    """Fails fast when the vendored oracle cannot import, so parity never
+    degrades into a synthetic fallback path. (ref: DL-001)"""
     subprocess.run(
         [
             sys.executable,
@@ -71,6 +74,9 @@

 def run_bundle(name: str, argv: list[str], fixture_home: Path) -> dict:
+    """Runs the pinned upstream CLI under the fixture home so payloads reflect
+    executable oracle behavior across settings flows. (ref: DL-001)"""
     ensure_bundle()
     fixture_home.mkdir(parents=True, exist_ok=True)


```


**CC-M-001-004** (tests/parity_cli.rs) - implements CI-M-001-003

**Code:**

```diff
--- a/tests/parity_cli.rs
+++ b/tests/parity_cli.rs
@@ -88,10 +88,66 @@
     payload
 }
 
+fn parity_payload(value: &Value) -> Value {
+    json!({
+        "args": value.get("args").cloned().unwrap_or(Value::Null),
+        "exit_code": value.get("exit_code").cloned().unwrap_or(Value::Null),
+        "stdout": value.get("stdout").cloned().unwrap_or(Value::Null),
+        "stderr": value.get("stderr").cloned().unwrap_or(Value::Null),
+        "last_used": value.get("last_used").cloned().unwrap_or(Value::Null),
+    })
+}
+
+fn version_stdout(value: &Value) -> &str {
+    value["stdout"]
+        .as_str()
+        .expect("version payload should include stdout text")
+}
+
+fn assert_version_banner_shape(label: &str, value: &Value) {
+    let trimmed = version_stdout(value)
+        .strip_suffix('\n')
+        .expect("version banner should end with a newline");
+    let mut parts = trimmed.split_whitespace();
+    assert_eq!(
+        parts.next(),
+        Some("claude-monitor"),
+        "{label} banner should keep the upstream binary name"
+    );
+    let version = parts
+        .next()
+        .expect("{label} banner should include a version token");
+    assert!(
+        !version.is_empty(),
+        "{label} banner version token should not be empty"
+    );
+    assert!(
+        parts.next().is_none(),
+        "{label} banner should contain only the binary name and version"
+    );
+}
+
+fn assert_cli_parity(snapshot: &str, scenario: &str, args: &[&str], seed: Option<&str>) {
+    let oracle_home = unique_home(&format!("{scenario}-oracle"));
+    let rust_home = unique_home(&format!("{scenario}-rust"));
+    seed_home(&oracle_home, seed);
+    seed_home(&rust_home, seed);
+
+    let oracle = run_oracle(scenario, &oracle_home);
+    let rust = run_rust(args, &rust_home);
+
+    assert_eq!(
+        parity_payload(&rust),
+        parity_payload(&oracle),
+        "rust output diverged from oracle for {scenario}"
+    );
+    insta::assert_json_snapshot!(snapshot, json!({ "oracle": oracle, "rust": rust }));
+}
+
 #[test]
 fn version_banner_matches_oracle_snapshot() {
-    let oracle_home = unique_home("oracle-version");
-    let rust_home = unique_home("rust-version");
+    let oracle_home = unique_home("cli-version-oracle");
+    let rust_home = unique_home("cli-version-rust");
     seed_home(&oracle_home, None);
     seed_home(&rust_home, None);
 
@@ -100,22 +156,19 @@
 
     assert_eq!(oracle["exit_code"], json!(0));
     assert_eq!(rust["exit_code"], json!(0));
+    assert_eq!(oracle["oracle_mode"], json!("bundle"));
+    assert_eq!(oracle["stderr"], json!(""));
+    assert_eq!(rust["stderr"], json!(""));
+    assert_eq!(oracle["last_used"], Value::Null);
+    assert_eq!(rust["last_used"], Value::Null);
+    assert_version_banner_shape("oracle", &oracle);
+    assert_version_banner_shape("rust", &rust);
     insta::assert_json_snapshot!("cli-version", json!({ "oracle": oracle, "rust": rust }));
 }
 
 #[test]
 fn defaults_and_last_used_match_oracle_snapshot() {
-    let oracle_home = unique_home("oracle-defaults");
-    let rust_home = unique_home("rust-defaults");
-    seed_home(&oracle_home, None);
-    seed_home(&rust_home, None);
-
-    let oracle = run_oracle("cli-defaults", &oracle_home);
-    let rust = run_rust(&[], &rust_home);
-
-    assert_eq!(oracle["exit_code"], json!(0));
-    assert_eq!(rust["exit_code"], json!(0));
-    insta::assert_json_snapshot!("cli-defaults", json!({ "oracle": oracle, "rust": rust }));
+    assert_cli_parity("cli-defaults", "cli-defaults", &[], None);
 }
 
 #[test]
@@ -134,32 +187,23 @@
         "--refresh-per-second",
         "1.5",
     ];
-    let oracle_home = unique_home("oracle-overrides");
-    let rust_home = unique_home("rust-overrides");
-    seed_home(&oracle_home, None);
-    seed_home(&rust_home, None);
-
-    let oracle = run_oracle("cli-overrides", &oracle_home);
-    let rust = run_rust(&args, &rust_home);
-
-    assert_eq!(oracle["exit_code"], json!(0));
-    assert_eq!(rust["exit_code"], json!(0));
-    insta::assert_json_snapshot!("cli-overrides", json!({ "oracle": oracle, "rust": rust }));
+
+    assert_cli_parity("cli-overrides", "cli-overrides", &args, None);
 }
 
 #[test]
 fn clear_flow_removes_persisted_last_used_file() {
     let seed = r#"{"view":"daily"}"#;
-    let oracle_home = unique_home("oracle-clear");
-    let rust_home = unique_home("rust-clear");
+    let oracle_home = unique_home("cli-clear-oracle");
+    let rust_home = unique_home("cli-clear-rust");
     seed_home(&oracle_home, Some(seed));
     seed_home(&rust_home, Some(seed));
 
     let oracle = run_oracle("cli-clear", &oracle_home);
     let rust = run_rust(&["--clear"], &rust_home);
 
-    assert_eq!(oracle["exit_code"], json!(0));
-    assert_eq!(rust["exit_code"], json!(0));
+    assert_eq!(oracle["oracle_mode"], json!("bundle"));
+    assert_eq!(parity_payload(&rust), parity_payload(&oracle));
     assert!(
         !rust_home.join(".claude-monitor/last_used.json").exists(),
         "clear should remove last_used.json"

```

**Documentation:**

```diff
--- a/tests/parity_cli.rs
+++ b/tests/parity_cli.rs
@@ -88,6 +88,9 @@
     payload
 }

+/// Narrows parity assertions to the contract fields that must match across
+/// oracle and Rust runs, so snapshots cannot bless unrelated drift. (ref: DL-001)
 fn parity_payload(value: &Value) -> Value {
     json!({
         "args": value.get("args").cloned().unwrap_or(Value::Null),
@@ -98,6 +101,9 @@
     })
 }

+/// Keeps banner parsing shared so both executables are held to the same shape
+/// contract prior to snapshot review. (ref: DL-001)
 fn version_stdout(value: &Value) -> &str {
     value["stdout"]
         .as_str()
@@ -105,6 +111,9 @@
         .expect("version payload should include stdout text")
 }

+/// Verifies the upstream binary name and tokenized version format, which are
+/// part of the CLI contract independent of exact version text. (ref: DL-001)
 fn assert_version_banner_shape(label: &str, value: &Value) {
     let trimmed = version_stdout(value)
         .strip_suffix('\n')
@@ -125,6 +134,9 @@
         "{label} banner should contain only the binary name and version"
     );
 }

+/// Reuses the same oracle and Rust harness for each scenario so every CLI
+/// regression test measures direct contract equality. (ref: DL-001, DL-005)
 fn assert_cli_parity(snapshot: &str, scenario: &str, args: &[&str], seed: Option<&str>) {
     let oracle_home = unique_home(&format!("{scenario}-oracle"));
     let rust_home = unique_home(&format!("{scenario}-rust"));

```


**CC-M-001-005** (tests/vendor/claude-code-usage-monitor.manifest.json) - implements CI-M-001-004

**Code:**

```diff
--- a/tests/vendor/claude-code-usage-monitor.manifest.json
+++ b/tests/vendor/claude-code-usage-monitor.manifest.json
@@ -11,9 +11,28 @@
     "cli-version",
     "cli-clear"
   ],
+  "oracle_env": {
+    "TZ": "UTC",
+    "NO_COLOR": "1"
+  },
+  "required_files": [
+    "pyproject.toml",
+    "src/claude_monitor/__init__.py",
+    "src/claude_monitor/__main__.py",
+    "src/claude_monitor/_version.py",
+    "src/claude_monitor/error_handling.py"
+  ],
+  "required_roots": [
+    "src/claude_monitor/cli",
+    "src/claude_monitor/core",
+    "src/claude_monitor/data",
+    "src/claude_monitor/monitoring",
+    "src/claude_monitor/terminal",
+    "src/claude_monitor/ui"
+  ],
   "notes": [
     "The vendored bundle is the parity oracle for executable behavior.",
-    "This bootstrap vendors a minimal subset anchored to a real upstream commit pin.",
-    "Refresh snapshots only after updating the manifest pin and recording the fixture input changes."
+    "Refresh the vendored bundle by copying the upstream executable subset needed by the CLI, settings, monitoring, terminal, and table flows.",
+    "Parity tests fail if the vendored oracle cannot import, rather than falling back to an emulated payload."
   ]
 }

```

**Documentation:**

```diff
--- a/tests/vendor/claude-code-usage-monitor.manifest.json
+++ b/tests/vendor/claude-code-usage-monitor.manifest.json
@@ -24,6 +24,7 @@
   "notes": [
     "The vendored bundle is the parity oracle for executable behavior.",
     "Refresh the vendored bundle by copying the upstream executable subset needed by the CLI, settings, monitoring, terminal, and table flows.",
+    "Manifest-driven inventory keeps bundle-mode parity aligned with upstream imports. (ref: DL-001)",
     "Parity tests fail if the vendored oracle cannot import, rather than falling back to an emulated payload."
   ]
 }

```


**CC-M-001-006** (tests/fixtures/contract/README.md) - implements CI-M-001-005

**Code:**

```diff
--- a/tests/fixtures/contract/README.md
+++ b/tests/fixtures/contract/README.md
@@ -12,9 +12,9 @@
 
 ## Oracle Refresh Flow
 
-1. Update `tests/vendor/claude-code-usage-monitor.manifest.json` with the upstream commit pin.
-2. Run `python3 tests/support/fetch_upstream_oracle.py` to verify or refresh the vendored subset.
-3. Run `python3 tests/support/oracle_runner.py --scenario ...` to inspect the oracle payloads.
+1. Update `tests/vendor/claude-code-usage-monitor.manifest.json` when the upstream commit pin or executable import roots change.
+2. Run `python3 tests/support/fetch_upstream_oracle.py --check-pin` to verify the vendored executable subset, or rerun without `--check-pin` to refresh it from the pinned source.
+3. Run `python3 tests/support/oracle_runner.py --scenario ...` to confirm the real vendored bundle still imports and produces the expected payload.
 4. Re-record Rust snapshots only after the oracle payloads and fixture contents are stable.
 
 ## Sanitization Rules
@@ -22,3 +22,4 @@
 - Keep only the files required for the scenario under `home/`.
 - Replace user-specific paths and identifiers with deterministic placeholders.
 - Treat executable upstream behavior as the contract when fixture content and prose disagree.
+- Treat oracle import failures as harness failures; do not replace them with emulated payloads.

```

**Documentation:**

```diff
--- a/tests/fixtures/contract/README.md
+++ b/tests/fixtures/contract/README.md
@@ -15,6 +15,8 @@
 2. Run `python3 tests/support/fetch_upstream_oracle.py --check-pin` to verify the vendored executable subset, or rerun without `--check-pin` to refresh it from the pinned source.
 3. Run `python3 tests/support/oracle_runner.py --scenario ...` to confirm the real vendored bundle still imports and produces the expected payload.
 4. Re-record Rust snapshots only after the oracle payloads and fixture contents are stable.
+
+Bundle-mode execution is the CLI parity contract boundary for this fixture set. (ref: DL-001)

 ## Sanitization Rules

```


**CC-M-001-007** (tests/snapshots/parity_cli__cli-defaults.snap) - implements CI-M-001-003

**Code:**

```diff
--- a/tests/snapshots/parity_cli__cli-defaults.snap
+++ b/tests/snapshots/parity_cli__cli-defaults.snap
@@ -14,7 +14,7 @@
       "timezone": "UTC",
       "view": "realtime"
     },
-    "oracle_mode": "emulated",
+    "oracle_mode": "bundle",
     "scenario": "cli-defaults",
     "source_commit": "06f0fe11e694b8619f63f8b0db10dbdc5e7e5a44",
     "stderr": "",

```

**CC-M-001-008** (tests/snapshots/parity_cli__cli-overrides.snap) - implements CI-M-001-003

**Code:**

```diff
--- a/tests/snapshots/parity_cli__cli-overrides.snap
+++ b/tests/snapshots/parity_cli__cli-overrides.snap
@@ -27,7 +27,7 @@
       "timezone": "UTC",
       "view": "daily"
     },
-    "oracle_mode": "emulated",
+    "oracle_mode": "bundle",
     "scenario": "cli-overrides",
     "source_commit": "06f0fe11e694b8619f63f8b0db10dbdc5e7e5a44",
     "stderr": "",

```

**CC-M-001-009** (tests/snapshots/parity_cli__cli-version.snap) - implements CI-M-001-003

**Code:**

```diff
--- a/tests/snapshots/parity_cli__cli-version.snap
+++ b/tests/snapshots/parity_cli__cli-version.snap
@@ -9,7 +9,7 @@
     ],
     "exit_code": 0,
     "last_used": null,
-    "oracle_mode": "emulated",
+    "oracle_mode": "bundle",
     "scenario": "cli-version",
     "source_commit": "06f0fe11e694b8619f63f8b0db10dbdc5e7e5a44",
     "stderr": "",

```

**CC-M-001-010** (tests/snapshots/parity_cli__cli-clear.snap) - implements CI-M-001-003

**Code:**

```diff
--- a/tests/snapshots/parity_cli__cli-clear.snap
+++ b/tests/snapshots/parity_cli__cli-clear.snap
@@ -9,7 +9,7 @@
     ],
     "exit_code": 0,
     "last_used": null,
-    "oracle_mode": "emulated",
+    "oracle_mode": "bundle",
     "scenario": "cli-clear",
     "source_commit": "06f0fe11e694b8619f63f8b0db10dbdc5e7e5a44",
     "stderr": "",

```

### Milestone 2: Retain Limit Warnings Through The Report Pipeline

**Files**: src/parser/entries.rs, src/runtime/orchestrator.rs, src/analysis/limits.rs, tests/parity_ingest.rs, tests/parity_analysis.rs, tests/fixtures/ingest/mixed-events.jsonl

**Requirements**:

- Keep raw limit-candidate events available even when they carry zero tokens or no usage payload
- Build blocks from normalized usage entries without losing the raw warning stream
- Attach detected limit events to matching session blocks and shared report state

**Acceptance Criteria**:

- A zero-token system warning still reaches detect_limit_events after normalization
- Block construction stays driven by token-bearing usage entries only
- cargo test --test parity_ingest and cargo test --test parity_analysis cover the warning regression

**Tests**:

- cargo test --test parity_ingest
- cargo test --test parity_analysis

#### Code Intent

- **CI-M-002-001** `src/parser/entries.rs::normalize_usage_entries`: Return token-bearing UsageEntry values for block math while preserving a raw event collection that still contains zero-token system and tool_result messages eligible for limit detection. Track preserved_with_tokens as a diagnostic counter so token-bearing warning events are visible rather than silently dropped. (refs: DL-002, DL-005)
- **CI-M-002-002** `src/runtime/orchestrator.rs::load_report_state`: Pass the preserved raw event stream into detect_limit_events after block construction so report warnings come from the same decoded JSONL corpus as block totals. (refs: DL-002)
- **CI-M-002-003** `tests/parity_analysis.rs`: Lock the regression with a fixture or inline raw event that has no usage tokens but still produces a limit warning attached to the correct block and report state. (refs: DL-002, DL-005)
- **CI-M-002-004** `src/analysis/limits.rs::detect_limit_events`: Consume the preserved raw event stream and classify zero-token system or tool_result warnings so limit messages survive normalization and attach to the correct session blocks. (refs: DL-002, DL-005)
- **CI-M-002-005** `tests/parity_ingest.rs`: Assert that mixed raw events keep zero-token warning rows available through ingestion and orchestration instead of dropping them before limit detection runs. (refs: DL-002, DL-005)
- **CI-M-002-006** `tests/fixtures/ingest/mixed-events.jsonl`: Add a raw-event fixture that mixes token-bearing usage rows with zero-token system or tool_result limit warnings to lock the regression inputs shared by ingest and analysis tests. (refs: DL-002, DL-005)

#### Code Changes

**CC-M-002-001** (src/parser/entries.rs) - implements CI-M-002-001

**Code:**

```diff
--- a/src/parser/entries.rs
+++ b/src/parser/entries.rs
@@ -17,6 +17,7 @@
 pub struct EntryNormalization {
     pub entries: Vec<UsageEntry>,
     pub retained_raw_events: Vec<RawUsageEvent>,
+    pub preserved_with_tokens: usize,
     pub skipped_zero_tokens: usize,
     pub skipped_before_cutoff: usize,
     pub skipped_duplicates: usize,
@@ -28,6 +29,7 @@
     cutoff: Option<OffsetDateTime>,
 ) -> EntryNormalization {
     let mut accepted = Vec::new();
+    let mut preserved_raw_events = Vec::new();
     let mut seen = BTreeSet::new();
     let mut report = EntryNormalization::default();
 
@@ -38,6 +39,11 @@
 
         if cutoff.is_some_and(|limit| timestamp < limit) {
             report.skipped_before_cutoff += 1;
+            continue;
+        }
+
+        if should_preserve_raw_event(&event.payload) {
+            if has_nonzero_tokens(&event.payload) {
+                report.preserved_with_tokens += 1;
+            }
+            preserved_raw_events.push(event.clone());
             continue;
         }
 
@@ -69,7 +75,9 @@
 
     accepted.sort_by(|left, right| left.0.timestamp.cmp(&right.0.timestamp));
     report.entries = accepted.iter().map(|(entry, _)| entry.clone()).collect();
-    report.retained_raw_events = accepted.into_iter().map(|(_, event)| event).collect();
+    preserved_raw_events.extend(accepted.into_iter().map(|(_, event)| event));
+    preserved_raw_events.sort_by_key(event_sort_key);
+    report.retained_raw_events = preserved_raw_events;
     report
 }
 
@@ -149,3 +157,15 @@
         .unwrap_or("unknown")
         .to_lowercase()
 }
+
+fn should_preserve_raw_event(payload: &Value) -> bool {
+    matches!(payload.get("type").and_then(Value::as_str), Some("system" | "tool_result"))
+}
+
+fn has_nonzero_tokens(payload: &Value) -> bool {
+    payload
+        .get("usage")
+        .and_then(|u| {
+            let input = u.get("input_tokens").and_then(Value::as_u64).unwrap_or(0);
+            let output = u.get("output_tokens").and_then(Value::as_u64).unwrap_or(0);
+            if input + output > 0 { Some(()) } else { None }
+        })
+        .is_some()
+}
+
+fn event_sort_key(event: &RawUsageEvent) -> (i64, String, usize) {
+    (
+        parse_timestamp(&event.payload)
+            .map(|ts| ts.unix_timestamp())
+            .unwrap_or(i64::MAX),
+        event.source_file.display().to_string(),
+        event.line_number,
+    )
+}

```

**Documentation:**

```diff
--- a/src/parser/entries.rs
+++ b/src/parser/entries.rs
@@ -39,6 +39,8 @@

+        // System and tool_result rows stay in the raw stream because limit
+        // warnings can be meaningful even when token totals are zero. (ref: DL-002)
     if should_preserve_raw_event(&event.payload) {
         preserved_raw_events.push(event.clone());
         continue;
@@ -157,6 +161,9 @@
 }

+/// Keeps warning-only rows available to limit detection while usage filtering
+/// removes entries that do not contribute token totals. (ref: DL-002)
 fn should_preserve_raw_event(payload: &Value) -> bool {
     matches!(payload.get("type").and_then(Value::as_str), Some("system" | "tool_result"))
 }
@@ -161,6 +168,9 @@
 }

+/// Sorts preserved raw rows deterministically; unparseable timestamps sort last
+/// so they do not interfere with block attachment. (ref: DL-002)
 fn event_sort_key(event: &RawUsageEvent) -> (i64, String, usize) {
     (
         parse_timestamp(&event.payload)
+            .map(|ts| ts.unix_timestamp())
+            .unwrap_or(i64::MAX),

```


**CC-M-002-002** (src/runtime/orchestrator.rs) - implements CI-M-002-002

**Code:**

```diff
--- a/src/runtime/orchestrator.rs
+++ b/src/runtime/orchestrator.rs
@@ -3,7 +3,7 @@
 use crate::analysis::{calculate_custom_limit, detect_limit_events, transform_to_blocks};
 use crate::config::ResolvedConfig;
 use crate::discovery::{collect_jsonl_files, discover_roots, select_primary_root};
-use crate::parser::{DecodedJsonl, decode_jsonl_file, normalize_usage_entries};
+use crate::parser::{DecodedJsonl, EntryNormalization, decode_jsonl_file, normalize_usage_entries};
 use crate::report::ReportState;
 
 pub fn load_report_state(_resolved: &ResolvedConfig) -> anyhow::Result<ReportState> {
@@ -22,9 +22,13 @@
         decoded.diagnostics.extend(file_decoded.diagnostics);
     }
 
-    let normalized = normalize_usage_entries(decoded, None);
-    let mut blocks = transform_to_blocks(&normalized.entries, now);
-    let limits = detect_limit_events(&normalized.retained_raw_events, &mut blocks);
+    let EntryNormalization {
+        entries,
+        retained_raw_events,
+        ..
+    } = normalize_usage_entries(decoded, None);
+    let mut blocks = transform_to_blocks(&entries, now);
+    let limits = detect_limit_events(&retained_raw_events, &mut blocks);
     let _custom_limit = calculate_custom_limit(&blocks);
     Ok(ReportState::from_blocks(now, blocks, limits))
 }

```

**Documentation:**

```diff
--- a/src/runtime/orchestrator.rs
+++ b/src/runtime/orchestrator.rs
@@ -22,6 +22,8 @@
         decoded.diagnostics.extend(file_decoded.diagnostics);
     }

+    // Usage entries drive block math, while preserved raw rows keep zero-token
+    // warnings available for limit detection in the same load pass. (ref: DL-002)
     let EntryNormalization {
         entries,
         retained_raw_events,

```


**CC-M-002-003** (tests/parity_analysis.rs) - implements CI-M-002-003

**Code:**

```diff
--- a/tests/parity_analysis.rs
+++ b/tests/parity_analysis.rs
@@ -1,12 +1,18 @@
 use std::path::PathBuf;
 
-use serde_json::json;
 use time::macros::datetime;
 
 use cmonitor_rs::analysis::{calculate_custom_limit, detect_limit_events, transform_to_blocks};
+use cmonitor_rs::discovery::JsonlFile;
 use cmonitor_rs::domain::{TokenUsage, UsageEntry};
-use cmonitor_rs::parser::RawUsageEvent;
+use cmonitor_rs::parser::{decode_jsonl_file, normalize_usage_entries};
 use cmonitor_rs::report::{ReportState, build_daily_rows};
+
+fn fixture_path(relative: &str) -> PathBuf {
+    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
+        .join("tests/fixtures/ingest")
+        .join(relative)
+}
 
 fn usage_entry(timestamp: time::OffsetDateTime, total: u64) -> UsageEntry {
     UsageEntry {
@@ -43,24 +49,28 @@
 }
 
 #[test]
-fn limit_detection_assigns_warnings_to_block_ranges() {
-    let entries = vec![usage_entry(datetime!(2026-03-14 12:15 UTC), 10)];
-    let mut blocks = transform_to_blocks(&entries, datetime!(2026-03-14 12:30 UTC));
-    let limits = detect_limit_events(
-        &[RawUsageEvent {
-            source_file: PathBuf::from("fixture.jsonl"),
-            line_number: 10,
-            payload: json!({
-                "timestamp": "2026-03-14T12:20:00Z",
-                "type": "system",
-                "content": "Rate limit reached until later"
-            }),
-        }],
-        &mut blocks,
+fn limit_detection_uses_shared_mixed_event_fixture() {
+    let path = fixture_path("mixed-events.jsonl");
+    let decoded = decode_jsonl_file(&JsonlFile {
+        root: path.parent().expect("fixture parent").to_path_buf(),
+        path: path.clone(),
+    })
+    .expect("fixture jsonl should decode");
+    let normalized =
+        normalize_usage_entries(decoded, Some(datetime!(2026-03-14 11:59:30 UTC)));
+    let mut blocks = transform_to_blocks(&normalized.entries, datetime!(2026-03-14 12:30 UTC));
+    let limits = detect_limit_events(&normalized.retained_raw_events, &mut blocks);
+    let report = ReportState::from_blocks(datetime!(2026-03-14 12:30 UTC), blocks, limits.clone());
+
+    assert_eq!(limits.len(), 2);
+    assert_eq!(report.limits.len(), 2);
+    assert_eq!(report.blocks[0].limits.len(), 2);
+    assert!(limits.iter().any(|limit| limit.message.contains("Rate limit")));
+    assert!(
+        limits
+            .iter()
+            .any(|limit| limit.message.contains("Usage limit"))
     );
-
-    assert_eq!(limits.len(), 1);
-    assert_eq!(blocks[0].limits.len(), 1);
 }
 
 #[test]

```

**Documentation:**

```diff
--- a/tests/parity_analysis.rs
+++ b/tests/parity_analysis.rs
@@ -6,6 +6,9 @@
 use cmonitor_rs::parser::{decode_jsonl_file, normalize_usage_entries};
 use cmonitor_rs::report::{ReportState, build_daily_rows};

+/// Shares the ingest fixture with analysis tests so preserved-warning coverage
+/// cannot drift between normalization and block-level limit detection. (ref: DL-002)
 fn fixture_path(relative: &str) -> PathBuf {
     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
         .join("tests/fixtures/ingest")
@@ -49,6 +52,9 @@
 }

 #[test]
+/// Uses the mixed-event fixture end to end so warning preservation and limit
+/// attachment stay coupled in one regression seam. (ref: DL-002, DL-005)
 fn limit_detection_uses_shared_mixed_event_fixture() {
     let path = fixture_path("mixed-events.jsonl");
     let decoded = decode_jsonl_file(&JsonlFile {

```


**CC-M-002-004** (src/analysis/limits.rs) - implements CI-M-002-004

**Code:**

```diff
--- a/src/analysis/limits.rs
+++ b/src/analysis/limits.rs
@@ -5,12 +5,12 @@
 use crate::parser::RawUsageEvent;
 
 pub fn detect_limit_events(
-    events: &[RawUsageEvent],
+    raw_events: &[RawUsageEvent],
     blocks: &mut [SessionBlock],
 ) -> Vec<LimitEvent> {
     let mut detected = Vec::new();
 
-    for event in events {
+    for event in raw_events {
         let Some(limit) = parse_limit_event(event) else {
             continue;
         };
@@ -34,7 +34,7 @@
         return None;
     }
 
-    let content = event.payload.get("content").and_then(Value::as_str)?;
+    let content = limit_message_content(&event.payload)?;
     let lowered = content.to_lowercase();
     if !lowered.contains("limit") && !lowered.contains("rate") {
         return None;
@@ -57,7 +57,24 @@
     Some(LimitEvent {
         kind,
         timestamp,
-        message: content.to_owned(),
+        message: content,
         reset_at: None,
     })
 }
+
+fn limit_message_content(payload: &Value) -> Option<String> {
+    if let Some(content) = payload.get("content").and_then(Value::as_str) {
+        return Some(content.to_owned());
+    }
+
+    let content = payload.get("content").and_then(Value::as_array)?;
+    let parts = content
+        .iter()
+        .filter_map(|item| item.get("text").and_then(Value::as_str).or_else(|| item.as_str()))
+        .collect::<Vec<_>>();
+    if parts.is_empty() {
+        None
+    } else {
+        Some(parts.join(" "))
+    }
+}

```

**Documentation:**

```diff
--- a/src/analysis/limits.rs
+++ b/src/analysis/limits.rs
@@ -57,6 +57,9 @@
 }

+/// Accepts both plain-string and structured content payloads because upstream
+/// limit warnings do not arrive under a single message schema. (ref: DL-002)
 fn limit_message_content(payload: &Value) -> Option<String> {
     if let Some(content) = payload.get("content").and_then(Value::as_str) {
         return Some(content.to_owned());

```


**CC-M-002-005** (tests/parity_ingest.rs) - implements CI-M-002-005

**Code:**

```diff
--- a/tests/parity_ingest.rs
+++ b/tests/parity_ingest.rs
@@ -2,6 +2,7 @@
 use std::path::{Path, PathBuf};
 use std::time::{SystemTime, UNIX_EPOCH};
 
+use serde_json::Value;
 use time::macros::datetime;
 
 use cmonitor_rs::discovery::{JsonlFile, RootSource, collect_jsonl_files, discover_roots_with};
@@ -95,6 +96,17 @@
     assert_eq!(normalized.skipped_before_cutoff, 1);
     assert_eq!(normalized.skipped_zero_tokens, 1);
     assert_eq!(normalized.skipped_duplicates, 1);
+    let preserved_warnings = normalized
+        .retained_raw_events
+        .iter()
+        .filter(|event| {
+            matches!(
+                event.payload.get("type").and_then(Value::as_str),
+                Some("system" | "tool_result")
+            )
+        })
+        .collect::<Vec<_>>();
+    assert_eq!(preserved_warnings.len(), 2);
     assert_eq!(normalized.entries[0].model, "claude-3-5-haiku-20241022");
     assert_eq!(normalized.entries[0].tokens.total_tokens(), 11);
     assert_eq!(normalized.entries[1].message_id.as_deref(), Some("m-1"));
@@ -124,8 +136,15 @@
             .iter()
             .all(|entry| entry.tokens.total_tokens() > 0)
     );
-    assert_eq!(
-        normalized.retained_raw_events.len(),
-        normalized.entries.len()
-    );
+    assert!(normalized.entries.iter().all(|entry| {
+        normalized.retained_raw_events.iter().any(|event| {
+            event.source_file == entry.source_file && event.line_number == entry.line_number
+        })
+    }));
+    assert!(normalized.retained_raw_events.iter().any(|event| {
+        matches!(
+            event.payload.get("type").and_then(Value::as_str),
+            Some("system")
+        )
+    }));
 }

```

**Documentation:**

```diff
--- a/tests/parity_ingest.rs
+++ b/tests/parity_ingest.rs
@@ -95,6 +96,8 @@
     assert_eq!(normalized.skipped_before_cutoff, 1);
     assert_eq!(normalized.skipped_zero_tokens, 1);
     assert_eq!(normalized.skipped_duplicates, 1);
+    // Preserved raw warnings keep zero-token system rows visible to the
+    // limit-detection pass without polluting usage totals. (ref: DL-002)
     let preserved_warnings = normalized
         .retained_raw_events
         .iter()
@@ -136,6 +145,8 @@
         .all(|entry| entry.tokens.total_tokens() > 0)
     );
     assert!(normalized.entries.iter().all(|entry| {
+        // The retained stream still carries warning-only rows so analysis sees
+        // the same raw evidence that ingest observed. (ref: DL-002)
         normalized.retained_raw_events.iter().any(|event| {
             event.source_file == entry.source_file && event.line_number == entry.line_number
         })

```


**CC-M-002-006** (tests/fixtures/ingest/mixed-events.jsonl) - implements CI-M-002-006

**Code:**

```diff
--- a/tests/fixtures/ingest/mixed-events.jsonl
+++ b/tests/fixtures/ingest/mixed-events.jsonl
@@ -1,6 +1,8 @@
 {"timestamp":"2026-03-14T11:59:00Z","message_id":"m-old","request_id":"r-old","model":"Claude-3-7-Sonnet-20250219","usage":{"input_tokens":2,"output_tokens":1}}
 {"timestamp":
+{"timestamp":"2026-03-14T12:00:00Z","type":"system","content":"Rate limit reached until later"}
 {"timestamp":"2026-03-14T12:00:00Z","message_id":"m-zero","request_id":"r-zero","model":"Claude-3-7-Sonnet-20250219","usage":{"input_tokens":0,"output_tokens":0}}
 {"timestamp":"2026-03-14T12:00:01Z","message_id":"m-1","request_id":"r-1","model":"Claude-3-7-Sonnet-20250219","usage":{"input_tokens":10,"output_tokens":2}}
+{"timestamp":"2026-03-14T12:30:00Z","type":"tool_result","content":"Usage limit nearly exhausted"}
 {"timestamp":"2026-03-14T12:00:02Z","message_id":"m-1","request_id":"r-1","model":"Claude-3-7-Sonnet-20250219","usage":{"input_tokens":10,"output_tokens":2}}
 {"timestamp":"2026-03-14T13:00:00+01:00","message":{"id":"m-2","model":"Claude-3-5-Haiku-20241022"},"requestId":"r-2","usage":{"input_tokens":5,"output_tokens":1,"cache_creation_tokens":2,"cache_read_tokens":3},"cost":0.25}

```

**Documentation:**

```diff
--- a/tests/fixtures/ingest/README.md
+++ b/tests/fixtures/ingest/README.md
@@ -1,4 +1,5 @@
 Fixture notes for ingest parity:

 - `sample-home/` is a stable recursive directory tree used to verify JSONL file discovery order.
 - `mixed-events.jsonl` mixes malformed rows, cutoff rows, zero-token rows, duplicate composite ids, and accepted rows.
+- `mixed-events.jsonl` also carries warning-only system and tool_result rows so limit detection sees the raw evidence preserved by normalization. (ref: DL-002)
 - The ingest tests intentionally keep fixtures local and deterministic so parser behavior can evolve without terminal output concerns.

```


### Milestone 3: Align Table Aggregation With Resolved Timezone

**Files**: Cargo.toml, src/report/daily_monthly.rs, src/runtime/table_mode.rs, tests/parity_tables.rs, tests/parity_analysis.rs

**Requirements**:

- Group daily and monthly rows by the resolved CLI timezone instead of raw UTC timestamps
- Format row labels in the same timezone named in the table title
- Keep daily monthly and realtime totals sourced from one ReportState

**Acceptance Criteria**:

- A block that crosses a UTC date boundary groups into the expected local day and month labels
- Table titles and row labels reference the same timezone
- cargo test --test parity_tables and cargo test --test parity_analysis cover the timezone regression

**Tests**:

- cargo test --test parity_tables
- cargo test --test parity_analysis

#### Code Intent

- **CI-M-003-001** `src/report/daily_monthly.rs::build_rows`: Accept timezone context and derive grouping keys plus rendered labels from block timestamps converted into the resolved CLI timezone. (refs: DL-003, DL-005)
- **CI-M-003-002** `src/runtime/table_mode.rs::run_table_mode`: Pass the resolved CLI timezone through the daily and monthly row builders so the title and aggregated rows describe the same local calendar buckets. (refs: DL-003)
- **CI-M-003-003** `tests/parity_tables.rs`: Add a deterministic cross-midnight timezone case that proves local labels and token totals stay stable for both daily and monthly output. (refs: DL-003, DL-005)
- **CI-M-003-004** `tests/parity_analysis.rs`: Assert on shared ReportState under a fixed non-UTC timezone so cross-midnight blocks prove daily and monthly aggregation boundaries before table rendering snapshots are consulted. (refs: DL-003, DL-005)

#### Code Changes

**CC-M-003-001** (src/report/daily_monthly.rs) - implements CI-M-003-001

**Code:**

```diff
--- a/src/report/daily_monthly.rs
+++ b/src/report/daily_monthly.rs
@@ -1,13 +1,18 @@
 use std::collections::BTreeMap;
 
+use chrono::{DateTime, Utc};
+use chrono_tz::Tz;
 use serde::{Deserialize, Serialize};
 use time::format_description::FormatItem;
 use time::macros::format_description;
+use time::{OffsetDateTime, UtcOffset};
 
 use crate::report::ReportState;
 
 static DAY_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");
 static MONTH_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]");
+static OFFSET_FORMAT: &[FormatItem<'static>] =
+    format_description!("[offset_hour sign:mandatory]:[offset_minute]");
 
 #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
 pub struct AggregateRow {
@@ -17,26 +22,34 @@
     pub models: Vec<String>,
 }
 
-pub fn build_daily_rows(report: &ReportState) -> Vec<AggregateRow> {
-    build_rows(report, DAY_FORMAT)
+enum GroupingTimezone {
+    Fixed(UtcOffset),
+    Named(Tz),
 }
 
-pub fn build_monthly_rows(report: &ReportState) -> Vec<AggregateRow> {
-    build_rows(report, MONTH_FORMAT)
+pub fn build_daily_rows(report: &ReportState, timezone: &str) -> Vec<AggregateRow> {
+    build_rows(report, DAY_FORMAT, "%Y-%m-%d", timezone)
 }
 
-fn build_rows(report: &ReportState, formatter: &[FormatItem<'static>]) -> Vec<AggregateRow> {
+pub fn build_monthly_rows(report: &ReportState, timezone: &str) -> Vec<AggregateRow> {
+    build_rows(report, MONTH_FORMAT, "%Y-%m", timezone)
+}
+
+fn build_rows(
+    report: &ReportState,
+    formatter: &[FormatItem<'static>],
+    chrono_formatter: &str,
+    timezone: &str,
+) -> Vec<AggregateRow> {
     let mut grouped = BTreeMap::<String, AggregateRow>::new();
+    let timezone = resolve_timezone(timezone);
 
     for block in &report.blocks {
         if block.is_gap {
             continue;
         }
 
-        let label = block
-            .start_time
-            .format(formatter)
-            .unwrap_or_else(|_| "unknown".to_owned());
+        let label = format_label(block.start_time, formatter, chrono_formatter, &timezone);
         let row = grouped
             .entry(label.clone())
             .or_insert_with(|| AggregateRow {
@@ -56,3 +69,36 @@
 
     grouped.into_values().collect()
 }
+
+fn resolve_timezone(timezone: &str) -> GroupingTimezone {
+    if timezone.eq_ignore_ascii_case("utc") || timezone.eq_ignore_ascii_case("z") {
+        GroupingTimezone::Fixed(UtcOffset::UTC)
+    } else if let Ok(offset) = UtcOffset::parse(timezone, OFFSET_FORMAT) {
+        GroupingTimezone::Fixed(offset)
+    } else if let Ok(named) = timezone.parse::<Tz>() {
+        GroupingTimezone::Named(named)
+    } else {
+        GroupingTimezone::Fixed(UtcOffset::UTC)
+    }
+}
+
+fn format_label(
+    timestamp: OffsetDateTime,
+    formatter: &[FormatItem<'static>],
+    chrono_formatter: &str,
+    timezone: &GroupingTimezone,
+) -> String {
+    match timezone {
+        GroupingTimezone::Fixed(offset) => timestamp
+            .to_offset(*offset)
+            .format(formatter)
+            .unwrap_or_else(|_| "unknown".to_owned()),
+        GroupingTimezone::Named(named) => chrono_label(timestamp, *named, chrono_formatter),
+    }
+}
+
+fn chrono_label(timestamp: OffsetDateTime, timezone: Tz, formatter: &str) -> String {
+    DateTime::<Utc>::from_timestamp(timestamp.unix_timestamp(), timestamp.nanosecond())
+        .map(|value| value.with_timezone(&timezone).format(formatter).to_string())
+        .unwrap_or_else(|| "unknown".to_owned())
+}

```

**Documentation:**

```diff
--- a/src/report/daily_monthly.rs
+++ b/src/report/daily_monthly.rs
@@ -22,6 +22,9 @@
 pub struct AggregateRow {
     pub label: String,
     pub total_tokens: u64,
 }

+/// Carries either fixed offsets or named regions so grouping can follow the
+/// resolved CLI timezone across UTC, offset, and region inputs. (ref: DL-003)
 enum GroupingTimezone {
     Fixed(UtcOffset),
     Named(Tz),
@@ -26,6 +29,9 @@
     Named(Tz),
 }

+/// Groups blocks on the requested day boundary so daily labels agree with the
+/// timezone named in table output. (ref: DL-003)
 pub fn build_daily_rows(report: &ReportState, timezone: &str) -> Vec<AggregateRow> {
     build_rows(report, DAY_FORMAT, "%Y-%m-%d", timezone)
 }
@@ -31,6 +37,9 @@
     build_rows(report, DAY_FORMAT, "%Y-%m-%d", timezone)
 }

+/// Groups blocks on the requested month boundary so monthly rollups follow the
+/// same timezone contract as daily output. (ref: DL-003)
 pub fn build_monthly_rows(report: &ReportState, timezone: &str) -> Vec<AggregateRow> {
     build_rows(report, MONTH_FORMAT, "%Y-%m", timezone)
 }
@@ -35,6 +44,9 @@
 pub fn build_monthly_rows(report: &ReportState, timezone: &str) -> Vec<AggregateRow> {
     build_rows(report, MONTH_FORMAT, "%Y-%m", timezone)
 }

+/// Converts timestamps at aggregation entry so labels and totals share one
+/// calendar boundary for the entire table build. (ref: DL-003)
 fn build_rows(
     report: &ReportState,
     formatter: &[FormatItem<'static>],
@@ -69,6 +81,9 @@

     grouped.into_values().collect()
 }

+/// Resolves UTC, fixed offsets, and named regions into one grouping model that
+/// preserves the CLI timezone contract. (ref: DL-003)
 fn resolve_timezone(timezone: &str) -> GroupingTimezone {
     if timezone.eq_ignore_ascii_case("utc") || timezone.eq_ignore_ascii_case("z") {
         GroupingTimezone::Fixed(UtcOffset::UTC)
@@ -81,6 +96,8 @@
     }
 }

+/// Keeps grouping keys and rendered labels on the same converted timestamp.
+/// (ref: DL-003)
 fn format_label(
     timestamp: OffsetDateTime,
     formatter: &[FormatItem<'static>],
@@ -96,6 +113,9 @@
         GroupingTimezone::Named(named) => chrono_label(timestamp, *named, chrono_formatter),
     }
 }

+/// Uses chrono-tz only where named-region DST rules affect day and month
+/// boundaries. (ref: DL-003)
 fn chrono_label(timestamp: OffsetDateTime, timezone: Tz, formatter: &str) -> String {
     DateTime::<Utc>::from_timestamp(timestamp.unix_timestamp(), timestamp.nanosecond())
         .map(|value| value.with_timezone(&timezone).format(formatter).to_string())

```


**CC-M-003-002** (src/runtime/table_mode.rs) - implements CI-M-003-002

**Code:**

```diff
--- a/src/runtime/table_mode.rs
+++ b/src/runtime/table_mode.rs
@@ -25,11 +25,11 @@
     match view {
         View::Monthly => println!(
             "{}",
-            table::render_table(&title, &build_monthly_rows(&report))
+            table::render_table(&title, &build_monthly_rows(&report, &resolved.cli.timezone))
         ),
         _ => println!(
             "{}",
-            table::render_table(&title, &build_daily_rows(&report))
+            table::render_table(&title, &build_daily_rows(&report, &resolved.cli.timezone))
         ),
     }
     Ok(ExitCode::SUCCESS)

```

**Documentation:**

```diff
--- a/src/runtime/table_mode.rs
+++ b/src/runtime/table_mode.rs
@@ -25,6 +25,8 @@
     match view {
+        // Row builders receive the resolved timezone so labels and the table
+        // title describe the same calendar boundary. (ref: DL-003)
         View::Monthly => println!(
             "{}",
             table::render_table(&title, &build_monthly_rows(&report, &resolved.cli.timezone))

```


**CC-M-003-003** (tests/parity_tables.rs) - implements CI-M-003-003

**Code:**

```diff
--- a/tests/parity_tables.rs
+++ b/tests/parity_tables.rs
@@ -33,9 +33,10 @@
     );
     let report = ReportState::from_blocks(datetime!(2026-03-14 18:00 UTC), blocks, Vec::new());
     let output = format!(
-        "{}\n{}",
+        "{}
+{}",
         summary::render_summary(&report),
-        table::render_table("daily usage", &build_daily_rows(&report))
+        table::render_table("daily usage", &build_daily_rows(&report, "UTC"))
     );
 
     insta::assert_snapshot!("daily-table", output);
@@ -49,12 +50,28 @@
     );
     let report = ReportState::from_blocks(datetime!(2026-03-14 18:00 UTC), blocks, Vec::new());
     let output = format!(
-        "{}\n{}",
+        "{}
+{}",
         summary::render_summary(&report),
-        table::render_table("monthly usage", &build_monthly_rows(&report))
+        table::render_table("monthly usage", &build_monthly_rows(&report, "UTC"))
     );
 
     insta::assert_snapshot!("monthly-table", output);
+}
+
+#[test]
+fn daily_and_monthly_rows_use_requested_timezone_name() {
+    let blocks = transform_to_blocks(
+        &[entry(datetime!(2026-03-31 23:30 UTC), 20)],
+        datetime!(2026-04-01 01:00 UTC),
+    );
+    let report = ReportState::from_blocks(datetime!(2026-04-01 01:00 UTC), blocks, Vec::new());
+
+    let daily = build_daily_rows(&report, "Europe/Berlin");
+    let monthly = build_monthly_rows(&report, "Europe/Berlin");
+
+    assert_eq!(daily[0].label, "2026-04-01");
+    assert_eq!(monthly[0].label, "2026-04");
 }
 
 #[test]

```

**Documentation:**

```diff
--- a/tests/parity_tables.rs
+++ b/tests/parity_tables.rs
@@ -60,6 +60,9 @@
 }

 #[test]
+/// Cross-midnight fixtures prove named timezone grouping keeps rendered labels
+/// on the requested day and month boundaries. (ref: DL-003, DL-005)
 fn daily_and_monthly_rows_use_requested_timezone_name() {
     let blocks = transform_to_blocks(
         &[entry(datetime!(2026-03-31 23:30 UTC), 20)],

```


**CC-M-003-004** (tests/parity_analysis.rs) - implements CI-M-003-004

**Code:**

```diff
--- a/tests/parity_analysis.rs
+++ b/tests/parity_analysis.rs
@@ -6,7 +6,7 @@
 use cmonitor_rs::discovery::JsonlFile;
 use cmonitor_rs::domain::{TokenUsage, UsageEntry};
 use cmonitor_rs::parser::{decode_jsonl_file, normalize_usage_entries};
-use cmonitor_rs::report::{ReportState, build_daily_rows};
+use cmonitor_rs::report::{ReportState, build_daily_rows, build_monthly_rows};
 
 fn fixture_path(relative: &str) -> PathBuf {
     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
@@ -92,9 +92,23 @@
     let entries = vec![usage_entry(datetime!(2026-03-14 12:15 UTC), 10)];
     let blocks = transform_to_blocks(&entries, datetime!(2026-03-14 18:00 UTC));
     let report = ReportState::from_blocks(datetime!(2026-03-14 18:00 UTC), blocks, Vec::new());
-    let rows = build_daily_rows(&report);
+    let rows = build_daily_rows(&report, "UTC");
 
     assert_eq!(rows[0].total_tokens, report.totals.total_tokens);
+}
+
+#[test]
+fn report_rows_use_requested_timezone_boundary_before_table_rendering() {
+    let entries = vec![usage_entry(datetime!(2026-03-31 23:30 UTC), 10)];
+    let blocks = transform_to_blocks(&entries, datetime!(2026-04-01 01:00 UTC));
+    let report = ReportState::from_blocks(datetime!(2026-04-01 01:00 UTC), blocks, Vec::new());
+    let daily = build_daily_rows(&report, "Europe/Berlin");
+    let monthly = build_monthly_rows(&report, "Europe/Berlin");
+
+    assert_eq!(daily[0].label, "2026-04-01");
+    assert_eq!(monthly[0].label, "2026-04");
+    assert_eq!(daily[0].total_tokens, report.totals.total_tokens);
+    assert_eq!(monthly[0].total_tokens, report.totals.total_tokens);
 }
 
 #[test]

```

**Documentation:**

```diff
--- a/tests/parity_analysis.rs
+++ b/tests/parity_analysis.rs
@@ -101,6 +101,9 @@
 }

 #[test]
+/// Checks report-row boundaries at the shared aggregation seam so timezone
+/// coverage does not depend on table rendering. (ref: DL-003, DL-005)
 fn report_rows_use_requested_timezone_boundary_before_table_rendering() {
     let entries = vec![usage_entry(datetime!(2026-03-31 23:30 UTC), 10)];
     let blocks = transform_to_blocks(&entries, datetime!(2026-04-01 01:00 UTC));

```


**CC-M-003-005** (Cargo.toml) - implements CI-M-003-001, CI-M-004-001

**Code:**

```diff
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -25,6 +25,9 @@

 [dependencies]
 anyhow = "1.0"
+chrono = "0.4"
+chrono-tz = "0.10"
 clap = { version = "4.5", features = ["derive"] }
+ctrlc = "3.4"
 serde = { version = "1.0", features = ["derive"] }
 serde_json = "1.0"

```

**Documentation:**

```diff
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -25,6 +25,9 @@

 [dependencies]
 anyhow = "1.0"
+# Named-region grouping needs chrono-tz, while time continues to own the CLI
+# timestamp surface and fixed-offset formatting. (ref: DL-003, DL-006)
 chrono = "0.4"
 chrono-tz = "0.10"
 clap = { version = "4.5", features = ["derive"] }
+# Safe cross-platform SIGINT handling without unsafe FFI. (ref: DL-007)
+ctrlc = "3.4"
 serde = { version = "1.0", features = ["derive"] }

```


### Milestone 4: Restore Realtime Loop Contract

**Files**: Cargo.toml, src/runtime/realtime.rs, src/runtime/terminal.rs, tests/parity_realtime.rs

**Requirements**:

- Keep the realtime frame visible by holding TerminalGuard for the duration of the refresh loop
- Use resolved refresh settings to drive repeated renders instead of returning after one print
- Keep the empty-data path explicit without entering alternate screen

**Acceptance Criteria**:

- Realtime mode with report data does not drop alternate screen immediately after the first render
- A controlled test seam exercises repeated render cadence without hanging the suite
- No-data execution still prints the explicit message and exits successfully

**Tests**:

- cargo test --test parity_realtime

#### Code Intent

- **CI-M-004-001** `src/runtime/realtime.rs::run_realtime_mode`: Own a refresh loop that renders the shared report state repeatedly and keeps TerminalGuard alive until an explicit exit condition or ctrlc-based interruption path completes. (refs: DL-004, DL-005, DL-007)
- **CI-M-004-002** `src/runtime/terminal.rs::TerminalGuard`: Flush alternate-screen transitions as part of terminal lifecycle management so rendered frames and teardown sequences are observable at the right times. (refs: DL-004)
- **CI-M-004-003** `tests/parity_realtime.rs`: Cover the no-data fast path and a controlled data-present loop path so realtime regressions fail without relying on manual terminal inspection. (refs: DL-004, DL-005)

#### Code Changes

**CC-M-004-001** (src/runtime/realtime.rs) - implements CI-M-004-001

**Code:**

```diff
--- a/src/runtime/realtime.rs
+++ b/src/runtime/realtime.rs
@@ -1,17 +1,105 @@
+use std::io::{self, Write};
 use std::process::ExitCode;
+use std::sync::atomic::{AtomicBool, Ordering};
+use std::thread;
+use std::time::{Duration, Instant};

-use crate::config::ResolvedConfig;
+use crate::config::{Cli, ResolvedConfig};
+use crate::report::ReportState;
 use crate::runtime::orchestrator::load_report_state;
 use crate::runtime::terminal::TerminalGuard;
 use crate::ui::realtime;

+static INTERRUPTED: AtomicBool = AtomicBool::new(false);
+
 pub fn run_realtime_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
-    let report = load_report_state(resolved)?;
+    install_interrupt_handler()?;
+    run_realtime_loop(
+        &resolved.cli,
+        &mut io::stdout(),
+        || load_report_state(resolved),
+        |duration| thread::sleep(duration),
+        LoopControl::from_env(),
+    )
+}
+
+fn run_realtime_loop<Load, Sleep, W>(
+    cli: &Cli,
+    out: &mut W,
+    mut load_report: Load,
+    mut sleep: Sleep,
+    mut control: LoopControl,
+) -> anyhow::Result<ExitCode>
+where
+    Load: FnMut() -> anyhow::Result<ReportState>,
+    Sleep: FnMut(Duration),
+    W: Write,
+{
+    let mut report = load_report()?;
     if report.blocks.is_empty() {
-        println!("No Claude data directory found");
-    } else {
-        let _guard = TerminalGuard::enter(&resolved.cli)?;
-        println!("{}", realtime::render_realtime(&report));
+        writeln!(out, "No Claude data directory found")?;
+        return Ok(ExitCode::SUCCESS);
     }
-    Ok(ExitCode::SUCCESS)
+
+    let _guard = TerminalGuard::enter(cli)?;
+    let display_interval = Duration::from_secs_f64(1.0 / cli.refresh_per_second);
+    let data_interval = Duration::from_secs(cli.refresh_rate);
+    let mut next_reload_at = Instant::now() + data_interval;
+
+    loop {
+        render_frame(out, &report)?;
+        if control.should_exit_after_frame() || interrupted() {
+            return Ok(ExitCode::SUCCESS);
+        }
+        sleep(display_interval);
+        if interrupted() {
+            return Ok(ExitCode::SUCCESS);
+        }
+        if Instant::now() >= next_reload_at {
+            report = load_report()?;
+            next_reload_at = Instant::now() + data_interval;
+        }
+    }
 }
+
+fn render_frame<W>(out: &mut W, report: &ReportState) -> anyhow::Result<()>
+where
+    W: Write,
+{
+    write!(out, "\x1b[H\x1b[2J{}\n", realtime::render_realtime(report))?;
+    out.flush()?;
+    Ok(())
+}
+
+struct LoopControl {
+    max_frames: Option<usize>,
+    rendered_frames: usize,
+}
+
+impl LoopControl {
+    fn from_env() -> Self {
+        Self {
+            max_frames: std::env::var("CMONITOR_TEST_MAX_FRAMES")
+                .ok()
+                .and_then(|value| value.parse::<usize>().ok()),
+            rendered_frames: 0,
+        }
+    }
+
+    fn should_exit_after_frame(&mut self) -> bool {
+        self.rendered_frames += 1;
+        self.max_frames
+            .is_some_and(|max_frames| self.rendered_frames >= max_frames)
+    }
+}
+
+fn interrupted() -> bool {
+    INTERRUPTED.load(Ordering::Relaxed)
+}
+
+fn install_interrupt_handler() -> anyhow::Result<()> {
+    INTERRUPTED.store(false, Ordering::Relaxed);
+    ctrlc::set_handler(move || {
+        INTERRUPTED.store(true, Ordering::Relaxed);
+    })?;
+    Ok(())
+}

```

**Documentation:**

```diff
--- a/src/runtime/realtime.rs
+++ b/src/runtime/realtime.rs
@@ -9,6 +9,9 @@
 use crate::runtime::orchestrator::load_report_state;
 use crate::runtime::terminal::TerminalGuard;
 use crate::ui::realtime;

+/// Keeps the terminal guard alive for the full refresh loop so alternate-screen
+/// output persists until an explicit exit path fires. (ref: DL-004)
 pub fn run_realtime_mode(resolved: &ResolvedConfig) -> anyhow::Result<ExitCode> {
     install_interrupt_handler()?;
     run_realtime_loop(
@@ -21,6 +24,9 @@
     )
 }

+/// Owns repeated renders, reload cadence, and bounded test exits in one loop so
+/// realtime behavior stays visible and measurable. (ref: DL-004)
 fn run_realtime_loop<Load, Sleep, W>(
     cli: &Cli,
     out: &mut W,
@@ -55,6 +61,9 @@
     }
 }

+/// Flushes each frame immediately because alternate-screen teardown happens on
+/// guard drop rather than at print boundaries. (ref: DL-004)
 fn render_frame<W>(out: &mut W, report: &ReportState) -> anyhow::Result<()>
 where
     W: Write,
@@ -66,6 +75,9 @@
     Ok(())
 }

+/// Encodes the explicit frame bound used by automated tests without changing
+/// the interactive refresh contract. (ref: DL-004)
 struct LoopControl {
     max_frames: Option<usize>,
     rendered_frames: usize,
@@ -74,6 +86,9 @@
 }

 impl LoopControl {
+    /// Reads the frame bound from environment so regression tests can stop the
+    /// loop deterministically at a chosen render count. (ref: DL-004)
     fn from_env() -> Self {
         Self {
             max_frames: std::env::var("CMONITOR_TEST_MAX_FRAMES")
@@ -84,6 +99,9 @@
         }
     }

+    /// Counts visible frames so automated checks assert the rendered contract
+    /// that users observe. (ref: DL-004)
     fn should_exit_after_frame(&mut self) -> bool {
         self.rendered_frames += 1;
         self.max_frames
@@ -94,6 +112,9 @@
     }
 }

+/// Checks the shared interrupt flag so ctrl-c exits through the same terminal
+/// cleanup path as bounded test runs. (ref: DL-004)
 fn interrupted() -> bool {
     INTERRUPTED.load(Ordering::Relaxed)
+@@ -98,6 +116,9 @@
 }

+/// Uses ctrlc crate to install a safe cross-platform interrupt handler that
+/// sets the shared flag so loop exit passes through terminal restoration. (ref: DL-004, DL-007)
 fn install_interrupt_handler() -> anyhow::Result<()> {
     INTERRUPTED.store(false, Ordering::Relaxed);

```


**CC-M-004-002** (src/runtime/terminal.rs) - implements CI-M-004-002

**Code:**

```diff
--- a/src/runtime/terminal.rs
+++ b/src/runtime/terminal.rs
@@ -1,3 +1,5 @@
+use std::io::{Write, stdout};
+
 use crate::compat::terminal_policy::{TerminalPolicy, default_terminal_policy};
 use crate::config::Cli;
 
@@ -10,6 +12,7 @@
         let policy = default_terminal_policy(cli);
         if policy.force_alternate_screen {
             print!("\x1b[?1049h");
+            stdout().flush()?;
         }
         Ok(Self { policy })
     }
@@ -19,6 +22,7 @@
     fn drop(&mut self) {
         if self.policy.force_alternate_screen {
             print!("\x1b[?1049l");
+            let _ = stdout().flush();
         }
     }
 }

```

**Documentation:**

```diff
--- a/src/runtime/terminal.rs
+++ b/src/runtime/terminal.rs
@@ -12,6 +12,9 @@
         let policy = default_terminal_policy(cli);
         if policy.force_alternate_screen {
             print!("\x1b[?1049h");
+            // Immediate flush makes alternate-screen entry observable to the
+            // realtime loop and test harness. (ref: DL-004)
             stdout().flush()?;
         }
         Ok(Self { policy })
@@ -22,6 +25,9 @@
     fn drop(&mut self) {
         if self.policy.force_alternate_screen {
             print!("\x1b[?1049l");
+            // Drop flushes the return escape so bounded exits and SIGINT both
+            // restore the main screen predictably. (ref: DL-004)
             let _ = stdout().flush();
         }
     }

```


**CC-M-004-003** (tests/parity_realtime.rs) - implements CI-M-004-003

**Code:**

```diff
--- a/tests/parity_realtime.rs
+++ b/tests/parity_realtime.rs
@@ -1,4 +1,7 @@
-//! Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+use std::fs;
+use std::path::{Path, PathBuf};
+use std::process::Command;
+use std::time::{SystemTime, UNIX_EPOCH};
 
 use cmonitor_rs::compat::terminal_policy::{TerminalPolicy, default_terminal_policy};
 use cmonitor_rs::config::{Cli, Plan, Theme, TimeFormat, View};
@@ -6,7 +9,36 @@
 use cmonitor_rs::ui::realtime::render_realtime;
 use time::macros::datetime;
 
-/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
+fn repo_root() -> PathBuf {
+    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
+}
+
+fn bin_path() -> &'static str {
+    env!("CARGO_BIN_EXE_cmonitor-rs")
+}
+
+fn unique_home(name: &str) -> PathBuf {
+    let nanos = SystemTime::now()
+        .duration_since(UNIX_EPOCH)
+        .expect("clock should be after epoch")
+        .as_nanos();
+    std::env::temp_dir().join(format!("cmonitor-rs-{name}-{nanos}"))
+}
+
+fn seed_realtime_home(home: &Path) {
+    if home.exists() {
+        fs::remove_dir_all(home).expect("remove old fixture home");
+    }
+    let project_dir = home.join(".claude/projects/demo-project");
+    fs::create_dir_all(&project_dir).expect("create realtime fixture project dir");
+    let payload = concat!(
+        r#"{"timestamp":"2099-01-01T00:15:00Z","message_id":"m-1","request_id":"r-1","model":"Claude-3-7-Sonnet-20250219","usage":{"input_tokens":10,"output_tokens":2},"cost":0.01}"#,
+        "
+",
+    );
+    fs::write(project_dir.join("session.jsonl"), payload).expect("write realtime fixture");
+}
+
 fn cli() -> Cli {
     Cli {
         plan: Plan::Custom,
@@ -27,7 +59,6 @@
 }
 
 #[test]
-/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn realtime_render_snapshot_is_deterministic() {
     let report = ReportState {
         generated_at: datetime!(2026-03-14 12:30 UTC),
@@ -55,7 +86,6 @@
 }
 
 #[test]
-/// Sanitized fixtures and oracle runs expose JSONL edge cases early. (ref: DL-005)
 fn parity_terminal_policy_keeps_non_tty_gate_disabled() {
     let policy = default_terminal_policy(&cli());
     assert_eq!(
@@ -66,3 +96,49 @@
         }
     );
 }
+
+#[test]
+fn realtime_no_data_fast_path_skips_alternate_screen() {
+    let home = unique_home("realtime-empty");
+    fs::create_dir_all(&home).expect("create empty home");
+
+    let output = Command::new(bin_path())
+        .current_dir(repo_root())
+        .env("HOME", &home)
+        .arg("--view")
+        .arg("realtime")
+        .output()
+        .expect("realtime binary should execute");
+
+    assert!(output.status.success());
+    let stdout = String::from_utf8_lossy(&output.stdout);
+    assert_eq!(stdout, "No Claude data directory found
+");
+    assert!(!stdout.contains("[?1049h"));
+}
+
+#[test]
+fn realtime_data_path_enters_alternate_screen_and_stops_after_test_frames() {
+    let home = unique_home("realtime-live");
+    seed_realtime_home(&home);
+
+    let output = Command::new(bin_path())
+        .current_dir(repo_root())
+        .env("HOME", &home)
+        .env("CMONITOR_TEST_MAX_FRAMES", "3")
+        .arg("--view")
+        .arg("realtime")
+        .arg("--refresh-rate")
+        .arg("1")
+        .arg("--refresh-per-second")
+        .arg("20")
+        .output()
+        .expect("realtime binary should execute");
+
+    assert!(output.status.success());
+    let stdout = String::from_utf8_lossy(&output.stdout);
+    assert!(stdout.contains("[?1049h"));
+    assert!(stdout.contains("[?1049l"));
+    assert!(stdout.matches("[H[2J").count() >= 3);
+    assert!(stdout.matches("active block:").count() >= 3);
+}

```

**Documentation:**

```diff
--- a/tests/parity_realtime.rs
+++ b/tests/parity_realtime.rs
@@ -9,6 +9,9 @@
 use cmonitor_rs::ui::realtime::render_realtime;
 use time::macros::datetime;

+/// Resolves the workspace root once so binary-driven realtime tests execute
+/// under the same relative-path assumptions as the CLI. (ref: DL-004)
 fn repo_root() -> PathBuf {
     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
 }
@@ -12,6 +15,9 @@
     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
 }

+/// Uses the compiled test binary so realtime coverage observes the terminal
+/// contract through the real CLI surface. (ref: DL-004)
 fn bin_path() -> &'static str {
     env!("CARGO_BIN_EXE_cmonitor-rs")
 }
@@ -16,6 +22,9 @@
     env!("CARGO_BIN_EXE_cmonitor-rs")
 }

+/// Allocates isolated homes so repeated-render tests can persist state without
+/// cross-test terminal or fixture leakage. (ref: DL-004)
 fn unique_home(name: &str) -> PathBuf {
     let nanos = SystemTime::now()
         .duration_since(UNIX_EPOCH)
@@ -24,6 +33,9 @@
     std::env::temp_dir().join(format!("cmonitor-rs-{name}-{nanos}"))
 }

+/// Seeds a minimal live dataset so the refresh loop can render multiple frames
+/// without depending on external Claude directories. (ref: DL-004)
 fn seed_realtime_home(home: &Path) {
     if home.exists() {
         fs::remove_dir_all(home).expect("remove old fixture home");
@@ -96,6 +108,9 @@
 }

 #[test]
+/// Guards the no-data fast path so alternate-screen mode stays reserved for
+/// actual realtime frames. (ref: DL-004)
 fn realtime_no_data_fast_path_skips_alternate_screen() {
     let home = unique_home("realtime-empty");
     fs::create_dir_all(&home).expect("create empty home");
@@ -113,6 +128,9 @@
 }

 #[test]
+/// Bounds the live path by frame count so repeated renders are asserted without
+/// manual interruption. (ref: DL-004, DL-005)
 fn realtime_data_path_enters_alternate_screen_and_stops_after_test_frames() {
     let home = unique_home("realtime-live");
     seed_realtime_home(&home);

```


## Execution Waves

- W-001: M-001
- W-002: M-002, M-003, M-004
