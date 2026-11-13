CREATE TABLE taiyi_candidates (
    id TEXT PRIMARY KEY,
    question_id TEXT NOT NULL REFERENCES questions(id),
    request_hash TEXT NOT NULL,
    snapshot_json TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(question_id, request_hash)
);

CREATE TRIGGER prevent_taiyi_candidate_update
BEFORE UPDATE ON taiyi_candidates
BEGIN
    SELECT RAISE(ABORT, 'saved taiyi candidate is immutable');
END;

CREATE TRIGGER prevent_taiyi_candidate_delete
BEFORE DELETE ON taiyi_candidates
BEGIN
    SELECT RAISE(ABORT, 'saved taiyi candidate is immutable');
END;

CREATE TRIGGER enforce_taiyi_locked_question
BEFORE INSERT ON taiyi_candidates
WHEN NOT EXISTS(SELECT 1 FROM questions WHERE id = NEW.question_id AND locked = 1)
BEGIN
    SELECT RAISE(ABORT, 'taiyi candidate requires a locked question');
END;
