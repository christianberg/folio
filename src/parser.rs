use chrono::NaiveDate;

use rust_decimal::Decimal;

use crate::types::{Ledger, ParseError, Posting, Tag, Transaction};

pub fn parse(input: &str) -> Result<Ledger, ParseError> {
    let mut transactions = Vec::new();
    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // A non-indented, non-empty line is a transaction date.
        if !line.starts_with(' ') && !line.starts_with('\t') {
            let date = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
                .map_err(|_| ParseError(format!("invalid date: {trimmed}")))?;

            let mut postings = Vec::new();
            while let Some(posting_line) = lines.peek() {
                if posting_line.starts_with(' ') || posting_line.starts_with('\t') {
                    let posting_trimmed = posting_line.trim();
                    if !posting_trimmed.is_empty() {
                        postings.push(parse_posting(posting_trimmed)?);
                    }
                    lines.next();
                } else {
                    break;
                }
            }

            transactions.push(Transaction { date, postings });
        }
    }

    Ok(Ledger { transactions })
}

fn parse_posting(line: &str) -> Result<Posting, ParseError> {
    // The last whitespace-separated token that looks like a number is the amount.
    // Everything before it is tags.
    let tokens: Vec<&str> = line.split_whitespace().collect();

    let amount_idx = tokens
        .iter()
        .rposition(|t| looks_like_amount(t))
        .ok_or_else(|| ParseError(format!("no amount found in posting: {line}")))?;

    let amount: Decimal = tokens[amount_idx]
        .parse()
        .map_err(|_| ParseError(format!("invalid amount: {}", tokens[amount_idx])))?;

    let tags = tokens[..amount_idx]
        .iter()
        .map(|t| parse_tag(t))
        .collect();

    Ok(Posting { tags, amount })
}

fn looks_like_amount(s: &str) -> bool {
    let s = s.strip_prefix('+').unwrap_or(s);
    s.parse::<Decimal>().is_ok()
}

fn parse_tag(s: &str) -> Tag {
    match s.split_once(':') {
        Some((key, value)) => Tag::KeyValue(key.to_string(), value.to_string()),
        None => Tag::Plain(s.to_string()),
    }
}
