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

    /// Ask for text input with a default value shown to the user.
    /// Returns None if the user cancelled.
    pub fn text_with_default(&self, message: &str, default: &str) -> Option<String> {
        match &self.0 {
            Inner::Real => inquire::Text::new(message).with_default(default).prompt().ok(),
            Inner::Null(q) => {
                let answer = q.lock().unwrap().pop_front().unwrap_or_default();
                Some(if answer.is_empty() { default.to_string() } else { answer })
            }
        }
    }

    /// Ask for text input with autocomplete suggestions shown as the user types.
    /// Returns None if the user cancelled or entered an empty string.
    pub fn text_with_completions(&self, message: &str, completions: &[String]) -> Option<String> {
        match &self.0 {
            Inner::Real => {
                let completer = TagCompleter { options: completions.to_vec() };
                inquire::Text::new(message).with_autocomplete(completer).prompt().ok()
            }
            Inner::Null(q) => q.lock().unwrap().pop_front(),
        }
    }

    /// Ask for a yes/no confirmation. Returns None if cancelled.
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
