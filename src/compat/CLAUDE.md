# CLAUDE.md

## Overview

The `compat` directory isolates upstream quirks and terminal-policy defaults that remain intentionally parity-bound.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Compat directory index and task map | Finding parity-preserving shims before behavioral changes |
| `mod.rs` | Compat module exports | Rewiring compatibility surface area or imports |
| `terminal_policy.rs` | Terminal policy defaults and deferred safety gates | Changing TTY compatibility behavior |
| `upstream.rs` | Upstream behavior adapters and parity helpers | Preserving or retiring upstream quirks |
