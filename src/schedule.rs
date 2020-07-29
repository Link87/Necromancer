use crate::entity::Entity;

pub struct Scheduler {}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {}
    }

    #[tokio::main]
    pub async fn schedule(&self, entities: &mut Vec<Entity>) {
        for entity in entities {
            self.awaken_entity(entity).await;
        }
    }

    async fn awaken_entity(&self, entity: &mut Entity) {
        for task in entity.tasks() {
            for statement in task.statements() {
                statement.execute();
            }
        }
    }
}
