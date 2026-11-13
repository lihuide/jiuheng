CREATE TABLE liuyao_checkpoints (
    question_id TEXT NOT NULL REFERENCES questions(id),
    event_count INTEGER NOT NULL CHECK(event_count BETWEEN 0 AND 6),
    transcript_json TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY(question_id, event_count)
);

CREATE TRIGGER prevent_liuyao_checkpoint_update
BEFORE UPDATE ON liuyao_checkpoints
BEGIN
    SELECT RAISE(ABORT, 'saved liuyao progress is immutable');
END;

CREATE TRIGGER prevent_liuyao_checkpoint_delete
BEFORE DELETE ON liuyao_checkpoints
BEGIN
    SELECT RAISE(ABORT, 'saved liuyao progress is immutable');
END;

CREATE TRIGGER enforce_liuyao_checkpoint_sequence
BEFORE INSERT ON liuyao_checkpoints
WHEN NEW.event_count != COALESCE((SELECT MAX(event_count) + 1 FROM liuyao_checkpoints WHERE question_id = NEW.question_id), 0)
    OR EXISTS(SELECT 1 FROM random_artifacts WHERE question_id = NEW.question_id AND method = 'liuyao')
BEGIN
    SELECT RAISE(ABORT, 'liuyao progress must append before completion');
END;

CREATE TRIGGER enforce_liuyao_checkpoint_finalization
BEFORE INSERT ON random_artifacts
WHEN NEW.method = 'liuyao'
    AND EXISTS(SELECT 1 FROM liuyao_checkpoints WHERE question_id = NEW.question_id)
    AND NOT EXISTS(SELECT 1 FROM liuyao_checkpoints WHERE question_id = NEW.question_id AND event_count = 6 AND transcript_json = NEW.entropy_events_json)
BEGIN
    SELECT RAISE(ABORT, 'completed liuyao must match saved progress');
END;
