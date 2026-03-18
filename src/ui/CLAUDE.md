# CLAUDE.md

## Overview

The `ui` directory renders report state into daily, monthly, and realtime terminal output.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | UI directory index and task map | Finding rendering entrypoints by view type |
| `mod.rs` | UI module exports | Rewiring renderer surface area or imports |
| `realtime.rs` | Live-session terminal rendering with cost-per-model section | Changing realtime output layout or active-session display |
| `session.rs` | Session-block table rendering (`--view session`) | Changing session-view columns or active-block marking |
| `sparkline.rs` | Unicode block-char sparkline from rolling token history | Changing sparkline width or sample bucketing |
| `summary.rs` | Shared summary formatting helpers | Adjusting cross-view summary presentation |
| `table.rs` | Daily and monthly table rendering with ThemePalette colors | Changing tabular layout, color output, or empty-state output |
