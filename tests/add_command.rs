use chrono::NaiveDate;
use folio::infrastructure::{Filesystem, Output, Prompt};

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 4, 6).unwrap()
}

struct RunResult {
    exit_code: i32,
    stdout: Vec<String>,
    stderr: Vec<String>,
    appended: String,
}

fn run(path: &str, existing: &str, answers: impl IntoIterator<Item = &'static str>) -> RunResult {
    let fs = Filesystem::create_null([(path, existing)]);
    let appends = fs.track_appends();
    let output = Output::create_null();
    let stdout = output.track_stdout();
    let stderr = output.track_stderr();
    let prompt = Prompt::create_null(answers);
    let exit_code = folio::commands::add::run(path, today(), &fs, &prompt, &output);
    let appended = appends.all().into_iter().map(|(_, c)| c).collect::<String>();
    RunResult { exit_code, stdout: stdout.all(), stderr: stderr.all(), appended }
}

fn run_new(answers: impl IntoIterator<Item = &'static str>) -> RunResult {
    let fs = Filesystem::create_null(std::iter::empty::<(&str, &str)>());
    let appends = fs.track_appends();
    let output = Output::create_null();
    let stdout = output.track_stdout();
    let stderr = output.track_stderr();
    let prompt = Prompt::create_null(answers);
    let exit_code = folio::commands::add::run("ledger.folio", today(), &fs, &prompt, &output);
    let appended = appends.all().into_iter().map(|(_, c)| c).collect::<String>();
    RunResult { exit_code, stdout: stdout.all(), stderr: stderr.all(), appended }
}

// Two-posting expense transaction. After posting 1 (unbalanced), the loop
// automatically continues — no "y" prompt. Posting 2 uses the default amount
// (empty = accept -45.00 balance). Then "n" to finish.
const SIMPLE_EXPENSE: &[&str] = &[
    "2026-04-06", // date
    "food", "type:expense", "", // posting 1 tags
    "45.00",      // posting 1 amount
    // unbalanced → loop continues automatically, no confirm prompt
    "checking", "type:asset", "", // posting 2 tags
    "",           // accept default amount (-45.00)
    "n",          // don't add another posting
];

#[test]
fn exits_zero_for_balanced_transaction() {
    assert_eq!(run_new(SIMPLE_EXPENSE.iter().copied()).exit_code, 0);
}

#[test]
fn prints_saved_confirmation() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert!(r.stdout.iter().any(|l| l.contains("saved")));
}

#[test]
fn appends_serialised_transaction_to_file() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert!(r.appended.contains("2026-04-06"));
    assert!(r.appended.contains("food type:expense"));
    assert!(r.appended.contains("45.00"));
    assert!(r.appended.contains("checking type:asset"));
    assert!(r.appended.contains("-45.00"));
}

#[test]
fn tags_are_sorted_alphabetically_in_output() {
    let r = run_new([
        "2026-04-06",
        "type:expense", "grocery", "food", "", "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert!(r.appended.contains("food grocery type:expense"), "tags should be sorted");
    assert!(r.appended.contains("checking type:asset"), "tags should be sorted");
}

#[test]
fn uses_default_date_when_input_is_empty() {
    let r = run_new([
        "",  // accept default date (2026-04-06)
        "type:expense", "food", "", "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.contains("2026-04-06"));
}

#[test]
fn retries_on_invalid_date() {
    let r = run_new([
        "not-a-date",   // invalid → error, re-prompt
        "2026-04-06",   // valid
        "type:expense", "food", "", "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stderr.iter().any(|l| l.contains("Invalid date")));
    assert!(r.appended.contains("2026-04-06"));
}

#[test]
fn retries_on_invalid_amount() {
    let r = run_new([
        "2026-04-06",
        "type:expense", "food", "",
        "oops",    // invalid → error, re-prompt
        "45.00",   // valid
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stderr.iter().any(|l| l.contains("Invalid amount")));
}

#[test]
fn forces_another_posting_when_unbalanced() {
    // After posting 1, transaction is unbalanced — loop continues without asking
    let r = run_new([
        "2026-04-06",
        "type:expense", "food", "", "45.00",
        // no confirm here; balance remaining message shown, then posting 2 prompt
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stdout.iter().any(|l| l.contains("Balance remaining")));
}

#[test]
fn default_amount_balances_transaction() {
    // After posting 1 with 45.00, the default for posting 2 should be -45.00
    // Accepting the default (empty) should produce a balanced transaction
    let r = run_new([
        "2026-04-06",
        "type:expense", "food", "", "45.00",
        "type:asset", "checking", "",
        "",   // accept default (-45.00)
        "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.contains("-45.00"));
}

#[test]
fn validates_type_tag_required() {
    let r = run_new([
        "2026-04-06",
        "food",           // tag without type:
        "",               // try to finish → error, re-prompt
        "type:expense",   // add missing type
        "",               // finish
        "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stderr.iter().any(|l| l.contains("type:")));
}

#[test]
fn rejects_duplicate_plain_tags() {
    let r = run_new([
        "2026-04-06",
        "food",
        "food",           // duplicate → error, re-prompt
        "type:expense",
        "",
        "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stderr.iter().any(|l| l.contains("Duplicate")));
}

#[test]
fn rejects_duplicate_key_tags() {
    let r = run_new([
        "2026-04-06",
        "type:expense",
        "type:asset",     // duplicate key → error, re-prompt
        "",
        "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stderr.iter().any(|l| l.contains("Duplicate")));
}

#[test]
fn rejects_tag_with_whitespace() {
    // In null mode the answer is the trimmed string — but we can pass a
    // whitespace-containing answer to exercise the validation path
    let r = run_new([
        "2026-04-06",
        "foo bar",        // whitespace → error, re-prompt
        "food",
        "type:expense",
        "",
        "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stderr.iter().any(|l| l.contains("whitespace")));
}

#[test]
fn appends_to_existing_file() {
    let existing = "2026-01-01\n    salary type:income 3000.00\n    checking type:asset -3000.00\n";
    let r = run("ledger.folio", existing, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.contains("2026-04-06"));
    assert!(!r.appended.contains("salary"), "should only append new tx, not rewrite file");
}

#[test]
fn exits_one_for_unparseable_existing_file() {
    let r = run("ledger.folio", "not a valid ledger!!!", SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 1);
    assert!(r.stderr.iter().any(|l| l.contains("Error")));
}
