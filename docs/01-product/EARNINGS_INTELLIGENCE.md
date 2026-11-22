# Earnings Intelligence Specification

**Version:** 1.0 product specification  
**Status:** Required MVP capability  
**Normative source:** [Product Definition](PRODUCT_DEFINITION.md)

## 1. Purpose

Track survey opportunities, time spent, completion outcomes, offered rewards, credited rewards, and withdrawable balances with the least possible manual bookkeeping.

The dashboard must distinguish observed facts from estimates and unverified claims. It must never present an offered reward as money earned.

## 2. Data acquisition hierarchy

Use sources in this order where available:

1. Explicitly authorized official platform integration, if one is supported and permitted.
2. User-initiated screen capture/OCR of survey listing, completion, reward, transaction, or balance screens.
3. In-session observation of visible state changes while the user has explicitly started a tracking session.
4. User correction only when information cannot be reliably observed or captured.

No platform credentials may be collected or stored for scraping. No terms circumvention or hidden endpoint access is permitted.

## 3. Earnings states

Every reward record must have a distinct state:
- offered: reward displayed before the survey; not earned.
- in_progress: survey attempt has started.
- completed_unverified: completion appears to have occurred, but reward credit is not yet confirmed.
- pending: platform shows a pending reward.
- credited: platform evidence shows reward credited to the account.
- withdrawable: platform evidence shows the amount is available for withdrawal.
- rejected_or_reversed: platform explicitly indicates rejection, reversal, or cancellation.
- unknown: evidence is insufficient or contradictory.

The dashboard must not combine these states into one undifferentiated earnings total.

## 4. Required dashboard measures

- Offered reward value and currency.
- Completed survey count.
- Completion-unverified and pending reward totals.
- Credited reward total.
- Withdrawable balance, only when directly evidenced.
- Time spent per survey attempt.
- Effective hourly rate calculated from credited rewards and measured time, with calculation scope shown.
- Screening outcomes and completion outcomes by platform, category, and persona.
- Data freshness and evidence source for imported/detected values.

Points, tokens, gift cards, and cash must retain their native unit until a documented conversion rate and date are available. Do not silently convert points to cash.

## 5. Evidence provenance

Every detected reward or balance must record:
- Platform/source identifier.
- Persona and survey attempt association when known.
- Amount and native currency/unit.
- Reward state.
- Observation timestamp.
- Evidence type (official integration, captured screen, in-session observation, user correction).
- Evidence reference or protected screenshot reference where retention is enabled.
- Confidence and verification state.

If two sources disagree, show the conflict and do not silently select the larger or more favorable value.

## 6. Time tracking

- Start timing when the user explicitly starts a survey attempt or confirms that a detected survey is in progress.
- Pause/stop timing when the user ends the attempt, leaves the tracked context, or the application detects a reliable terminal state, subject to user control.
- Allow correction of obvious measurement errors.
- Clearly distinguish active time from elapsed wall-clock time.
- Do not infer that a survey was completed merely because the timer stopped.

## 7. Automation and limitations

The application should automatically detect visible survey/reward information where technically possible and should avoid asking the user to type routine amounts or outcomes. However:
- It cannot know a hidden account balance from a screen that does not show it.
- It cannot independently confirm a platform-side credit without visible evidence or a permitted official integration.
- It cannot guarantee that a displayed offer will be paid.
- It must label estimates, detected states, and verified credits separately.

## 8. Acceptance criteria

- [ ] Offered rewards are never counted as credited earnings.
- [ ] Completion, pending, credited, and withdrawable states are separate.
- [ ] Reward amount retains its currency or native unit.
- [ ] Every detected amount has a timestamp and source provenance.
- [ ] Survey time is measured and can be corrected.
- [ ] Dashboard totals are reproducible from underlying reward records.
- [ ] Unverified data is visibly distinguished from confirmed platform evidence.
- [ ] Routine tracking does not require manual entry when the information is visible and reliably detectable.
