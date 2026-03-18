# CLAUDE.md

## Overview

The `parser` directory decodes raw JSONL events and normalizes them into typed usage entries.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Parser directory index and task map | Finding decode vs normalization responsibilities |
| `mod.rs` | Parser module exports | Rewiring parser surface area or imports |
| `entries.rs` | Entry normalization, cutoff filtering, dedupe, and token extraction | Changing ingest semantics or typed event shaping |
| `jsonl.rs` | Line-by-line JSONL decoding, parallel parsing, and malformed-line diagnostics | Debugging parse failures or raw event capture |

## Dedup Invariants

`DedupKey` scope is global across all files in a parsing run. The parallel path
(`parse_jsonl_files_parallel`) parses files concurrently via rayon but applies
dedup sequentially after merging, using the same `BTreeSet<DedupKey>` contract as
the sequential `parse_jsonl_file` path. Entries that lack both `message_id` and
`request_id` bypass dedup entirely in both paths.
