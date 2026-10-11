-- Runtime engine results and rule hits are append-only audit material. Full
-- semantic re-evaluation is performed in Rust before fusion; these triggers
-- provide an additional structural and cross-table boundary.

CREATE TRIGGER IF NOT EXISTS validate_engine_run_insert
BEFORE INSERT ON engine_runs
WHEN length(trim(NEW.implementation_version)) = 0
  OR length(NEW.implementation_version) > 128
  OR NEW.implementation_version GLOB '*[^a-z0-9._-]*'
  OR length(trim(NEW.started_at)) = 0
  OR NEW.completed_at IS NULL
  OR NEW.method NOT IN (
      'yarrow_stalk', 'chinese_astrology', 'taiyi', 'qimen_dunjia',
      'liuyao', 'meihua_yishu', 'four_pillars', 'ziwei_doushu', 'qizheng_siyu'
  )
  OR NEW.applicability_status NOT IN (
      'applicable', 'insufficient_evidence', 'missing_parameters',
      'unsupported_question_type', 'not_implemented'
  )
  OR CASE WHEN json_valid(NEW.applicability_reasons_json)
          THEN json_type(NEW.applicability_reasons_json) <> 'array'
          ELSE 1 END
  OR (
      NEW.chart_json IS NOT NULL
      AND CASE WHEN json_valid(NEW.chart_json)
               THEN json_type(NEW.chart_json) <> 'object'
               ELSE 1 END
  )
  OR (
      NEW.applicability_status = 'applicable'
      AND (
          NEW.chart_json IS NULL
          OR NEW.signals_json IS NULL
          OR NEW.evidence_tier IS NULL
          OR CASE WHEN json_valid(NEW.chart_json)
                  THEN json_type(NEW.chart_json) <> 'object'
                  ELSE 1 END
          OR CASE WHEN json_valid(NEW.signals_json)
                  THEN json_type(NEW.signals_json) <> 'object'
                  ELSE 1 END
          OR NEW.evidence_tier NOT IN (
              'early_core_classic', 'early_authoritative_commentary',
              'later_authoritative_commentary'
          )
      )
  )
  OR (
      NEW.applicability_status <> 'applicable'
      AND (NEW.signals_json IS NOT NULL OR NEW.evidence_tier IS NOT NULL)
  )
BEGIN
    SELECT RAISE(ABORT, 'invalid engine run audit structure');
END;

CREATE TRIGGER IF NOT EXISTS validate_rule_hit_insert
BEFORE INSERT ON rule_hits
WHEN NOT EXISTS (
        SELECT 1
        FROM engine_runs e
        JOIN rules r ON r.rule_id = NEW.rule_id
        WHERE e.id = NEW.engine_run_id
          AND e.method = r.method
          AND e.applicability_status = 'applicable'
          AND e.completed_at IS NOT NULL
    )
  OR CASE WHEN json_valid(NEW.input_snapshot_json)
          THEN json_type(NEW.input_snapshot_json) <> 'object'
          ELSE 1 END
  OR CASE WHEN json_valid(NEW.output_snapshot_json)
          THEN json_type(NEW.output_snapshot_json) <> 'object'
               OR COALESCE(json_type(NEW.output_snapshot_json, '$.matched'), '') <> 'true'
               OR COALESCE(json_type(NEW.output_snapshot_json, '$.signals'), '') <> 'object'
          ELSE 1 END
  OR CASE WHEN json_valid(NEW.evaluation_trace_json)
          THEN json_type(NEW.evaluation_trace_json) <> 'array'
          ELSE 1 END
BEGIN
    SELECT RAISE(ABORT, 'invalid rule hit audit structure');
END;

CREATE TRIGGER IF NOT EXISTS prevent_engine_run_update
BEFORE UPDATE ON engine_runs
BEGIN
    SELECT RAISE(ABORT, 'engine run is immutable');
END;

CREATE TRIGGER IF NOT EXISTS prevent_engine_run_delete
BEFORE DELETE ON engine_runs
BEGIN
    SELECT RAISE(ABORT, 'engine run is immutable');
END;

CREATE TRIGGER IF NOT EXISTS prevent_rule_hit_update
BEFORE UPDATE ON rule_hits
BEGIN
    SELECT RAISE(ABORT, 'rule hit is immutable');
END;

CREATE TRIGGER IF NOT EXISTS prevent_rule_hit_delete
BEFORE DELETE ON rule_hits
BEGIN
    SELECT RAISE(ABORT, 'rule hit is immutable');
END;
