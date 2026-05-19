CREATE TABLE blog_posts (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    html_content TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    author TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    featured BOOLEAN NOT NULL DEFAULT false,
    published BOOLEAN NOT NULL DEFAULT true,
    reading_time_minutes INTEGER NOT NULL,
    -- EMBEDDING_DIM = 384; must match fastembed::AllMiniLML6V2.
    embedding vector(384),
    content_format TEXT NOT NULL DEFAULT 'markdown',
    series_slug TEXT,
    series_title TEXT,
    series_position INTEGER,
    source TEXT NOT NULL DEFAULT 'api',
    content_hash TEXT,
    CONSTRAINT blog_posts_slug_idx UNIQUE (slug)
);

CREATE INDEX blog_posts_published_at_idx ON blog_posts (published_at);
CREATE INDEX blog_posts_tags_idx ON blog_posts USING gin (tags);
CREATE INDEX blog_posts_series_idx ON blog_posts (series_slug);
CREATE INDEX blog_posts_embedding_hnsw_idx
    ON blog_posts USING hnsw (embedding vector_cosine_ops);

CREATE TABLE blog_post_tags (
    post_id BIGINT NOT NULL REFERENCES blog_posts(id) ON DELETE CASCADE,
    tag_id BIGINT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (post_id, tag_id)
);

CREATE INDEX blog_post_tags_tag_id_idx ON blog_post_tags (tag_id);

INSERT INTO schema_migrations (brick, version, name)
VALUES
    ('blog', 1, 'initial_blog_schema'),
    ('blog', 2, 'add_series_fields'),
    ('blog', 3, 'add_declarative_source_fields');
