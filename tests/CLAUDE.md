# CLAUDE.md

## Overview

The `tests` directory holds the fixture-backed parity suites plus the oracle, snapshot, and fixture assets they depend on.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Test directory index and task map | Finding parity coverage by behavior area |
| `fixtures/` | Sanitized parity inputs and edge-case datasets | Refreshing fixture coverage or reproducing ingest/table/realtime cases |
| `parity_analysis.rs` | Session-block, limit, and shared-report parity tests | Changing analysis math or block invariants |
| `parity_cli.rs` | CLI contract and persisted-config parity tests | Modifying flags, version output, or clear flows |
| `parity_ingest.rs` | Discovery and JSONL normalization parity tests | Changing root discovery or ingest semantics |
| `parity_realtime.rs` | Realtime rendering and terminal-policy parity tests | Modifying live output or terminal defaults |
| `parity_tables.rs` | Daily/monthly table snapshot tests | Changing tabular output or empty-state rendering |
| `snapshots/` | Stored user-facing output baselines | Reviewing expected CLI and table output changes |
| `support/` | Vendored-oracle fetch and execution helpers | Updating upstream parity harnessing |
| `vendor/` | Pinned upstream oracle metadata and bundle inventory | Auditing oracle provenance or refresh scope |
