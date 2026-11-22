# Survey Fit Learning Specification

**Version:** 1.0 product specification  
**Status:** Required MVP capability  
**Normative source:** [Product Definition](PRODUCT_DEFINITION.md)

## 1. Purpose

Learn which survey opportunities are historically more compatible with each truthful persona, using explicit eligibility criteria and observed survey outcomes. The system helps prioritize opportunities; it does not guarantee qualification or alter answers to manipulate screeners.

## 2. Two separate signals

### 2.1 Deterministic eligibility matching

Before a survey attempt, compare explicit survey criteria with persona facts. Each criterion receives one state:
- match
- mismatch
- unknown
- stale
- conflict

This layer is explainable and evidence-based. Unknown is not a match. A known mismatch must be surfaced rather than concealed.

### 2.2 Historical outcome learning

After an attempt, associate the observed outcome with:
- Survey platform.
- Persona ID.
- Survey category and topic.
- Explicit eligibility criteria, where available.
- Reward offer and estimated duration.
- Attempt timestamp.
- Outcome label and evidence source.
- Whether the outcome is confirmed or uncertain.

The system may learn historical patterns by persona, platform, and category when enough data exists.

## 3. Outcome taxonomy

Keep outcomes separate:
- qualified
- screened_out
- quota_full
- completed
- quality_rejected
- technical_failure
- abandoned_by_user
- pending
- unknown

A survey may have a sequence of events (qualified, completed, reward pending, credited); these are not mutually exclusive states of one generic success flag. Model the attempt lifecycle explicitly.

## 4. Prediction and confidence rules

- Do not display a percentage prediction until a minimum sample and calibration requirements are established in technical/evaluation design.
- Always show sample size, observation window, and uncertainty.
- Separate eligibility match from historical completion/credit rates.
- Avoid pooling different personas or materially different survey types.
- Account for recency and changing platform behavior.
- Use a baseline or uncertainty-aware estimate when data is sparse; label it as insufficient history.
- Validate predictions on later, held-out attempts, not on the same data used to train them.
- Prevent data leakage: predictors may use only information available before the attempt; outcomes are labels observed afterward.
- Do not claim causation from observational correlations.

## 5. Learning behavior

The model may identify that certain categories, criteria, platforms, or reward/time combinations have historically produced better outcomes for a persona. It may rank opportunities by transparent factors such as:
- Explicit eligibility match.
- Historical qualification/completion evidence.
- Reward value and duration.
- Data freshness and confidence.
- User-defined practical preferences.

The system must explain which factors contributed to a ranking. It must not optimize the persona's answers to appear eligible.

## 6. Integrity and safety constraints

- Never change or recommend changing a personal fact to improve a score.
- Never recommend a false answer to pass screening.
- Never infer that a persona qualifies solely from previous success.
- Treat each new survey's explicit criteria as authoritative for that survey.
- Do not use outcomes from one persona as personal facts for another persona.
- Distinguish screening failure from quota exhaustion, technical failure, and quality rejection.

## 7. Evaluation

Before presenting predictive percentages, evaluate:
- Calibration on chronologically held-out attempts.
- Precision/recall for qualification and completion labels where meaningful.
- Performance by platform/category/persona with minimum sample thresholds.
- Drift over time.
- Confidence behavior under sparse data.
- Whether rankings improve useful opportunity selection without encouraging misrepresentation.

## 8. Acceptance criteria

- [ ] Explicit criteria matching is separate from statistical historical learning.
- [ ] Unknown criteria are never silently treated as matches.
- [ ] Outcomes are recorded with persona, platform, time, and evidence provenance.
- [ ] Screening failure, quota full, technical failure, and quality rejection are distinct.
- [ ] Sparse history is labeled insufficient rather than shown as a confident prediction.
- [ ] Predictions are validated on future/held-out data before display.
- [ ] The system never recommends false answers or persona manipulation.
