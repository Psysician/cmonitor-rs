# cmonitor-rs

`cmonitor-rs` is a fresh Rust rewrite scaffold for [`Maciek-roboblog/Claude-Code-Usage-Monitor`](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor).

This repository is intentionally separate from the existing [`Psysician/c-monitor`](https://github.com/Psysician/c-monitor) Python fork. The initial parity target is the original upstream monitor behavior, not the fork's added multi-provider features.

## Current Status

- Cargo binary project with CI, lint/test commands, and issue templates
- Initial CLI scaffold for plan/view/theme/refresh configuration
- Project docs capturing upstream parity, runtime architecture, and the first milestone

## Planned Compatibility

The first parity line targets the upstream Claude monitor CLI:

- realtime, daily, and monthly monitoring views
- plan selection for `pro`, `max5`, `max20`, and `custom`
- refresh, theme, timezone, time-format, reset-hour, and logging flags
- local Claude session analysis with terminal-oriented output

Fork-only provider features from the current `Psysician/c-monitor` repository, such as Codex or dual-provider monitoring, are intentionally deferred until the upstream-compatible baseline exists.

## Development

Prerequisites:

- Rust `1.93.0`
- `make`

Verification:

```bash
make verify
```

Current binary behavior is intentionally limited to a contract-aware placeholder:

```bash
cargo run -- --plan max20 --view daily --theme dark
```

The CLI parser is present, but the session reader, analytics engine, and terminal UI are not implemented yet.

## Repository Docs

- `docs/parity-inventory.md`: upstream CLI and runtime behaviors to preserve
- `docs/architecture-notes.md`: planned runtime loop, state model, and module boundaries
- `plans/m1-bootstrap-and-contract-harness.md`: first implementation milestone and acceptance criteria

## License

MIT

