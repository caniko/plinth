ALTER TABLE portfolio_items
    ADD COLUMN project_url TEXT,
    ADD COLUMN links JSONB NOT NULL DEFAULT '[]'::jsonb;

INSERT INTO schema_migrations (brick, version, name)
VALUES ('portfolio', 2, 'project_link_schema');
