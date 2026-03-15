# CLAUDE.md

## Overview

The `parser` directory decodes raw JSONL events and normalizes them into typed usage entries.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Parser directory index and task map | Finding decode vs normalization responsibilities |
| `mod.rs` | Parser module exports | Rewiring parser surface area or imports |
| `entries.rs` | Entry normalization, cutoff filtering, dedupe, and token extraction | Changing ingest semantics or typed event shaping |
| `jsonl.rs` | Line-by-line JSONL decoding and malformed-line diagnostics | Debugging parse failures or raw event capture |
