# Data Export and Deletion Specification

**Status:** Normative specification for SC-012 (v1.0).

---

## 1. Data Export Specification

### 1.1 Scope
Survey Copilot supports two export scopes:
1. **Full Application Export (`full_application`)**: Exports all user-owned data across all personas, sessions, and history.
2. **Single Persona Export (`single_persona`)**: Strictly exports only the selected persona and its owned records. No data from other personas is included.

### 1.2 Schema Versioning
- **Schema URL**: `https://survey-copilot.app/schemas/export-v1.json`
- **Current Schema Version**: `1.0.0`
- **Application Version**: `0.1.0` (or active release version)

### 1.3 JSON Document Structure
```json
{
  "$schema": "https://survey-copilot.app/schemas/export-v1.json",
  "schemaVersion": "1.0.0",
  "appVersion": "0.1.0",
  "exportTimestamp": "2026-11-23T01:50:00.000Z",
  "exportScope": "full_application",
  "targetPersonaId": null,
  "targetPersonaName": null,
  "summary": {
    "exportScope": "full_application",
    "targetPersonaId": null,
    "targetPersonaName": null,
    "totalPersonas": 2,
    "totalFacts": 10,
    "totalSources": 8,
    "totalHistoryEntries": 4,
    "totalPreferences": 2,
    "totalSessions": 3,
    "totalQuestions": 12,
    "totalRecommendations": 5,
    "totalAnswers": 5,
    "totalFitResults": 2,
    "totalAttempts": 1,
    "sensitiveFactsCount": 2,
    "highlySensitiveFactsCount": 1,
    "hasSensitiveData": true,
    "estimatedRecordsCount": 47,
    "warningMessage": "Perhatian Privasi: Berkas ekspor memuat 2 fakta sensitif..."
  },
  "data": {
    "personas": [...],
    "facts": [...],
    "factSources": [...],
    "factHistory": [...],
    "personaPreferences": [...],
    "surveySessions": [...],
    "screenCaptures": [...],
    "surveyQuestions": [...],
    "answerRecommendations": [...],
    "answerHistory": [...],
    "surveyOpportunities": [...],
    "surveyOpportunityCriteria": [...],
    "surveyFitResults": [...],
    "surveyAttempts": [...],
    "surveyOutcomeEvents": [...],
    "surveyMemory": [...]
  }
}
```

### 1.4 Absolute Secret & Credential Exclusion
- Under no circumstances are API keys, provider tokens, or Windows Credential Manager references exported.
- The `provider_configs` table and `credential_ref` fields are strictly excluded from export queries.
- Serialized export output undergoes an automatic security audit checking for `credential_ref`, `credentialRef`, `apiKey`, and `api_key`. If detected, export execution fails immediately.

### 1.5 Survey Memory & NQI Forward Compatibility
- The `data.surveyMemory` collection is reserved for Next Question Intelligence and Survey Memory records, preserving question sequences, branch transitions, and recognition fingerprints without breaking schema version `1.0.0`.

---

## 2. Data Deletion Specification

### 2.1 Single Persona Deletion
- Enforces strict foreign key cascade in SQLite.
- Deletes associated temporary screenshot files on disk referenced in `screen_captures`.
- If the deleted persona is active, active status is automatically transferred to the next available candidate persona in the same atomic transaction.
- Returns `PersonaDependencySummary` verifying deleted record counts.

### 2.2 Full Application Wipe (`data_wipe_all`)
- **Safety Safeguard**: Requires user confirmation text to exactly match `"HAPUS SEMUA DATA"`.
- **Database Wipe**: Deletes all rows in user data tables inside a transaction.
- **Foreign Key Integrity**: Leaves 0 orphan records.
- **Filesystem Cleanup**: Unlinks all temporary screenshot and capture files on disk.
- **SQLite WAL Checkpoint**: Executes `PRAGMA wal_checkpoint(TRUNCATE);` to flush and truncate WAL log files.
- **SQLite VACUUM**: Executes `VACUUM;` to reclaim unused disk space and clear residual deleted pages from the `.sqlite3` file.
- **Migration & Operational State**: Preserves `_migrations` table and re-inserts default `provider_pool_settings` so the application remains operable.

### 2.3 Transparent Disclosures (Unaffected External Items)
The application explicitly documents and reports to the user that:
1. Exported JSON files saved outside the app directory by the user are not automatically deleted.
2. External backups, OS shadow copies, or cloud replicas outside app boundaries cannot be deleted.
3. Media filesystem slack and unallocated disk clusters are governed by the OS filesystem and are not guaranteed to undergo cryptographic overwrite (secure erase).
