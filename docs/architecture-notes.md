# cmonitor-rs architecture notes

## Runtime loop

1. Load CLI arguments and merge saved `last_used.json` preferences.
2. Discover Claude data roots and preserve the upstream first-root compatibility decision.
3. Collect and decode JSONL files into raw events plus malformed-line diagnostics.
4. Normalize usage entries, build five-hour session blocks, detect limit events, and compute custom-plan limits.
5. Project the analysis output into one shared report state.
6. Render either deterministic daily or monthly tables, or hand the same report state to realtime mode.

## Shared report boundary

- `parser` owns raw-event decoding and typed entry normalization.
- `analysis` owns session blocks, limit detection, and P90 custom-limit math.
- `report` owns the renderer-neutral totals and grouped aggregates.
- `ui` only turns report state into strings or terminal widgets.
- `runtime` owns orchestration, refresh cadence, and terminal lifecycle.
- `compat` isolates upstream quirks such as first-root selection, latent `session` view handling, and alternate-screen defaults.

## Implementation guardrails

- Daily and monthly views must consume the same `ReportState` as realtime mode.
- Table mode must remain deterministic and avoid alternate-screen control.
- Intentional safety fixes stay behind compatibility gates until the parity matrix is green.
- Fixture-backed oracle comparisons outrank README prose when the two disagree.
