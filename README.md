# cmonitor-rs

`cmonitor-rs` is a clean Rust rewrite of [`Maciek-roboblog/Claude-Code-Usage-Monitor`](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor).

The shipped parity line stays focused on the upstream Claude-only monitor contract: local Claude JSONL ingestion, five-hour session blocks, custom-plan P90 limits, and terminal-first daily, monthly, and realtime views.

## Verification

- `cargo test --test parity_cli`
- `cargo test --test parity_ingest`
- `cargo test --test parity_analysis`
- `cargo test --test parity_tables`
- `cargo test --test parity_realtime`
- `make verify`

## Scope

- Preserve upstream executable behavior before semantic fixes.
- Keep the rewrite as a single crate with explicit internal module seams.
- Use a vendored upstream oracle plus sanitized fixture homes for parity evidence.
- Keep local Claude file analysis and terminal-first operation as the core product behavior.

## Regression Guardrails

- CLI parity only counts when the vendored upstream monitor runs in bundle mode from the pinned manifest; emulated fallback is a harness failure, not parity evidence.
- Token math stays driven by typed usage entries, but zero-token `system` and `tool_result` events remain available for limit-warning detection until report assembly finishes.
- Daily and monthly grouping must use the resolved CLI timezone for both bucket boundaries and rendered labels.
- Realtime mode keeps terminal ownership for the full refresh loop and skips alternate-screen setup on the explicit no-data fast path.

## Deferred Until Post-Parity

- Alias takeover or packaging claims for `claude-monitor`, `cmonitor`, `ccmonitor`, and `ccm`.
- Fork-only provider expansion from `Psysician/c-monitor`.
- Safer non-TTY terminal behavior beyond the compatibility-scoped gate.

## Current Architecture

- `config`: CLI parsing plus `last_used.json` persistence.
- `discovery`: Claude root discovery and JSONL file enumeration.
- `parser`: raw JSONL decoding and typed usage-entry normalization.
- `analysis`: session blocks, limit detection, and custom-limit math.
- `report`: shared renderer-neutral report state.
- `ui` and `runtime`: table rendering, realtime rendering, and terminal lifecycle.
- `compat`: isolated upstream quirks and terminal-policy defaults.

## Behavioral Invariants

- Discovery records every matching Claude root but still consumes only the first discovered root until parity evidence proves a broader selection policy is safe.
- JSONL ingestion sorts by timestamp, drops malformed and zero-token rows, and deduplicates only when both `message_id` and `request_id` are present.
- Session blocks round start times down to UTC hours, span five hours, split on block end or five-hour inactivity, and only future-ending non-gap blocks count as active.
- Custom-plan limits come from completed non-gap block P90, then completed sessions, then the default custom minimum.
- Daily, monthly, and realtime surfaces all derive totals from the same report state instead of maintaining separate math.

## Why The Crate Looks Like This

- The rewrite stays in one crate so parity work lands behind stable module seams before any crate-splitting ceremony is introduced.
- Compatibility shims intentionally preserve upstream quirks until fixture-backed parity is green, so user-visible differences remain measurable instead of accidental.

## License

MIT
