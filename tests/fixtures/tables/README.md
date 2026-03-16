Fixture notes for table parity:

- Table snapshots are built from deterministic in-memory report state so layout regressions stay obvious.
- Daily and monthly snapshots share the same report pipeline and differ only in the aggregate grouping label.
- Empty-state coverage stays explicit because table mode must not depend on realtime terminal control.
