# Folio — Session Status

## Last updated: 2026-04-06

## What was done

### PR #1 — CLAUDE.md (merged)
- Added project-wide testing conventions based on James Shore's Testing Without Mocks, adapted for Rust
- Documents Nullable infrastructure wrappers, Output Tracking, Signature Shielding patterns
- Forbids `mockall`/`mockito` in logic tests

### PR #2 — Milestone 1: parser and validation (merged)
- Cargo project skeleton: lib crate + `folio` binary, `chrono` + `rust_decimal` deps
- `src/types.rs`: `Ledger`, `Transaction`, `Posting`, `Tag`, `ParseError` types
- `src/parser.rs`: parses multi-posting transactions from plain text
- 16 passing tests covering parsing and all validations

**Validations implemented:**
- Postings in a transaction must sum to zero
- No duplicate plain tags on a posting
- No duplicate keys among key:value tags
- Every posting must have exactly one `type:*` tag with a valid value (asset/liability/equity/income/expense)
- Colon in tag value raises an error (quoted tags planned for later)
- Numeric plain tags raise an error (ambiguous with amounts)
- Blank lines inside a transaction raise an error

## What's next

Milestone 1 (remaining):
- Serialisation with canonical tag ordering (alphabetical)
- Round-trip test: parse → serialise → parse produces identical result
- Auto-balance posting (one posting per transaction may omit the amount)

Later milestones: query engine, prediction engine, TUI entry, bank import, reporting CLI.

## Blockers

None.
