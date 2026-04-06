use folio::commands::check;
use folio::infrastructure::{Args, Filesystem, Output};

const VALID: &str = "\
2026-04-03
    food type:expense   45.00
    checking type:asset -45.00
";

const INVALID: &str = "\
2026-04-03
    food type:expense   45.00
    checking type:asset -40.00
";

struct Result {
    exit_code: i32,
    stdout: Vec<String>,
    stderr: Vec<String>,
}

fn run(path: &str, content: &str) -> Result {
    let fs = Filesystem::create_null([(path, content)]);
    let output = Output::create_null();
    let stdout = output.track_stdout();
    let stderr = output.track_stderr();
    let exit_code = check::run(path, &fs, &output);
    Result { exit_code, stdout: stdout.all(), stderr: stderr.all() }
}

#[test]
fn exits_zero_for_valid_file() {
    assert_eq!(run("ledger.folio", VALID).exit_code, 0);
}

#[test]
fn prints_ok_to_stdout_for_valid_file() {
    let r = run("ledger.folio", VALID);
    assert!(r.stdout.iter().any(|l| l.contains("ok")), "expected 'ok' in stdout: {r:?}", r = r.stdout);
}

#[test]
fn exits_nonzero_for_invalid_file() {
    assert_eq!(run("ledger.folio", INVALID).exit_code, 1);
}

#[test]
fn prints_error_to_stderr_for_invalid_file() {
    let r = run("ledger.folio", INVALID);
    assert!(!r.stderr.is_empty(), "expected error on stderr");
    assert!(r.stdout.is_empty(), "expected no stdout on failure");
}

#[test]
fn dispatches_check_subcommand_via_args() {
    let fs = Filesystem::create_null([("ledger.folio", VALID)]);
    let output = Output::create_null();
    let stdout = output.track_stdout();
    let args = Args::create_null(["folio", "check", "ledger.folio"]);
    let code = folio::run(args, &fs, &output);
    assert_eq!(code, 0);
    assert!(stdout.all().iter().any(|l| l.contains("ok")));
}

#[test]
fn exits_nonzero_when_file_not_found() {
    let fs = Filesystem::create_null(std::iter::empty::<(&str, &str)>());
    let output = Output::create_null();
    let stderr = output.track_stderr();
    let code = check::run("missing.folio", &fs, &output);
    assert_eq!(code, 1);
    assert!(!stderr.all().is_empty(), "expected error on stderr");
}
