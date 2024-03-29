use std::sync::Arc;
use std::time::Duration;

use async_recursion::async_recursion;
use log::{debug, error};
use smol_str::SmolStr;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time;

use super::state::State;
use super::Message;
use crate::scroll::entity::{Entity, Species};
use crate::scroll::expression::Expr;
use crate::scroll::statement::Stmt;
use crate::scroll::task::Task;
use crate::value::Value;

// static DEMON_RESAMPLE_COUNT_RNG_DISTRIBUTION: Lazy<Uniform<u64>> = Lazy::new(|| Uniform::from(0..=5));

pub type Candle = Arc<SmolStr>;

// Represents a summoned creature. Fields are read-only.
pub struct Spirit<'a> {
    name: SmolStr,
    creature: &'a Entity,
    sender: UnboundedSender<Message>,
}

struct RunningTask {
    active: bool,
}

impl RunningTask {
    fn new() -> RunningTask {
        RunningTask { active: true }
    }

    fn active(&self) -> bool {
        self.active
    }

    fn active_mut(&mut self) -> &mut bool {
        &mut self.active
    }
}

impl<'a: 'static> Spirit<'a> {
    pub fn summon(
        name: SmolStr,
        creature: &'a Entity,
        sender: UnboundedSender<Message>,
    ) -> Arc<Spirit<'a>> {
        Arc::new(Spirit {
            name,
            creature,
            sender,
        })
    }

    pub async fn unleash(self: Arc<Self>, state: Arc<State>, _candle: Candle) {
        match self.creature.species() {
            Species::Zombie => {
                for task in self.creature.tasks().values() {
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&self).perform(Arc::clone(&state), task)).await
                    {
                        error!("{}", e);
                    }
                }
            }
            Species::Ghost => {
                for task in self.creature.tasks().values() {
                    if let Err(e) =
                        tokio::spawn(Arc::clone(&self).perform(Arc::clone(&state), task)).await
                    {
                        error!("{}", e);
                    }
                    time::sleep(Duration::from_millis(fastrand::u64(500..=10_000))).await;
                }
            }
            Species::Vampire => {
                let mut tasks: Vec<&Task> = self.creature.tasks().values().collect();
                fastrand::shuffle(&mut tasks);
                for task in tasks {
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
                todo!()
            //     let sample_size = fastrand::usize(1..=10 * self.creature.tasks().len());
            //     let mut task_ids: Vec<usize> =
            //         iter::repeat_with(|| fastrand::usize(0..self.creature.tasks().len()))
            //             .take(sample_size)
            //             .collect();

            //     debug!("Djinn task order {:?}", &task_ids);
            //     while !task_ids.is_empty() {
            //         let mut tasks = Vec::new();
            //         for _ in
            //             1..=fastrand::usize(1..=(f32::ceil(task_ids.len() as f32 / 5.0) as usize))
            //         {
            //             let selected = task_ids.pop().unwrap();
            //             let task = self.creature.tasks().get_index(selected).unwrap();
            //             tasks.push(tokio::spawn(tokio::spawn(
            //                 Arc::clone(&self).perform(Arc::clone(&state), task),
            //             )));
            //         }
            //         for e in future::join_all(tasks)
            //             .await
            //             .into_iter()
            //             .filter_map(|t| t.err())
            //         {
            //             error!("{}", e);
            //         }
            //     }
            }
        }
    }

    // perform a task asynchronously
    async fn perform(self: Arc<Self>, state: Arc<State>, task: &'a Task) {
        debug!("{} performing task {}", self.name, task.name());
        let mut running_task = RunningTask::new();
        self.exec_stmts(&state, &mut running_task, task.statements())
            .await;
    }

    // #[async_recursion]
    async fn exec_stmts(
        &self,
        state: &Arc<State>,
        task: &mut RunningTask,
        stmts: &'a Vec<Stmt>,
    ) {
        debug!("{} executing statements {:?}", self.name, stmts);
        for stmt in stmts {
            // wait until entity is active
            loop {
                if state.knowledge().get(&self.name).unwrap().active() {
                    break;
                } else {
                    // sleep until notified, then check again
                    state.notifier().notified().await;
                }
            }
            // execute one statement at a time
            // let other tasks perform and check for being active again before next statement
            self.exec_stmt(state, task, stmt).await;

            // check if task is still active
            if !task.active() {
                // abort since a task cannot be reactivated
                break;
            }

            tokio::task::yield_now().await;
        }
    }

    #[async_recursion]
    async fn exec_stmt(&self, state: &Arc<State>, task: &mut RunningTask, stmt: &'a Stmt) {
        match stmt {
            Stmt::Animate(None) => {
                debug!(
                    "{} (Species {}) tries to animate itself",
                    self.name,
                    self.creature.species(),
                );
                self.send_message(Message::Animate(self.name.clone()));
            }
            Stmt::Animate(Some(other_name)) => {
                debug!("{} tries to animate {}", self.name, other_name);
                self.send_message(Message::Animate(other_name.clone()));
            }
            Stmt::Banish(None) => {
                debug!("{} banishing itself", self.name);
                set_active(&state, self.name.as_str(), false);
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
                self.send_message(Message::Disturb(self.name.clone()));
            }
            Stmt::Disturb(Some(other_name)) => {
                debug!("{} tries to disturb {}", self.name, other_name);
                self.send_message(Message::Disturb(other_name.clone()));
            }
            Stmt::Forget(None) => {
                debug!("{} forgets its value", self.name);
                set_value(&state, self.name.as_str(), Value::default())
            }
            Stmt::Forget(Some(other_name)) => {
                debug!("{} makes {} forget its value", self.name, other_name);
                set_value(&state, other_name, Value::default())
            }
            Stmt::Invoke(None) => {
                debug!("{} invoking a new copy of itself", self.name);
                self.send_message(Message::Invoke(self.name.clone()));
            }
            Stmt::Invoke(Some(other_name)) => {
                debug!("{} invoking a new copy of {}", self.name, other_name);
                self.send_message(Message::Invoke(other_name.clone()));
            }
            Stmt::Remember(None, exprs) => {
                let value = self.eval_exprs(&state, exprs);
                debug!("{} remembering {} (self)", self.name, value);
                set_value(&state, self.name.as_str(), value)
            }
            Stmt::Remember(Some(other_name), exprs) => {
                let value = self.eval_exprs(&state, exprs);
                debug!("{} remembering {} (from {})", other_name, value, self.name);
                set_value(&state, other_name, value)
            }
            Stmt::Say(name, exprs) => {
                let value = self.eval_exprs(&state, exprs);
                match name {
                    None => debug!("{} saying {:?} (is {})", self.name, exprs, value),
                    Some(other_name) => debug!("{} saying {:?} (is {})", other_name, exprs, value),
                }
                self.send_message(Message::Say(value));
            }
            Stmt::ShambleUntil(expr, stmts) => loop {
                let cond = self.eval_standalone_expr(&state, expr);
                debug!(
                    "{} shambling until {:?} is true (currently {})",
                    self.name, expr, cond
                );
                match cond {
                    Value::Boolean(true) => {
                        break;
                    }
                    Value::Boolean(false) => {
                        self.exec_stmts(&state, task, stmts).await;
                    }
                    value => panic!("Not a boolean: {}", value),
                }
            },
            Stmt::ShambleAround(stmts) => loop {
                debug!("{} shambling around", self.name);
                self.exec_stmts(&state, task, stmts).await;
            },
            Stmt::Stumble => {
                debug!("{} stumbling", self.name);
                *task.active_mut() = false;
            }
            Stmt::Taste(expr, stmts1, stmts2) => {
                let cond = self.eval_standalone_expr(&state, expr);
                debug!("{} tasting {:?} (tastes like {})...", self.name, expr, cond);
                match cond {
                    Value::Boolean(true) => {
                        debug!("...{} likes the taste", self.name);
                        self.exec_stmts(&state, task, stmts1).await;
                    }
                    Value::Boolean(false) => {
                        debug!("...{} hates the taste", self.name);
                        self.exec_stmts(&state, task, stmts2).await;
                    }
                    value => panic!("Not a boolean: {}", value),
                }
            }
        }
    }

    fn eval_exprs(&self, state: &Arc<State>, exprs: &Vec<Expr>) -> Value {
        debug!("{} evaluating expressions {:?}", self.name, exprs);
        let mut stack = vec![Value::default()];
        for index in (0..exprs.len()).rev() {
            let expr = exprs.get(index).unwrap();
            self.eval_expr(state, expr, &mut stack);
            debug!(
                "{} evaluating expression {:?} (Stack {:?})",
                self.name, expr, stack
            );
        }
        stack.pop().unwrap()
    }

    fn eval_standalone_expr(&self, state: &Arc<State>, expr: &Expr) -> Value {
        let mut stack = vec![Value::default()];
        self.eval_expr(state, expr, &mut stack);
        debug!(
            "{} evaluating standalone expression {:?} to {}",
            self.name,
            expr,
            stack.last().unwrap()
        );
        let value = stack.pop().unwrap();
        value
    }

    /// Evaluate the expression. The stack is modified accordingly. The returned value is put on top of the stack as well.
    fn eval_expr(&self, state: &Arc<State>, expr: &Expr, stack: &mut Vec<Value>) {
        match expr {
            Expr::Moan(None) => {
                *stack.last_mut().unwrap() = get_value(state, self.name.as_str()) + stack.last().unwrap();
            }
            Expr::Moan(Some(other_name)) => {
                *stack.last_mut().unwrap() = get_value(state, other_name) + stack.last().unwrap();
            }
            Expr::Remembering(None, value) => {
                stack.push(Value::Boolean(value == get_value(state, self.name.as_str())))
            }
            Expr::Remembering(Some(other_name), value) => {
                stack.push(Value::Boolean(value == get_value(state, other_name)))
            }
            Expr::Rend => {
                let top = &stack.pop().unwrap();
                *stack.last_mut().unwrap() = stack.last().unwrap() / top;
            }
            Expr::Turn => {
                *stack.last_mut().unwrap() = -stack.last().unwrap();
            }
            Expr::Value(value) => stack.push(value.clone()),
        }
    }

    fn send_message(&self, message: Message) {
        self.sender
            .send(message)
            .expect("Message receiver dropped before task could finish!");
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
