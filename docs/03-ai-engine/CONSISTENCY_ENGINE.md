# Consistency Engine

## 1. Purpose
Identify material contradictions between a proposed response, current profile facts, and relevant confirmed answer history.

## 2. Comparison dimensions
- Same attribute and subject.
- Compatible time period.
- Same units and measurement basis.
- Current versus historical validity.
- Source reliability and verification status.
- Whether one value is an estimate or range.

## 3. Outcomes
- `consistent`: no material conflict found.
- `potential_conflict`: values may differ due to date, unit, or interpretation.
- `conflict`: incompatible claims for the same scope and period.
- `not_comparable`: insufficient metadata to compare.

## 4. Resolution
Show each conflicting value, its date, status, and source. Ask the user to clarify or update the relevant fact. Do not choose a value based on which might improve survey acceptance.

## 5. False-positive control
Do not flag legitimate changes over time as contradictions. Normalize units only when conversion is unambiguous. Keep the original value and unit for auditability.
