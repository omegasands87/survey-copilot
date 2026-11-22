# Survey Fit Learning Specification

**Version:** 1.0  
**Status:** Required MVP capability  
**Normative source:** PRODUCT_DEFINITION.md

## 1. Purpose

Help the user decide which survey opportunities appear relevant to each real persona. The app combines deterministic matching of visible eligibility criteria with carefully qualified historical outcome patterns. It does not promise qualification or manipulate answers.

## 2. Deterministic eligibility matching

Before a survey attempt, compare explicit visible criteria with current persona facts. Each criterion receives one state:
- match
- mismatch
- unknown
- stale
- conflict

Unknown is never treated as a match. Every result must show the criterion and supporting fact or missing information.

## 3. Historical outcome learning

If the user records or confirms an observed survey outcome, associate it with:
- Persona ID.
- Platform label, if known.
- Survey category/topic, if known.
- Explicit eligibility criteria, if captured.
- Attempt timestamp.
- Outcome label and evidence source.
- Whether the outcome is confirmed or uncertain.

Outcome labels:
- qualified
- screened_out
- quota_full
- completed
- quality_rejected
- technical_failure
- abandoned_by_user
- pending
- unknown

These labels describe survey fit and attempt outcome only. They do not create an earnings, reward, balance, payout, or income ledger.

## 4. Prediction rules

- Do not display a percentage prediction until minimum sample and calibration requirements are defined and tested.
- Show sample size, observation window, and uncertainty.
- Keep deterministic criterion matching separate from historical outcome estimates.
- Do not pool materially different personas or survey categories without a validated method.
- Use chronological holdout data to validate estimates.
- Avoid data leakage: predictors use only information available before an attempt.
- Do not claim causation from observational correlations.
- Sparse history must be labeled insufficient.

## 5. Ranking

The app may sort opportunities using transparent factors:
- Explicit eligibility match.
- Historical outcome evidence, where sufficient.
- Survey category relevance.
- User-selected preferences.

Do not use reward or income tracking as a v1.0 feature. Do not recommend changing a personal fact or answer to improve fit.

## 6. Integrity constraints

- Never change or recommend changing a personal fact to improve a score.
- Never recommend a false answer to pass screening.
- Never infer qualification solely from previous success.
- Use each survey's explicit criteria as authoritative for that survey.
- Keep outcomes attributed to the actual persona used.

## 7. Acceptance criteria

- [ ] Explicit matching is separate from historical learning.
- [ ] Unknown criteria are never silently treated as matches.
- [ ] Outcomes are attributed to persona and attempt.
- [ ] Screening failure, quota full, technical failure, and quality rejection are distinct.
- [ ] Sparse history is labeled insufficient.
- [ ] Predictions are validated on chronologically held-out data.
- [ ] No reward, balance, payout, or income ledger is implemented.
- [ ] The system never recommends false answers or persona manipulation.
