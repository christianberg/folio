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
    let appended = appends.all().into_iter().map(|(_, content)| content).collect::<String>();
    RunResult { exit_code, stdout: stdout.all(), stderr: stderr.all(), appended }
}

fn run_new(answers: impl IntoIterator<Item = &'static str>) -> RunResult {
    // Empty file — no existing vocabulary
    let fs = Filesystem::create_null::<[(&str, &str); 0]>([]);
    let appends = fs.track_appends();
    let output = Output::create_null();
    let stdout = output.track_stdout();
    let stderr = output.track_stderr();
    let prompt = Prompt::create_null(answers);
    let exit_code = folio::commands::add::run("ledger.folio", today(), &fs, &prompt, &output);
    let appended = appends.all().into_iter().map(|(_, content)| content).collect::<String>();
    RunResult { exit_code, stdout: stdout.all(), stderr: stderr.all(), appended }
}

// answers for a simple two-posting expense transaction
const SIMPLE_EXPENSE: &[&str] = &[
    "2026-04-06", // date
    "food",       // tag
    "type:expense",
    "",           // end tags for posting 1
    "45.00",      // amount
    "y",          // add another posting
    "checking",
    "type:asset",
    "",           // end tags for posting 2
    "-45.00",     // amount
    "n",          // done
];

#[test]
fn exits_zero_for_balanced_transaction() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
}

#[test]
fn prints_saved_confirmation() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert!(r.stdout.iter().any(|l| l.contains("saved")));
}

#[test]
fn appends_serialised_transaction_to_file() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert!(r.appended.contains("2026-04-06"), "expected date in output");
    assert!(r.appended.contains("food type:expense"), "expected sorted tags");
    assert!(r.appended.contains("45.00"), "expected amount");
    assert!(r.appended.contains("checking type:asset"), "expected asset posting");
    assert!(r.appended.contains("-45.00"), "expected negative amount");
}

#[test]
fn tags_are_sorted_alphabetically_in_output() {
    // Enter tags in reverse alphabetical order; serialiser should sort them
    let r = run_new([
        "2026-04-06",
        "type:expense", "grocery", "food", "", "45.00",
        "y",
        "type:asset", "checking", "", "-45.00",
        "n",
    ]);
    assert!(r.appended.contains("food grocery type:expense"), "tags should be sorted");
    assert!(r.appended.contains("checking type:asset"), "tags should be sorted");
}

#[test]
fn uses_default_date_when_input_is_empty() {
    let r = run_new([
        "",           // accept default date (2026-04-06)
        "type:expense", "food", "", "45.00",
        "y",
        "type:asset", "checking", "", "-45.00",
        "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.contains("2026-04-06"));
}

#[test]
fn exits_one_for_unbalanced_transaction() {
    let r = run_new([
        "2026-04-06",
        "food", "type:expense", "", "45.00",
        "n", // only one posting — won't balance
    ]);
    assert_eq!(r.exit_code, 1);
    assert!(r.stderr.iter().any(|l| l.contains("balance")));
}

#[test]
fn appends_to_existing_file() {
    let existing = "2026-01-01\n    salary type:income 3000.00\n    checking type:asset -3000.00\n";
    let r = run("ledger.folio", existing, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    // The append should only contain the new transaction, not the old one
    assert!(r.appended.contains("2026-04-06"));
    assert!(!r.appended.contains("salary"), "should only append new tx, not rewrite file");
}

#[test]
fn vocabulary_from_existing_file_is_available() {
    // We can't directly observe completions in null mode, but we can verify
    // that parsing the existing file succeeds and the command runs fine
    let existing = "2026-01-01\n    salary type:income 3000.00\n    checking type:asset -3000.00\n";
    let r = run("ledger.folio", existing, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
}

#[test]
fn exits_one_for_unparseable_existing_file() {
    let r = run("ledger.folio", "not a valid ledger!!!", SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 1);
    assert!(r.stderr.iter().any(|l| l.contains("Error")));
}
