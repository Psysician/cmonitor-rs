# cmonitor-rs parity inventory

## Source of truth

- Behavioral target: `Maciek-roboblog/Claude-Code-Usage-Monitor`
- Non-target for bootstrap: provider extensions from `Psysician/c-monitor`

## CLI contract to match

- `--plan` with `pro`, `max5`, `max20`, and `custom`
- `--custom-limit-tokens`
- `--view` with `realtime`, `daily`, and `monthly`
- `--timezone`
- `--time-format`
- `--theme`
- `--refresh-rate`
- `--refresh-per-second`
- `--reset-hour`
- `--log-level`
- `--log-file`
- `--debug`
- `--clear`

## Runtime behavior invariants

- Analyze Claude session data from local files rather than a remote API
- Maintain terminal-first operation for the primary user experience
- Preserve default plan/view semantics before changing UX or terminology
- Rebuild confidence with deterministic fixtures and state-transition tests

## Explicitly deferred from bootstrap

- Codex-only or dual-provider monitoring
- Memory-budget gates and other fork-only observability additions
- Final alias packaging for `claude-monitor`, `cmonitor`, `ccmonitor`, and `ccm`

