use folio::{parse, ParseError, Tag};

#[test]
fn parses_a_simple_expense_transaction() {
    let input = "\
2026-04-03
    food grocery type:expense +45.00
    budget:food checking type:asset -45.00
";

    let ledger = parse(input).expect("should parse without error");

    assert_eq!(ledger.transactions.len(), 1);

    let tx = &ledger.transactions[0];
    assert_eq!(tx.date.to_string(), "2026-04-03");
    assert_eq!(tx.postings.len(), 2);

    let expense = &tx.postings[0];
    assert_eq!(expense.amount.to_string(), "45.00");
    assert!(expense.tags.contains(&Tag::Plain("food".into())));
    assert!(expense.tags.contains(&Tag::Plain("grocery".into())));
    assert!(expense.tags.contains(&Tag::KeyValue("type".into(), "expense".into())));

    let asset = &tx.postings[1];
    assert_eq!(asset.amount.to_string(), "-45.00");
    assert!(asset.tags.contains(&Tag::KeyValue("budget".into(), "food".into())));
    assert!(asset.tags.contains(&Tag::Plain("checking".into())));
    assert!(asset.tags.contains(&Tag::KeyValue("type".into(), "asset".into())));
}

#[test]
fn rejects_unbalanced_transaction() {
    let input = "\
2026-04-03
    food grocery type:expense +45.00
    budget:food checking type:asset -40.00
";

    let err = parse(input).expect_err("should fail for unbalanced transaction");
    assert!(
        matches!(err, ParseError::UnbalancedTransaction { .. }),
        "expected UnbalancedTransaction, got: {err:?}",
    );
}

#[test]
fn rejects_duplicate_key_on_posting() {
    let input = "\
2026-04-03
    food type:expense type:income +45.00
    checking type:asset -45.00
";

    let err = parse(input).expect_err("should fail for duplicate key");
    assert!(
        matches!(err, ParseError::DuplicateKey { .. }),
        "expected DuplicateKey, got: {err:?}",
    );
}

#[test]
fn rejects_duplicate_plain_tag_on_posting() {
    let input = "\
2026-04-03
    food food type:expense +45.00
    checking type:asset -45.00
";

    let err = parse(input).expect_err("should fail for duplicate plain tag");
    assert!(
        matches!(err, ParseError::DuplicateTag { .. }),
        "expected DuplicateTag, got: {err:?}",
    );
}

#[test]
fn rejects_posting_with_no_type_tag() {
    let input = "\
2026-04-03
    food grocery +45.00
    checking -45.00
";

    let err = parse(input).expect_err("should fail for missing type tag");
    assert!(
        matches!(err, ParseError::MissingTypeTag { .. }),
        "expected MissingTypeTag, got: {err:?}",
    );
}

#[test]
fn rejects_posting_with_two_type_tags() {
    let input = "\
2026-04-03
    food type:expense type:income +45.00
    checking type:asset -45.00
";

    let err = parse(input).expect_err("should fail for two type tags");
    // type:* shares the same key, so DuplicateKey fires before MissingTypeTag would.
    assert!(
        matches!(err, ParseError::DuplicateKey { ref key, .. } if key == "type"),
        "expected DuplicateKey for 'type', got: {err:?}",
    );
}

#[test]
fn rejects_posting_with_invalid_type_value() {
    let input = "\
2026-04-03
    food type:snack +45.00
    checking type:asset -45.00
";

    let err = parse(input).expect_err("should fail for invalid type value");
    assert!(
        matches!(err, ParseError::InvalidTypeValue { ref value, .. } if value == "snack"),
        "expected InvalidTypeValue for 'snack', got: {err:?}",
    );
}
