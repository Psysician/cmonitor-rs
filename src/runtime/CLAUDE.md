# CLAUDE.md

## Overview

The `runtime` directory orchestrates loading report state and routing it into table or realtime terminal flows.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Runtime directory index and task map | Locating orchestration and terminal lifecycle code |
| `mod.rs` | Runtime module exports | Rewiring runtime surface area or imports |
| `orchestrator.rs` | Report-state loading, delta cache, and root selection | Changing end-to-end execution flow or caching behavior |
| `realtime.rs` | Realtime loop, refresh scheduling, and sparkline history | Adjusting live refresh behavior, no-data handling, or sparkline |
| `table_mode.rs` | Daily/monthly/session/JSON runtime entrypoints | Modifying non-realtime execution paths |
| `terminal.rs` | Terminal setup, teardown, and alternate-screen handling | Changing terminal lifecycle behavior |
| `theme.rs` | Theme resolution and terminal styling defaults | Adjusting color or style selection |

## Execution Model Notes

- `DeltaCache` is owned by `run_realtime_mode` and passed to `load_report_state`
  on every cycle. Table mode passes `None`; caching has no value in one-shot runs.
- `--output json` short-circuits before the terminal loop in both `run_realtime_mode`
  and `run_table_mode`, so no alternate-screen setup occurs for JSON export.
- `--multi-root` is gated behind a flag; the default single-root path is unchanged
  and all existing parity tests exercise that path without the flag.
