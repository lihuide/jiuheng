CREATE TABLE IF NOT EXISTS source_editions (
    source_id TEXT PRIMARY KEY,
    canonical_title TEXT NOT NULL,
    attributed_author TEXT NOT NULL,
    historical_period TEXT NOT NULL,
    edition_title TEXT NOT NULL,
    edition_type TEXT NOT NULL CHECK (edition_type IN ('facsimile', 'collated', 'critical', 'transcription')),
    edition_year TEXT NOT NULL,
    holding_institution TEXT NOT NULL,
    catalog_identifier TEXT NOT NULL,
    facsimile_reference TEXT NOT NULL,
    digital_checksum TEXT NOT NULL,
    notes TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS source_passages (
    passage_id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES source_editions(source_id),
    volume TEXT NOT NULL,
    chapter TEXT NOT NULL,
    folio_or_page TEXT NOT NULL,
    original_text TEXT NOT NULL,
    normalized_text TEXT NOT NULL,
    textual_variants TEXT NOT NULL,
    transcription_status TEXT NOT NULL CHECK (transcription_status IN ('unreviewed', 'verified', 'disputed')),
    content_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS rule_source_links (
    rule_id TEXT NOT NULL REFERENCES rules(rule_id),
    passage_id TEXT NOT NULL REFERENCES source_passages(passage_id),
    relation TEXT NOT NULL CHECK (relation IN ('primary', 'commentary', 'variant', 'counterevidence')),
    notes TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY(rule_id, passage_id, relation)
);

CREATE TABLE IF NOT EXISTS rule_evidence_state (
    rule_id TEXT PRIMARY KEY REFERENCES rules(rule_id),
    evidence_bundle_hash TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS rule_reviews (
    review_id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES rules(rule_id),
    review_kind TEXT NOT NULL CHECK (review_kind IN ('bibliography', 'transcription', 'logic', 'implementation')),
    decision TEXT NOT NULL CHECK (decision IN ('approved', 'rejected', 'needs_work')),
    reviewer TEXT NOT NULL,
    evidence_notes TEXT NOT NULL,
    rule_content_hash TEXT NOT NULL,
    evidence_bundle_hash TEXT NOT NULL,
    reviewed_at TEXT NOT NULL,
    UNIQUE(rule_id, review_kind, reviewer, rule_content_hash, evidence_bundle_hash)
);

CREATE TABLE IF NOT EXISTS rule_test_cases (
    test_case_id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES rules(rule_id),
    name TEXT NOT NULL,
    input_json TEXT NOT NULL,
    expected_output_json TEXT NOT NULL,
    boundary_class TEXT NOT NULL,
    source_basis TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS rule_test_runs (
    run_id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES rules(rule_id),
    rule_content_hash TEXT NOT NULL,
    implementation_version TEXT NOT NULL,
    test_suite_hash TEXT NOT NULL,
    passed_count INTEGER NOT NULL CHECK (passed_count >= 0),
    failed_count INTEGER NOT NULL CHECK (failed_count >= 0),
    total_count INTEGER NOT NULL CHECK (total_count >= 0),
    environment_json TEXT NOT NULL,
    executed_at TEXT NOT NULL,
    CHECK (passed_count + failed_count = total_count)
);

CREATE TABLE IF NOT EXISTS rule_test_suite_state (
    rule_id TEXT PRIMARY KEY REFERENCES rules(rule_id),
    test_suite_hash TEXT NOT NULL,
    enabled_case_count INTEGER NOT NULL CHECK (enabled_case_count >= 0),
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_source_passages_source ON source_passages(source_id);
CREATE INDEX IF NOT EXISTS idx_rule_source_links_rule ON rule_source_links(rule_id);
CREATE INDEX IF NOT EXISTS idx_rule_reviews_gate ON rule_reviews(rule_id, rule_content_hash, evidence_bundle_hash, review_kind, decision);
CREATE INDEX IF NOT EXISTS idx_rule_test_cases_rule ON rule_test_cases(rule_id, enabled);
CREATE INDEX IF NOT EXISTS idx_rule_test_runs_rule ON rule_test_runs(rule_id, rule_content_hash, executed_at DESC);

CREATE TRIGGER IF NOT EXISTS invalidate_test_suite_after_insert
AFTER INSERT ON rule_test_cases
BEGIN
    INSERT INTO rule_test_suite_state(rule_id, test_suite_hash, enabled_case_count, updated_at)
    VALUES(NEW.rule_id, '', (SELECT COUNT(*) FROM rule_test_cases WHERE rule_id = NEW.rule_id AND enabled = 1), CURRENT_TIMESTAMP)
    ON CONFLICT(rule_id) DO UPDATE SET
        test_suite_hash = '',
        enabled_case_count = excluded.enabled_case_count,
        updated_at = excluded.updated_at;
END;

CREATE TRIGGER IF NOT EXISTS invalidate_test_suite_after_update
AFTER UPDATE ON rule_test_cases
BEGIN
    INSERT INTO rule_test_suite_state(rule_id, test_suite_hash, enabled_case_count, updated_at)
    VALUES(OLD.rule_id, '', (SELECT COUNT(*) FROM rule_test_cases WHERE rule_id = OLD.rule_id AND enabled = 1), CURRENT_TIMESTAMP)
    ON CONFLICT(rule_id) DO UPDATE SET
        test_suite_hash = '',
        enabled_case_count = excluded.enabled_case_count,
        updated_at = excluded.updated_at;
    INSERT INTO rule_test_suite_state(rule_id, test_suite_hash, enabled_case_count, updated_at)
    VALUES(NEW.rule_id, '', (SELECT COUNT(*) FROM rule_test_cases WHERE rule_id = NEW.rule_id AND enabled = 1), CURRENT_TIMESTAMP)
    ON CONFLICT(rule_id) DO UPDATE SET
        test_suite_hash = '',
        enabled_case_count = excluded.enabled_case_count,
        updated_at = excluded.updated_at;
END;

CREATE TRIGGER IF NOT EXISTS invalidate_test_suite_after_delete
AFTER DELETE ON rule_test_cases
BEGIN
    INSERT INTO rule_test_suite_state(rule_id, test_suite_hash, enabled_case_count, updated_at)
    VALUES(OLD.rule_id, '', (SELECT COUNT(*) FROM rule_test_cases WHERE rule_id = OLD.rule_id AND enabled = 1), CURRENT_TIMESTAMP)
    ON CONFLICT(rule_id) DO UPDATE SET
        test_suite_hash = '',
        enabled_case_count = excluded.enabled_case_count,
        updated_at = excluded.updated_at;
END;

CREATE TRIGGER IF NOT EXISTS invalidate_evidence_after_link_insert
AFTER INSERT ON rule_source_links
BEGIN
    INSERT INTO rule_evidence_state(rule_id, evidence_bundle_hash, updated_at)
    VALUES(NEW.rule_id, '', CURRENT_TIMESTAMP)
    ON CONFLICT(rule_id) DO UPDATE SET
        evidence_bundle_hash = '',
        updated_at = excluded.updated_at;
END;

CREATE TRIGGER IF NOT EXISTS invalidate_evidence_after_link_update
AFTER UPDATE ON rule_source_links
BEGIN
    INSERT INTO rule_evidence_state(rule_id, evidence_bundle_hash, updated_at)
    VALUES(OLD.rule_id, '', CURRENT_TIMESTAMP)
    ON CONFLICT(rule_id) DO UPDATE SET
        evidence_bundle_hash = '',
        updated_at = excluded.updated_at;
    INSERT INTO rule_evidence_state(rule_id, evidence_bundle_hash, updated_at)
    VALUES(NEW.rule_id, '', CURRENT_TIMESTAMP)
    ON CONFLICT(rule_id) DO UPDATE SET
        evidence_bundle_hash = '',
        updated_at = excluded.updated_at;
END;

CREATE TRIGGER IF NOT EXISTS invalidate_evidence_after_link_delete
AFTER DELETE ON rule_source_links
BEGIN
    INSERT INTO rule_evidence_state(rule_id, evidence_bundle_hash, updated_at)
    VALUES(OLD.rule_id, '', CURRENT_TIMESTAMP)
    ON CONFLICT(rule_id) DO UPDATE SET
        evidence_bundle_hash = '',
        updated_at = excluded.updated_at;
END;

CREATE TRIGGER IF NOT EXISTS invalidate_evidence_after_passage_update
AFTER UPDATE ON source_passages
BEGIN
    UPDATE rule_evidence_state
    SET evidence_bundle_hash = '', updated_at = CURRENT_TIMESTAMP
    WHERE rule_id IN (
        SELECT rule_id FROM rule_source_links WHERE passage_id = NEW.passage_id
    );
END;

CREATE TRIGGER IF NOT EXISTS invalidate_evidence_after_edition_update
AFTER UPDATE ON source_editions
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

CREATE VIEW IF NOT EXISTS rule_release_readiness AS
SELECT
    r.rule_id,
    r.content_hash AS rule_content_hash,
    CASE WHEN r.reviewed = 1
              AND r.stability = 'stable'
              AND r.evidence_tier <> 'unverified'
              AND length(trim(r.original_text)) > 0
              AND length(trim(r.source_reference)) > 0
         THEN 1 ELSE 0 END AS rule_fields_ready,
    CASE WHEN EXISTS (
        SELECT 1
        FROM rule_source_links l
        JOIN source_passages p ON p.passage_id = l.passage_id
        JOIN source_editions e ON e.source_id = p.source_id
        JOIN rule_evidence_state s ON s.rule_id = l.rule_id
        WHERE l.rule_id = r.rule_id
          AND l.relation = 'primary'
          AND p.transcription_status = 'verified'
          AND length(s.evidence_bundle_hash) > 0
          AND length(trim(e.catalog_identifier)) > 0
          AND length(trim(e.facsimile_reference)) > 0
          AND length(trim(e.digital_checksum)) > 0
    ) THEN 1 ELSE 0 END AS primary_source_ready,
    CASE WHEN
        EXISTS (SELECT 1 FROM rule_reviews v JOIN rule_evidence_state s ON s.rule_id = v.rule_id WHERE v.rule_id = r.rule_id AND v.rule_content_hash = r.content_hash AND v.evidence_bundle_hash = s.evidence_bundle_hash AND length(s.evidence_bundle_hash) > 0 AND v.review_kind = 'bibliography' AND v.decision = 'approved')
        AND EXISTS (SELECT 1 FROM rule_reviews v JOIN rule_evidence_state s ON s.rule_id = v.rule_id WHERE v.rule_id = r.rule_id AND v.rule_content_hash = r.content_hash AND v.evidence_bundle_hash = s.evidence_bundle_hash AND length(s.evidence_bundle_hash) > 0 AND v.review_kind = 'transcription' AND v.decision = 'approved')
        AND EXISTS (SELECT 1 FROM rule_reviews v JOIN rule_evidence_state s ON s.rule_id = v.rule_id WHERE v.rule_id = r.rule_id AND v.rule_content_hash = r.content_hash AND v.evidence_bundle_hash = s.evidence_bundle_hash AND length(s.evidence_bundle_hash) > 0 AND v.review_kind = 'logic' AND v.decision = 'approved')
        AND EXISTS (SELECT 1 FROM rule_reviews v JOIN rule_evidence_state s ON s.rule_id = v.rule_id WHERE v.rule_id = r.rule_id AND v.rule_content_hash = r.content_hash AND v.evidence_bundle_hash = s.evidence_bundle_hash AND length(s.evidence_bundle_hash) > 0 AND v.review_kind = 'implementation' AND v.decision = 'approved')
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
