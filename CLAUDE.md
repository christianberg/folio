# Folio — Project Conventions

## Testing Philosophy: No Mocks

This project uses James Shore's "Testing Without Mocks" pattern language, adapted for Rust.
Reference: https://www.jamesshore.com/v2/projects/nullables/testing-without-mocks

Tests are narrow, state-based, and sociable. Infrastructure is isolated via Nullables, not mocks.

**Non-negotiable rules:**
- No mock crates (`mockall`, `mockito`, `mock_instant`, etc.) in logic tests
- Never assert on whether a method was called — check outputs and state instead
- Infrastructure (filesystem, clocks, network, CLI args) must always be wrapped; logic code never calls `std::fs`, `SystemTime`, `reqwest`, `std::env::args`, etc. directly
- Every infrastructure wrapper must support zero-impact instantiation via `create_null()`

---

## Architecture

```
src/
  main.rs               # Entry point: wires real infrastructure, calls folio::run()
  lib.rs                # Public API + top-level run() dispatch
  infrastructure/       # Infrastructure Wrappers only
    args.rs             # CLI argument parsing (clap)
    filesystem.rs       # File I/O
    output.rs           # stdout/stderr
  commands/             # Command logic — pure, receives infrastructure via injection
    check.rs
tests/
  check_command.rs      # Command tests using nullables (no real I/O)
  integration.rs        # Narrow integration + e2e tests (real I/O)
```

Logic code receives infrastructure via function parameters. It never imports from `std::fs`,
`std::time::SystemTime`, `std::env`, etc. directly.

---

## Infrastructure Wrappers

Every wrapper lives in `src/infrastructure/` and exposes `create()` (real) and `create_null()`
(no external I/O). See the existing wrappers as canonical examples:

> `src/infrastructure/filesystem.rs` — configurable-response nullable (map of path → content)
> `src/infrastructure/output.rs` — output-tracking nullable (stdout/stderr)
> `src/infrastructure/args.rs` — CLI arg parsing; `create_null(args)` uses clap's `parse_from`

### Configurable-response wrapper (e.g. Filesystem)

```rust
pub struct Filesystem(Inner);

enum Inner {
    Real,
    Null(HashMap<String, String>),
}

impl Filesystem {
    pub fn create() -> Self { Self(Inner::Real) }

    pub fn create_null(files: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        Self(Inner::Null(files.into_iter().map(|(k, v)| (k.into(), v.into())).collect()))
    }

    pub fn read_to_string(&self, path: &str) -> Result<String, std::io::Error> {
        match &self.0 {
            Inner::Real => std::fs::read_to_string(path),
            Inner::Null(files) => files.get(path).cloned()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, ...)),
        }
    }
}
```

### Output-tracking wrapper (e.g. Output)

The wrapper holds a `Weak` reference; the `OutputTracker` holds the `Arc`. When no tracker is
registered nothing is allocated. When the tracker is dropped, accumulation stops automatically.
Tracking works on **both real and null instances**.

```rust
pub struct Output {
    stdout: Mutex<Option<Weak<Mutex<Vec<String>>>>>,
    real: bool,
}

impl Output {
    pub fn println(&self, msg: &str) {
        if let Some(arc) = self.stdout.lock().unwrap().as_ref().and_then(Weak::upgrade) {
            arc.lock().unwrap().push(msg.to_string());
        }
        if self.real { println!("{msg}"); }
    }

    pub fn track_stdout(&self) -> OutputTracker {
        let arc = Arc::new(Mutex::new(Vec::new()));
        *self.stdout.lock().unwrap() = Some(Arc::downgrade(&arc));
        OutputTracker(arc)
    }
}

pub struct OutputTracker(Arc<Mutex<Vec<String>>>);

impl OutputTracker {
    pub fn all(&self) -> Vec<String> { self.0.lock().unwrap().clone() }
}
```

---

## Writing Tests

### DO: State-based tests with a `run()` helper (Signature Shielding)

```rust
fn exits_zero_for_valid_file() {
    assert_eq!(run("ledger.folio", VALID).exit_code, 0);
}

struct RunResult { exit_code: i32, stdout: Vec<String>, stderr: Vec<String> }

fn run(path: &str, content: &str) -> RunResult {
    let fs = Filesystem::create_null([(path, content)]);
    let output = Output::create_null();
    let stdout = output.track_stdout();
    let stderr = output.track_stderr();
    let exit_code = check::run(path, &fs, &output);
    RunResult { exit_code, stdout: stdout.all(), stderr: stderr.all() }
}
```

### DO: Output tracking for side effects

```rust
fn prints_ok_to_stdout_for_valid_file() {
    let r = run("ledger.folio", VALID);
    assert!(r.stdout.iter().any(|l| l.contains("ok")));
}
```

### DON'T: Mock crates or interaction assertions

```rust
// NEVER:
mock.expect_now_ms().returning(|| 1000);
mock.assert_called_once();
```

---

## Narrow Integration Tests

`tests/integration.rs` is structured in four sections:

- **Per-wrapper narrow tests** — hit real I/O for one wrapper only (real file read, real stdout)
- **Null parity tests** — verify the null instance returns the same error kinds as the real one; catches null drift
- **Args narrow test** — `create_null` only; `create()` is a one-liner wrapping clap, too thin for a narrow test
- **`e2e` section** — spawns the real binary via `env!("CARGO_BIN_EXE_folio")` for full-stack smoke tests

---

## Workflow

- **Red/green TDD:** write a failing test first, then write the minimum code to make it pass.
- **Atomic commits:** each commit covers one piece of functionality — tests and implementation together.
- **Always run `cargo test` before committing.** A pre-commit hook enforces this, but run it manually too.

---

## Anti-Patterns to Avoid

| Anti-pattern | Preferred alternative |
|---|---|
| `mockall`, `mockito` in logic tests | Nullables + Output Trackers |
| `assert!(mock.was_called())` | Assert on return values or tracked output |
| Calling `std::fs` directly in logic | Inject a `Filesystem` wrapper |
| `SystemTime::now()` in logic | Inject a `Clock` wrapper |
| `std::env::args()` / `clap::Parser::parse()` in logic | Inject an `Args` wrapper |
| Tracking only on null instances | Use `Weak`/`Arc` so tracking works on real instances too |
| Giant test setup repeated per test | Centralise in a `run()` helper (Signature Shielding) |
