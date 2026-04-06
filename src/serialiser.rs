use crate::types::{Tag, Transaction};

pub fn serialise(transaction: &Transaction) -> String {
    let mut lines = vec![transaction.date.to_string()];
    for posting in &transaction.postings {
        let mut tags: Vec<String> = posting.tags.iter().map(tag_to_string).collect();
        tags.sort();
        let tag_part = tags.join(" ");
        lines.push(format!("    {tag_part} {}", posting.amount));
    }
    lines.join("\n")
}

fn tag_to_string(tag: &Tag) -> String {
    match tag {
        Tag::Plain(s) => s.clone(),
        Tag::KeyValue(k, v) => format!("{k}:{v}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Posting, Transaction};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn amount(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    fn plain(s: &str) -> Tag {
        Tag::Plain(s.to_string())
    }

    fn kv(k: &str, v: &str) -> Tag {
        Tag::KeyValue(k.to_string(), v.to_string())
    }

    #[test]
    fn serialises_date_line() {
        let tx = Transaction {
            date: date("2026-04-06"),
            postings: vec![
                Posting { tags: vec![plain("food"), kv("type", "expense")], amount: amount("45.00") },
                Posting { tags: vec![plain("checking"), kv("type", "asset")], amount: amount("-45.00") },
            ],
        };
        assert!(serialise(&tx).starts_with("2026-04-06\n"));
    }

    #[test]
    fn tags_sorted_alphabetically_within_posting() {
        let tx = Transaction {
            date: date("2026-04-06"),
            postings: vec![Posting {
                tags: vec![kv("type", "expense"), plain("grocery"), plain("food")],
                amount: amount("45.00"),
            }],
        };
        let s = serialise(&tx);
        // food < grocery < type:expense alphabetically
        assert!(s.contains("    food grocery type:expense 45"));
    }

    #[test]
    fn two_posting_transaction_round_trips() {
        let input = "\
2026-04-06
    food grocery type:expense 45.00
    budget:food checking type:asset -45.00";
        let ledger = crate::parser::parse(input).unwrap();
        let serialised = serialise(&ledger.transactions[0]);
        assert_eq!(serialised, input);
    }
}
