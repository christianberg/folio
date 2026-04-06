use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct Ledger {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug)]
pub struct Transaction {
    pub date: NaiveDate,
    pub postings: Vec<Posting>,
}

#[derive(Debug)]
pub struct Posting {
    pub tags: Vec<Tag>,
    pub amount: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    Plain(String),
    KeyValue(String, String),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidDate { line: String },
    InvalidAmount { token: String },
    MissingAmount { line: String },
    UnbalancedTransaction { date: NaiveDate, sum: Decimal },
    DuplicateKey { key: String, line: String },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidDate { line } => write!(f, "invalid date: {line}"),
            ParseError::InvalidAmount { token } => write!(f, "invalid amount: {token}"),
            ParseError::MissingAmount { line } => write!(f, "no amount found in posting: {line}"),
            ParseError::UnbalancedTransaction { date, sum } => {
                write!(f, "transaction on {date} does not balance (sum: {sum})")
            }
            ParseError::DuplicateKey { key, line } => {
                write!(f, "duplicate key '{key}' in posting: {line}")
            }
        }
    }
}

impl std::error::Error for ParseError {}
