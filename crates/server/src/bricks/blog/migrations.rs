use crate::bricks::BrickMigration;

/// Return the list of database migrations for the blog brick.
pub fn blog_migrations() -> Vec<BrickMigration> {
    vec![
        BrickMigration {
            brick: "blog",
            version: 1,
            name: "initial_blog_schema",
            up: "",
        },
        BrickMigration {
            brick: "blog",
            version: 2,
            name: "add_series_fields",
            up: "",
        },
        BrickMigration {
            brick: "blog",
            version: 3,
            name: "add_declarative_source_fields",
            up: "",
        },
    ]
}
