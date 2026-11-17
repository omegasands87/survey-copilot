# Answer Engine

## 1. Evidence statuses
| Status | Meaning | Draft behavior |
|---|---|---|
| `supported` | Current confirmed evidence adequately answers the question | Draft permitted |
| `partially_supported` | Evidence answers only part of the question | Partial draft with explicit gap, or clarification |
| `insufficient_information` | Required evidence is absent | No factual draft; ask for input |
| `conflicting_evidence` | Material evidence conflicts | No definitive draft |
| `user_input_required` | User must supply or confirm a detail | Ask targeted question |
| `not_applicable` | Question does not apply based on explicit user-confirmed evidence | Explain basis; user decides response |

## 2. Workflow
1. Validate question and request.
2. Classify intent and constraints.
3. Retrieve relevant confirmed facts.
4. Filter stale, disputed, sensitive, and irrelevant facts.
5. Run consistency checks.
6. Construct minimized provider context.
7. Generate structured recommendation.
8. Validate schema and fact references.
9. Apply local support checks.
10. Display draft or abstention with rationale.
11. Persist only after the appropriate user action.

## 3. Drafting rules
- Preserve the user's meaning; do not embellish.
- Match requested units and answer format without changing the underlying fact.
- If a range is requested but only a point value exists, do not fabricate a range.
- Do not convert an estimate into an exact fact.
- For subjective questions, ask the user for their view unless a clearly relevant preference has been explicitly recorded.
- For sensitive attributes, require explicit inclusion consent for each relevant request.

## 4. User actions
A recommendation is not an answer submitted to a platform. The user may edit, confirm, reject, or copy it. Confirmation records approval of the draft only.

## 5. Prohibited behavior
No deceptive screening strategy, invented response, automatic form completion, auto-submit, or circumvention of platform controls.
