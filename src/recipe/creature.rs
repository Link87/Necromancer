use std::borrow::Borrow;
use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};

use indexmap::IndexSet;

use super::task::Task;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Creature<'a> {
    name: &'a str,
    species: Species,
    active: bool,
    memory: Value,
    tasks: IndexSet<Task<'a>>,
}

impl<'a> Creature<'a> {
    pub fn summon(
        name: &'a str,
        species: Species,
        active: bool,
        memory: Value,
        tasks: IndexSet<Task<'a>>,
    ) -> Creature<'a> {
        Creature {
            name,
            species,
            active,
            memory,
            tasks,
        }
    }

    pub fn species(&self) -> Species {
        self.species
    }

    pub fn name(&self) -> &'a str {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn moan(&self) -> &Value {
        &self.memory
    }

    pub fn tasks(&self) -> &IndexSet<Task> {
        &self.tasks
    }
}

impl PartialEq<Creature<'_>> for Creature<'_> {
    fn eq(&self, other: &Creature) -> bool {
        self.name == other.name
    }
}

impl Eq for Creature<'_> {}

impl Hash for Creature<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Borrow<str> for Creature<'_> {
    fn borrow(&self) -> &str {
        return self.name.borrow();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Species {
    Zombie,
    Ghost,
    Vampire,
    Demon,
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
