//! Scrolls are the internal representation of ZOMBIE source code. This module and its submodules contain the data type definitions for recipes.
use std::collections::HashMap;

use creature::Creature;

pub mod creature;
pub mod expression;
pub mod statement;
pub mod task;

/// A mysterious scroll with instructions for necromancers and their summoning rituals.
///
/// Contains a list of creatures to summon.
#[derive(Debug, Clone, PartialEq)]
pub struct Scroll<'a> {
    // use hash map to store values on heap.
    creatures: HashMap<&'a str, Creature<'a>>,
}

impl<'a> Scroll<'a> {
    /// Create a new recipe from a set of creatures.
    fn new(creatures: HashMap<&'a str, Creature<'a>>) -> Scroll<'a> {
        Scroll { creatures }
    }

    /// Return the creatures listed in the recipe.
    pub fn creatures(&self) -> &HashMap<&'a str, Creature> {
        &self.creatures
    }
}

impl<'a> From<Vec<Creature<'a>>> for Scroll<'a> {
    fn from(creatures: Vec<Creature<'a>>) -> Scroll<'a> {
        Scroll::new(creatures.into_iter().map(|c| (c.name(), c)).collect())
    }
}
