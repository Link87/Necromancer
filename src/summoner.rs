use std::sync::Arc;
use std::time::Duration;

use dashmap::{DashMap, DashSet};
use futures::future::{self, AbortHandle, Abortable};
use futures::stream::{FuturesUnordered, StreamExt};
use lazy_static::lazy_static;
use log::{debug, error, warn};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::seq::index;
use rand::{Rng, SeedableRng};
use tokio::io::{self, AsyncWriteExt};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::task::JoinHandle;
use tokio::time;

use crate::recipe::creature::{Creature, Species};
use crate::recipe::expression::Expression;
use crate::recipe::statement::Statement;
use crate::recipe::Recipe;
use crate::value::Value;

lazy_static! {
    static ref GHOST_RNG_DISTRIBUTION: Uniform<u64> = Uniform::from(500..=10000);
    static ref DEMON_RESAMPLE_COUNT_RNG_DISTRIBUTION: Uniform<u64> = Uniform::from(0..=5);
}

pub struct Scheduler {
    recipe: Arc<Recipe>,
}

impl Scheduler {
    pub fn new(recipe: Recipe) -> Scheduler {
        Scheduler {
            recipe: Arc::new(recipe),
        }
    }

    #[tokio::main(flavor = "multi_thread")]
    pub async fn schedule(self) {
        let creatures = self.recipe.creatures();

        let env = Arc::new(SharedEnv::new());
        for creature in creatures {
            env.creatures()
                .insert(String::from(creature.name()), EntityData::from(creature));
        }
        debug!("{:?}", env);

        let mut futures = Vec::with_capacity(creatures.len());
        let (tx, mut rx) = mpsc::unbounded_channel();

        // for aborting
        let abort_handles = Arc::new(RwLock::new(Vec::with_capacity(creatures.len())));
        // count how many copies of an entity are alive
        let candles: Arc<DashSet<Arc<String>>> = Arc::new(DashSet::new());

        for creature in creatures {
            let awakened = AwakenedEntity::new(
                String::from(creature.name()),
                Arc::clone(&self.recipe),
                Arc::clone(&env),
                UnboundedSender::clone(&tx),
            );
            let candle = Arc::new(String::from(creature.name()));
            candles.insert(Arc::clone(&candle));

            let (handle, registration) = AbortHandle::new_pair();
            abort_handles.write().await.push(handle);

            let future = Abortable::new(tokio::spawn(awakened.execute(candle)), registration);
            futures.push(future);
        }

        let tasks = Arc::new(Mutex::new(
            futures
                .into_iter()
                .collect::<FuturesUnordered<Abortable<JoinHandle<()>>>>(),
        ));

        // kill program if every entity is inactive
        let env_watchdog = Arc::clone(&env);
        let abort_handles_watchdog = Arc::clone(&abort_handles);
        let candles_watchdog = Arc::clone(&candles);
        let watchdog = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                debug!("Watchdog tick.");
                if env_watchdog.creatures().iter().all(|c| {
                    !c.value().active()
                        || Arc::strong_count(&candles_watchdog.get(c.key()).unwrap()) <= 1
                }) {
                    warn!("Watchdog triggered! Aborting: only inactive tasks left.");
                    for handle in abort_handles_watchdog.read().await.iter() {
                        handle.abort()
                    }
                }
            }
        });

        // wait for messages to arrive
        // runs indefinetly as it holds both sender and receiver refs
        let tasks_message_handler = Arc::clone(&tasks);
        let abort_handles_message_handler = Arc::clone(&abort_handles);
        let candles_message_handler = Arc::clone(&candles);
        let message_handler = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    Message::Invoke(ref name) => {
                        // spawn new entity and add to awaited futures
                        let awakened = AwakenedEntity::new(
                            String::from(name),
                            Arc::clone(&self.recipe),
                            Arc::clone(&env),
                            UnboundedSender::clone(&tx),
                        );
                        let candle: Arc<String> =
                            Arc::clone(&candles_message_handler.get(name).unwrap());

                        let (handle, registration) = AbortHandle::new_pair();
                        abort_handles_message_handler.write().await.push(handle);

                        tasks_message_handler.lock().await.push(Abortable::new(
                            tokio::spawn(awakened.execute(candle)),
                            registration,
                        ));
                    }
                }
            }
        });

        // iterate until a None appears, all tasks are finished then
        while let Some(_) = tasks.lock().await.next().await {}

        // watchdog useless now
        watchdog.abort();

        // Messages are no longer needed.
        // Necessary since message does not exit on its own.
        message_handler.abort();
    }
}

struct AwakenedEntity {
    name: String,
    recipe: Arc<Recipe>,
    env: Arc<SharedEnv>,
    sender: UnboundedSender<Message>,
}

impl AwakenedEntity {
    fn new(
        name: String,
        recipe: Arc<Recipe>,
        env: Arc<SharedEnv>,
        sender: UnboundedSender<Message>,
    ) -> AwakenedEntity {
        AwakenedEntity {
            name,
            recipe,
            env,
            sender,
        }
    }

    async fn execute(self, _candle: Arc<String>) {
        let awakened = Arc::new(self);
        let creature = awakened
            .recipe
            .creatures()
            .get(awakened.name.as_str())
            .unwrap();
        match creature.species() {
            Species::Zombie => {
                for task in creature.tasks() {
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&awakened).execute_task(String::from(task.name())))
                            .await
                    {
                        error!("{}", e);
                    }
                }
            }
            Species::Ghost => {
                let mut rng = SmallRng::from_entropy();
                for task in creature.tasks() {
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&awakened).execute_task(String::from(task.name())))
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
            Species::Vampire => {
                let mut rng = SmallRng::from_entropy();
                let sample =
                    index::sample(&mut rng, creature.tasks().len(), creature.tasks().len());
                for index in sample {
                    let task = creature.tasks().get_index(index).unwrap();
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&awakened).execute_task(String::from(task.name())))
                            .await
                    {
                        error!("{}", e);
                    }
                }
            }
            Species::Demon => {
                let mut rng = SmallRng::from_entropy();
                let mut sample =
                    index::sample(&mut rng, creature.tasks().len(), creature.tasks().len())
                        .into_vec();
                for _ in 0..=DEMON_RESAMPLE_COUNT_RNG_DISTRIBUTION.sample(&mut rng) {
                    let resample_size = rng.gen_range(0..=creature.tasks().len() / 3);
                    sample.extend(index::sample(
                        &mut rng,
                        creature.tasks().len(),
                        resample_size,
                    ));
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
                        let task = creature.tasks().get_index(selected).unwrap();
                        tasks.push(tokio::spawn(
                            Arc::clone(&awakened).execute_task(String::from(task.name())),
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
            Species::Djinn => {
                let mut rng = SmallRng::from_entropy();
                let sample_size = rng.gen_range(1..=10 * creature.tasks().len());
                let distribution: Uniform<usize> = Uniform::from(0..creature.tasks().len());
                let mut sample: Vec<usize> = distribution
                    .sample_iter(&mut rng)
                    .take(sample_size)
                    .collect();

                debug!("Djinn task order {:?}", &sample);
                while !sample.is_empty() {
                    let mut tasks = Vec::new();
                    for _ in 1..=rng.gen_range(1..=(f32::ceil(sample.len() as f32 / 5.0) as i64)) {
                        let selected = sample.pop().unwrap();
                        let task = creature.tasks().get_index(selected).unwrap();
                        tasks.push(tokio::spawn(
                            Arc::clone(&awakened).execute_task(String::from(task.name())),
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
            .recipe
            .creatures()
            .get(self.name.as_str())
            .unwrap()
            .tasks()
            .get(task_name.as_str())
            .unwrap()
            .statements()
        {
            loop {
                if self.env.creatures().get(&self.name).unwrap().active() {
                    break;
                } else {
                    self.env.notifier().notified().await;
                }
            }
            execute_statement(&self.env, &self.name, &self.recipe, &self.sender, statement).await;
            tokio::task::yield_now().await;
        }
    }
}

async fn execute_statement(
    env: &Arc<SharedEnv>,
    entity_name: &str,
    recipe: &Arc<Recipe>,
    sender: &UnboundedSender<Message>,
    statement: &Statement,
) {
    match statement {
        Statement::Animate => {
            let species = recipe.creatures().get(entity_name).unwrap().species();
            debug!(
                "{} (Species {}) tries to animate itself (wtf?)",
                entity_name, species,
            );
            if matches!(species, Species::Zombie) {
                set_active(env, entity_name, true);
            }
        }
        Statement::AnimateNamed(other_name) => {
            let species = recipe
                .creatures()
                .get(other_name.as_str())
                .unwrap()
                .species();
            debug!(
                "{} tries to animate {} (Species {})",
                entity_name, other_name, species
            );
            if matches!(species, Species::Zombie) {
                set_active(env, other_name, true);
            }
        }
        Statement::Banish => {
            debug!("{} banishing itself", entity_name);
            set_active(env, entity_name, false);
        }
        Statement::BanishNamed(other_name) => {
            debug!("{} banishing {}", entity_name, other_name);
            set_active(env, other_name, false);
        }
        Statement::Disturb => {
            let species = recipe.creatures().get(entity_name).unwrap().species();
            debug!(
                "{} (Species {}) tries to disturb itself (wtf?)",
                entity_name, species,
            );
            if matches!(species, Species::Ghost) {
                set_active(env, entity_name, true);
            }
        }
        Statement::DisturbNamed(other_name) => {
            let species = recipe
                .creatures()
                .get(other_name.as_str())
                .unwrap()
                .species();
            debug!(
                "{} tries to disturb {} (Species {})",
                entity_name, other_name, species,
            );
            if matches!(species, Species::Ghost) {
                set_active(env, other_name, true);
            }
        }
        Statement::Forget => {
            debug!("{} forgets its value", entity_name);
            set_value(env, entity_name, Value::default())
        }
        Statement::ForgetNamed(other_name) => {
            debug!("{} makes {} forget its value", entity_name, other_name);
            set_value(env, other_name, Value::default())
        }
        Statement::Invoke => {
            debug!("{} invoking a new copy of itself", entity_name);
            sender
                .send(Message::Invoke(String::from(entity_name)))
                .expect("Message receiver dropped before task could finish!");
        }
        Statement::InvokeNamed(other_name) => {
            debug!("{} invoking a new copy of {}", entity_name, other_name);
            sender
                .send(Message::Invoke(String::from(other_name)))
                .expect("Message receiver dropped before task could finish!");
        }
        Statement::Remember(exprs) => {
            let value = evaluate_expressions(exprs);
            debug!("{} remembering {} (self)", entity_name, value);
            set_value(env, entity_name, value)
        }
        Statement::RememberNamed(other_name, exprs) => {
            let value = evaluate_expressions(exprs);
            debug!("{} remembering {}", other_name, value);
            set_value(env, other_name, value)
        }
        Statement::SayNamed(_, exprs) | Statement::Say(exprs) => {
            let value = evaluate_expressions(exprs);
            let mut stdout = io::stdout();
            stdout
                .write_all((format!("{}\n", value)).as_bytes())
                .await
                .expect("Could not output text!");
        }
    }
}

fn evaluate_expressions(expr: &Vec<Expression>) -> Value {
    todo!()
}

fn set_active(env: &Arc<SharedEnv>, entity_name: &str, active: bool) {
    env.creatures().alter(entity_name, |_, mut data| {
        *data.active_mut() = active;
        data
    });
    if active {
        env.notifier().notify_waiters();
    }
}

fn set_value(env: &Arc<SharedEnv>, entity_name: &str, value: Value) {
    env.creatures().alter(entity_name, |_, mut data| {
        *data.value_mut() = value;
        data
    });
}

#[derive(Debug, Clone)]
enum Message {
    Invoke(String),
}

#[derive(Debug, Default)]
struct SharedEnv {
    creatures: DashMap<String, EntityData>,
    notifier: Notify,
}

impl SharedEnv {
    fn new() -> SharedEnv {
        SharedEnv {
            creatures: DashMap::new(),
            notifier: Notify::new(),
        }
    }

    fn creatures(&self) -> &DashMap<String, EntityData> {
        &self.creatures
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

impl From<&Creature> for EntityData {
    fn from(creature: &Creature) -> EntityData {
        EntityData::new(Value::from(creature.moan()), creature.active())
    }
}
