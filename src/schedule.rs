use std::io::{self, Write};

use crate::entity::Entity;
use crate::parse::SyntaxTree;
use crate::statement::{Statement, StatementCmd};

pub struct Scheduler {}

pub struct EntitySummoner<'a> {
    entity: &'a Entity,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {}
    }

    #[tokio::main(flavor = "multi_thread")]
    pub async fn schedule(&self, syntax_tree: &SyntaxTree) {
        let entities = syntax_tree.entities();
        for entity in entities {
            EntitySummoner::new(entity).awaken().await;
        }
    }
}

impl<'a> EntitySummoner<'a> {
    fn new(entity: &Entity) -> EntitySummoner {
        EntitySummoner { entity }
    }

    async fn awaken(&self) {
        for task in self.entity.tasks() {
            for statement in task.statements() {
                execute_statement(statement);
            }
        }
    }
}

#[allow(unused_must_use)]
fn execute_statement(statement: &Statement) {
    match statement.cmd() {
        StatementCmd::Say(arg) => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle.write_all((format!("{}", arg)).as_bytes());
        }
        _ => unimplemented!()
    }
}
