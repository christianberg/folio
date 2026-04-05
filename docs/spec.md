# Folio — Tag-Based Plain-Text Accounting Specification

## Overview

A plain-text double-entry accounting system that replaces hierarchical account names with tag sets. Instead of declaring a rigid account hierarchy upfront, the user applies tags freely to postings and chooses grouping dimensions at report time. The system is strict double-entry under the hood, with ergonomic sugar for common patterns.

---

## Core Concepts

### Tags

A tag is a short, whitespace-free string applied to a posting. Tags are unordered. Two syntactic forms exist:

| Form | Example | Constraint |
|------|---------|------------|
| Plain tag | `food` | Any number per posting |
| Key:value tag | `type:expense` | At most one tag per key per posting |

Key:value tags enforce mutual exclusivity within a key. Multiple keys are independent.

Special characters (e.g. `!`) are allowed in tag names. There are no reserved plain tags — only reserved keys (see below).

Quoted tags allow whitespace: `note:"birthday dinner"`, `ref:"INV-2024-042"`.

### Reserved Keys

Exactly five key:value pairs carry accounting semantics. Every posting must have exactly one `type:*` tag.

| Tag | Meaning |
|-----|---------|
| `type:asset` | Asset account |
| `type:liability` | Liability account |
| `type:equity` | Equity account |
| `type:income` | Income |
| `type:expense` | Expense |

All other keys (`budget:*`, `transit:*`, and any user-defined keys) are user-defined and carry no built-in semantics beyond mutual exclusivity within their key.

### Accounts

There are no declared accounts. Each unique combination of tags on a posting implicitly defines an account. Querying and reporting select postings by tag membership, not by account path.

---

## File Format

### Encoding

- Plain UTF-8 text files
- One or more files; convention TBD (e.g. one file per year)
- Lines beginning with `#` are comments
- Tool-generated entries sort tags alphabetically within each posting; manual entry may use any order

### Transaction Structure

```
DATE
    [DATE] TAGS... AMOUNT [CURRENCY]
    [DATE] TAGS... AMOUNT [CURRENCY]
    ...
```

- **Transaction line:** a date in `YYYY-MM-DD` format, alone on the line
- **Posting lines:** indented, one per posting
- **Blank lines** separate transactions

### Posting Line Fields (in order)

| Field | Required | Notes |
|-------|----------|-------|
| Date | Optional | Override date for this posting only (see Transit Sugar) |
| Tags | Mandatory | Whitespace-separated; quoted tags allowed |
| Amount | Mandatory* | Decimal number with sign; one posting per transaction may omit amount to auto-balance |
| Currency | Optional | ISO 4217 code; reserved for v2 |

### Amount Sign Convention

- Positive amounts increase the balance of the tagged account
- `type:expense` postings are positive (expense increases)
- Offsetting `type:asset` postings are negative
- All postings in a transaction must sum to zero

### Examples

**Simple expense:**
```
2026-04-03
    food grocery type:expense                +45.00
    budget:food checking type:asset          -45.00
```

**Income:**
```
2026-04-01
    salary type:income                       +3000.00
    budget:unallocated checking type:asset   -3000.00
```

**Budget fill:**
```
2026-04-01
    budget:unallocated checking type:asset   +600.00
    budget:food checking type:asset          -400.00
    budget:car checking type:asset
```

**Debit card purchase with transit sugar (see below):**
```
2026-03-28
    car gas type:expense                     +45.00
    2026-03-31 budget:car checking transit:visa type:asset -45.00
```

---

## Special Syntax: Transit Sugar

When a posting carries a `transit:KEY` tag and an override date different from the transaction date, the system automatically expands it into three postings:

**As written:**
```
2026-03-28
    car gas type:expense                              +45.00
    2026-03-31 budget:car checking transit:visa type:asset -45.00
```

**Expanded internally:**
```
2026-03-28
    car gas type:expense                              +45.00
    transit:visa type:liability                       -45.00   ← auto-generated

2026-03-31
    transit:visa type:liability                       +45.00   ← auto-generated
    budget:car checking transit:visa type:asset       -45.00
```

The `transit:visa` liability balance should always net to zero once all settlements are recorded. A non-zero balance indicates a pending (unsettled) transaction. This is an automatic integrity check.

The `transit:KEY` tag names the transit liability implicitly — no account declaration required.

---

## Budgeting

Budgets are real envelope accounts, not reporting targets. Budget envelopes are implicit sub-accounts of an asset (e.g. `checking`) distinguished by their `budget:*` tag.

**Flow:**

1. Income arrives into `budget:unallocated`
2. A budget-fill transaction moves funds from `budget:unallocated` to named envelopes
3. Expenses draw down envelopes via the `budget:*` tag on the asset posting

`budget:unallocated` balance:
- Positive: income received but not yet allocated
- Negative: over-allocated (spending exceeds filled envelopes)

Budget envelopes support arbitrary use cases: monthly allowances, annual bill reserves, long-term savings goals. All are expressed as standard transactions.

---

## Prediction Engine

### Principle

Tag predictions are derived entirely from the plaintext data files. No separate model files are stored. A cache may be used for performance but is always derived from source data.

### Algorithm

Naïve Bayes over tag co-occurrence, computed from historical postings. Inputs to prediction:

- Tags already entered on the current posting
- Tags on other postings in the same transaction
- Posting amount (optional signal)

### Behaviour During Entry

1. User enters one or more tags
2. System computes posterior probability for all other known tags
3. Tags above a threshold (default 90%) are **auto-applied** and shown visibly
4. Tags below threshold are shown in descending probability order for manual selection
5. Auto-applied tags can be removed; suggested tags can be accepted

### Bootstrap

The model learns from the first transaction. Strong correlations (e.g. `grocery` → `type:expense`, `visa` → `type:liability`) emerge within days of normal use. No cold-start configuration required.

### Model Updates

The model updates live during data entry as new tags are confirmed. Editing or deleting historical entries triggers re-derivation from source.

---

## Bank Statement Import

1. System parses statement (CSV or similar)
2. For each merchant string token: if the token matches an existing tag (exact or fuzzy/substring), apply it and use it to trigger predictions
3. Unrecognised tokens are presented as candidate new tags for the user to accept or skip
4. User confirms, amends, and resolves each imported transaction interactively
5. Confirmed transactions are appended to the data file in standard format

---

## Query Language

### Syntax

```
TAGS... [-TAGS...] 
```

- Space-separated tags are AND conditions
- `-tag` excludes postings with that tag (AND NOT)
- Key:value tags are valid filter terms
- `budget` (bare key) matches any posting with a `budget:*` tag
- No OR operator; if OR is needed, a unifying tag should be added to the data instead

**Examples:**

| Query | Meaning |
|-------|---------|
| `food type:expense` | All food expenses |
| `checking type:asset` | All checking account postings |
| `type:expense -vacation` | All expenses except vacation |
| `budget:car` | All postings drawing from the car envelope |
| `transit:visa` | All visa debit card transit postings |

---

## Report Types

### 1. Ledger

Lists all postings matching the tag filter, in date order, with running balance.

- For `type:asset` and `type:liability`: shows opening balance at start of period, then running balance
- For `type:income` and `type:expense`: no opening balance (period totals only)

### 2. Group by Time

Collects all matching postings and sums amounts over a time period (daily / weekly / monthly / yearly). Shows net change per period.

### 3. Balance Sheet Snapshot

All `type:asset` and `type:liability` postings summed to a given date. Shows net worth.

### 4. P&L / Cash Flow

All `type:income` and `type:expense` postings in a date range, summed. Difference is savings rate.

### 5. Breakdown by Tag Dimension

User specifies a dimension: an ordered list of tags expected to be mutually exclusive across the data.

```
breakdown [food transport housing leisure] for type:expense monthly
```

Each posting matching the base filter is assigned to the first matching dimension tag. Postings matching none fall into "other." Postings matching multiple emit a warning (data quality hint).

### 6. Cross-Cutting Intersection

AND queries naturally express cross-cutting reports that hierarchical systems cannot:

```
vacation food type:expense
```

Returns all expenses tagged both `vacation` and `food` — e.g. restaurant meals while travelling.

### 7. Anomaly / Data Hygiene Search

Filter postings by tag combinations and amount thresholds for data quality checks:

```
shell type:expense -car
```

Finds shell-tagged expenses not tagged car — likely a data entry error.

The prediction engine can also flag transactions where high-confidence co-occurrence expectations are violated.

---

## Implementation Milestones

### Milestone 1: Data Model and Parser

- Define and document the complete file format grammar (EBNF or similar)
- Implement parser for transaction/posting/tag/amount syntax
- Implement validation: postings sum to zero, exactly one `type:*` per posting, at most one value per key
- Implement transit sugar expansion
- Round-trip test: parse → serialise → parse produces identical result
- Canonical tag ordering (alphabetical) for tool-generated output

**Deliverable:** library crate/module with parse and validate functions, comprehensive test suite.

### Milestone 2: Query Engine

- Implement tag filter evaluation (AND, AND NOT, key wildcard)
- Implement date range filtering
- Implement posting retrieval and balance accumulation
- Implement the seven report types (ledger, group-by-time, balance sheet, P&L, breakdown, intersection, anomaly search)
- Implement breakdown with user-specified dimension and "other" bucket

**Deliverable:** query API usable from CLI and future UI layers.

### Milestone 3: Prediction Engine

- Implement Naïve Bayes tag co-occurrence model derived from parsed data
- Inputs: tags on current posting, tags on sibling postings, posting amount
- Output: ranked list of candidate tags with probabilities
- Configurable auto-apply threshold (default 90%)
- Live update during entry session
- Optional on-disk cache with invalidation on file change

**Deliverable:** prediction API returning ranked tag candidates given partial posting state.

### Milestone 4: Interactive Entry (CLI/TUI)

- Interactive transaction entry using prediction engine
- Tag completion from known vocabulary
- Auto-applied tags shown visibly, removable
- Suggested tags shown in probability order, selectable
- Support for multi-posting transactions and split postings
- Transit sugar: user enters override date and `transit:KEY`; expansion is automatic
- Append confirmed transaction to data file in canonical form

**Deliverable:** working TUI entry flow using ratatui (Rust) or equivalent.

### Milestone 5: Bank Statement Import

- CSV parser for common bank export formats (configurable column mapping)
- Merchant string tokenisation and fuzzy matching against known tag vocabulary
- Per-transaction interactive confirmation using prediction engine
- Append confirmed transactions to data file

**Deliverable:** import command with interactive confirmation flow.

### Milestone 6: Reporting CLI

- Command-line interface for all report types
- Date range flags, dimension specification for breakdown reports
- Plain-text tabular output
- Optional CSV export

**Deliverable:** complete CLI covering all query and report types.

---

## Implementation Notes

### Executable

The binary is named `folio`. All CLI commands are subcommands: `folio add`, `folio import`, `folio report`, `folio balance`.

### Language

Rust is recommended:
- Single compiled binary, no runtime dependency
- Strong performance for parsing and numeric work
- Good CLI/TUI ecosystem (clap, ratatui)
- Correctness guarantees well-suited to financial data

### Non-Goals (v1)

- Multi-currency support (prepare data model but do not implement)
- Migration from beancount/ledger
- Graphical UI (TUI is sufficient)
- Network sync or multi-user access

### Design Principles

- All data in plaintext files; no hidden state
- Prediction model is always derived from data, never authoritative
- If OR is needed in a query, that signals a missing tag — add the tag, not the operator
- Categorisation (overlapping tags) and budgeting (mutually exclusive `budget:*` tags) are separate concerns handled by the same mechanism
- Transit sugar is purely ergonomic; the expanded form is always valid double-entry
