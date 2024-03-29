use std::fmt::{Display, Formatter, Result};

use indexmap::IndexMap;
use smol_str::SmolStr;

use super::task::Task;
use crate::value::Value;

pub type TaskList = IndexMap<SmolStr, Task>;

#[derive(Debug, Clone)]
pub struct Entity {
    name: SmolStr,
    species: Species,
    active: bool,
    memory: Value,
    tasks: TaskList,
}

impl Entity {
    pub fn summon(
        name: &str,
        species: Species,
        active: bool,
        memory: Value,
        tasks: TaskList,
    ) -> Entity {
        Entity {
            name: SmolStr::from(name),
            species,
            active,
            memory,
            tasks,
        }
    }

    pub fn species(&self) -> Species {
        self.species
    }

    pub fn name(&self) -> SmolStr {
        self.name.clone()
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn moan(&self) -> &Value {
        &self.memory
    }

    pub fn tasks(&self) -> &TaskList {
        &self.tasks
    }
}

/// The different kinds of species that a [`Creature`] can belong to.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Species {
    /// Zombies process their active tasks in sequence, beginning from the first task defined,
    /// as quickly as they can. They perform each task exactly once.
    Zombie,
    /// Ghosts process their active tasks in sequence, beginning from the first task defined,
    /// but they may wait for an undefined time before beginning and between each task.
    /// They eventually perform each task exactly once.
    Ghost,
    /// Vampires process their active tasks in random order, as quickly as they can.
    /// They perform each task exactly once, and complete one task before beginning the next.
    Vampire,
    /// Demons process their active tasks in random order, as quickly as they can.
    /// They may decide to perform tasks multiple times before becoming inactive, but will perform
    /// each task at least once. They may perform multiple tasks at the same time.
    /// They may also summon additional demons exactly like themselves.
    Demon,
    /// Djinn process their active tasks in random order, as quickly as they can. They may decide
    /// to perform each task multiple times, or not at all, before becoming inactive.
    /// They may perform multiple tasks at the same time.
    Djinn,
}

impl Display for Species {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Species::Zombie => write!(fmt, "Zombie"),
            Species::Ghost => write!(fmt, "Ghost"),
            Species::Vampire => write!(fmt, "Vampire"),
            Species::Demon => write!(fmt, "Demon"),
            Species::Djinn => write!(fmt, "Djinn"),
        }
    }
}
