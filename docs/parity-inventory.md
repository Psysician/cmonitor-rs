# cmonitor-rs parity inventory

## Source of truth

- Behavioral target: `Maciek-roboblog/Claude-Code-Usage-Monitor`
- Contract authority: upstream executable behavior plus upstream tests
- Explicit non-target: provider extensions from `Psysician/c-monitor`

## Fixture-backed contract cases

- CLI defaults: no saved config, no explicit flags, realtime view default, custom plan default, and upstream version banner shape.
- CLI overrides: `--plan`, `--view`, `--theme`, `--timezone`, `--refresh-rate`, and `--refresh-per-second` override stored values without mutating unrelated fields.
- Saved config persistence: theme, timezone, time format, refresh rate, reset hour, view, and custom-limit state live in `~/.claude-monitor/last_used.json`.
- Clear flow: `--clear` removes the saved config file before continuing the terminal path.
- Oracle pinning: every executable comparison runs against the commit recorded in `tests/vendor/claude-code-usage-monitor.manifest.json`.

## Runtime behavior invariants

- Analyze Claude session data from local files rather than a remote API.
- Maintain terminal-first operation for the primary user experience.
- Preserve five-hour UTC-rounded session-block semantics before intentional fixes.
- Preserve custom-plan P90 fallback order before semantic cleanup.
- Route known upstream quirks through compatibility helpers instead of silently redesigning them.

## Deferred divergence list

- First-discovered-root-only behavior remains compatible until multi-root fixtures prove a safe change.
- Latent `session` view handling remains compatibility-scoped until the renderer contract is fully covered.
- Realtime versus aggregate token-accounting drift remains compatibility-scoped until shared report parity is green.
- Alias takeover and packaging parity stay out of scope until the full fixture matrix passes.
- Fork-only provider features remain explicitly deferred from the parity line.
