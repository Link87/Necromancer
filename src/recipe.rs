//! Recipes are the internal representation of ZOMBIE source code. This module and its submodules contain the data type definitions for recipes.
use std::collections::HashMap;

use creature::Creature;

pub mod creature;
pub mod expression;
pub mod statement;
pub mod task;

/// A recipe used by necromancers in their mysterious rituals.
///
/// Contains a list of creatures to summon.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe<'a> {
    // use hash map to store values on heap.
    creatures: HashMap<&'a str, Creature<'a>>,
}

impl<'a> Recipe<'a> {
    /// Create a new recipe from a set of creatures.
    fn new(creatures: HashMap<&'a str, Creature<'a>>) -> Recipe<'a> {
        Recipe { creatures }
    }

    /// Return the creatures listed in the recipe.
    pub fn creatures(&self) -> &HashMap<&'a str, Creature> {
        &self.creatures
    }
}

impl<'a> From<Vec<Creature<'a>>> for Recipe<'a> {
    fn from(creatures: Vec<Creature<'a>>) -> Recipe<'a> {
        Recipe::new(creatures.into_iter().map(|c| (c.name(), c)).collect())
    }
}
