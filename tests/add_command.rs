use chrono::NaiveDate;
use folio::commands::add::{ask_tags, type_tag_validator};
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
    fn always_includes_five_default_type_tags() {
        let ledger = parse("").unwrap();
        let vocab = tag_vocabulary(&ledger);
        for tag in &["type:asset", "type:equity", "type:expense", "type:income", "type:liability"] {
            assert!(vocab.contains(&tag.to_string()), "missing {tag}");
        }
    }

    #[test]
    fn default_type_tags_not_duplicated_when_already_in_file() {
        let ledger = parse(SINGLE_EXPENSE).unwrap(); // contains type:expense and type:asset
        let vocab = tag_vocabulary(&ledger);
        assert_eq!(vocab.iter().filter(|t| t.as_str() == "type:expense").count(), 1);
        assert_eq!(vocab.iter().filter(|t| t.as_str() == "type:asset").count(), 1);
    }

    #[test]
    fn empty_ledger_vocabulary_contains_only_defaults() {
        let ledger = parse("").unwrap();
        let vocab = tag_vocabulary(&ledger);
        assert_eq!(vocab.len(), 5);
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

// ── type_tag_validator (pure function) ────────────────────────────────────────

#[test]
fn type_tag_validator_accepts_selection_with_type_tag() {
    assert_eq!(type_tag_validator(&["food", "type:expense"]), None);
}

#[test]
fn type_tag_validator_rejects_selection_without_type_tag() {
    assert!(type_tag_validator(&["food", "checking"]).is_some());
}

#[test]
fn type_tag_validator_rejects_empty_selection() {
    assert!(type_tag_validator(&[]).is_some());
}

// ── Tag entry (ask_tags) ──────────────────────────────────────────────────────
// Tests for tag validation and collection, called directly — no date or amount
// answers needed.
// Phase 1: multi_select answer (comma-separated). Phase 2: single text answer.

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

// Phase 1: multi_select — answers are comma-encoded selections.
// Phase 2: single text answer — space-separated new tags (or "" for none).

#[test]
fn ask_tags_phase_1_collects_selected_tags() {
    // The type: requirement is enforced by inquire's validator in real mode;
    // in null mode we just supply a valid answer directly.
    let vocab = &["type:expense", "type:asset", "food"];
    let r = run_ask_tags(vocab, &[
        "type:expense,food", // phase 1: both selected
        "",                  // phase 2: no additional tags
    ]);
    let tags = r.tags.unwrap();
    assert!(tags.contains(&Tag::Plain("food".to_string())));
    assert!(tags.contains(&Tag::KeyValue("type".to_string(), "expense".to_string())));
}

#[test]
fn ask_tags_collects_valid_tags() {
    let vocab = &["type:expense"];
    let r = run_ask_tags(vocab, &[
        "type:expense", // phase 1
        "food",         // phase 2: one new tag
    ]);
    let tags = r.tags.unwrap();
    assert!(tags.contains(&Tag::Plain("food".to_string())));
    assert!(tags.contains(&Tag::KeyValue("type".to_string(), "expense".to_string())));
}

#[test]
fn ask_tags_phase_2_accepts_space_separated_tags() {
    let vocab = &["type:expense"];
    let r = run_ask_tags(vocab, &[
        "type:expense",  // phase 1
        "food coffee",   // phase 2: two new tags in one input
    ]);
    let tags = r.tags.unwrap();
    assert!(tags.contains(&Tag::Plain("food".to_string())));
    assert!(tags.contains(&Tag::Plain("coffee".to_string())));
}

#[test]
fn ask_tags_rejects_duplicate_plain_tag_in_phase_2() {
    // "food" selected in phase 1; entering "food" again in phase 2 is silently skipped.
    let vocab = &["type:expense", "food"];
    let r = run_ask_tags(vocab, &[
        "type:expense,food", // phase 1: both selected
        "food",              // phase 2: duplicate — should be skipped with warning
    ]);
    let tags = r.tags.unwrap();
    assert_eq!(tags.iter().filter(|t| **t == Tag::Plain("food".to_string())).count(), 1);
    assert!(r.stderr.iter().any(|l| l.contains("Duplicate")));
}

#[test]
fn ask_tags_rejects_duplicate_key_tag_in_phase_2() {
    let vocab = &["type:expense", "type:asset"];
    let r = run_ask_tags(vocab, &[
        "type:expense",  // phase 1
        "type:asset",    // phase 2: duplicate key — skipped with warning
    ]);
    let tags = r.tags.unwrap();
    assert_eq!(tags.iter().filter(|t| matches!(t, Tag::KeyValue(k, _) if k == "type")).count(), 1);
    assert!(r.stderr.iter().any(|l| l.contains("Duplicate")));
}

#[test]
fn ask_tags_returns_none_on_cancellation() {
    // Empty answer queue → prompt returns None → ask_tags propagates it
    let r = run_ask_tags(&["type:expense"], &[]);
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

/// Run `folio add` against a new (empty) file. Default type tags are always in vocabulary,
/// so each posting still needs a multi_select answer.
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

// Minimal two-posting answer sequence.
// Per posting: phase-1 multi_select (must include type:), phase-2 text (space-separated new tags).
const SIMPLE_EXPENSE: &[&str] = &[
    "2026-04-06",    // date_select
    "type:expense",  // phase 1 posting 1: select type tag
    "food",          // phase 2 posting 1: additional tags
    "45.00",         // amount
    "type:asset",    // phase 1 posting 2
    "checking",      // phase 2 posting 2
    "",              // amount: accept default -45.00
    "n",             // no more postings
];

const EXISTING_FILE: &str =
    "2026-01-01\n    salary type:income 3000.00\n    checking type:asset -3000.00\n";

#[test]
fn smoke_records_two_transactions_in_one_session() {
    let r = run_new([
        // Transaction 1
        "2026-04-06",
        "type:expense", "food", "45.00",
        "type:asset", "checking", "", "n",
        // Transaction 2
        "2026-04-07",
        "type:expense", "coffee", "3.50",
        "type:asset", "checking", "", "n",
        // Ctrl-D at date prompt (queue exhausted)
    ]);
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.contains("2026-04-06"), "first transaction missing");
    assert!(r.appended.contains("2026-04-07"), "second transaction missing");
}

#[test]
fn smoke_ctrl_d_at_date_exits_cleanly() {
    // Queue runs out immediately at the first date prompt — should exit 0 with nothing saved.
    let r = run_new([]);
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.is_empty());
}

#[test]
fn smoke_displays_transaction_before_saving() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    // The serialised transaction must appear in stdout so the user can verify it
    assert!(r.stdout.iter().any(|l| l.contains("2026-04-06")), "date not displayed");
    assert!(r.stdout.iter().any(|l| l.contains("food")), "tags not displayed");
    assert!(r.stdout.iter().any(|l| l.contains("45.00")), "amount not displayed");
}

#[test]
fn smoke_saves_balanced_transaction() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(r.stdout.iter().any(|l| l.contains("saved")));
    assert!(r.appended.contains("2026-04-06"));
    assert!(r.appended.contains("food type:expense"));
    assert!(r.appended.contains("checking type:asset"));
}

#[test]
fn smoke_uses_default_date() {
    let r = run_new(["", "type:expense", "food", "45.00", "type:asset", "checking", "", "n"]);
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.contains("2026-04-06"));
}

#[test]
fn smoke_shows_balance_remaining_and_forces_another_posting() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert!(r.stdout.iter().any(|l| l.contains("Balance remaining")));
}

#[test]
fn smoke_default_amount_balances_transaction() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert!(r.appended.contains("-45.00"));
}

#[test]
fn smoke_exits_one_for_unparseable_file() {
    let r = run("ledger.folio", "not valid", SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 1);
    assert!(r.stderr.iter().any(|l| l.contains("Error")));
}

// ── Separator tests ────────────────────────────────────────────────────────────

#[test]
fn no_leading_newline_for_new_file() {
    let r = run_new(SIMPLE_EXPENSE.iter().copied());
    assert!(!r.appended.starts_with('\n'));
}

#[test]
fn separates_with_blank_line_when_file_ends_with_newline() {
    let r = run("ledger.folio", EXISTING_FILE, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.starts_with('\n'));
}

#[test]
fn no_extra_blank_line_when_file_already_ends_with_blank_line() {
    let existing = format!("{EXISTING_FILE}\n");
    let r = run("ledger.folio", &existing, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(!r.appended.starts_with('\n'));
}

#[test]
fn handles_file_without_trailing_newline() {
    let existing = EXISTING_FILE.trim_end_matches('\n');
    let r = run("ledger.folio", existing, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(r.appended.starts_with("\n\n"));
}

#[test]
fn appends_only_new_content_not_whole_file() {
    let r = run("ledger.folio", EXISTING_FILE, SIMPLE_EXPENSE.iter().copied());
    assert_eq!(r.exit_code, 0);
    assert!(!r.appended.contains("salary"));
}
