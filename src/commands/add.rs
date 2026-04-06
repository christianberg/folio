use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::infrastructure::{Filesystem, Output, Prompt};
use crate::serialiser;
use crate::types::{Ledger, Posting, Tag, Transaction};
use crate::{parser, ParseError};

pub fn run(path: &str, today: NaiveDate, fs: &Filesystem, prompt: &Prompt, output: &Output) -> i32 {
    let vocabulary = match load_vocabulary(path, fs, output) {
        Some(v) => v,
        None => return 1,
    };

    let date = match ask_date(today, prompt, output) {
        Some(d) => d,
        None => return 0,
    };

    let postings = match ask_postings(&vocabulary, prompt, output) {
        Some(p) => p,
        None => return 0,
    };

    let sum: Decimal = postings.iter().map(|p| p.amount).sum();
    if !sum.is_zero() {
        output.eprintln(&format!("Transaction does not balance (sum: {sum})"));
        return 1;
    }

    let transaction = Transaction { date, postings };
    let serialised = serialiser::serialise(&transaction);

    if let Err(e) = fs.append_str(path, &format!("{serialised}\n")) {
        output.eprintln(&format!("Error writing {path}: {e}"));
        return 1;
    }

    output.println("Transaction saved.");
    0
}

fn load_vocabulary(path: &str, fs: &Filesystem, output: &Output) -> Option<Vec<String>> {
    match fs.read_to_string(path) {
        Ok(content) => match parser::parse(&content) {
            Ok(ledger) => Some(tag_vocabulary(&ledger)),
            Err(e) => {
                output.eprintln(&format!("Error parsing {path}: {e}"));
                None
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Some(vec![]),
        Err(e) => {
            output.eprintln(&format!("Error reading {path}: {e}"));
            None
        }
    }
}

fn ask_date(today: NaiveDate, prompt: &Prompt, output: &Output) -> Option<NaiveDate> {
    let input = prompt.text_with_default("Date", &today.to_string())?;
    match NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d") {
        Ok(d) => Some(d),
        Err(_) => {
            output.eprintln(&format!("Invalid date: {input}"));
            None
        }
    }
}

fn ask_postings(vocabulary: &[String], prompt: &Prompt, output: &Output) -> Option<Vec<Posting>> {
    let mut postings = Vec::new();
    loop {
        let n = postings.len() + 1;
        output.println(&format!("Posting {n}"));

        let tags = ask_tags(vocabulary, prompt, output)?;
        let amount = ask_amount(prompt, output)?;
        postings.push(Posting { tags, amount });

        match prompt.confirm("Add another posting?", false)? {
            false => break,
            true => continue,
        }
    }
    Some(postings)
}

fn ask_tags(vocabulary: &[String], prompt: &Prompt, output: &Output) -> Option<Vec<Tag>> {
    let mut tags = Vec::new();
    loop {
        let input = prompt.text_with_completions("  Tag (empty to finish)", vocabulary)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            break;
        }
        match parse_tag(trimmed) {
            Ok(tag) => tags.push(tag),
            Err(e) => output.eprintln(&format!("  Invalid tag: {e}")),
        }
    }
    Some(tags)
}

fn ask_amount(prompt: &Prompt, output: &Output) -> Option<Decimal> {
    let input = prompt.text_with_completions("  Amount", &[])?;
    match input.trim().parse::<Decimal>() {
        Ok(amount) => Some(amount),
        Err(_) => {
            output.eprintln(&format!("  Invalid amount: {}", input.trim()));
            None
        }
    }
}

fn parse_tag(s: &str) -> Result<Tag, ParseError> {
    match s.split_once(':') {
        Some((key, value)) => {
            if value.contains(':') {
                return Err(ParseError::ColonInTagValue {
                    tag: s.to_string(),
                    line: s.to_string(),
                });
            }
            Ok(Tag::KeyValue(key.to_string(), value.to_string()))
        }
        None => {
            if s.parse::<Decimal>().is_ok() {
                return Err(ParseError::NumericTag {
                    tag: s.to_string(),
                    line: s.to_string(),
                });
            }
            Ok(Tag::Plain(s.to_string()))
        }
    }
}

fn tag_vocabulary(ledger: &Ledger) -> Vec<String> {
    use std::collections::BTreeSet;
    let mut tags: BTreeSet<String> = BTreeSet::new();
    for tx in &ledger.transactions {
        for posting in &tx.postings {
            for tag in &posting.tags {
                tags.insert(match tag {
                    Tag::Plain(s) => s.clone(),
                    Tag::KeyValue(k, v) => format!("{k}:{v}"),
                });
            }
        }
    }
    tags.into_iter().collect()
}
