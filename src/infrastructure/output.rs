use std::sync::{Arc, Mutex};

pub struct Output(Inner);

enum Inner {
    Real,
    Null {
        stdout: Arc<Mutex<Vec<String>>>,
        stderr: Arc<Mutex<Vec<String>>>,
    },
}

pub struct OutputTracker(Arc<Mutex<Vec<String>>>);

impl OutputTracker {
    pub fn all(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

impl Output {
    pub fn create() -> Self {
        Self(Inner::Real)
    }

    pub fn create_null() -> Self {
        Self(Inner::Null {
            stdout: Arc::new(Mutex::new(Vec::new())),
            stderr: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn println(&self, msg: &str) {
        match &self.0 {
            Inner::Real => println!("{msg}"),
            Inner::Null { stdout, .. } => stdout.lock().unwrap().push(msg.to_string()),
        }
    }

    pub fn eprintln(&self, msg: &str) {
        match &self.0 {
            Inner::Real => eprintln!("{msg}"),
            Inner::Null { stderr, .. } => stderr.lock().unwrap().push(msg.to_string()),
        }
    }

    pub fn track_stdout(&self) -> OutputTracker {
        match &self.0 {
            Inner::Null { stdout, .. } => OutputTracker(Arc::clone(stdout)),
            Inner::Real => panic!("track_stdout called on real Output"),
        }
    }

    pub fn track_stderr(&self) -> OutputTracker {
        match &self.0 {
            Inner::Null { stderr, .. } => OutputTracker(Arc::clone(stderr)),
            Inner::Real => panic!("track_stderr called on real Output"),
        }
    }
}
