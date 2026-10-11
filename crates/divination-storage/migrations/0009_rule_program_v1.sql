-- Free-form expressions are no longer executable. Existing locally-authored
-- rules must be re-imported and re-reviewed with the versioned JSON programs.
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
