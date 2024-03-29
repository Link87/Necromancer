use std::sync::Arc;
use std::time::Duration;

use dashmap::DashSet;
use futures::future::{AbortHandle, Abortable};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use log::{debug, warn};
use smol_str::SmolStr;
use state::State;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time;

use crate::necro::summon::{Candle, Spirit};
use crate::scroll::entity::{Entity, Species};
use crate::scroll::{EntityList, Scroll};
use crate::value::Value;

mod state;
mod summon;

pub struct Necromancer {
    scroll: Scroll,
}

impl Necromancer {
    pub fn unroll(scroll: Scroll) -> Necromancer {
        Necromancer { scroll }
    }

    // calling this runs the interpreter
    // `Ritual` owns any data that is needed for managing the entities from a 'top-level' view.
    // In addition, `State` holds any data that is needed from within the entities. Both are Arc<>,
    // since they're shared between threads.
    // Ritual spawns a tokio task for every entity. Every entity itself spawns a tokio task for each
    // of their tasks.
    #[tokio::main(flavor = "multi_thread")]
    pub async fn initiate(self) {
        // we need a static reference to the AST
        // TODO rewrite (this is too hacky imo)
        let scroll: &'static Scroll = Box::leak(Box::new(self.scroll));

        let creatures = scroll.creatures();
        let ritual = Ritual::new(creatures).await;

        // Abort futures (i.e. kill program) if every entity is inactive.
        // poll `Ritual::watchdog()` every second.
        let ritual_wd = Arc::clone(&ritual);
        let watchdog = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                debug!("Watchdog tick.");
                Ritual::watchdog(Arc::clone(&ritual_wd)).await;
            }
        });

        // wait for messages to arrive
        // runs indefinetly as it holds both sender and receiver refs
        let ritual_msg = Arc::clone(&ritual);
        let message_handler = tokio::spawn(async move {
            while let Some(message) = Ritual::received(Arc::clone(&ritual_msg)).await {
                match message {
                    Message::Animate(name) => {
                        let creature = creatures.get(&name).unwrap();
                        if matches!(creature.species(), Species::Zombie) {
                            Arc::clone(&ritual_msg).summon(creature).await;
                        }
                    }
                    Message::Disturb(name) => {
                        let creature = creatures.get(&name).unwrap();
                        if matches!(creature.species(), Species::Ghost) {
                            Arc::clone(&ritual_msg).summon(creature).await;
                        }
                    }
                    Message::Invoke(name) => {
                        let creature = creatures.get(&name).unwrap();
                        Arc::clone(&ritual_msg).summon(creature).await;
                    }
                    Message::Say(value) => {
                        println!("{}", value);
                    }
                }
            }
        });

        Ritual::finished(ritual).await;

        // watchdog useless now
        watchdog.abort();

        // Messages are no longer needed.
        // Necessary since message does not exit on its own.
        message_handler.abort();
    }
}

pub struct Ritual {
    /// The global state. Reference shared with the [`Spirit`]s.
    state: Arc<State>,
    /// Collection of `Future`s that are associated with an entity.
    /// A future completes when the corresponding entity is finished,
    /// i.e. the Tokio task finishes.
    /// [`Abortable`] provides a way to abort the computation.
    tasks: RwLock<FuturesUnordered<Abortable<JoinHandle<()>>>>,
    /// [`AbortHandles`] for aborting the computations.
    abort_handles: RwLock<Vec<AbortHandle>>,
    /// A candle is lit for every copy of an entity. This is used to count
    /// how many copies of an entity are alive.
    /// The `Ritual` is finished if all candles go out and the program can be killed.
    candles: DashSet<Candle>,
    /// Sender of an unbounded channel. To be distibuted to the entities.
    sender: UnboundedSender<Message>,
    /// Receiver of an unbounded channel. To be kept to receive messages from entities.
    receiver: Mutex<UnboundedReceiver<Message>>,
}

impl<'a: 'static> Ritual {
    /// Prepare the ritual and summon any of the listed creatures.
    async fn new(entities: &'a EntityList) -> Arc<Ritual> {
        let (tx, rx) = mpsc::unbounded_channel();
        let ritual = Arc::new(Ritual {
            state: Arc::new(State::from(entities.values())),
            tasks: RwLock::new(FuturesUnordered::new()),
            abort_handles: RwLock::new(Vec::new()),
            candles: DashSet::new(),
            sender: tx,
            receiver: Mutex::new(rx),
        });

        debug!("{:?}", ritual.state);

        for creature in entities.values() {
            Self::summon(Arc::clone(&ritual), creature).await;
        }

        ritual
    }

    /// Summon a creature in the [`Ritual`].
    async fn summon(self: Arc<Self>, creature: &'a Entity) {
        let spirit = Spirit::summon(
            creature.name(),
            creature,
            UnboundedSender::clone(&self.sender),
        );
        // light a candle
        let candle = Arc::new(creature.name());
        self.candles.insert(Arc::clone(&candle));

        // handle for killing the entity
        let (abort_handle, abort_reg) = AbortHandle::new_pair();
        self.abort_handles.write().await.push(abort_handle);

        // spawn the task and create corresponding future
        let state = Arc::clone(&self.state);
        let join_handle = tokio::spawn(spirit.unleash(state, candle));
        let future = Abortable::new(join_handle, abort_reg);
        self.tasks.read().await.push(future); // TODO Potential dead-lock with (1)
    }

    /// Poll the watchdog
    async fn watchdog(self: Arc<Self>) {
        if self.state.knowledge().iter().all(|c| {
            !c.value().active() || Arc::strong_count(&self.candles.get(c.key()).unwrap()) <= 1
        }) {
            warn!("Watchdog triggered! Aborting: only inactive tasks left.");
            for handle in self.abort_handles.read().await.iter() {
                handle.abort()
            }
        }
    }

    async fn received(self: Arc<Self>) -> Option<Message> {
        self.receiver.lock().await.recv().await
    }

    /// Use the returned `Future` to `await` the end of the ritual.
    async fn finished(self: Arc<Self>) {
        // iterate until a None appears, all tasks are finished then
        while let Some(_) = self.tasks.write().await.next().await {} // TODO Potential dead-lock (1)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Animate(SmolStr),
    Disturb(SmolStr),
    Invoke(SmolStr),
    Say(Value),
}
