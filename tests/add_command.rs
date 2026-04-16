use chrono::NaiveDate;
use folio::commands::add::ask_tags;
use folio::infrastructure::{Clock, Filesystem, Output, Prompt};
use folio::Tag;

// ── Vocabulary ────────────────────────────────────────────────────────────────
// Pure function tests — no I/O, no answer sequences.

mod vocabulary {
    use folio::commands::add::tag_vocabulary;
    use folio::parse;

    const SINGLE_EXPENSE: &str = "\
2026-01-01
    food type:expense 45.00
    checking type:asset -45.00
";

    const TWO_EXPENSES: &str = "\
2026-01-01
    food type:expense 45.00
    checking type:asset -45.00

2026-01-02
    food type:expense 20.00
    checking type:asset -20.00
";

    #[test]
    fn empty_ledger_returns_empty_vocabulary() {
        let ledger = parse("").unwrap();
        assert!(tag_vocabulary(&ledger).is_empty());
    }

    #[test]
    fn extracts_plain_tags_from_ledger() {
        let ledger = parse(SINGLE_EXPENSE).unwrap();
        let vocab = tag_vocabulary(&ledger);
        assert!(vocab.contains(&"food".to_string()));
        assert!(vocab.contains(&"checking".to_string()));
    }

    #[test]
    fn extracts_key_value_tags_from_ledger() {
        let ledger = parse(SINGLE_EXPENSE).unwrap();
        let vocab = tag_vocabulary(&ledger);
        assert!(vocab.contains(&"type:expense".to_string()));
        assert!(vocab.contains(&"type:asset".to_string()));
    }

    #[test]
    fn deduplicates_tags_across_transactions() {
        let ledger = parse(TWO_EXPENSES).unwrap();
        let vocab = tag_vocabulary(&ledger);
        assert_eq!(vocab.iter().filter(|t| t.as_str() == "food").count(), 1);
    }

    #[test]
    fn vocabulary_is_sorted_alphabetically() {
        let ledger = parse(
            "2026-01-01\n\
             \x20   zebra type:expense 45.00\n\
             \x20   apple type:asset -45.00\n",
        )
        .unwrap();
        let vocab = tag_vocabulary(&ledger);
        let mut sorted = vocab.clone();
        sorted.sort();
        assert_eq!(vocab, sorted);
    }
}

// ── Tag entry (ask_tags) ──────────────────────────────────────────────────────
// Tests for tag validation and collection, called directly — no date or amount
// answers needed.

struct TagsResult {
    tags: Option<Vec<Tag>>,
    stderr: Vec<String>,
}

fn run_ask_tags(vocabulary: &[&str], answers: &[&str]) -> TagsResult {
    let vocab: Vec<String> = vocabulary.iter().map(|s| s.to_string()).collect();
    let output = Output::create_null();
    let stderr = output.track_stderr();
    let prompt = Prompt::create_null(answers.iter().copied());
    let tags = ask_tags(&vocab, &prompt, &output);
    TagsResult { tags, stderr: stderr.all() }
}

#[test]
fn ask_tags_requires_type_tag() {
    let r = run_ask_tags(
        &[],
        &["food", "", "type:expense", ""], // empty without type → error; then type added
    );
    assert!(r.tags.is_some());
    assert!(r.stderr.iter().any(|l| l.contains("type:")));
}

#[test]
fn ask_tags_rejects_duplicate_plain_tags() {
    let r = run_ask_tags(&[], &["food", "food", "type:expense", ""]);
    assert!(r.tags.is_some());
    assert!(r.stderr.iter().any(|l| l.contains("Duplicate")));
}

#[test]
fn ask_tags_rejects_duplicate_key_tags() {
    let r = run_ask_tags(&[], &["type:expense", "type:asset", "food", ""]);
    assert!(r.tags.is_some());
    assert!(r.stderr.iter().any(|l| l.contains("Duplicate")));
}

#[test]
fn ask_tags_rejects_whitespace_in_tag() {
    let r = run_ask_tags(&[], &["foo bar", "food", "type:expense", ""]);
    assert!(r.tags.is_some());
    assert!(r.stderr.iter().any(|l| l.contains("whitespace")));
}

#[test]
fn ask_tags_collects_valid_tags() {
    let r = run_ask_tags(&[], &["food", "type:expense", ""]);
    let tags = r.tags.unwrap();
    assert!(tags.contains(&Tag::Plain("food".to_string())));
    assert!(tags.contains(&Tag::KeyValue("type".to_string(), "expense".to_string())));
}

#[test]
fn ask_tags_multi_select_pre_fills_from_vocabulary() {
    // Vocabulary is non-empty → multi_select fires first.
    // Comma-encoded answer selects two tags simultaneously; both must appear in output.
    // (If multi_select didn't fire, phase-2 would consume the comma string as one bad tag.)
    let r = run_ask_tags(
        &["type:expense", "food"],
        &["type:expense,food", ""], // multi_select: both; phase-2: immediately done
    );
    let tags = r.tags.unwrap();
    assert!(tags.contains(&Tag::KeyValue("type".to_string(), "expense".to_string())));
    assert!(tags.contains(&Tag::Plain("food".to_string())));
    assert!(r.stderr.is_empty());
}

#[test]
fn ask_tags_returns_none_on_cancellation() {
    // Empty answer queue → prompt returns None → ask_tags propagates it
    let r = run_ask_tags(&[], &[]);
    assert!(r.tags.is_none());
}

// ── Full command smoke tests ───────────────────────────────────────────────────
// A small number of end-to-end tests covering file I/O, serialisation, and the
// overall prompt flow. These use short answer sequences because tag-validation
// and vocabulary logic are already covered above.

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 4, 6).unwrap()
}

struct RunResult {
    exit_code: i32,
    stdout: Vec<String>,
    stderr: Vec<String>,
    appended: String,
}

/// Run `folio add` against a pre-existing file. Vocabulary is non-empty (the file
/// has one salary transaction), so each posting needs a multi_select answer.
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

/// Run `folio add` against a new (empty) file. No vocabulary → no multi_select phase.
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

// Minimal two-posting answer sequence for a new (empty) file.
// No multi_select phase because vocabulary is empty.
const SIMPLE_EXPENSE_NEW: &[&str] = &[
    "2026-04-06",         // date_select
    "food", "type:expense", "",  // tags for posting 1
    "45.00",              // amount
    "checking", "type:asset", "",  // tags for posting 2
    "",                   // amount: accept default -45.00
    "n",                  // no more postings
];

// Same transaction against a file with existing content.
// Each posting needs an empty multi_select answer (no pre-selection from vocab).
const SIMPLE_EXPENSE_EXISTING: &[&str] = &[
    "2026-04-06",         // date_select
    "",                   // multi_select posting 1
    "food", "type:expense", "",  // tags for posting 1
    "45.00",              // amount
    "",                   // multi_select posting 2
    "checking", "type:asset", "",  // tags for posting 2
    "",                   // amount: accept default -45.00
    "n",                  // no more postings
];

const EXISTING_FILE: &str =
    "2026-01-01\n    salary type:income 3000.00\n    checking type:asset -3000.00\n";

#[test]
fn smoke_saves_balanced_transaction() {
    let r = run_new(SIMPLE_EXPENSE_NEW.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(r.stdout.iter().any(|l| l.contains("saved")));
    assert!(r.appended.contains("2026-04-06"));
    assert!(r.appended.contains("food type:expense"));
    assert!(r.appended.contains("checking type:asset"));
}

#[test]
fn smoke_uses_default_date() {
    let r = run_new(["", "food", "type:expense", "", "45.00", "checking", "type:asset", "", "", "n"]);
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.contains("2026-04-06"));
}

#[test]
fn smoke_shows_balance_remaining_and_forces_another_posting() {
    let r = run_new(SIMPLE_EXPENSE_NEW.iter().copied());
    assert!(r.stdout.iter().any(|l| l.contains("Balance remaining")));
}

#[test]
fn smoke_default_amount_balances_transaction() {
    let r = run_new(SIMPLE_EXPENSE_NEW.iter().copied());
    assert!(r.appended.contains("-45.00"));
}

#[test]
fn smoke_exits_one_for_unparseable_file() {
    let r = run("ledger.folio", "not valid", SIMPLE_EXPENSE_EXISTING.iter().copied());
    assert_eq!(r.exit_code, 1);
    assert!(r.stderr.iter().any(|l| l.contains("Error")));
}

// ── Separator tests ────────────────────────────────────────────────────────────

#[test]
fn no_leading_newline_for_new_file() {
    let r = run_new(SIMPLE_EXPENSE_NEW.iter().copied());
    assert!(!r.appended.starts_with('\n'));
}

#[test]
fn separates_with_blank_line_when_file_ends_with_newline() {
    let r = run("ledger.folio", EXISTING_FILE, SIMPLE_EXPENSE_EXISTING.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.starts_with('\n'));
}

#[test]
fn no_extra_blank_line_when_file_already_ends_with_blank_line() {
    let existing = format!("{EXISTING_FILE}\n");
    let r = run("ledger.folio", &existing, SIMPLE_EXPENSE_EXISTING.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(!r.appended.starts_with('\n'));
}

#[test]
fn handles_file_without_trailing_newline() {
    let existing = EXISTING_FILE.trim_end_matches('\n');
    let r = run("ledger.folio", existing, SIMPLE_EXPENSE_EXISTING.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.starts_with("\n\n"));
}

#[test]
fn appends_only_new_content_not_whole_file() {
    let r = run("ledger.folio", EXISTING_FILE, SIMPLE_EXPENSE_EXISTING.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(!r.appended.contains("salary"));
}
