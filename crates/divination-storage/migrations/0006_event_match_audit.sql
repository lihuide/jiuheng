ALTER TABLE event_revisions
    ADD COLUMN match_algorithm_version TEXT;

ALTER TABLE event_revisions
    ADD COLUMN match_exact_digest INTEGER
        CHECK (match_exact_digest IS NULL OR match_exact_digest IN (0, 1));

ALTER TABLE event_revisions
    ADD COLUMN match_score_percent INTEGER
        CHECK (match_score_percent IS NULL OR match_score_percent BETWEEN 0 AND 100);

ALTER TABLE event_revisions
    ADD COLUMN match_reasons_json TEXT;

DROP TRIGGER IF EXISTS prevent_locked_question_mutation;

CREATE TRIGGER prevent_locked_question_mutation
BEFORE UPDATE OF category, subcategory, ancient_question_type, question_text,
    subject_text, object_text, target_event, options_json, parameters_json,
    location_json, divination_time_bjt, target_time_text, locked_at_bjt,
    event_fingerprint_version, event_fingerprint, normalized_terms_json
ON questions
WHEN OLD.locked = 1
BEGIN
    SELECT RAISE(ABORT, 'locked question is immutable');
END;
