use std::iter;
use std::sync::Arc;
use std::time::Duration;

use async_recursion::async_recursion;
use dashmap::{DashMap, DashSet};
use fastrand::Rng;
use futures::future::{self, AbortHandle, Abortable};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{debug, error, warn};
use once_cell::sync::Lazy;
use tokio::io::{self, AsyncWriteExt};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::task::JoinHandle;
use tokio::time;

use crate::scroll::creature::{Creature, Species};
use crate::scroll::expression::Expr;
use crate::scroll::statement::Stmt;
use crate::scroll::task::Task;
use crate::scroll::Scroll;
use crate::value::Value;

use super::state::State;
use super::Message;

// static GHOST_RNG_DISTRIBUTION: Lazy<Uniform<u64>> = Lazy::new(|| Uniform::from(500..=10000));
// static DEMON_RESAMPLE_COUNT_RNG_DISTRIBUTION: Lazy<Uniform<u64>> = Lazy::new(|| Uniform::from(0..=5));

pub type Candle<'a> = Arc<&'a str>;

// Represents a summoned creature. Fields are read-only.
pub struct Spirit<'a> {
    name: &'a str,
    creature: &'a Creature<'a>,
    // sender: UnboundedSender<Message>,
}

impl<'a: 'static> Spirit<'a> {
    pub fn summon(
        name: &'a str,
        creature: &'a Creature<'a>, // sender: UnboundedSender<Message>,
    ) -> Arc<Spirit<'a>> {
        Arc::new(Spirit {
            name,
            creature,
            // sender,
        })
    }

    pub async fn unleash(self: Arc<Self>, state: Arc<State>, _candle: Candle<'a>) {
        match self.creature.species() {
            Species::Zombie => {
                for task in self.creature.tasks() {
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&self).perform(Arc::clone(&state), task)).await
                    {
                        error!("{}", e);
                    }
                }
            }
            Species::Ghost => {
                for task in self.creature.tasks() {
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&self).perform(Arc::clone(&state), task)).await
                    {
                        error!("{}", e);
                    }
                    time::sleep(Duration::from_millis(fastrand::u64(500..=10_000))).await;
                }
            }
            Species::Vampire => {
                let mut task_ids: Vec<usize> = (0..self.creature.tasks().len()).collect();
                fastrand::shuffle(&mut task_ids);
                for idx in task_ids {
                    let task = self.creature.tasks().get_index(idx).unwrap();
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&self).perform(Arc::clone(&state), task)).await
                    {
                        error!("{}", e);
                    }
                }
            }
            Species::Demon => {
                // TODO fix demon
                todo!();
                // let mut rng = SmallRng::from_entropy();
                // let mut sample =
                //     index::sample(&mut rng, creature.tasks().len(), creature.tasks().len())
                //         .into_vec();
                // for _ in 0..=DEMON_RESAMPLE_COUNT_RNG_DISTRIBUTION.sample(&mut rng) {
                //     let resample_size = rng.gen_range(0..=creature.tasks().len() / 3);
                //     sample.extend(index::sample(
                //         &mut rng,
                //         creature.tasks().len(),
                //         resample_size,
                //     ));
                // }

                // debug!("Demon task order {:?}", &sample);
                // while !sample.is_empty() {
                //     if rng.gen_ratio(33, 100 * sample.len() as u32) {
                //         awakened
                //             .sender
                //             .send(Message::Invoke(String::from(&awakened.name)))
                //             .expect("Message receiver dropped before task could finish!");
                //         debug!("Spawning helper demon!");
                //     }
                //     let mut tasks = Vec::new();
                //     for _ in 1..=rng.gen_range(1..=(f32::ceil(sample.len() as f32 / 5.0) as i64)) {
                //         let selected = sample.pop().unwrap();
                //         let task = creature.tasks().get_index(selected).unwrap();
                //         tasks.push(tokio::spawn(
                //             Arc::clone(&awakened).perform(String::from(task.name())),
                //         ));
                //     }
                //     for e in future::join_all(tasks)
                //         .await
                //         .into_iter()
                //         .filter_map(|t| t.err())
                //     {
                //         error!("{}", e);
                //     }
                // }
            }
            Species::Djinn => {
                let sample_size = fastrand::usize(1..=10 * self.creature.tasks().len());
                let mut task_ids: Vec<usize> =
                    iter::repeat_with(|| fastrand::usize(0..self.creature.tasks().len()))
                        .take(sample_size)
                        .collect();

                debug!("Djinn task order {:?}", &task_ids);
                while !task_ids.is_empty() {
                    let mut tasks = Vec::new();
                    for _ in
                        1..=fastrand::usize(1..=(f32::ceil(task_ids.len() as f32 / 5.0) as usize))
                    {
                        let selected = task_ids.pop().unwrap();
                        let task = self.creature.tasks().get_index(selected).unwrap();
                        tasks.push(tokio::spawn(tokio::spawn(
                            Arc::clone(&self).perform(Arc::clone(&state), task),
                        )));
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

    // perform a task asynchronously
    async fn perform(self: Arc<Self>, state: Arc<State>, task: &'a Task<'a>) {
        self.exec_stmts(state, task.statements());
    }

    #[async_recursion]
    async fn exec_stmts(&self, state: Arc<State>, stmts: &'a Vec<Stmt<'a>>) {
        for stmt in stmts {
            // wait until active
            loop {
                if state.knowledge().get(self.name).unwrap().active() {
                    break;
                } else {
                    // sleep until notified, then check again
                    state.notifier().notified().await;
                }
            }
            // execute one statement at a time
            // let other tasks perform and check for being active again before next statement
            self.exec_stmt(Arc::clone(&state), stmt).await;
            tokio::task::yield_now().await;
        }
    }

    // TODO implement message passing for spawning
    async fn exec_stmt(&self, state: Arc<State>, stmt: &'a Stmt<'a>) {
        match stmt {
            Stmt::Animate(None) => {
                debug!(
                    "{} (Species {}) tries to animate itself",
                    self.name,
                    self.creature.species(),
                );
                todo!();
                // if matches!(species, Species::Zombie) {
                //     set_active(state, self.name, true);
                // }
            }
            Stmt::Animate(Some(other_name)) => {
                debug!("{} tries to animate {}", self.name, other_name);
                todo!();
                // if matches!(species, Species::Zombie) {
                //     set_active(state, other_name, true);
                // }
            }
            Stmt::Banish(None) => {
                debug!("{} banishing itself", self.name);
                set_active(&state, self.name, false);
            }
            Stmt::Banish(Some(other_name)) => {
                debug!("{} banishing {}", self.name, other_name);
                set_active(&state, other_name, false);
            }
            Stmt::Disturb(None) => {
                debug!(
                    "{} (Species {}) tries to disturb itself",
                    self.name,
                    self.creature.species(),
                );
                todo!();
                // if matches!(species, Species::Ghost) {
                //     set_active(state, self.name, true);
                // }
            }
            Stmt::Disturb(Some(other_name)) => {
                debug!("{} tries to disturb {}", self.name, other_name);
                todo!();
                // if matches!(species, Species::Ghost) {
                //     set_active(state, other_name, true);
                // }
            }
            Stmt::Forget(None) => {
                debug!("{} forgets its value", self.name);
                set_value(&state, self.name, Value::default())
            }
            Stmt::Forget(Some(other_name)) => {
                debug!("{} makes {} forget its value", self.name, other_name);
                set_value(&state, other_name, Value::default())
            }
            Stmt::Invoke(None) => {
                debug!("{} invoking a new copy of itself", self.name);
                todo!();
                // sender
                //     .send(Message::Invoke(String::from(self.name)))
                //     .expect("Message receiver dropped before task could finish!");
            }
            Stmt::Invoke(Some(other_name)) => {
                debug!("{} invoking a new copy of {}", self.name, other_name);
                todo!();
                // sender
                //     .send(Message::Invoke(String::from(other_name)))
                //     .expect("Message receiver dropped before task could finish!");
            }
            Stmt::Remember(None, exprs) => {
                let value = self.eval_exprs(&state, exprs);
                debug!("{} remembering {} (self)", self.name, value);
                set_value(&state, self.name, value)
            }
            Stmt::Remember(Some(other_name), exprs) => {
                let value = self.eval_exprs(&state, exprs);
                debug!("{} remembering {} (from {})", other_name, value, self.name);
                set_value(&state, other_name, value)
            }
            Stmt::Say(_, exprs) => {
                let value = self.eval_exprs(&state, exprs);
                let mut stdout = io::stdout();
                stdout
                    .write_all((format!("{}\n", value)).as_bytes())
                    .await
                    .expect("Could not output text!");
            }
            Stmt::ShambleUntil(expr, stmts) => loop {
                let cond = self.eval_standalone_expr(&state, expr);
                match cond {
                    Value::Boolean(true) => {
                        self.exec_stmts(Arc::clone(&state), stmts).await;
                    }
                    Value::Boolean(false) => {}
                    value => panic!("Not a boolean: {}", value),
                }
            },
            Stmt::ShambleAround(stmts) => loop {
                self.exec_stmts(Arc::clone(&state), stmts).await;
            },
            Stmt::Stumble => {
                // TODO
                // pass join handle here and call abort()?
                // or make executed task a separate struct that has a cancel flag
                // issue is that being `active` is only possible for creatures, not tasks
                todo!();
            }
            Stmt::Taste(expr, stmts1, stmts2) => {
                let cond = self.eval_standalone_expr(&state, expr);
                match cond {
                    Value::Boolean(true) => {
                        self.exec_stmts(Arc::clone(&state), stmts1).await;
                    }
                    Value::Boolean(false) => {
                        self.exec_stmts(Arc::clone(&state), stmts2).await;
                    }
                    value => panic!("Not a boolean: {}", value),
                }
            }
        }
    }

    fn eval_exprs(&self, state: &Arc<State>, exprs: &Vec<Expr>) -> Value {
        let mut stack = Vec::new();
        for index in (0..exprs.len()).rev() {
            let expr = exprs.get(index).unwrap();
            self.eval_expr(state, expr, &mut stack);
        }
        stack.pop().unwrap()
    }

    fn eval_standalone_expr(&self, state: &Arc<State>, expr: &Expr) -> Value {
        let mut stack = Vec::new();
        self.eval_expr(state, expr, &mut stack);
        stack.pop().unwrap()
    }

    /// Evaluate the expression. The stack is modified accordingly. The returned value is put on top of the stack as well.
    fn eval_expr(&self, state: &Arc<State>, expr: &Expr, stack: &mut Vec<Value>) {
        match expr {
            Expr::Moan(None) => stack.push(get_value(state, self.name)),
            Expr::Moan(Some(other_name)) => stack.push(get_value(state, other_name)),
            Expr::Remembering(None, value)  => {
                stack.push(Value::Boolean(value == get_value(state, self.name)))
            }
            Expr::Remembering(Some(other_name), value) => {
                stack.push(Value::Boolean(value == get_value(state, other_name)))
            }
            Expr::Rend => {
                let fst = &stack.pop().unwrap();
                let snd = &stack.pop().unwrap();
                stack.push(snd / fst);
            }
            Expr::Turn => {
                *stack.last_mut().unwrap() = -stack.last().unwrap();
            }
            Expr::Value(value) => stack.push(value.clone()),
        }
    }
}

fn set_active(state: &State, name: &str, active: bool) {
    state.knowledge().alter(name, |_, mut spirit| {
        *spirit.active_mut() = active;
        spirit
    });
    if active {
        state.notifier().notify_waiters();
    }
}

fn get_value(state: &State, name: &str) -> Value {
    state.knowledge().get(name).unwrap().memory().clone()
}

fn set_value(state: &State, name: &str, value: Value) {
    state.knowledge().alter(name, |_, mut spirit| {
        *spirit.memory_mut() = value;
        spirit
    });
}
