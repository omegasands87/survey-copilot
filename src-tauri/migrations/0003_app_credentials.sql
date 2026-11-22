-- Migration 0003: Opaque credential reference metadata table
-- Stores only non-sensitive metadata and opaque references to Windows Credential Manager.
-- Actual secrets/API keys are NEVER stored in SQLite.

CREATE TABLE IF NOT EXISTS app_credentials (
  id TEXT PRIMARY KEY,
  provider_type TEXT NOT NULL CHECK(provider_type IN ('gemini', 'openai_compatible', 'anthropic_compatible', 'custom_http')),
  label TEXT NOT NULL CHECK(length(label) BETWEEN 1 AND 100),
  credential_ref TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL DEFAULT 'configured' CHECK(status IN ('configured', 'unconfigured', 'needs_update', 'inaccessible')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_credentials_ref ON app_credentials(credential_ref);
CREATE INDEX IF NOT EXISTS idx_credentials_provider ON app_credentials(provider_type);
