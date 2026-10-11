-- A checksum declaration is not proof that the corresponding facsimile bytes
-- were ever read.  Version 12 requires a locally computed, byte-length-bound
-- verification record that matches the source edition's canonical SHA-256.
CREATE TABLE IF NOT EXISTS source_facsimile_verifications (
    source_id TEXT PRIMARY KEY REFERENCES source_editions(source_id) ON DELETE CASCADE,
    digital_checksum TEXT NOT NULL,
    byte_length INTEGER NOT NULL CHECK (byte_length > 0),
    file_name TEXT NOT NULL,
    verifier_version TEXT NOT NULL,
    verified_at TEXT NOT NULL,
    CHECK (length(digital_checksum) = 71),
    CHECK (substr(digital_checksum, 1, 7) = 'sha256:'),
    CHECK (substr(digital_checksum, 8) NOT GLOB '*[^0-9a-f]*'),
    CHECK (length(trim(file_name)) > 0),
    CHECK (length(trim(verifier_version)) > 0),
    CHECK (length(trim(verified_at)) > 0)
);

CREATE TRIGGER IF NOT EXISTS invalidate_evidence_after_facsimile_verification_insert
AFTER INSERT ON source_facsimile_verifications
BEGIN
    UPDATE rule_evidence_state
    SET evidence_bundle_hash = '', updated_at = CURRENT_TIMESTAMP
    WHERE rule_id IN (
        SELECT l.rule_id
        FROM rule_source_links l
        JOIN source_passages p ON p.passage_id = l.passage_id
        WHERE p.source_id = NEW.source_id
    );
END;

CREATE TRIGGER IF NOT EXISTS invalidate_evidence_after_facsimile_verification_update
AFTER UPDATE ON source_facsimile_verifications
BEGIN
    UPDATE rule_evidence_state
    SET evidence_bundle_hash = '', updated_at = CURRENT_TIMESTAMP
    WHERE rule_id IN (
        SELECT l.rule_id
        FROM rule_source_links l
        JOIN source_passages p ON p.passage_id = l.passage_id
        WHERE p.source_id IN (OLD.source_id, NEW.source_id)
    );
END;

CREATE TRIGGER IF NOT EXISTS invalidate_evidence_after_facsimile_verification_delete
AFTER DELETE ON source_facsimile_verifications
BEGIN
    UPDATE rule_evidence_state
    SET evidence_bundle_hash = '', updated_at = CURRENT_TIMESTAMP
    WHERE rule_id IN (
        SELECT l.rule_id
        FROM rule_source_links l
        JOIN source_passages p ON p.passage_id = l.passage_id
        WHERE p.source_id = OLD.source_id
    );
END;

-- Any rule reviewed before actual byte verification must be explicitly
-- re-imported and re-reviewed against the stronger evidence bundle.
UPDATE rules
SET reviewed = 0,
    updated_at = CURRENT_TIMESTAMP;

DROP VIEW IF EXISTS rule_release_readiness;

CREATE VIEW rule_release_readiness AS
SELECT
    r.rule_id,
    r.content_hash AS rule_content_hash,
    CASE WHEN r.reviewed = 1
              AND r.stability = 'stable'
              AND r.evidence_tier <> 'unverified'
              AND length(trim(r.original_text)) > 0
              AND length(trim(r.source_reference)) > 0
              AND length(r.condition_expression) <= 32768
              AND CASE WHEN json_valid(r.condition_expression)
                       THEN json_extract(r.condition_expression, '$.schema_version') = 1
                            AND json_type(r.condition_expression, '$.condition') = 'object'
                       ELSE 0 END
              AND length(r.result_expression) <= 32768
              AND CASE WHEN json_valid(r.result_expression)
                       THEN json_extract(r.result_expression, '$.schema_version') = 1
                            AND json_type(r.result_expression, '$.signals') = 'object'
                       ELSE 0 END
              AND (
                  r.timing_expression IS NULL
                  OR (
                      length(r.timing_expression) <= 32768
                      AND CASE WHEN json_valid(r.timing_expression)
                               THEN json_extract(r.timing_expression, '$.schema_version') = 1
                                    AND json_type(r.timing_expression, '$.window') = 'text'
                               ELSE 0 END
                  )
              )
              AND (
                  (r.action_key IS NULL
                   AND r.action_polarity IS NULL
                   AND r.action_expression IS NULL)
                  OR
                  (length(trim(r.action_key)) > 0
                   AND r.action_key NOT GLOB '*[^a-z0-9._-]*'
                   AND substr(r.action_key, 1, 1) GLOB '[a-z0-9]'
                   AND substr(r.action_key, -1, 1) GLOB '[a-z0-9]'
                   AND r.action_polarity IN ('do', 'avoid')
                   AND length(trim(r.action_expression)) > 0)
              )
         THEN 1 ELSE 0 END AS rule_fields_ready,
    CASE WHEN EXISTS (
        SELECT 1
        FROM rule_source_links l
        JOIN source_passages p ON p.passage_id = l.passage_id
        JOIN source_editions e ON e.source_id = p.source_id
        JOIN source_facsimile_verifications v ON v.source_id = e.source_id
        JOIN rule_evidence_state s ON s.rule_id = l.rule_id
        WHERE l.rule_id = r.rule_id
          AND l.relation = 'primary'
          AND p.transcription_status = 'verified'
          AND length(trim(p.folio_or_page)) > 0
          AND length(trim(p.original_text)) > 0
          AND length(s.evidence_bundle_hash) > 0
          AND e.edition_type IN ('facsimile', 'collated', 'critical')
          AND length(trim(e.catalog_identifier)) > 0
          AND length(trim(e.facsimile_reference)) > 0
          AND length(e.digital_checksum) = 71
          AND substr(e.digital_checksum, 1, 7) = 'sha256:'
          AND substr(e.digital_checksum, 8) NOT GLOB '*[^0-9a-f]*'
          AND v.digital_checksum = e.digital_checksum
          AND v.byte_length > 0
          AND v.verifier_version = 'jiuheng.sha256.v1'
          AND length(trim(v.file_name)) > 0
          AND length(trim(v.verified_at)) > 0
    ) THEN 1 ELSE 0 END AS primary_source_ready,
    CASE WHEN
        EXISTS (SELECT 1 FROM rule_reviews v JOIN rule_evidence_state s ON s.rule_id = v.rule_id WHERE v.rule_id = r.rule_id AND v.rule_content_hash = r.content_hash AND v.evidence_bundle_hash = s.evidence_bundle_hash AND length(s.evidence_bundle_hash) > 0 AND v.review_kind = 'bibliography' AND v.decision = 'approved')
        AND EXISTS (SELECT 1 FROM rule_reviews v JOIN rule_evidence_state s ON s.rule_id = v.rule_id WHERE v.rule_id = r.rule_id AND v.rule_content_hash = r.content_hash AND v.evidence_bundle_hash = s.evidence_bundle_hash AND length(s.evidence_bundle_hash) > 0 AND v.review_kind = 'transcription' AND v.decision = 'approved')
        AND EXISTS (SELECT 1 FROM rule_reviews v JOIN rule_evidence_state s ON s.rule_id = v.rule_id WHERE v.rule_id = r.rule_id AND v.rule_content_hash = r.content_hash AND v.evidence_bundle_hash = s.evidence_bundle_hash AND length(s.evidence_bundle_hash) > 0 AND v.review_kind = 'logic' AND v.decision = 'approved')
        AND EXISTS (SELECT 1 FROM rule_reviews v JOIN rule_evidence_state s ON s.rule_id = v.rule_id WHERE v.rule_id = r.rule_id AND v.rule_content_hash = r.content_hash AND v.evidence_bundle_hash = s.evidence_bundle_hash AND v.review_kind = 'implementation' AND v.decision = 'approved')
        AND NOT EXISTS (SELECT 1 FROM rule_reviews v JOIN rule_evidence_state s ON s.rule_id = v.rule_id WHERE v.rule_id = r.rule_id AND v.rule_content_hash = r.content_hash AND v.evidence_bundle_hash = s.evidence_bundle_hash AND v.decision = 'rejected')
    THEN 1 ELSE 0 END AS reviews_ready,
    CASE WHEN
        EXISTS (SELECT 1 FROM rule_test_cases c WHERE c.rule_id = r.rule_id AND c.enabled = 1)
        AND EXISTS (
            SELECT 1 FROM rule_test_suite_state s
            WHERE s.rule_id = r.rule_id
              AND length(s.test_suite_hash) > 0
              AND s.enabled_case_count = (SELECT COUNT(*) FROM rule_test_cases c WHERE c.rule_id = r.rule_id AND c.enabled = 1)
        )
        AND EXISTS (
            SELECT 1 FROM rule_test_runs t
            JOIN rule_test_suite_state s ON s.rule_id = t.rule_id
            WHERE t.rule_id = r.rule_id
              AND t.rule_content_hash = r.content_hash
              AND t.implementation_version = r.implementation_version
              AND t.test_suite_hash = s.test_suite_hash
              AND t.failed_count = 0
              AND t.total_count = s.enabled_case_count
              AND t.executed_at = (
                  SELECT MAX(t2.executed_at) FROM rule_test_runs t2
                  WHERE t2.rule_id = r.rule_id
                    AND t2.rule_content_hash = r.content_hash
                    AND t2.implementation_version = r.implementation_version
              )
        )
    THEN 1 ELSE 0 END AS tests_ready
FROM rules r;
