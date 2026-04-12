use crate::bricks::BrickMigration;

pub fn portfolio_migrations() -> Vec<BrickMigration> {
    vec![BrickMigration {
        brick: "portfolio",
        version: 1,
        name: "initial_portfolio_schema",
        up: r#"
                DEFINE TABLE portfolio_items SCHEMAFULL;

                DEFINE FIELD slug ON TABLE portfolio_items TYPE string;
                DEFINE FIELD title ON TABLE portfolio_items TYPE string;
                DEFINE FIELD description ON TABLE portfolio_items TYPE string;
                DEFINE FIELD content ON TABLE portfolio_items TYPE option<string>;
                DEFINE FIELD html_content ON TABLE portfolio_items TYPE option<string>;
                DEFINE FIELD tech_stack ON TABLE portfolio_items TYPE array<string>;
                DEFINE FIELD link ON TABLE portfolio_items TYPE option<string>;
                DEFINE FIELD demo ON TABLE portfolio_items TYPE option<string>;
                DEFINE FIELD image_url ON TABLE portfolio_items TYPE option<string>;
                DEFINE FIELD date ON TABLE portfolio_items TYPE datetime;
                DEFINE FIELD featured ON TABLE portfolio_items TYPE bool DEFAULT false;
                DEFINE FIELD order ON TABLE portfolio_items TYPE int DEFAULT 0;

                -- Indexes
                DEFINE INDEX portfolio_items_slug_idx ON TABLE portfolio_items COLUMNS slug UNIQUE;
                DEFINE INDEX portfolio_items_date_idx ON TABLE portfolio_items COLUMNS date;
                DEFINE INDEX portfolio_items_order_idx ON TABLE portfolio_items COLUMNS order;
            "#,
    }]
}
