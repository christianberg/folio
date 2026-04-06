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

### PR #6 — folio add command + serialiser (open, CI green)
- `folio add <path>` — interactive transaction entry via `inquire`-based prompts
- Tag entry shows live-filtered completions drawn from all tags in the existing ledger file
- `src/serialiser.rs`: canonical form (tags alphabetical within each posting), used on write
- `Prompt` infrastructure wrapper: `create()` / `create_null(answers)` — answer queue for tests
- `Filesystem` gains `append_str` + `track_appends()` (same Weak/Arc pattern as Output)
- 9 add command tests, 2 serialiser tests, 5 new integration tests

## What's next

Milestone decisions made this session:
- **Transit sugar deferred** — will add once core system is functional
- **Auto-balance posting deferred** — less relevant with interactive entry flow

Milestone 2 (remaining):
- Query engine: tag filter evaluation (AND, AND NOT, key wildcard), date range filtering
- Report types: ledger, balance sheet, P&L

Later milestones: prediction engine (Naïve Bayes), full TUI (ratatui), bank import, reporting CLI.

## Blockers

None. PR #6 awaiting review/merge.
