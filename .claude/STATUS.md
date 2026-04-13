# Folio — Session Status

## Last updated: 2026-04-13

## What was done

### PR #1 — CLAUDE.md (merged)
- Added project-wide testing conventions based on James Shore's Testing Without Mocks, adapted for Rust

### PR #2 — Milestone 1: parser and validation (merged)
- Cargo project skeleton: lib crate + `folio` binary, `chrono` + `rust_decimal` deps
- `src/types.rs`: `Ledger`, `Transaction`, `Posting`, `Tag`, `ParseError` types
- `src/parser.rs`: parses multi-posting transactions from plain text
- 16 passing tests

### PR #3 — folio check command + infrastructure layer (merged)
- `folio check <path>` — validates a ledger file, exits 0/1
- `Filesystem`, `Output`, `Args` infrastructure wrappers
- `folio::run(args, fs, output)` — testable top-level entry point
- 32 passing tests

### PR #6 — folio add command + serialiser (merged)
- `folio add <path>` — interactive transaction entry with live tag completion
- `Clock` infrastructure wrapper (replaces direct `chrono::Local` calls)
- `Prompt` infrastructure wrapper (nullable `inquire`)
- `Filesystem` gains `append_str` + `track_appends()`
- `src/serialiser.rs`: canonical form (tags alphabetical), blank-line separator logic
- Validation: whitespace in tags, duplicates, required `type:` tag, retry on bad input
- UX: balance-remaining message, default balancing amount, forced balance before done
- 65 passing tests total
- CLAUDE.md updated: TDD workflow, atomic commits, `cargo test` before committing
- Pre-commit git hook installed (runs `cargo test`)

## What's next

Milestone 1 (remaining — deferred):
- Transit sugar expansion
- Auto-balance posting

Milestone 2:
- Query engine: tag filter evaluation (AND, AND NOT, key wildcard), date range filtering
- Report types: ledger, balance sheet, P&L, group-by-time, breakdown, anomaly search

Later milestones: prediction engine (Naïve Bayes), full TUI (ratatui), bank import, reporting CLI.

## Blockers

None.
