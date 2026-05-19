CREATE TABLE site_content (
    id BIGSERIAL PRIMARY KEY,
    key TEXT NOT NULL,
    title TEXT,
    content TEXT NOT NULL,
    html_content TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT site_content_key_idx UNIQUE (key)
);

CREATE TABLE tags (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT tags_name_idx UNIQUE (name),
    CONSTRAINT tags_slug_idx UNIQUE (slug)
);

INSERT INTO schema_migrations (brick, version, name)
VALUES ('core', 2, 'core_schema');
