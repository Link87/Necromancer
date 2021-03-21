use crate::task::Task;
use crate::value::Value;

use std::fmt::{Display, Formatter, Result};

use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    kind: EntityKind,
    name: String,
    is_active: bool,
    memory: Value,
    tasks: IndexMap<String, Task>,
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
        tasks: IndexMap<String, Task>,
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

    pub fn tasks(&self) -> &IndexMap<String, Task> {
        &self.tasks
    }
}

impl Display for EntityKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            EntityKind::Zombie => write!(fmt, "Zombie"),
            EntityKind::Ghost => write!(fmt, "Ghost"),
            EntityKind::Vampire => write!(fmt, "Vampire"),
            EntityKind::Demon => write!(fmt, "Demon"),
            EntityKind::Djinn => write!(fmt, "Djinn"),
        }
    }
}
