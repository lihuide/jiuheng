CREATE TRIGGER IF NOT EXISTS enforce_released_rule_hit_insert
BEFORE INSERT ON rule_hits
WHEN NOT EXISTS (
    SELECT 1
    FROM rule_release_readiness gate
    WHERE gate.rule_id = NEW.rule_id
      AND gate.rule_fields_ready = 1
      AND gate.primary_source_ready = 1
      AND gate.reviews_ready = 1
      AND gate.tests_ready = 1
)
BEGIN
    SELECT RAISE(ABORT, 'rule evidence gate rejected rule hit');
END;

CREATE TRIGGER IF NOT EXISTS enforce_released_rule_hit_update
BEFORE UPDATE OF rule_id ON rule_hits
WHEN NOT EXISTS (
    SELECT 1
    FROM rule_release_readiness gate
    WHERE gate.rule_id = NEW.rule_id
      AND gate.rule_fields_ready = 1
      AND gate.primary_source_ready = 1
      AND gate.reviews_ready = 1
      AND gate.tests_ready = 1
)
BEGIN
    SELECT RAISE(ABORT, 'rule evidence gate rejected rule hit');
END;
