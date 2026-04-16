use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::VecDeque;
use std::sync::Mutex;

pub struct Prompt(Inner);

enum Inner {
    Real,
    Null(Mutex<VecDeque<String>>),
}

impl Prompt {
    pub fn create() -> Self {
        Self(Inner::Real)
    }

    pub fn create_null(answers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self(Inner::Null(Mutex::new(answers.into_iter().map(|a| a.into()).collect())))
    }

    /// Calendar date picker. Returns None if the user cancelled.
    pub fn date_select(&self, message: &str, default: NaiveDate) -> Option<NaiveDate> {
        match &self.0 {
            Inner::Real => inquire::DateSelect::new(message).with_default(default).prompt().ok(),
            Inner::Null(q) => {
                let answer = q.lock().unwrap().pop_front().unwrap_or_default();
                if answer.is_empty() {
                    Some(default)
                } else {
                    NaiveDate::parse_from_str(answer.trim(), "%Y-%m-%d").ok()
                }
            }
        }
    }

    /// Multi-select from a list of options. Returns selected items.
    /// `preselected` items are shown as already checked (by value match against options).
    /// In null mode, a single answer encodes selections as a comma-separated string;
    /// empty string means no selections.
    pub fn multi_select(
        &self,
        message: &str,
        options: &[String],
        preselected: &[String],
    ) -> Option<Vec<String>> {
        match &self.0 {
            Inner::Real => {
                let defaults: Vec<usize> = preselected
                    .iter()
                    .filter_map(|s| options.iter().position(|o| o == s))
                    .collect();
                inquire::MultiSelect::new(message, options.to_vec())
                    .with_default(&defaults)
                    .prompt()
                    .ok()
            }
            Inner::Null(q) => {
                let answer = q.lock().unwrap().pop_front()?;
                if answer.is_empty() {
                    Some(vec![])
                } else {
                    Some(answer.split(',').map(|s| s.trim().to_string()).collect())
                }
            }
        }
    }

    /// Plain text input. Returns None if the user cancelled.
    pub fn text(&self, message: &str) -> Option<String> {
        match &self.0 {
            Inner::Real => inquire::Text::new(message).prompt().ok(),
            Inner::Null(q) => q.lock().unwrap().pop_front(),
        }
    }

    /// Text input with autocomplete suggestions shown as the user types.
    /// Returns None if the user cancelled.
    pub fn text_with_completions(&self, message: &str, completions: &[String]) -> Option<String> {
        match &self.0 {
            Inner::Real => {
                let completer = TagCompleter { options: completions.to_vec() };
                inquire::Text::new(message).with_autocomplete(completer).prompt().ok()
            }
            Inner::Null(q) => q.lock().unwrap().pop_front(),
        }
    }

    /// Decimal number input with optional default. Retries automatically on invalid input.
    /// Returns None if the user cancelled.
    pub fn decimal(&self, message: &str, default: Option<Decimal>) -> Option<Decimal> {
        match &self.0 {
            Inner::Real => match default {
                Some(d) => inquire::CustomType::<Decimal>::new(message)
                    .with_error_message("Please enter a valid decimal number")
                    .with_default(d)
                    .prompt()
                    .ok(),
                None => inquire::CustomType::<Decimal>::new(message)
                    .with_error_message("Please enter a valid decimal number")
                    .prompt()
                    .ok(),
            },
            Inner::Null(q) => {
                let answer = q.lock().unwrap().pop_front()?;
                if answer.trim().is_empty() {
                    default
                } else {
                    answer.trim().parse().ok()
                }
            }
        }
    }

    /// Yes/no confirmation. Returns None if cancelled.
    pub fn confirm(&self, message: &str, default: bool) -> Option<bool> {
        match &self.0 {
            Inner::Real => inquire::Confirm::new(message).with_default(default).prompt().ok(),
            Inner::Null(q) => {
                let answer = q.lock().unwrap().pop_front()?;
                Some(matches!(answer.to_lowercase().as_str(), "y" | "yes" | "true"))
            }
        }
    }
}

#[derive(Clone)]
struct TagCompleter {
    options: Vec<String>,
}

impl inquire::Autocomplete for TagCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let input_lower = input.to_lowercase();
        Ok(self.options.iter().filter(|o| o.to_lowercase().contains(&input_lower)).cloned().collect())
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(highlighted)
    }
}
