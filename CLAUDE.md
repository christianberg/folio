# Folio — Project Conventions

## Testing Philosophy: No Mocks

This project uses James Shore's "Testing Without Mocks" pattern language, adapted for Rust.
Reference: https://www.jamesshore.com/v2/projects/nullables/testing-without-mocks

Tests are narrow, state-based, and sociable. Infrastructure is isolated via Nullables, not mocks.

**Non-negotiable rules:**
- No mock crates (`mockall`, `mockito`, `mock_instant`, etc.) in logic tests
- Never assert on whether a method was called — check outputs and state instead
- Infrastructure (filesystem, clocks, network) must always be wrapped; logic code never calls `std::fs`, `SystemTime`, `reqwest`, etc. directly
- Every infrastructure wrapper must support zero-impact instantiation via `create_null()`

---

## Architecture

```
src/
  main.rs               # Entry point: wires real infrastructure, calls run()
  lib.rs                # Re-exports; top-level integration
  logic/                # Pure logic — no I/O, no infrastructure calls
  infrastructure/       # Infrastructure Wrappers only
    mod.rs
    clock.rs            # Example canonical Nullable (see below)
    filesystem.rs
tests/
  *.rs                  # Integration tests (hit real I/O, run separately)
```

Logic code is entirely in `src/logic/`. It receives infrastructure via injected trait objects or generic bounds. It never imports from `std::fs`, `std::net`, `std::time::SystemTime`, `std::env`, etc. directly.

---

## Infrastructure Wrappers

Every piece of infrastructure **must** live in `src/infrastructure/` and implement this structure:

```rust
pub struct Clock {
    inner: ClockInner,
}

enum ClockInner {
    Real,
    Null { now_ms: u64 },
}

impl Clock {
    /// Production instance — uses real system time.
    pub fn create() -> Self {
        Clock { inner: ClockInner::Real }
    }

    /// Nullable instance — no real I/O; configurable via options.
    pub fn create_null(now_ms: u64) -> Self {
        Clock { inner: ClockInner::Null { now_ms } }
    }

    pub fn now_ms(&self) -> u64 {
        match &self.inner {
            ClockInner::Real => {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            }
            ClockInner::Null { now_ms } => *now_ms,
        }
    }
}
```

Once a canonical wrapper exists in the repo, point to it here:
> See `src/infrastructure/clock.rs` for a worked example.

---

## Output Tracking

To observe side effects in tests without mocks, infrastructure wrappers expose an output tracker:

```rust
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct OutputTracker<T> {
    data: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone> OutputTracker<T> {
    pub fn new() -> Self { Self::default() }
    pub fn track(&self, item: T) { self.data.lock().unwrap().push(item); }
    pub fn all(&self) -> Vec<T> { self.data.lock().unwrap().clone() }
}
```

Infrastructure wrappers accept an optional `OutputTracker` and record their side effects into it:

```rust
pub struct Filesystem {
    tracker: Option<OutputTracker<WriteEvent>>,
    // ...
}

pub struct WriteEvent { pub path: String, pub content: String }

impl Filesystem {
    pub fn track_writes(&self) -> OutputTracker<WriteEvent> {
        // attach and return tracker
    }
}
```

Tests then assert on `.all()` rather than on mock expectations.

---

## Writing Tests

### DO: State-based tests with a `run()` helper (Signature Shielding)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_amount_correctly() {
        let result = run(Input { amount_cents: 4500, ..Input::default() });
        assert_eq!(result.formatted, "+45.00");
    }

    struct Input {
        amount_cents: i64,
        currency: &'static str,
    }

    impl Default for Input {
        fn default() -> Self {
            Input { amount_cents: 0, currency: "USD" }
        }
    }

    struct Output {
        formatted: String,
    }

    fn run(input: Input) -> Output {
        let clock = Clock::create_null(1_000_000);
        let svc = PostingFormatter::new(clock);
        let formatted = svc.format(input.amount_cents, input.currency);
        Output { formatted }
    }
}
```

The `run()` helper centralises setup. Adding a new dependency means editing `run()`, not every test.

### DO: Output tracking for side effects

```rust
#[test]
fn writes_transaction_to_file() {
    let fs = Filesystem::create_null();
    let writes = fs.track_writes();
    let ledger = Ledger::new(fs);
    ledger.append(transaction());
    assert_eq!(writes.all().len(), 1);
    assert!(writes.all()[0].content.contains("type:expense"));
}
```

### DON'T: Mock crates

```rust
// NEVER:
use mockall::automock;
#[automock]
trait Clock { fn now_ms(&self) -> u64; }
let mut mock = MockClock::new();
mock.expect_now_ms().returning(|| 1000);
```

### DON'T: Interaction assertions

```rust
// NEVER check whether a method was called — check what the system produced instead.
mock.assert_called_once(); // ← never
```

---

## Narrow Integration Tests

Infrastructure wrappers get their own integration tests in `tests/` that hit real systems (real filesystem, real HTTP). These are the **only** tests allowed to do real I/O. Run them separately from unit tests:

```
cargo test                        # unit tests only (no real I/O)
cargo test --test integration     # integration tests
```

---

## Anti-Patterns to Avoid

| Anti-pattern | Preferred alternative |
|---|---|
| `mockall`, `mockito` in logic tests | Nullables + Output Trackers |
| `assert!(mock.was_called())` | Assert on return values or tracked output |
| Calling `std::fs` directly in logic | Inject a `Filesystem` wrapper |
| `SystemTime::now()` in logic | Inject a `Clock` wrapper |
| `std::env::var()` in logic | Inject an `Env` wrapper |
| Giant test setup repeated per test | Centralise in a `run()` helper (Signature Shielding) |
