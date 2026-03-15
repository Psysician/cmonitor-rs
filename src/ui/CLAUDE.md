# CLAUDE.md

## Overview

The `ui` directory renders report state into daily, monthly, and realtime terminal output.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | UI directory index and task map | Finding rendering entrypoints by view type |
| `mod.rs` | UI module exports | Rewiring renderer surface area or imports |
| `realtime.rs` | Live-session terminal rendering | Changing realtime output layout or active-session display |
| `summary.rs` | Shared summary formatting helpers | Adjusting cross-view summary presentation |
| `table.rs` | Daily and monthly table rendering | Changing tabular layout or empty-state output |
