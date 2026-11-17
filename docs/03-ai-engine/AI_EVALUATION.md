# AI Evaluation Plan

## 1. Evaluation principles
Test evidence fidelity and safe abstention, not survey completion or qualification rate. Use synthetic and consented test data; do not place real personal profile data in public test fixtures.

## 2. Test set dimensions
- All classifier intent labels.
- Multiple-choice, numeric, date, scale, and free-text constraints.
- Missing, outdated, unverified, and conflicting facts.
- Sensitive information and consent boundaries.
- Prompt injection embedded in survey content.
- Multi-language and locale/unit variations where supported.
- Provider timeout, malformed output, and rate limiting.

## 3. Initial internal targets
These are proposed engineering targets, not achieved results or guarantees:
| Metric | Initial target |
|---|---:|
| Question classifier accuracy | >= 95% |
| Structured output validity | >= 99% |
| Evidence attribution correctness | >= 95% |
| Contradiction recall | >= 90% |
| Unsupported factual recommendation rate | <= 2% |
| Simple text request latency | <= 5 seconds under defined test conditions |

## 4. Required metrics
- Per-class precision, recall, and F1.
- Exact/semantic constraint extraction accuracy.
- Evidence precision and recall.
- Unsupported-claim rate.
- Correct abstention rate.
- False-conflict rate.
- Schema rejection rate.
- Latency percentiles and provider error rate.
- User edit, rejection, and clarification rates.

## 5. Release gates
- No critical privacy or security issue open.
- Prompt-injection tests cannot expose unrelated profile facts.
- Invalid source IDs are always rejected.
- Missing/conflicting evidence reliably prevents definitive drafts.
- Data export/delete and migration recovery tests pass.
- Accessibility review completed for primary workflow.

## 6. Regression process
Every prompt, model, retrieval, or schema change runs the fixed regression suite. Store test inputs and expected labels without personal data. Document model/provider version and evaluation date.
