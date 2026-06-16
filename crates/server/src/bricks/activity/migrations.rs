use crate::bricks::BrickMigration;

/// Return the list of database migrations for the activity brick.
pub fn activity_migrations() -> Vec<BrickMigration> {
    vec![BrickMigration {
        brick: "activity",
        version: 1,
        name: "initial_activity_schema",
        up: "",
    }]
}
