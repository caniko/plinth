use crate::bricks::BrickMigration;

/// Return the list of database migrations for the todo brick.
pub fn todo_migrations() -> Vec<BrickMigration> {
    vec![BrickMigration {
        brick: "todo",
        version: 1,
        name: "initial_todo_schema",
        up: "",
    }]
}
