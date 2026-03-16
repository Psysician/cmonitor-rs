# CLAUDE.md

## Overview

The `discovery` directory finds Claude data roots and enumerates JSONL files in deterministic order.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Discovery directory index and task map | Locating root and file-enumeration logic |
| `mod.rs` | Discovery module exports | Rewiring discovery surface area or imports |
| `jsonl_files.rs` | Recursive JSONL file collection and stable ordering | Changing ingest file selection or traversal behavior |
| `roots.rs` | Standard root lookup, custom root handling, and primary-root selection | Modifying discovery precedence or compatibility rules |
