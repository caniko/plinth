CREATE TABLE portfolio_items (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    content TEXT,
    html_content TEXT,
    tech_stack TEXT[] NOT NULL DEFAULT '{}',
    link TEXT,
    demo TEXT,
    image_url TEXT,
    date TIMESTAMPTZ NOT NULL,
    featured BOOLEAN NOT NULL DEFAULT false,
    "order" INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT portfolio_items_slug_idx UNIQUE (slug)
);

CREATE INDEX portfolio_items_date_idx ON portfolio_items (date);
CREATE INDEX portfolio_items_order_idx ON portfolio_items ("order");

INSERT INTO schema_migrations (brick, version, name)
VALUES ('portfolio', 1, 'initial_portfolio_schema');
