CREATE TABLE todos (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    content TEXT,
    html_content TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    completed BOOLEAN NOT NULL DEFAULT false,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    "order" INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT todos_slug_idx UNIQUE (slug)
);

CREATE INDEX todos_completed_idx ON todos (completed);
CREATE INDEX todos_order_idx ON todos ("order");
CREATE INDEX todos_tags_idx ON todos USING gin (tags);

CREATE TABLE todo_tags (
    todo_id BIGINT NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
    tag_id BIGINT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (todo_id, tag_id)
);

CREATE INDEX todo_tags_tag_id_idx ON todo_tags (tag_id);

INSERT INTO schema_migrations (brick, version, name)
VALUES ('todo', 1, 'initial_todo_schema');
