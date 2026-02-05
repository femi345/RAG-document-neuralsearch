CREATE TABLE connectors (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL,
    source_type     TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    credentials     JSONB NOT NULL DEFAULT '{}',
    scopes          TEXT[],
    last_sync_at    TIMESTAMPTZ,
    sync_cursor     TEXT,
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, source_type)
);
