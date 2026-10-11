ALTER TABLE locations ADD COLUMN code_system TEXT NOT NULL DEFAULT 'legacy_seed';
ALTER TABLE locations ADD COLUMN source_record_id TEXT NOT NULL DEFAULT '';
ALTER TABLE locations ADD COLUMN source_url TEXT NOT NULL DEFAULT '';
ALTER TABLE locations ADD COLUMN source_license TEXT NOT NULL DEFAULT '';
ALTER TABLE locations ADD COLUMN source_modified_date TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_locations_hierarchy
    ON locations(province, city, district, street);
