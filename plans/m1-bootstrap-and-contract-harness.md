# Milestone 1: bootstrap and contract harness

## Goal

Establish an implementation-ready Rust baseline for `cmonitor-rs` with fixture-driven validation around the upstream command contract and the core session-state pipeline.

## Deliverables

- Stable CLI surface for upstream monitor flags and defaults
- Fixture corpus covering representative Claude session inputs
- Deterministic state snapshots for realtime, daily, and monthly views
- Initial parser/domain scaffolding for local session discovery and plan thresholds

## Acceptance criteria

- `make verify` passes in CI and locally
- State and fixture tests can validate defaults and view transitions deterministically
- The team can start analytics and TUI work without reopening CLI or scope decisions
- Fork-only provider expansion remains explicitly deferred

## First implementation slice

- Build fixture-driven session discovery and normalization
- Model upstream plan defaults and threshold calculations
- Produce daily/monthly derived state before implementing the full realtime renderer

