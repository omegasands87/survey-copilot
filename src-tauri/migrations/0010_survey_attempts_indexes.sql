-- Migration 0010: Indexes for Survey Attempts and Outcome Learning (SC-052)
-- Optimizes filtering by opportunity, outcome status, and observed events.

CREATE INDEX IF NOT EXISTS idx_survey_attempts_opp ON survey_attempts(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_survey_attempts_outcome ON survey_attempts(current_outcome);
CREATE INDEX IF NOT EXISTS idx_survey_outcome_events_label ON survey_outcome_events(outcome_label);
CREATE INDEX IF NOT EXISTS idx_survey_outcome_events_confidence ON survey_outcome_events(confidence);
