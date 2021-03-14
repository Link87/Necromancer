use crate::task::Task;
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    kind: EntityKind,
    name: String,
    is_active: bool,
    memory: Value,
    tasks: Vec<Task>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntityKind {
    Zombie,
    Ghost,
    Vampire,
    Demon,
    Djinn,
}

impl Entity {
    pub fn summon(
        kind: EntityKind,
        name: String,
        is_active: bool,
        memory: Value,
        tasks: Vec<Task>,
    ) -> Entity {
        Entity {
            kind,
            name,
            is_active,
            memory,
            tasks,
        }
    }

    pub fn kind(&self) -> EntityKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.is_active
    }

    pub fn moan(&self) -> &Value {
        &self.memory
    }

    pub fn tasks(&self) -> &Vec<Task> {
        &self.tasks
    }
}
