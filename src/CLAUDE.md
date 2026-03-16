# CLAUDE.md

## Overview

The `src` root contains crate entrypoints plus the implementation subdirectories that power config, ingest, analysis, reporting, and terminal runtime flows.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Source-root file index and task map | Finding crate entrypoints before deeper module work |
| `analysis/` | Session-block, limit, and custom-plan calculations | Changing report math or warning attachment |
| `compat/` | Upstream-compatibility shims and terminal-policy defaults | Preserving upstream quirks or policy gates |
| `config/` | CLI parsing and persisted last-used settings | Changing flags or config resolution |
| `discovery/` | Claude root enumeration and JSONL file discovery | Modifying root search or input collection |
| `domain/` | Shared typed models for ingest, analysis, and reporting | Adjusting cross-layer data contracts |
| `lib.rs` | Public module exports and crate seams | Wiring new modules or tracing cross-layer dependencies |
| `main.rs` | CLI entrypoint and process exit handling | Changing binary startup or top-level argument dispatch |
| `parser/` | JSONL decoding and usage-entry normalization | Changing ingest semantics or raw-event handling |
| `report/` | Shared renderer-neutral report state and row builders | Modifying daily/monthly aggregation or totals |
| `runtime/` | End-to-end orchestration, realtime loop, and terminal lifecycle | Changing execution flow or terminal ownership |
| `ui/` | Terminal renderers and summary formatting | Adjusting visible table or realtime output |
