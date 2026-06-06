use crate::bricks::BrickMigration;

pub fn portfolio_migrations() -> Vec<BrickMigration> {
    vec![
        BrickMigration {
            brick: "portfolio",
            version: 1,
            name: "initial_portfolio_schema",
            up: "",
        },
        BrickMigration {
            brick: "portfolio",
            version: 2,
            name: "project_link_schema",
            up: "",
        },
    ]
}
