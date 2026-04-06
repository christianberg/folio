use std::sync::{Arc, Mutex};

pub struct Output {
    stdout: Arc<Mutex<Vec<String>>>,
    stderr: Arc<Mutex<Vec<String>>>,
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
            stdout: Arc::new(Mutex::new(Vec::new())),
            stderr: Arc::new(Mutex::new(Vec::new())),
            real: true,
        }
    }

    pub fn create_null() -> Self {
        Self {
            stdout: Arc::new(Mutex::new(Vec::new())),
            stderr: Arc::new(Mutex::new(Vec::new())),
            real: false,
        }
    }

    pub fn println(&self, msg: &str) {
        self.stdout.lock().unwrap().push(msg.to_string());
        if self.real {
            println!("{msg}");
        }
    }

    pub fn eprintln(&self, msg: &str) {
        self.stderr.lock().unwrap().push(msg.to_string());
        if self.real {
            eprintln!("{msg}");
        }
    }

    pub fn track_stdout(&self) -> OutputTracker {
        OutputTracker(Arc::clone(&self.stdout))
    }

    pub fn track_stderr(&self) -> OutputTracker {
        OutputTracker(Arc::clone(&self.stderr))
    }
}
