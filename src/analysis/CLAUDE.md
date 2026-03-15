# CLAUDE.md

## Overview

The `analysis` directory turns normalized usage entries into session blocks, limit events, and custom-plan limits.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `CLAUDE.md` | Analysis directory index and task map | Locating block and limit calculations |
| `blocks.rs` | Session-block construction, gap insertion, and active-block logic | Changing block boundaries or inactivity rules |
| `limits.rs` | Limit-event detection and warning attachment | Modifying limit classification or warning ranges |
| `mod.rs` | Analysis module exports | Rewiring analysis surface area or imports |
| `p90.rs` | P90 custom-limit calculation and fallbacks | Adjusting percentile logic or custom-plan defaults |
