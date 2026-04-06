use folio::parse;

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
}
