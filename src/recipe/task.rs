use super::statement::Statement;

use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Task {
    name: String,
    active: bool,
    statements: Vec<Statement>,
}

impl Task {
    pub fn new(name: String, active: bool, statements: Vec<Statement>) -> Task {
        Task {
            name,
            active,
            statements,
        }
    }

    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn statements<'a>(&self) -> &Vec<Statement> {
        &self.statements
    }
}

/// Two tasks are considered equal, if and only if their names are equal.
impl PartialEq<Task> for Task {
    fn eq(&self, other: &Task) -> bool {
        self.name == other.name
    }
}

impl Eq for Task {}

impl Hash for Task {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Borrow<str> for Task {
    fn borrow(&self) -> &str {
        return self.name.borrow();
    }
}
