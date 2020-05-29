use crate::task::Task;

#[derive(Debug)]
pub struct Entity {
    kind: EntityKind,
    name: String,
    is_active: bool,
    memory: Memory,
    tasks: Vec<Task>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Memory {
    Number(i64),
    None,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
        memory: Memory,
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

    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.is_active
    }

    pub fn remember(&mut self, value: Memory) {
        self.memory = value;
    }

    pub fn moan(&self) -> Memory {
        self.memory
    }

    pub fn tasks<'a>(&'a self) -> &'a Vec<Task> {
        &self.tasks
    }
}
