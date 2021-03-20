use std::ops::Not;
use std::sync::Arc;

use dashmap::DashMap;
use futures::stream::{FuturesUnordered, StreamExt};
use log::{debug, error};
use tokio::io::{self, AsyncWriteExt};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;

use crate::entity::{Entity, EntityKind};
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
        
        let mut futures = Vec::with_capacity(entities.len());
        let (tx, mut rx) = mpsc::unbounded_channel();
                
        for (name, _) in entities {
            let awakened = AwakenedEntity::new(
                String::from(name),
                Arc::clone(&self.syntax_tree),
                Arc::clone(&env),
                UnboundedSender::clone(&tx),
            );
            futures.push(tokio::spawn(awakened.execute()));
        }
        
        let tasks = Arc::new(Mutex::new(
            futures
            .into_iter()
            .collect::<FuturesUnordered<JoinHandle<()>>>(),
        ));
        let tasks_push = Arc::clone(&tasks);
        
        // wait for messages to arrive
        // runs indefinetly as it holds both sender and receiver refs
        let message_handler = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    Message::Invoke(name) => {
                        // spawn new entity and add to awaited futures
                        let awakened = AwakenedEntity::new(
                            String::from(name),
                            Arc::clone(&self.syntax_tree),
                            Arc::clone(&env),
                            UnboundedSender::clone(&tx),
                        );
                        tasks_push
                            .lock()
                            .await
                            .push(tokio::spawn(awakened.execute()));
                    }
                }
            }
        });

        // iterate until a None appears, all tasks are finished then
        while let Some(_) = tasks.lock().await.next().await {}

        // Messages are no longer needed.
        // Necessary since message does not exit on its own.
        message_handler.abort();
    }
}

struct AwakenedEntity {
    name: String,
    syntax_tree: Arc<SyntaxTree>,
    env: Arc<SharedEnv>,
    sender: UnboundedSender<Message>,
}

impl AwakenedEntity {
    fn new(
        name: String,
        syntax_tree: Arc<SyntaxTree>,
        env: Arc<SharedEnv>,
        sender: UnboundedSender<Message>,
    ) -> AwakenedEntity {
        AwakenedEntity {
            name,
            syntax_tree,
            env,
            sender,
        }
    }

    async fn execute(self) {
        let awakened = Arc::new(self);
        let entity = awakened.syntax_tree.entities().get(&awakened.name).unwrap();
        match entity.kind() {
            EntityKind::Zombie => {
                for (task_name, _) in entity.tasks() {
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&awakened).execute_task(String::from(task_name)))
                            .await
                    {
                        error!("{}", e);
                    }
                }
            }
            _ => unimplemented!(),
        }
    }

    async fn execute_task(self: Arc<AwakenedEntity>, task_name: String) {
        for statement in self
            .syntax_tree
            .entities()
            .get(&self.name)
            .unwrap()
            .tasks()
            .get(&task_name)
            .unwrap()
            .statements()
        {
            loop {
                if self.env.entities().get(&self.name).unwrap().active() {
                    break;
                } else {
                    self.env.notifier().notified().await;
                }
            }
            execute_statement(
                &self.env,
                &self.name,
                &self.syntax_tree,
                &self.sender,
                statement,
            )
            .await;
            tokio::task::yield_now().await;
        }
    }
}

async fn execute_statement(
    env: &Arc<SharedEnv>,
    entity_name: &str,
    syntax_tree: &Arc<SyntaxTree>,
    sender: &UnboundedSender<Message>,
    statement: &Statement,
) {
    match statement.cmd() {
        StatementCmd::Animate => {
            let entity_kind = syntax_tree.entities().get(entity_name).unwrap().kind();
            debug!(
                "{} (Kind {}) tries to animate itself (wtf?)",
                entity_name, entity_kind,
            );
            if matches!(entity_kind, EntityKind::Zombie) {
                set_active(env, entity_name, true);
            }
        }
        StatementCmd::AnimateNamed(other_name) => {
            let entity_kind = syntax_tree.entities().get(other_name).unwrap().kind();
            debug!(
                "{} tries to animate {} (Kind {})",
                entity_name, other_name, entity_kind
            );
            if matches!(entity_kind, EntityKind::Zombie) {
                set_active(env, other_name, true);
            }
        }
        StatementCmd::Banish => {
            debug!("{} banishing itself", entity_name);
            set_active(env, entity_name, false);
        }
        StatementCmd::BanishNamed(other_name) => {
            debug!("{} banishing {}", entity_name, other_name);
            set_active(env, other_name, false);
        }
        StatementCmd::Disturb => {
            let entity_kind = syntax_tree.entities().get(entity_name).unwrap().kind();
            debug!(
                "{} (Kind {}) tries to disturb itself (wtf?)",
                entity_name, entity_kind,
            );
            if matches!(entity_kind, EntityKind::Ghost) {
                set_active(env, entity_name, true);
            }
        }
        StatementCmd::DisturbNamed(other_name) => {
            let entity_kind = syntax_tree.entities().get(other_name).unwrap().kind();
            debug!(
                "{} tries to disturb {} (Kind {})",
                entity_name, other_name, entity_kind,
            );
            if matches!(entity_kind, EntityKind::Ghost) {
                set_active(env, other_name, true);
            }
        }
        StatementCmd::Forget => {
            debug!("{} forgets its value", entity_name);
            set_value(env, entity_name, &Value::default())
        }
        StatementCmd::ForgetNamed(other_name) => {
            debug!("{} makes {} forget its value", entity_name, other_name);
            set_value(env, other_name, &Value::default())
        }
        StatementCmd::Invoke => {
            debug!("{} invoking a new copy of itself\n", entity_name);
            sender
                .send(Message::Invoke(String::from(entity_name)))
                .expect("Message receiver dropped before task could finish!");
        }
        StatementCmd::InvokeNamed(other_name) => {
            debug!("{} invoking a new copy of {}", entity_name, other_name);
            sender
                .send(Message::Invoke(String::from(other_name)))
                .expect("Message receiver dropped before task could finish!");
        }
        StatementCmd::Remember(value) => {
            debug!("{} remembering {} (self)\n", entity_name, value);
            set_value(env, entity_name, value)
        }
        StatementCmd::RememberNamed(other_name, value) => {
            debug!("{} remembering {}", other_name, value);
            set_value(env, other_name, value)
        }
        StatementCmd::SayNamed(_, arg) | StatementCmd::Say(arg) => {
            let mut stdout = io::stdout();
            stdout
                .write_all((format!("{}\n", arg)).as_bytes())
                .await
                .expect("Could not output text!");
        }
    }
}

fn set_active(env: &Arc<SharedEnv>, entity_name: &str, active: bool) {
    env.entities().alter(entity_name, |_, mut data| {
        *data.active_mut() = active;
        data
    });
    if active {
        env.notifier().notify_waiters();
    }
}

fn set_value(env: &Arc<SharedEnv>, entity_name: &str, value: &Value) {
    env.entities().alter(entity_name, |_, mut data| {
        *data.value_mut() = Value::from(value);
        data
    });
}

#[derive(Debug, Clone)]
enum Message {
    Invoke(String),
}

#[derive(Debug, Default)]
struct SharedEnv {
    entities: DashMap<String, EntityData>,
    notifier: Notify,
}

impl SharedEnv {
    fn new() -> SharedEnv {
        SharedEnv {
            entities: DashMap::new(),
            notifier: Notify::new(),
        }
    }

    fn entities(&self) -> &DashMap<String, EntityData> {
        &self.entities
    }

    fn notifier(&self) -> &Notify {
        &self.notifier
    }
}

/// Holds owned data of an entity.
///
/// Is a reduced version of [`Entity`] that allows mutability,
/// for keeping track of entity data while executing code.
///
/// Given an [`Entity`], an instance of [`EntityData`] can be
/// created using [`EntityData::from`].
#[derive(Clone, Debug, Default)]
struct EntityData {
    value: Value,
    active: bool,
}

impl EntityData {
    fn new(value: Value, active: bool) -> EntityData {
        EntityData { value, active }
    }

    fn _value(&self) -> &Value {
        &self.value
    }

    fn active(&self) -> bool {
        self.active
    }

    fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    fn active_mut(&mut self) -> &mut bool {
        &mut self.active
    }
}

impl From<&Entity> for EntityData {
    fn from(entity: &Entity) -> EntityData {
        EntityData::new(Value::from(entity.moan()), entity.active())
    }
}
