-- Migration 0004: Provider configuration and health indexes
-- Adds index on credential_ref for fast reference lookups
-- Ensures provider_configs and provider_health queries are optimized

CREATE INDEX IF NOT EXISTS idx_provider_configs_cred_ref ON provider_configs(credential_ref);
CREATE INDEX IF NOT EXISTS idx_provider_health_state ON provider_health(state, cooldown_until);
