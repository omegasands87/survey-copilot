# Question Classifier

## 1. Supported intent labels
- `demographic`: age range, household composition, location at an appropriate granularity.
- `employment`: occupation, industry, work status, tenure.
- `financial`: income, spending, financial products; sensitive by default.
- `purchase_history`: past purchases or ownership.
- `brand_usage`: awareness, usage, preference, frequency.
- `health_lifestyle`: health or lifestyle information; sensitive by default.
- `preference_opinion`: subjective preference or opinion.
- `behavior_frequency`: frequency or recurring behavior.
- `knowledge_awareness`: familiarity or knowledge.
- `eligibility_screening`: screening question; no eligibility prediction.
- `other`: unsupported or unclear intent.

## 2. Extracted constraints
- Expected answer type: free text, single choice, multiple choice, number, date, scale, yes/no.
- Time window and reference date.
- Units, range, precision, and locale.
- Required scope (self, household, organization).
- Whether question asks for fact, estimate, or opinion.
- Ambiguity and missing definitions.

## 3. Confidence and abstention
Classifier confidence describes interpretation confidence only. If wording is ambiguous, show the ambiguity and ask for clarification rather than silently selecting an interpretation.

## 4. Evaluation
Measure per-class precision/recall, confusion matrix, constraint extraction accuracy, and abstention on ambiguous or out-of-scope questions. Include adversarial survey text in the test set.
