CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE documents (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL,
    source_type     TEXT NOT NULL,
    source_id       TEXT NOT NULL,
    title           TEXT NOT NULL,
    source_url      TEXT,
    content_hash    TEXT NOT NULL,
    chunk_count     INTEGER NOT NULL DEFAULT 0,
    mime_type       TEXT,
    metadata        JSONB DEFAULT '{}',
    indexed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, source_type, source_id)
);

CREATE INDEX idx_documents_user_source ON documents(user_id, source_type);
CREATE INDEX idx_documents_content_hash ON documents(content_hash);
