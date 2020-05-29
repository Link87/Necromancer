#[derive(Debug)]
pub struct Task {
    name: String,
    is_active: bool,
}

impl Task {
    pub fn new(name: String, is_active: bool) -> Task {
        Task { name, is_active }
    }

    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.is_active
    }
}
