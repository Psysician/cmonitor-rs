# CLAUDE.md

## Overview

The `report` directory assembles the shared renderer-neutral state consumed by table and realtime views.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Report directory index and task map | Finding shared totals and report-state ownership |
| `daily_monthly.rs` | Daily and monthly report-row aggregation with per-model cost breakdown (`AggregateRow.per_model`) | Changing grouped totals, per-model breakdowns, or period summaries |
| `mod.rs` | Report module exports | Rewiring report surface area or imports |
| `model.rs` | Report-state, totals, and view-facing data models | Changing renderer input structures or shared totals |
