use folio::{parse, Tag};

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
