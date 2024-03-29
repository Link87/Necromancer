use smol_str::SmolStr;

use super::statement::Stmt;

#[derive(Debug, Clone)]
pub struct Task {
    name: SmolStr,
    active: bool,
    stmts: Vec<Stmt>,
}

impl Task {
    pub fn new(name: &str, active: bool, stmts: Vec<Stmt>) -> Task {
        Task {
            name: SmolStr::from(name),
            active,
            stmts,
        }
    }

    pub fn name(&self) -> SmolStr {
        self.name.clone()
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn statements(&self) -> &Vec<Stmt> {
        &self.stmts
    }
}
