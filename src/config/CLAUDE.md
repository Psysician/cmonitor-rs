# CLAUDE.md

## Overview

The `config` directory owns CLI parsing, persisted last-used settings, and resolved runtime configuration.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Config directory index and task map | Finding CLI/config ownership quickly |
| `mod.rs` | CLI structs, config persistence, and resolved settings | Modifying flags, defaults, or saved state behavior |

## CLI Flags Added

| Flag | Type | Default | Purpose |
| --- | --- | --- | --- |
| `--since <duration>` | `Option<String>` | none | Filters JSONL files by mtime; accepts `<N>h` or `<N>d` |
| `--output <format>` | `OutputFormat` | `text` | `text` renders terminal UI; `json` serializes `ReportState` to stdout |
| `--multi-root` | `bool` | `false` | Aggregates all discovered roots instead of the first |

## Key Types

- `OutputFormat` -- `Text` / `Json` enum controlling render path; `Serialize`/`Deserialize` derived
- `Cli.since_threshold` -- resolved `SystemTime` threshold from `--since`; `None` when flag absent or unparseable
- `Cli.output` -- resolved `OutputFormat`; defaults to `Text`
- `Cli.multi_root` -- resolved `bool`; defaults to `false`
