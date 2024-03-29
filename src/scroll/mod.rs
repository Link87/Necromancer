//! Scrolls are the internal representation of ZOMBIE source code. This module and its submodules contain the data type definitions for recipes.
use std::collections::HashMap;

use entity::Entity;
use smol_str::SmolStr;

pub mod entity;
pub mod expression;
pub mod statement;
pub mod task;

pub type EntityList = HashMap<SmolStr, Entity>;

/// A mysterious scroll with instructions for necromancers and their summoning rituals.
///
/// Contains a list of creatures to summon.
#[derive(Debug, Clone)]
pub struct Scroll {
    // use hash map to store values on heap.
    entities: EntityList,
}

impl Scroll {
    /// Create a new recipe from a set of creatures.
    fn new(creatures: EntityList) -> Scroll {
        Scroll { entities: creatures }
    }

    /// Return the creatures listed in the recipe.
    pub fn creatures(&self) -> &EntityList {
        &self.entities
    }
}

impl From<Vec<Entity>> for Scroll {
    fn from(creatures: Vec<Entity>) -> Scroll {
        Scroll::new(creatures.into_iter().map(|c| (c.name(), c)).collect())
    }
}
