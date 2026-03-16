Fixture notes for ingest parity:

- `sample-home/` is a stable recursive directory tree used to verify JSONL file discovery order.
- `mixed-events.jsonl` mixes malformed rows, cutoff rows, zero-token rows, duplicate composite ids, and accepted rows.
- `mixed-events.jsonl` also carries warning-only system and tool_result rows so limit detection sees the raw evidence preserved by normalization. (ref: DL-002)
- The ingest tests intentionally keep fixtures local and deterministic so parser behavior can evolve without terminal output concerns.
