use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

pub struct Filesystem {
    inner: Inner,
    append_tracker: Mutex<Option<Weak<Mutex<Vec<(String, String)>>>>>,
}

enum Inner {
    Real,
    Null(Mutex<HashMap<String, String>>),
}

impl Filesystem {
    pub fn create() -> Self {
        Self { inner: Inner::Real, append_tracker: Mutex::new(None) }
    }

    pub fn create_null(files: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        Self {
            inner: Inner::Null(Mutex::new(
                files.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            )),
            append_tracker: Mutex::new(None),
        }
    }

    pub fn read_to_string(&self, path: &str) -> Result<String, std::io::Error> {
        match &self.inner {
            Inner::Real => std::fs::read_to_string(path),
            Inner::Null(files) => files.lock().unwrap().get(path).cloned().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("file not found: {path}"),
                )
            }),
        }
    }

    pub fn append_str(&self, path: &str, content: &str) -> Result<(), std::io::Error> {
        if let Some(arc) = self.append_tracker.lock().unwrap().as_ref().and_then(Weak::upgrade) {
            arc.lock().unwrap().push((path.to_string(), content.to_string()));
        }
        match &self.inner {
            Inner::Real => {
                use std::io::Write;
                let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
                f.write_all(content.as_bytes())
            }
            Inner::Null(files) => {
                files.lock().unwrap().entry(path.to_string()).or_default().push_str(content);
                Ok(())
            }
        }
    }

    pub fn track_appends(&self) -> AppendTracker {
        let arc = Arc::new(Mutex::new(Vec::new()));
        *self.append_tracker.lock().unwrap() = Some(Arc::downgrade(&arc));
        AppendTracker(arc)
    }
}

pub struct AppendTracker(Arc<Mutex<Vec<(String, String)>>>);

impl AppendTracker {
    pub fn all(&self) -> Vec<(String, String)> {
        self.0.lock().unwrap().clone()
    }
}
