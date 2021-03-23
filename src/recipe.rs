use std::collections::HashSet;
use std::iter::FromIterator;

use creature::Creature;

pub mod creature;
pub mod expression;
pub mod statement;
pub mod task;

#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    creatures: HashSet<Creature>,
}

impl Recipe {
    fn new(creatures: HashSet<Creature>) -> Recipe {
        Recipe { creatures }
    }

    pub fn creatures(&self) -> &HashSet<Creature> {
        &self.creatures
    }
}

impl From<Vec<Creature>> for Recipe {
    fn from(creatures: Vec<Creature>) -> Recipe {
        Recipe::new(creatures.into_iter().collect())
    }
}

impl FromIterator<Creature> for Recipe {
    fn from_iter<I: IntoIterator<Item = Creature>>(creatures: I) -> Recipe {
        Recipe::new(creatures.into_iter().collect())
    }
}
