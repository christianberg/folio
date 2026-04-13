use chrono::NaiveDate;

pub struct Clock(Inner);

enum Inner {
    Real,
    Null(NaiveDate),
}

impl Clock {
    pub fn create() -> Self {
        Self(Inner::Real)
    }

    pub fn create_null(date: NaiveDate) -> Self {
        Self(Inner::Null(date))
    }

    pub fn today(&self) -> NaiveDate {
        match &self.0 {
            Inner::Real => chrono::Local::now().date_naive(),
            Inner::Null(d) => *d,
        }
    }
}
