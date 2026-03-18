# CLAUDE.md

## Overview

The `domain` directory defines the typed models shared across ingest, analysis, and reporting.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Domain directory index and task map | Locating shared data structures before behavior changes |
| `mod.rs` | Domain module exports | Rewiring shared type visibility |
| `plan.rs` | Usage-plan and limit-domain types | Adjusting plan naming or limit metadata |
| `session_block.rs` | Session-block, gap-block, warning models, and per-model stats (`model_stats`) | Changing block semantics, per-model breakdown, or downstream report fields |
| `usage_entry.rs` | Typed usage-entry and token-usage models | Modifying ingest output or token accounting fields |
