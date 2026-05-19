use crate::bricks::BrickMigration;

pub fn todo_migrations() -> Vec<BrickMigration> {
    vec![BrickMigration {
        brick: "todo",
        version: 1,
        name: "initial_todo_schema",
        up: "",
    }]
}
