# CLAUDE.md

## Overview

The `runtime` directory orchestrates loading report state and routing it into table or realtime terminal flows.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Runtime directory index and task map | Locating orchestration and terminal lifecycle code |
| `mod.rs` | Runtime module exports | Rewiring runtime surface area or imports |
| `orchestrator.rs` | Report-state loading and top-level runtime dispatch | Changing end-to-end execution flow |
| `realtime.rs` | Realtime loop and refresh scheduling | Adjusting live refresh behavior or no-data handling |
| `table_mode.rs` | Daily/monthly runtime entrypoints | Modifying non-realtime execution paths |
| `terminal.rs` | Terminal setup, teardown, and alternate-screen handling | Changing terminal lifecycle behavior |
| `theme.rs` | Theme resolution and terminal styling defaults | Adjusting color or style selection |
