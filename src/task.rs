use crate::statement::Statement;

#[derive(Debug)]
pub struct Task {
    name: String,
    is_active: bool,
    statements: Vec<Statement>,
}

impl Task {
    pub fn new(name: String, is_active: bool, statements: Vec<Statement>) -> Task {
        Task {
            name,
            is_active,
            statements,
        }
    }

    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.is_active
    }

    pub fn statements<'a>(&'a mut self) -> &'a mut Vec<Statement> {
        &mut self.statements
    }
}
