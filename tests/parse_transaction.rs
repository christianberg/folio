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
    food budget:food budget:car type:expense +45.00
    checking type:asset -45.00
";

    let err = parse(input).expect_err("should fail for duplicate key");
    assert!(
        matches!(err, ParseError::DuplicateKey { ref key, .. } if key == "budget"),
        "expected DuplicateKey for 'budget', got: {err:?}",
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
fn parses_transaction_covering_all_five_account_types() {
    let input = "\
2026-04-01
    salary type:income                 +3000.00
    checking type:asset                -2000.00
    savings type:asset                 -500.00
    mortgage type:liability            +300.00
    retained-earnings type:equity      -700.00
    rent type:expense                  -100.00
";

    let ledger = parse(input).expect("should parse without error");
    let tx = &ledger.transactions[0];
    assert_eq!(tx.postings.len(), 6);

    let types: Vec<&str> = tx.postings.iter().filter_map(|p| {
        p.tags.iter().find_map(|t| match t {
            folio::Tag::KeyValue(k, v) if k == "type" => Some(v.as_str()),
            _ => None,
        })
    }).collect();

    assert!(types.contains(&"income"));
    assert!(types.contains(&"asset"));
    assert!(types.contains(&"liability"));
    assert!(types.contains(&"equity"));
    assert!(types.contains(&"expense"));
}

#[test]
fn rejects_unbalanced_transaction_with_multiple_postings() {
    let input = "\
2026-04-01
    salary type:income    +3000.00
    checking type:asset   -2000.00
    savings type:asset    -500.00
";
    // sum is +500.00, not zero

    let err = parse(input).expect_err("should fail for unbalanced multi-posting transaction");
    assert!(
        matches!(err, ParseError::UnbalancedTransaction { .. }),
        "expected UnbalancedTransaction, got: {err:?}",
    );
}

#[test]
fn parses_multiple_transactions() {
    let input = "\
2026-04-01
    salary type:income   +3000.00
    checking type:asset  -3000.00

2026-04-03
    food type:expense    +45.00
    checking type:asset  -45.00
";

    let ledger = parse(input).expect("should parse without error");
    assert_eq!(ledger.transactions.len(), 2);
    assert_eq!(ledger.transactions[0].date.to_string(), "2026-04-01");
    assert_eq!(ledger.transactions[1].date.to_string(), "2026-04-03");
}

#[test]
fn parses_unsigned_positive_amount() {
    let input = "\
2026-04-03
    food type:expense    45.00
    checking type:asset  -45.00
";

    let ledger = parse(input).expect("should parse without error");
    let expense = &ledger.transactions[0].postings[0];
    assert_eq!(expense.amount.to_string(), "45.00");
}

#[test]
fn ignores_comments_and_blank_lines() {
    let input = "\
# this is a comment

2026-04-03
    # inline comment on a posting line
    food type:expense    45.00
    checking type:asset  -45.00

# trailing comment
";

    let ledger = parse(input).expect("should parse without error");
    assert_eq!(ledger.transactions.len(), 1);
    assert_eq!(ledger.transactions[0].postings.len(), 2);
}

#[test]
fn parses_empty_input() {
    let ledger = parse("").expect("should parse empty input without error");
    assert_eq!(ledger.transactions.len(), 0);
}

#[test]
fn rejects_blank_line_inside_transaction() {
    let input = "\
2026-04-03
    food type:expense  45.00

    checking type:asset  -45.00
";

    let err = parse(input).expect_err("should fail for blank line inside transaction");
    assert!(
        matches!(err, ParseError::BlankLineInTransaction { .. }),
        "expected BlankLineInTransaction, got: {err:?}",
    );
}

#[test]
fn rejects_colon_in_tag_value() {
    let input = "\
2026-04-03
    ref:INV:001 type:expense  45.00
    checking type:asset       -45.00
";

    let err = parse(input).expect_err("should fail for colon in tag value");
    assert!(
        matches!(err, ParseError::ColonInTagValue { ref tag, .. } if tag == "ref:INV:001"),
        "expected ColonInTagValue, got: {err:?}",
    );
}

#[test]
fn rejects_numeric_plain_tag() {
    let input = "\
2026-04-03
    food 2024 type:expense  45.00
    checking type:asset     -45.00
";

    let err = parse(input).expect_err("should fail for numeric plain tag");
    assert!(
        matches!(err, ParseError::NumericTag { ref tag, .. } if tag == "2024"),
        "expected NumericTag, got: {err:?}",
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
