use chrono::NaiveDate;

use crate::types::{Ledger, ParseError, Posting, Transaction};

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
    if line.is_empty() {
        return Err(ParseError("empty posting line".into()));
    }
    Ok(Posting {})
}
