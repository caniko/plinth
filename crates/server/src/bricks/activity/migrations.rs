use crate::bricks::BrickMigration;

pub fn activity_migrations() -> Vec<BrickMigration> {
    vec![BrickMigration {
        brick: "activity",
        version: 1,
        name: "initial_activity_schema",
        up: "",
    }]
}
