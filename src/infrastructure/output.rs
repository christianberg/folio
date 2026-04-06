use std::sync::{Arc, Mutex, Weak};

pub struct Output {
    stdout: Mutex<Option<Weak<Mutex<Vec<String>>>>>,
    stderr: Mutex<Option<Weak<Mutex<Vec<String>>>>>,
    real: bool,
}

pub struct OutputTracker(Arc<Mutex<Vec<String>>>);

impl OutputTracker {
    pub fn all(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

impl Output {
    pub fn create() -> Self {
        Self {
            stdout: Mutex::new(None),
            stderr: Mutex::new(None),
            real: true,
        }
    }

    pub fn create_null() -> Self {
        Self {
            stdout: Mutex::new(None),
            stderr: Mutex::new(None),
            real: false,
        }
    }

    pub fn println(&self, msg: &str) {
        if let Some(arc) = self.stdout.lock().unwrap().as_ref().and_then(Weak::upgrade) {
            arc.lock().unwrap().push(msg.to_string());
        }
        if self.real {
            println!("{msg}");
        }
    }

    pub fn eprintln(&self, msg: &str) {
        if let Some(arc) = self.stderr.lock().unwrap().as_ref().and_then(Weak::upgrade) {
            arc.lock().unwrap().push(msg.to_string());
        }
        if self.real {
            eprintln!("{msg}");
        }
    }

    pub fn track_stdout(&self) -> OutputTracker {
        let arc = Arc::new(Mutex::new(Vec::new()));
        *self.stdout.lock().unwrap() = Some(Arc::downgrade(&arc));
        OutputTracker(arc)
    }

    pub fn track_stderr(&self) -> OutputTracker {
        let arc = Arc::new(Mutex::new(Vec::new()));
        *self.stderr.lock().unwrap() = Some(Arc::downgrade(&arc));
        OutputTracker(arc)
    }
}
