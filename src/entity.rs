use crate::task::Task;
use crate::value::Value;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    kind: EntityKind,
    name: String,
    is_active: bool,
    memory: Value,
    tasks: HashMap<String, Task>,
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
        tasks: HashMap<String, Task>,
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

    pub fn tasks(&self) -> &HashMap<String, Task> {
        &self.tasks
    }
}
