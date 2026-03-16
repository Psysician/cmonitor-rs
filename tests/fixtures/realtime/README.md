Fixture notes for realtime parity:

- Realtime snapshots are derived from shared report state rather than a separate live-only model.
- Terminal policy coverage stays focused on the parity baseline: alternate screen on, non-TTY safety gate off.
- The runtime loop reuses the same orchestrator loader as table mode, so behavior differences stay isolated to rendering and terminal lifecycle.
