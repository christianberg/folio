use std::collections::HashMap;

pub struct Filesystem(Inner);

enum Inner {
    Real,
    Null(HashMap<String, String>),
}

impl Filesystem {
    pub fn create() -> Self {
        Self(Inner::Real)
    }

    pub fn create_null(files: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        Self(Inner::Null(
            files.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
        ))
    }

    pub fn read_to_string(&self, path: &str) -> Result<String, std::io::Error> {
        match &self.0 {
            Inner::Real => std::fs::read_to_string(path),
            Inner::Null(files) => files.get(path).cloned().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("file not found: {path}"),
                )
            }),
        }
    }
}
