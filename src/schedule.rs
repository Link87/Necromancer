use std::io::{self, Write};
use std::sync::Arc;

use dashmap::DashMap;
use log::debug;

use crate::entity::Entity;
use crate::parse::SyntaxTree;
use crate::statement::{Statement, StatementCmd};
use crate::value::Value;

pub struct Scheduler {
    syntax_tree: Arc<SyntaxTree>,
}

impl Scheduler {
    pub fn new(syntax_tree: SyntaxTree) -> Scheduler {
        Scheduler {
            syntax_tree: Arc::new(syntax_tree),
        }
    }

    #[tokio::main(flavor = "multi_thread")]
    pub async fn schedule(self) {
        let entities = self.syntax_tree.entities();

        let env = Arc::new(SharedEnv::new());
        for (name, entity) in entities {
            env.entities()
                .insert(String::from(name), EntityData::from(entity));
        }
        debug!("{:?}", env);

        for (name, _) in entities {
            let awakened = AwakenedEntity::new(String::from(name), self.syntax_tree.clone(), env.clone());
            tokio::spawn(awakened.execute());
        }
    }
}

struct AwakenedEntity{
    name: String,
    syntax_tree: Arc<SyntaxTree>,
    env: Arc<SharedEnv>,
}

impl AwakenedEntity {
    fn new(name: String, syntax_tree: Arc<SyntaxTree>, env: Arc<SharedEnv>) -> AwakenedEntity {
        AwakenedEntity { name, syntax_tree, env }
    }

    async fn execute(self) {
        let awakened = Arc::new(self);
        for (task_name, _) in awakened.syntax_tree.entities().get(&awakened.name).unwrap().tasks() {
            tokio::spawn(awakened.clone().execute_task(String::from(task_name)));
        }
    }

    async fn execute_task(self: Arc<AwakenedEntity>, task_name: String) {
        for statement in self.syntax_tree.entities().get(&self.name).unwrap().tasks().get(&task_name).unwrap().statements() {
           execute_statement(&self.env, &self.name, statement);
        }
    }
}

#[allow(unused_must_use)]
fn execute_statement(env: &Arc<SharedEnv>, entity_name: &str, statement: &Statement) {
    match statement.cmd() {
        StatementCmd::SayNamed(_, arg) | StatementCmd::Say(arg) => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle.write_all((format!("{}\n", arg)).as_bytes());
        }
        StatementCmd::RememberNamed(name, value) => {
            env.entities().alter(name, |_, mut data| {
                *data.value_mut() = Value::from(value);
                data
            });
        }
        StatementCmd::Remember(value) => {
            env.entities().alter(entity_name, |_, mut data| {
                *data.value_mut() = Value::from(value);
                data
            });
        }
    }
}

#[derive(Debug, Clone, Default)]
struct SharedEnv {
    entities: DashMap<String, EntityData>,
}

impl SharedEnv {
    fn new() -> SharedEnv {
        SharedEnv {
            entities: DashMap::new(),
        }
    }

    fn entities(&self) -> &DashMap<String, EntityData> {
        &self.entities
    }
}

/// Holds owned data of an entity.
///
/// Is a reduced version of `crate::entity::Entity` that allows mutability,
/// for keeping track of entity data while executing code.
///
/// Given an `crate::entity::Entity`, an instance of `EntityData` can be
/// created using `EntityData::from`.
#[derive(Clone, Debug, Default)]
struct EntityData {
    value: Value,
    active: bool,
}

impl EntityData {
    fn new(value: Value, active: bool) -> EntityData {
        EntityData { value, active }
    }

    fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    fn _active_mut(&mut self) -> &mut bool {
        &mut self.active
    }
}

impl From<&Entity> for EntityData {
    fn from(entity: &Entity) -> EntityData {
        EntityData::new(Value::from(entity.moan()), entity.active())
    }
}
