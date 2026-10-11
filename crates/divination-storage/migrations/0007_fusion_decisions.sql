CREATE TABLE IF NOT EXISTS fusion_decisions (
    question_id TEXT PRIMARY KEY REFERENCES questions(id),
    algorithm_version TEXT NOT NULL,
    signal_schema_version INTEGER NOT NULL,
    decision_json TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS prevent_fusion_decision_update
BEFORE UPDATE ON fusion_decisions
BEGIN
    SELECT RAISE(ABORT, 'fusion decision is immutable');
END;

CREATE TRIGGER IF NOT EXISTS prevent_fusion_decision_delete
BEFORE DELETE ON fusion_decisions
BEGIN
    SELECT RAISE(ABORT, 'fusion decision is immutable');
END;
