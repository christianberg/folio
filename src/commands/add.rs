use rust_decimal::Decimal;
use std::collections::HashSet;

use crate::infrastructure::{Clock, Filesystem, Output, Prompt};
use crate::serialiser;
use crate::types::{Ledger, Posting, Tag, Transaction};
use crate::{parser, ParseError};

pub fn run(path: &str, clock: &Clock, fs: &Filesystem, prompt: &Prompt, output: &Output) -> i32 {
    let existing_content = match fs.read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            output.eprintln(&format!("Error reading {path}: {e}"));
            return 1;
        }
    };

    let vocabulary = if existing_content.is_empty() {
        vec![]
    } else {
        match parser::parse(&existing_content) {
            Ok(ledger) => tag_vocabulary(&ledger),
            Err(e) => {
                output.eprintln(&format!("Error parsing {path}: {e}"));
                return 1;
            }
        }
    };

    let date = match prompt.date_select("Date", clock.today()) {
        Some(d) => d,
        None => return 0,
    };

    let postings = match ask_postings(&vocabulary, prompt, output) {
        Some(p) => p,
        None => return 0,
    };

    let transaction = Transaction { date, postings };
    let serialised = serialiser::serialise(&transaction);
    let prefix = append_prefix(&existing_content);

    if let Err(e) = fs.append_str(path, &format!("{prefix}{serialised}\n")) {
        output.eprintln(&format!("Error writing {path}: {e}"));
        return 1;
    }

    output.println("Transaction saved.");
    0
}

fn append_prefix(existing: &str) -> &'static str {
    if existing.is_empty() || existing.ends_with("\n\n") {
        ""
    } else if existing.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    }
}

fn ask_postings(vocabulary: &[String], prompt: &Prompt, output: &Output) -> Option<Vec<Posting>> {
    let mut postings: Vec<Posting> = Vec::new();
    loop {
        let n = postings.len() + 1;
        output.println(&format!("Posting {n}"));

        let tags = ask_tags(vocabulary, prompt, output)?;

        let current_sum: Decimal = postings.iter().map(|p| p.amount).sum();
        let balance_default = if postings.is_empty() { None } else { Some(-current_sum) };
        let amount = prompt.decimal("  Amount", balance_default)?;
        postings.push(Posting { tags, amount });

        let new_sum: Decimal = postings.iter().map(|p| p.amount).sum();
        if !new_sum.is_zero() {
            output.println(&format!("  Balance remaining: {}", -new_sum));
            continue; // must enter another posting to balance
        }

        match prompt.confirm("Add another posting?", false)? {
            false => break,
            true => continue,
        }
    }
    Some(postings)
}

pub fn ask_tags(vocabulary: &[String], prompt: &Prompt, output: &Output) -> Option<Vec<Tag>> {
    let mut tags: Vec<Tag> = Vec::new();
    let mut seen_plain: HashSet<String> = HashSet::new();
    let mut seen_keys: HashSet<String> = HashSet::new();

    // Phase 1: multi-select from existing vocabulary (skip if no vocabulary)
    if !vocabulary.is_empty() {
        let selected = prompt.multi_select("  Tags (select existing)", vocabulary)?;
        for s in &selected {
            match parse_tag(s) {
                Ok(tag) => {
                    match &tag {
                        Tag::Plain(name) => { seen_plain.insert(name.clone()); }
                        Tag::KeyValue(key, _) => { seen_keys.insert(key.clone()); }
                    }
                    tags.push(tag);
                }
                Err(_) => {} // vocabulary tags are already valid; skip silently
            }
        }
    }

    // Phase 2: loop for new tags
    loop {
        let input = prompt.text_with_completions("  Tag (empty to finish)", vocabulary)?;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            let has_type = tags.iter().any(|t| matches!(t, Tag::KeyValue(k, _) if k == "type"));
            if !has_type {
                output.eprintln(
                    "  A type: tag is required \
                     (type:asset, type:liability, type:equity, type:income, or type:expense)",
                );
                continue;
            }
            break;
        }

        if trimmed.contains(char::is_whitespace) {
            output.eprintln("  Tags cannot contain whitespace");
            continue;
        }

        let tag = match parse_tag(trimmed) {
            Ok(t) => t,
            Err(e) => {
                output.eprintln(&format!("  Invalid tag: {e}"));
                continue;
            }
        };

        let duplicate = match &tag {
            Tag::Plain(name) => !seen_plain.insert(name.clone()),
            Tag::KeyValue(key, _) => !seen_keys.insert(key.clone()),
        };
        if duplicate {
            output.eprintln(&format!("  Duplicate tag '{trimmed}', already added"));
            continue;
        }

        tags.push(tag);
    }
    Some(tags)
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

pub fn tag_vocabulary(ledger: &Ledger) -> Vec<String> {
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
