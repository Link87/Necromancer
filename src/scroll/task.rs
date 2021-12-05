use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

use super::statement::Stmt;

#[derive(Debug, Clone)]
pub struct Task<'a> {
    name: &'a str,
    active: bool,
    stmts: Vec<Stmt<'a>>,
}

impl<'a> Task<'a> {
    pub fn new(name: &'a str, active: bool, stmts: Vec<Stmt<'a>>) -> Task<'a> {
        Task {
            name,
            active,
            stmts,
        }
    }

    pub fn name(&self) -> &'a str {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn statements(&self) -> &Vec<Stmt> {
        &self.stmts
    }
}

/// Two tasks are considered equal, if and only if their names are equal.
impl PartialEq<Task<'_>> for Task<'_> {
    fn eq(&self, other: &Task) -> bool {
        self.name == other.name
    }
}

impl Eq for Task<'_> {}

impl Hash for Task<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<'a> Borrow<str> for Task<'a> {
    fn borrow(&self) -> &'a str {
        return self.name;
    }
}
