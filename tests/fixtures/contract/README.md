# Contract Fixtures

These fixtures document the sanitized HOME layouts used by the parity harness.
Integration tests can override the HOME path at runtime, but every scenario
still follows the same directory contract rooted at `~/.claude-monitor`.

## Layout

- `defaults/home`: no pre-seeded `last_used.json`; proves default merge behavior.
- `overrides/home`: reserved for explicit CLI override scenarios.
- `clear/home`: contains a seeded `last_used.json` so `--clear` removes it.

## Oracle Refresh Flow

1. Update `tests/vendor/claude-code-usage-monitor.manifest.json` when the upstream commit pin or executable import roots change.
2. Run `python3 tests/support/fetch_upstream_oracle.py --check-pin` to verify the vendored executable subset, or rerun without `--check-pin` to refresh it from the pinned source.
3. Run `python3 tests/support/oracle_runner.py --scenario ...` to confirm the real vendored bundle still imports and produces the expected payload.
4. Re-record Rust snapshots only after the oracle payloads and fixture contents are stable.

Bundle-mode execution is the CLI parity contract boundary for this fixture set. (ref: DL-001)

## Sanitization Rules

- Keep only the files required for the scenario under `home/`.
- Replace user-specific paths and identifiers with deterministic placeholders.
- Treat executable upstream behavior as the contract when fixture content and prose disagree.
- Treat oracle import failures as harness failures; do not replace them with emulated payloads.
