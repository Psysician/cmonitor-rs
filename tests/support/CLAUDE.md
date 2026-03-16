# CLAUDE.md

## Overview

The `tests/support` directory contains helper scripts that pin and execute the vendored upstream oracle.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Test-support directory index and task map | Locating oracle acquisition and execution helpers |
| `fetch_upstream_oracle.py` | Vendored-oracle pin verification and refresh logic | Checking upstream provenance or refreshing the bundle |
| `oracle_runner.py` | Oracle scenario execution and snapshot payload generation | Comparing Rust output against vendored upstream behavior |
