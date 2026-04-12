use crate::bricks::BrickMigration;

pub fn blog_migrations() -> Vec<BrickMigration> {
    vec![
        BrickMigration {
            brick: "blog",
            version: 1,
            name: "initial_blog_schema",
            up: r#"
                DEFINE TABLE blog_posts SCHEMAFULL;

                DEFINE FIELD slug ON TABLE blog_posts TYPE string;
                DEFINE FIELD title ON TABLE blog_posts TYPE string;
                DEFINE FIELD description ON TABLE blog_posts TYPE string DEFAULT "";
                DEFINE FIELD content ON TABLE blog_posts TYPE string;
                DEFINE FIELD html_content ON TABLE blog_posts TYPE string;
                DEFINE FIELD published_at ON TABLE blog_posts TYPE datetime;
                DEFINE FIELD updated_at ON TABLE blog_posts TYPE option<datetime>;
                DEFINE FIELD author ON TABLE blog_posts TYPE string;
                DEFINE FIELD tags ON TABLE blog_posts TYPE array<string>;
                DEFINE FIELD featured ON TABLE blog_posts TYPE bool DEFAULT false;
                DEFINE FIELD published ON TABLE blog_posts TYPE bool DEFAULT true;
                DEFINE FIELD reading_time_minutes ON TABLE blog_posts TYPE int;
                DEFINE FIELD embedding ON TABLE blog_posts TYPE option<array<float>>;
                DEFINE FIELD content_format ON TABLE blog_posts TYPE string DEFAULT "markdown";

                -- Indexes
                DEFINE INDEX blog_posts_slug_idx ON TABLE blog_posts COLUMNS slug UNIQUE;
                DEFINE INDEX blog_posts_published_at_idx ON TABLE blog_posts COLUMNS published_at;
                DEFINE INDEX blog_posts_tags_idx ON TABLE blog_posts COLUMNS tags;

                -- Graph relation: blog_posts -> tagged -> tags
                DEFINE TABLE tagged SCHEMAFULL TYPE RELATION FROM blog_posts TO tags;
                DEFINE FIELD created_at ON TABLE tagged TYPE datetime;
            "#,
        },
        BrickMigration {
            brick: "blog",
            version: 2,
            name: "add_series_fields",
            up: r#"
                DEFINE FIELD series_slug ON TABLE blog_posts TYPE option<string>;
                DEFINE FIELD series_title ON TABLE blog_posts TYPE option<string>;
                DEFINE FIELD series_position ON TABLE blog_posts TYPE option<int>;

                DEFINE INDEX blog_posts_series_idx ON TABLE blog_posts COLUMNS series_slug;
            "#,
        },
        BrickMigration {
            brick: "blog",
            version: 3,
            name: "add_declarative_source_fields",
            up: r#"
                DEFINE FIELD source ON TABLE blog_posts TYPE string DEFAULT "api";
                DEFINE FIELD content_hash ON TABLE blog_posts TYPE option<string>;
            "#,
        },
    ]
}
