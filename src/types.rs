use chrono::NaiveDate;

pub struct Ledger {
    pub transactions: Vec<Transaction>,
}

pub struct Transaction {
    pub date: NaiveDate,
    pub postings: Vec<Posting>,
}

pub struct Posting {}

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: {}", self.0)
    }
}

impl std::error::Error for ParseError {}
