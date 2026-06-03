-- Activity brick: curated external contributions (PRs/issues) across forges,
-- ranked by impact x recency at read time. The `vector` extension is created
-- in 0001_init.sql. EMBEDDING_DIM = 384; must match fastembed::AllMiniLML6V2
-- and blog_posts.embedding.

CREATE TABLE activity_items (
    id BIGSERIAL PRIMARY KEY,
    forge TEXT NOT NULL,                         -- 'github' | 'codeberg'
    repo_owner TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    kind TEXT NOT NULL,                          -- 'pr' | 'issue'
    number INTEGER NOT NULL,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    body TEXT,
    state TEXT NOT NULL,                         -- 'open' | 'closed' | 'merged'
    created_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    merged_at TIMESTAMPTZ,
    impact SMALLINT NOT NULL DEFAULT 1 CHECK (impact BETWEEN 1 AND 10),
    additions INTEGER,
    deletions INTEGER,
    comments_count INTEGER,
    labels TEXT[] DEFAULT '{}',
    repo_stars INTEGER,
    embedding vector(384),
    fetched_at TIMESTAMPTZ NOT NULL,             -- snapshot/refresh time; drives the TTL
    featured BOOLEAN NOT NULL DEFAULT false,
    published BOOLEAN NOT NULL DEFAULT true,
    content_hash TEXT,
    CONSTRAINT activity_items_natural_key UNIQUE (forge, repo_owner, repo_name, kind, number)
);

CREATE INDEX activity_items_state_idx ON activity_items (state);
CREATE INDEX activity_items_featured_idx ON activity_items (featured);
CREATE INDEX activity_items_published_idx ON activity_items (published);
CREATE INDEX activity_items_labels_idx ON activity_items USING gin (labels);
CREATE INDEX activity_items_embedding_hnsw_idx
    ON activity_items USING hnsw (embedding vector_cosine_ops);

INSERT INTO schema_migrations (brick, version, name)
VALUES ('activity', 1, 'initial_activity_schema');
