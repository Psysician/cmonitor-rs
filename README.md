# cmonitor-rs

**Fast, visual Claude Code usage monitor written in Rust.**

A clean Rust rewrite of [Claude-Code-Usage-Monitor](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor) with parallel parsing, delta-reload caching, and a modern terminal dashboard.

```
 ╭─── CLAUDE CODE USAGE MONITOR ──────────────────────────╮
 │ custom plan · UTC · ◉ active                            │
 ╰─────────────────────────────────────────────────────────╯

   Tokens      ━━━━━━━╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌  26.3%   11,592 / 44,000
     in 4,210  out 7,382
   Cost        ━━━━━━━━━━━╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌  42.7%   $7.69 / $18
   Messages    ━━╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌   8.9%   4 / 45
   Cache       ◆ 88% hit  (31,200 read / 4,100 write)

 ─────────────────────────────────────────────────────────

   Time Left   ━━━━━━━━━━━━━━━╌╌╌╌╌╌╌╌╌╌╌           2h 15m
   Models      ━━━━━━━━━━━━━━━━━━━━━━━━━━  sonnet 100%

 ─────────────────────────────────────────────────────────

   ⚡ Burn       386.4 tok/min       $ Rate    $0.2563/min
   ⏱  Resets     2026-03-18 22:00 UTC

   Cost by Model
     sonnet   ━━━━━━━━━━━━ $7.6900   100%  11,592 tok

 ─────────────────────────────────────────────────────────
   Token History  ▁▂▃▄▅▆▇█▇▆▅▃▂▁▂▃▅▇█▇
   ◉ Active session · Ctrl+C to exit
```

## Features

- **53x less RAM** than the Python original via streaming typed parser
- **Rayon parallel parsing** with post-merge dedup for large JSONL sets
- **Delta-reload caching** in realtime mode — only re-parses changed files
- **Input/Output token split** with cache hit indicator
- **Plan cost limits** matching upstream (Pro $18, Max5 $35, Max20 $140)
- **P90 auto-detection** for custom plan token limits
- **Session view** (`--view session`) with duration, tokens, cost per session
- **Sparkline** token history in the realtime dashboard
- **JSON export** (`--output json`) for piping to other tools
- **Multi-root** (`--multi-root`) aggregates all discovered Claude data directories
- **`--since` filtering** (e.g. `--since 24h`, `--since 7d`)
- **3 themes**: dark (default), light, classic

## Install

### From release binary

Download from [Releases](https://github.com/Psysician/cmonitor-rs/releases):

```bash
# Linux x86_64
curl -L https://github.com/Psysician/cmonitor-rs/releases/latest/download/cmonitor-rs-linux-x86_64 -o cmonitor-rs
chmod +x cmonitor-rs
sudo mv cmonitor-rs /usr/local/bin/
```

### From source

```bash
cargo install --git https://github.com/Psysician/cmonitor-rs
```

### Build locally

```bash
git clone https://github.com/Psysician/cmonitor-rs
cd cmonitor-rs
cargo install --path .
```

## Usage

```bash
# Realtime dashboard (default)
cmonitor-rs

# Daily/monthly table views
cmonitor-rs --view daily
cmonitor-rs --view monthly

# Session history
cmonitor-rs --view session

# Plan selection
cmonitor-rs --plan pro          # Token limit: 44K, Cost limit: $18
cmonitor-rs --plan max5         # Token limit: 220K, Cost limit: $35
cmonitor-rs --plan max20        # Token limit: 880K, Cost limit: $140

# Filtering
cmonitor-rs --since 24h         # Only files from last 24 hours
cmonitor-rs --since 7d          # Only files from last 7 days

# Export
cmonitor-rs --output json       # JSON to stdout
cmonitor-rs --view daily --output json

# Multi-root
cmonitor-rs --multi-root        # Aggregate all Claude data directories

# Themes
cmonitor-rs --theme dark        # Default
cmonitor-rs --theme light
cmonitor-rs --theme classic
```

## Architecture

```
config/       CLI parsing + last_used.json persistence
discovery/    Claude root discovery + JSONL file enumeration
parser/       Streaming JSONL decode + typed entry normalization + rayon parallel
analysis/     Session blocks, limit detection, P90 custom limits
report/       Shared renderer-neutral report state
ui/           Realtime dashboard, table, session, sparkline renderers
runtime/      Orchestrator, delta cache, terminal lifecycle, theme
compat/       Upstream behavior shims
```

## Verification

```bash
cargo test --all     # 24 parity tests
make verify          # fmt + clippy + check + test
```

## License

MIT
