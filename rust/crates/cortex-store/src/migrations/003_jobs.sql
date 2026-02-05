CREATE TABLE jobs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL,
    connector_id    UUID REFERENCES connectors(id),
    job_type        TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'queued',
    total_items     INTEGER DEFAULT 0,
    processed_items INTEGER DEFAULT 0,
    error_message   TEXT,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_jobs_status ON jobs(status) WHERE status IN ('queued', 'running');
