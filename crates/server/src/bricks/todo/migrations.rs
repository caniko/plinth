use crate::bricks::BrickMigration;

pub fn todo_migrations() -> Vec<BrickMigration> {
    vec![BrickMigration {
        brick: "todo",
        version: 1,
        name: "initial_todo_schema",
        up: r#"
                DEFINE TABLE todos SCHEMAFULL;

                DEFINE FIELD slug ON TABLE todos TYPE string;
                DEFINE FIELD title ON TABLE todos TYPE string;
                DEFINE FIELD description ON TABLE todos TYPE string;
                DEFINE FIELD content ON TABLE todos TYPE option<string>;
                DEFINE FIELD html_content ON TABLE todos TYPE option<string>;
                DEFINE FIELD tags ON TABLE todos TYPE array<string>;
                DEFINE FIELD completed ON TABLE todos TYPE bool DEFAULT false;
                DEFINE FIELD completed_at ON TABLE todos TYPE option<datetime>;
                DEFINE FIELD created_at ON TABLE todos TYPE datetime;
                DEFINE FIELD order ON TABLE todos TYPE int DEFAULT 0;

                DEFINE INDEX todos_slug_idx ON TABLE todos COLUMNS slug UNIQUE;
                DEFINE INDEX todos_completed_idx ON TABLE todos COLUMNS completed;
                DEFINE INDEX todos_order_idx ON TABLE todos COLUMNS order;
                DEFINE INDEX todos_tags_idx ON TABLE todos COLUMNS tags;

                -- Graph relation: todos -> todo_tagged -> tags
                DEFINE TABLE todo_tagged SCHEMAFULL TYPE RELATION FROM todos TO tags;
                DEFINE FIELD created_at ON TABLE todo_tagged TYPE datetime;
            "#,
    }]
}
