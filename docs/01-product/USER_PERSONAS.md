# Target User and Usage Model

**Version:** 1.0 MVP  
**Normative source:** PRODUCT_DEFINITION.md

## 1. Primary operator

One individual uses Survey Copilot on a Windows 11 PC and may maintain multiple profiles for distinct real respondents whose information the operator is authorized to manage.

There are no cloud accounts, shared workspaces, or synchronized profiles in v1.0.

## 2. Main needs

- Understand survey questions without repeatedly interpreting unfamiliar wording.
- Read questions and answer choices from screenshots when text cannot be copied.
- Recall the correct information for the selected respondent.
- Keep answers consistent with that respondent's confirmed facts and previous answers.
- Receive an honest recommendation or a clear request for missing information.
- Compare visible survey criteria with respondent profiles.
- Avoid repetitive API setup and interruptions from individual Gemini key limits.

## 3. Design implications

- The primary workflow is choose persona → capture question → get recommendation.
- Evidence must be visible beside the recommendation.
- The active persona must be obvious before every analysis.
- Missing evidence is a valid result; the app must not invent an answer.
- Provider pool configuration should be automatic by default.
- Advanced provider controls belong in Settings.
- User edits and confirmations remain separate from AI output.
- No external survey action occurs as a side effect of analysis.

## 4. Multiple respondent personas

Each persona corresponds to a real individual. The user must have authority and, where appropriate, consent to manage that person's information. Facts, sources, preferences, and history remain isolated. The active persona is shown before analysis. Personas must not be used to misrepresent the actual respondent or bypass eligibility rules.

## 5. Out of scope

Organizations, team collaboration, cloud sync, browser automation, automatic form completion, survey account credentials, reward/income tracking, and fictional respondent identities are outside v1.0.
