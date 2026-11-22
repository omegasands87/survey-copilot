-- Migration 0002: Typed facts, fact history, conflict detection, and extended source metadata

-- 1. Extend profile_facts with fact_key, data_type, and unit
ALTER TABLE profile_facts ADD COLUMN fact_key TEXT;
ALTER TABLE profile_facts ADD COLUMN data_type TEXT NOT NULL DEFAULT 'string';
ALTER TABLE profile_facts ADD COLUMN unit TEXT;

-- 2. Populate fact_key from existing attribute for backward compatibility
UPDATE profile_facts SET fact_key = lower(replace(attribute, ' ', '_')) WHERE fact_key IS NULL;

-- 3. Extend fact_sources with title, valid_from, valid_until, and notes
ALTER TABLE fact_sources ADD COLUMN title TEXT;
ALTER TABLE fact_sources ADD COLUMN valid_from TEXT;
ALTER TABLE fact_sources ADD COLUMN valid_until TEXT;
ALTER TABLE fact_sources ADD COLUMN notes TEXT;

-- 4. Create fact_history table for auditing changes and value history
CREATE TABLE IF NOT EXISTS fact_history (
  id TEXT PRIMARY KEY,
  persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
  fact_id TEXT NOT NULL REFERENCES profile_facts(id) ON DELETE CASCADE,
  previous_value_json TEXT,
  new_value_json TEXT NOT NULL,
  previous_status TEXT,
  new_status TEXT NOT NULL,
  change_reason TEXT,
  source_id TEXT REFERENCES fact_sources(id) ON DELETE SET NULL,
  changed_at TEXT NOT NULL
);

-- 5. Indexes for fast lookup, history retrieval, and conflict detection
CREATE INDEX IF NOT EXISTS idx_fact_history_fact ON fact_history(fact_id, changed_at DESC);
CREATE INDEX IF NOT EXISTS idx_fact_history_persona ON fact_history(persona_id, changed_at DESC);
CREATE INDEX IF NOT EXISTS idx_facts_persona_key ON profile_facts(persona_id, fact_key);
