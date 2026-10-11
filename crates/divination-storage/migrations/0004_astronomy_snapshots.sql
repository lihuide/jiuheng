CREATE TABLE IF NOT EXISTS astronomy_snapshots (
    question_id TEXT PRIMARY KEY REFERENCES questions(id),
    snapshot_version INTEGER NOT NULL,
    snapshot_json TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS prevent_astronomy_snapshot_update
BEFORE UPDATE ON astronomy_snapshots
BEGIN
    SELECT RAISE(ABORT, 'astronomy snapshot is immutable');
END;

CREATE TRIGGER IF NOT EXISTS prevent_astronomy_snapshot_delete
BEFORE DELETE ON astronomy_snapshots
BEGIN
    SELECT RAISE(ABORT, 'astronomy snapshot is immutable');
END;
