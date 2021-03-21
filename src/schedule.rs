use std::sync::Arc;

use dashmap::DashMap;
use futures::future;
use futures::stream::{FuturesUnordered, StreamExt};
use lazy_static::lazy_static;
use log::{debug, error};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::seq::index;
use rand::{Rng, SeedableRng};
use std::time::Duration;
use tokio::io::{self, AsyncWriteExt};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;
use tokio::time;

use crate::entity::{Entity, EntityKind};
use crate::parse::SyntaxTree;
use crate::statement::{Statement, StatementCmd};
use crate::value::Value;

lazy_static! {
    static ref GHOST_RNG_DISTRIBUTION: Uniform<u64> = Uniform::from(500..=10000);
    static ref DEMON_RESAMPLE_COUNT_RNG_DISTRIBUTION: Uniform<u64> = Uniform::from(0..=5);
}

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

        // TODO watchdog

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
            EntityKind::Ghost => {
                let mut rng = SmallRng::from_entropy();
                for (task_name, _) in entity.tasks() {
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&awakened).execute_task(String::from(task_name)))
                            .await
                    {
                        error!("{}", e);
                    }
                    time::sleep(Duration::from_millis(
                        GHOST_RNG_DISTRIBUTION.sample(&mut rng),
                    ))
                    .await;
                }
            }
            EntityKind::Vampire => {
                let mut rng = SmallRng::from_entropy();
                let sample = index::sample(&mut rng, entity.tasks().len(), entity.tasks().len());
                for index in sample {
                    let (task_name, _) = entity.tasks().get_index(index).unwrap();
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&awakened).execute_task(String::from(task_name)))
                            .await
                    {
                        error!("{}", e);
                    }
                }
            }
            EntityKind::Demon => {
                let mut rng = SmallRng::from_entropy();
                let mut sample =
                    index::sample(&mut rng, entity.tasks().len(), entity.tasks().len()).into_vec();
                for _ in 0..=DEMON_RESAMPLE_COUNT_RNG_DISTRIBUTION.sample(&mut rng) {
                    let resample_size = rng.gen_range(0..=entity.tasks().len() / 3);
                    sample.extend(index::sample(&mut rng, entity.tasks().len(), resample_size));
                }

                debug!("Demon task order {:?}", &sample);
                while !sample.is_empty() {
                    if rng.gen_ratio(33, 100 * sample.len() as u32) {
                        awakened
                            .sender
                            .send(Message::Invoke(String::from(&awakened.name)))
                            .expect("Message receiver dropped before task could finish!");
                        debug!("Spawning helper demon!");
                    }
                    let mut tasks = Vec::new();
                    for _ in 1..=rng.gen_range(1..=(f32::ceil(sample.len() as f32 / 5.0) as i64)) {
                        let selected = sample.pop().unwrap();
                        let (task_name, _) = entity.tasks().get_index(selected).unwrap();
                        tasks.push(tokio::spawn(
                            Arc::clone(&awakened).execute_task(String::from(task_name)),
                        ));
                    }
                    for e in future::join_all(tasks)
                        .await
                        .into_iter()
                        .filter_map(|t| t.err())
                    {
                        error!("{}", e);
                    }
                }
            }
            EntityKind::Djinn => {
                let mut rng = SmallRng::from_entropy();
                let sample_size = rng.gen_range(1..=10 * entity.tasks().len());
                let distribution: Uniform<usize> = Uniform::from(0..entity.tasks().len());
                let mut sample: Vec<usize> =
                    distribution.sample_iter(&mut rng).take(sample_size).collect();

                debug!("Djinn task order {:?}", &sample);
                while !sample.is_empty() {
                    let mut tasks = Vec::new();
                    for _ in 1..=rng.gen_range(1..=(f32::ceil(sample.len() as f32 / 5.0) as i64)) {
                        let selected = sample.pop().unwrap();
                        let (task_name, _) = entity.tasks().get_index(selected).unwrap();
                        tasks.push(tokio::spawn(
                            Arc::clone(&awakened).execute_task(String::from(task_name)),
                        ));
                    }
                    for e in future::join_all(tasks)
                        .await
                        .into_iter()
                        .filter_map(|t| t.err())
                    {
                        error!("{}", e);
                    }
                }
            }
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
            debug!("{} invoking a new copy of itself", entity_name);
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
            debug!("{} remembering {} (self)", entity_name, value);
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
