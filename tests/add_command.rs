use chrono::NaiveDate;
use folio::infrastructure::{Clock, Filesystem, Output, Prompt};

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
    let clock = Clock::create_null(today());
    let output = Output::create_null();
    let stdout = output.track_stdout();
    let stderr = output.track_stderr();
    let prompt = Prompt::create_null(answers);
    let exit_code = folio::commands::add::run(path, &clock, &fs, &prompt, &output);
    let appended = appends.all().into_iter().map(|(_, c)| c).collect::<String>();
    RunResult { exit_code, stdout: stdout.all(), stderr: stderr.all(), appended }
}

fn run_new(answers: impl IntoIterator<Item = &'static str>) -> RunResult {
    let fs = Filesystem::create_null(std::iter::empty::<(&str, &str)>());
    let appends = fs.track_appends();
    let clock = Clock::create_null(today());
    let output = Output::create_null();
    let stdout = output.track_stdout();
    let stderr = output.track_stderr();
    let prompt = Prompt::create_null(answers);
    let exit_code = folio::commands::add::run("ledger.folio", &clock, &fs, &prompt, &output);
    let appended = appends.all().into_iter().map(|(_, c)| c).collect::<String>();
    RunResult { exit_code, stdout: stdout.all(), stderr: stderr.all(), appended }
}

// Two-posting expense. After posting 1 (unbalanced) the loop continues
// automatically. Posting 2 accepts the default balancing amount with "".
const SIMPLE_EXPENSE: &[&str] = &[
    "2026-04-06",
    "food", "type:expense", "",
    "45.00",
    "checking", "type:asset", "",
    "",   // accept default -45.00
    "n",
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
        "not-a-date",
        "2026-04-06",
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
        "oops",
        "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stderr.iter().any(|l| l.contains("Invalid amount")));
}

#[test]
fn forces_another_posting_when_unbalanced() {
    let r = run_new([
        "2026-04-06",
        "type:expense", "food", "", "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stdout.iter().any(|l| l.contains("Balance remaining")));
}

#[test]
fn default_amount_balances_transaction() {
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
        "food", "",           // no type: → error, re-prompt
        "type:expense", "",   // add it
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
        "food", "food", "type:expense", "", // duplicate food → error, then type:expense
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
        "type:expense", "type:asset", "food", "", // dup type → error
        "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stderr.iter().any(|l| l.contains("Duplicate")));
}

#[test]
fn rejects_tag_with_whitespace() {
    let r = run_new([
        "2026-04-06",
        "foo bar", "food", "type:expense", "",
        "45.00",
        "type:asset", "checking", "", "", "n",
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.stderr.iter().any(|l| l.contains("whitespace")));
}

// ── Separator / appending tests ────────────────────────────────────────────────

#[test]
fn no_leading_newline_for_new_file() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert!(!r.appended.starts_with('\n'), "new file should not start with newline");
}

#[test]
fn separates_with_blank_line_when_file_ends_with_newline() {
    let existing = "2026-01-01\n    salary type:income 3000.00\n    checking type:asset -3000.00\n";
    let r = run("ledger.folio", existing, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.starts_with('\n'), "should prepend blank line separator");
}

#[test]
fn no_extra_blank_line_when_file_already_ends_with_blank_line() {
    let existing = "2026-01-01\n    salary type:income 3000.00\n    checking type:asset -3000.00\n\n";
    let r = run("ledger.folio", existing, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(!r.appended.starts_with('\n'), "should not add extra blank line");
}

#[test]
fn handles_file_without_trailing_newline() {
    let existing = "2026-01-01\n    salary type:income 3000.00\n    checking type:asset -3000.00";
    let r = run("ledger.folio", existing, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.starts_with("\n\n"), "should add newline then blank line");
}

#[test]
fn appends_only_new_content_not_whole_file() {
    let existing = "2026-01-01\n    salary type:income 3000.00\n    checking type:asset -3000.00\n";
    let r = run("ledger.folio", existing, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(!r.appended.contains("salary"), "appended content should not include existing transactions");
}

#[test]
fn exits_one_for_unparseable_existing_file() {
    let r = run("ledger.folio", "not a valid ledger!!!", SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 1);
    assert!(r.stderr.iter().any(|l| l.contains("Error")));
}
