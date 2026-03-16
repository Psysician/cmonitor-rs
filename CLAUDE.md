# CLAUDE.md

## Overview

Repository root holds package metadata, top-level docs, and directory entrypoints for the parity-focused rewrite.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Root file index and task map | Locating package metadata or top-level docs |
| `.gitignore` | Ignored build outputs and local state paths | Adjusting tracked vs generated files |
| `Cargo.lock` | Locked Rust dependency graph | Auditing or refreshing resolved dependencies |
| `Cargo.toml` | Package metadata, crate targets, and dependency policy | Changing crate wiring or dependency versions |
| `docs/` | Architecture notes and parity reference material | Understanding high-level design or scope boundaries |
| `LICENSE` | MIT license text | Verifying reuse and distribution terms |
| `Makefile` | Formatting, lint, check, and test recipes | Running or changing repository verification commands |
| `plans/` | Planner-authored execution and review artifacts | Replaying milestone intent or audit trails |
| `README.md` | Product scope, parity guarantees, and architectural rationale | Understanding the shipped behavior or release positioning |
| `rust-toolchain.toml` | Pinned Rust toolchain version | Aligning compiler and tooling expectations |
| `src/` | Crate entrypoints and implementation modules | Tracing runtime behavior or changing code paths |
| `tests/` | Parity suites, fixtures, snapshots, and oracle assets | Updating regression coverage or fixture evidence |
