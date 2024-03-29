use dashmap::DashMap;
use smol_str::SmolStr;
use tokio::sync::Notify;

use crate::scroll::entity::Entity;
use crate::value::Value;

#[derive(Debug)]
pub struct State {
    knowledge: DashMap<SmolStr, SpiritState>,
    notifier: Notify,
}

impl State {
    fn new() -> State {
        State {
            knowledge: DashMap::new(),
            notifier: Notify::new(),
        }
    }

    pub fn knowledge(&self) -> &DashMap<SmolStr, SpiritState> {
        &self.knowledge
    }

    pub fn notifier(&self) -> &Notify {
        &self.notifier
    }
}

impl<'a, I: Iterator<Item = &'a Entity>> From<I> for State {
    fn from(creatures: I) -> Self {
        let state = State::new();
        for creature in creatures {
            state
                .knowledge
                .insert(creature.name(), SpiritState::from(creature));
        }
        state
    }
}

/// Holds owned data of an entity.
///
/// Is a reduced version of a [`Creature`] that allows mutability,
/// for keeping track of entity data while executing code.
///
/// Given a [`Creature`], an instance of [``] can be
/// created using [`EntityData::from`].
#[derive(Clone, Debug, Default)]
pub struct SpiritState {
    memory: Value,
    active: bool,
}

impl SpiritState {
    fn new(memory: Value, active: bool) -> SpiritState {
        SpiritState { memory, active }
    }

    pub fn memory(&self) -> &Value {
        &self.memory
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn memory_mut(&mut self) -> &mut Value {
        &mut self.memory
    }

    pub fn active_mut(&mut self) -> &mut bool {
        &mut self.active
    }
}

impl From<&Entity> for SpiritState {
    fn from(creature: &Entity) -> SpiritState {
        SpiritState::new(Value::from(creature.moan()), creature.active())
    }
}
