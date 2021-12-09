use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use crate::{
    necro::summon::{Candle, Spirit},
    run,
    scroll::{creature::Creature, Scroll},
};
use dashmap::DashSet;
use futures::{future::join, StreamExt};
use futures::{
    future::{AbortHandle, Abortable},
    stream::FuturesUnordered,
};
use log::{debug, warn};
use state::State;
use tokio::{
    sync::{mpsc, Mutex, RwLock},
    task::JoinHandle,
    time,
};

mod state;
mod summon;

pub struct Necromancer<'a> {
    scroll: Scroll<'a>,
}

impl Necromancer<'static> {
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
        
        
        // TODO
        // let (tx, mut rx) = mpsc::unbounded_channel();
        // wait for messages to arrive
        // runs indefinetly as it holds both sender and receiver refs
        // let tasks_message_handler = Arc::clone(&tasks);
        // let abort_handles_message_handler = Arc::clone(&abort_handles);
        // let candles_message_handler = Arc::clone(&candles);
        // let message_handler = tokio::spawn(async move {
        //     while let Some(message) = rx.recv().await {
        //         match message {
        //             Message::Invoke(ref name) => {
        //                 // spawn new entity and add to awaited futures
        //                 let awakened = Incarnation::materialise(
        //                     String::from(name),
        //                     Arc::clone(&self.recipe),
        //                     Arc::clone(&env),
        //                     UnboundedSender::clone(&tx),
        //                 );
        //                 let candle: Arc<String> =
        //                     Arc::clone(&candles_message_handler.get(name).unwrap());

        //                 let (handle, registration) = AbortHandle::new_pair();
        //                 abort_handles_message_handler.write().await.push(handle);

        //                 tasks_message_handler.lock().await.push(Abortable::new(
        //                     tokio::spawn(awakened.unleash(candle)),
        //                     registration,
        //                 ));
        //             }
        //         }
        //     }
        // });

        Ritual::end(ritual).await;

        // watchdog useless now
        watchdog.abort();

        // Messages are no longer needed.
        // Necessary since message does not exit on its own.
        // message_handler.abort();
    }
}

struct Ritual<'a> {
    /// The global state. Reference shared with the [`Spirit`]s.
    state: Arc<State>,
    /// Collection of `Future`s that are associated with an entity.
    /// A future completes when the corresponding entity is finished,
    /// i.e. the Tokio task finishes.
    /// [`Abortable`] provides a way to abort the computation.
    tasks: FuturesUnordered<Abortable<JoinHandle<()>>>,
    /// [`AbortHandles`] for aborting the computations.
    abort_handles: Vec<AbortHandle>,
    /// A candle is lit for every copy of an entity. This is used to count
    /// how many copies of an entity are alive.
    /// The `Ritual` is finished if all candles go out and the program can be killed.
    candles: HashSet<Candle<'a>>,
}

impl<'a: 'static> Ritual<'a> {
    /// Prepare the ritual and summon any of the listed creatures.
    async fn new(creatures: &'a HashMap<&'a str, Creature<'a>>) -> Arc<RwLock<Ritual<'a>>> {
        let ritual = Arc::new(RwLock::new(Ritual {
            state: Arc::new(State::from(creatures.values())),
            tasks: FuturesUnordered::new(),
            abort_handles: Vec::new(),
            candles: HashSet::new(),
        }));

        debug!("{:?}", ritual.read().await.state);

        for creature in creatures.values() {
            Self::summon(Arc::clone(&ritual), creature).await;
        }

        ritual
    }

    /// Summon a creature in the [`Ritual`].
    async fn summon(ritual: Arc<RwLock<Ritual<'a>>>, creature: &'a Creature<'a>) {
        let mut ritual = ritual.write().await;
        let spirit = Spirit::summon(
            creature.name(),
            creature,
            // UnboundedSender::clone(&tx),
        );
        // light a candle
        let candle = Arc::new(creature.name());
        ritual.candles.insert(Arc::clone(&candle));

        // handle for killing the entity
        let (abort_handle, abort_reg) = AbortHandle::new_pair();
        ritual.abort_handles.push(abort_handle);

        // spawn the task and create corresponding future
        let state = Arc::clone(&ritual.state);
        let join_handle = tokio::spawn(spirit.unleash(state, candle));
        let future = Abortable::new(join_handle, abort_reg);
        ritual.tasks.push(future);
    }

    /// Poll the watchdog
    async fn watchdog(ritual: Arc<RwLock<Ritual<'a>>>) {
        let ritual = ritual.read().await;
        if ritual.state.knowledge().iter().all(|c| {
            !c.value().active() || Arc::strong_count(ritual.candles.get(c.key()).unwrap()) <= 1
        }) {
            warn!("Watchdog triggered! Aborting: only inactive tasks left.");
            for handle in ritual.abort_handles.iter() {
                handle.abort()
            }
        }
    }

    /// Use the returned `Future` to `await` the end of the ritual.
    async fn end(ritual: Arc<RwLock<Self>>) {
        // iterate until a None appears, all tasks are finished then
        while let Some(_) = ritual.write().await.tasks.next().await {}
    }
}

#[derive(Debug, Clone)]
enum Message<'a> {
    Animate(&'a str),
    Disturb(&'a str),
    Invoke(&'a str),
}
