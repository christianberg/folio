# Folio — Session Status

## Last updated: 2026-04-06

## What was done

### PR #1 — CLAUDE.md (merged)
- Added project-wide testing conventions based on James Shore's Testing Without Mocks, adapted for Rust

### PR #2 — Milestone 1: parser and validation (merged)
- Cargo project skeleton: lib crate + `folio` binary, `chrono` + `rust_decimal` deps
- `src/types.rs`: `Ledger`, `Transaction`, `Posting`, `Tag`, `ParseError` types
- `src/parser.rs`: parses multi-posting transactions from plain text
- 16 passing tests

**Validations implemented:**
- Postings sum to zero; no duplicate plain/key tags; exactly one valid `type:*` per posting
- Colon in tag value and numeric plain tags rejected (quoted tags planned later)
- Blank lines inside a transaction rejected

### PR #3 — folio check command + infrastructure layer (merged)
- `folio check <path>` — validates a ledger file, exits 0/1
- `Filesystem` wrapper: `create()` / `create_null(files)`
- `Output` wrapper: `create()` / `create_null()`; `track_stdout/stderr()` works on both real and null instances via `Weak`/`Arc` (zero cost when untracked)
- `Args` wrapper: CLI arg parsing as infrastructure; `create_null(args)` for testable dispatch
- `folio::run(args, fs, output)` — testable top-level entry point; `main.rs` is 3 lines
- 32 passing tests: command tests (nullables only), narrow integration tests, null parity tests, e2e binary tests
- CLAUDE.md updated with real patterns learned from implementation

## What's next

Milestone 1 (remaining):
- Serialisation with canonical tag ordering (alphabetical)
- Round-trip test: parse → serialise → parse produces identical result
- Auto-balance posting (one posting per transaction may omit the amount)

Later milestones: query engine, prediction engine, TUI entry, bank import, reporting CLI.

## Blockers

None.
