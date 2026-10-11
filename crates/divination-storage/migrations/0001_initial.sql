PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS locations (
    id INTEGER PRIMARY KEY,
    administrative_code TEXT NOT NULL UNIQUE,
    province TEXT NOT NULL,
    city TEXT NOT NULL,
    district TEXT,
    street TEXT,
    longitude_deg REAL NOT NULL,
    latitude_deg REAL NOT NULL,
    coordinate_precision TEXT NOT NULL CHECK (coordinate_precision IN ('city_center', 'district_center', 'street_center')),
    source TEXT NOT NULL,
    dataset_version TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1))
);

CREATE TABLE IF NOT EXISTS rules (
    rule_id TEXT PRIMARY KEY,
    method TEXT NOT NULL,
    applicable_question_types_json TEXT NOT NULL,
    condition_expression TEXT NOT NULL,
    result_expression TEXT NOT NULL,
    timing_expression TEXT,
    priority INTEGER NOT NULL,
    evidence_tier TEXT NOT NULL,
    historical_period TEXT NOT NULL,
    book_title TEXT NOT NULL,
    volume TEXT NOT NULL,
    chapter TEXT NOT NULL,
    original_text TEXT NOT NULL,
    edition TEXT NOT NULL,
    source_reference TEXT NOT NULL,
    textual_variants TEXT NOT NULL,
    stability TEXT NOT NULL,
    implementation_version TEXT NOT NULL,
    reviewed INTEGER NOT NULL DEFAULT 0 CHECK (reviewed IN (0, 1)),
    content_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS questions (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL,
    subcategory TEXT NOT NULL,
    ancient_question_type TEXT NOT NULL,
    question_text TEXT NOT NULL,
    subject_text TEXT NOT NULL,
    object_text TEXT NOT NULL,
    target_event TEXT NOT NULL,
    options_json TEXT NOT NULL,
    parameters_json TEXT NOT NULL,
    location_json TEXT NOT NULL,
    divination_time_bjt TEXT NOT NULL,
    target_time_text TEXT NOT NULL,
    locked_at_bjt TEXT NOT NULL,
    event_fingerprint_version INTEGER NOT NULL,
    event_fingerprint TEXT NOT NULL,
    normalized_terms_json TEXT NOT NULL,
    locked INTEGER NOT NULL DEFAULT 1 CHECK (locked IN (0, 1)),
    state TEXT NOT NULL CHECK (state IN ('locked', 'completed', 'not_executable', 'failed')),
    final_conclusion TEXT,
    final_timing TEXT,
    final_evidence_tier TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_questions_event_fingerprint
    ON questions(event_fingerprint, created_at DESC);

CREATE TRIGGER IF NOT EXISTS prevent_locked_question_mutation
BEFORE UPDATE OF category, subcategory, ancient_question_type, question_text,
    subject_text, object_text, target_event, options_json, parameters_json,
    location_json, divination_time_bjt, target_time_text, event_fingerprint
ON questions
WHEN OLD.locked = 1
BEGIN
    SELECT RAISE(ABORT, 'locked question is immutable');
END;

CREATE TABLE IF NOT EXISTS event_revisions (
    id TEXT PRIMARY KEY,
    prior_question_id TEXT NOT NULL REFERENCES questions(id),
    new_question_id TEXT NOT NULL UNIQUE REFERENCES questions(id),
    change_description TEXT NOT NULL,
    created_at_bjt TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS engine_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id TEXT NOT NULL REFERENCES questions(id),
    method TEXT NOT NULL,
    implementation_version TEXT NOT NULL,
    applicability_status TEXT NOT NULL,
    applicability_reasons_json TEXT NOT NULL,
    chart_json TEXT,
    signals_json TEXT,
    evidence_tier TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    UNIQUE(question_id, method)
);

CREATE TABLE IF NOT EXISTS rule_hits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    engine_run_id INTEGER NOT NULL REFERENCES engine_runs(id),
    rule_id TEXT NOT NULL REFERENCES rules(rule_id),
    input_snapshot_json TEXT NOT NULL,
    output_snapshot_json TEXT NOT NULL,
    evaluation_trace_json TEXT NOT NULL,
    UNIQUE(engine_run_id, rule_id)
);

CREATE TABLE IF NOT EXISTS random_artifacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id TEXT NOT NULL REFERENCES questions(id),
    method TEXT NOT NULL,
    algorithm TEXT NOT NULL,
    algorithm_version TEXT NOT NULL,
    seed_hex TEXT NOT NULL,
    entropy_events_json TEXT NOT NULL,
    generated_values_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(question_id, method)
);

CREATE TABLE IF NOT EXISTS decision_traces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id TEXT NOT NULL REFERENCES questions(id),
    stage TEXT NOT NULL,
    sequence_no INTEGER NOT NULL,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(question_id, stage, sequence_no)
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

