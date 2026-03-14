# cmonitor-rs architecture notes

## Runtime loop

1. Load CLI/config state and resolve plan thresholds.
2. Discover and parse Claude session files from the configured data root.
3. Build an in-memory view model for usage, limits, and warning thresholds.
4. Drive a refresh loop that updates derived state on a fixed cadence.
5. Render terminal output without coupling parsing to presentation details.

## Planned module boundaries

- `config`: CLI parsing and persisted preferences
- `discovery`: file traversal and session selection
- `parser`: JSONL decoding and usage normalization
- `domain`: plans, thresholds, and usage state
- `analysis`: burn-rate, predictions, and warning computation
- `ui`: terminal layout and refresh loop

## Implementation guardrails

- Keep the data model reusable by daily/monthly views and realtime UI.
- Prefer fixture-driven session snapshots over synthetic one-off parsing tests.
- Defer provider expansion until the single-provider baseline is stable.

