-- Migration 0013: Privacy Settings and Diagnostics Logging
-- Creates app_diagnostic_logs table for audit trail with pre-redacted messages
-- Seeds default privacy settings into app_settings table

CREATE TABLE IF NOT EXISTS app_diagnostic_logs (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    level TEXT NOT NULL,
    category TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata_json TEXT
);

CREATE INDEX IF NOT EXISTS idx_diag_logs_ts ON app_diagnostic_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_diag_logs_category ON app_diagnostic_logs(category);

-- Ensure default privacy settings exist
INSERT OR IGNORE INTO app_settings (key, value_json, updated_at)
VALUES (
    'privacy_settings',
    '{"autoRedactLogs":true,"requireSensitiveConsentPrompt":true,"requireVisionConsentPrompt":true,"screenshotRetentionPolicy":"immediate_delete","autoCleanupTempOnExit":true,"logLevel":"INFO","updatedAt":"2026-01-01T00:00:00Z"}',
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
);
