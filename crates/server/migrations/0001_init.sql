-- Phase 02 Postgres schema initialization.
--
-- ID strategy: every content table uses BIGSERIAL. The previous record IDs
-- were opaque, and BIGSERIAL keeps the greenfield Postgres schema simple
-- without an additional UUID extension.

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE schema_migrations (
    brick TEXT NOT NULL,
    version INTEGER NOT NULL,
    name TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (brick, version)
);

INSERT INTO schema_migrations (brick, version, name)
VALUES ('core', 1, 'init');
