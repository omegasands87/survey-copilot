# Survey Copilot — High-Level Product and Engineering Overview

**Status:** Non-normative overview  
**Authority:** The detailed product contract is [Product Definition v1.0](01-product/PRODUCT_DEFINITION.md). This file is a navigation-level summary, not an implementation specification. If any statement here conflicts with detailed specifications, the detailed specification governs.

## Product purpose

Survey Copilot is a desktop survey earning assistant. It helps a user evaluate survey opportunities, read visible survey content, select the truthful respondent persona, understand questions and quality checks, prepare evidence-grounded answers, and track observed outcomes and rewards.

The product optimizes for efficient, accurate survey participation and transparent earnings records. It does not guarantee qualification, completion, or payment.

## Core product capabilities

1. **Multi-Persona Digital Persona:** maintain multiple real respondent profiles with isolated facts, sources, and survey histories.
2. **Survey Opportunity Analyzer:** extract visible reward, stated duration, topic, and eligibility criteria; compare criteria with each persona.
3. **Screen Intelligence:** user-initiated region/window capture, OCR, visual interpretation, and correction of extracted content.
4. **Question Intelligence:** classify intent, answer format, timeframe, units, conditions, and explicit attention instructions.
5. **Evidence-Based Answer Copilot:** retrieve relevant confirmed facts, check conflicts, recommend supported answers, or abstain.
6. **Survey Quality Guard:** flag ambiguous wording, negative phrasing, compound questions, timeframe mismatch, conditional logic, and explicit attention checks.
7. **Earnings Intelligence:** track survey attempts, time, offered rewards, completion, pending credit, credited rewards, and withdrawable amounts with evidence provenance.
8. **Survey Fit Learning:** learn persona-specific historical patterns from attributed outcomes, with sample size, uncertainty, and chronological validation.
9. **Review and History:** user reviews, edits, confirms, rejects, and manually transfers answers; preserve the original recommendation and outcome history.

## Interaction and integrity boundaries

- Screen capture is initiated by the user; silent background screen recording is not allowed.
- The user verifies OCR/visual extraction before answer generation.
- Multiple personas must represent real people whose information the user is authorized to manage.
- The application must never fabricate facts or recommend false screener answers.
- The application may rank opportunities using explicit fit evidence, historical outcomes, reward, and duration, but unknown criteria are not treated as matches.
- Offered rewards are not earnings. Credited and withdrawable amounts require evidence.
- v1.0 does not automate clicks, form filling, navigation, or survey submission.
- No CAPTCHA bypass, bot-detection evasion, platform protection circumvention, or hidden credential scraping.

## Architecture direction

The product remains local-first, with optional external AI processing. The exact desktop framework, database schema, screen capture implementation, OCR/vision providers, AI provider interface, credential storage, and security controls must be finalized in the Stage 2 architecture specifications. The earlier technology list is a candidate direction only, not a final architecture decision.

## Documentation map

- Product scope and acceptance: [Product Definition](01-product/PRODUCT_DEFINITION.md)
- Screen capture and OCR: [Screen Intelligence](01-product/SCREEN_INTELLIGENCE.md)
- Multiple profiles: [Multi-Persona Specification](01-product/MULTI_PERSONA_SPECIFICATION.md)
- Rewards and dashboard: [Earnings Intelligence](01-product/EARNINGS_INTELLIGENCE.md)
- Eligibility and historical learning: [Survey Fit Learning](01-product/SURVEY_FIT_LEARNING.md)
- Testable requirements: [Product Requirements](01-product/PRODUCT_REQUIREMENTS.md)
- Feature behavior: [Feature Specification](01-product/FEATURE_SPECIFICATION.md)
- End-to-end flows: [User Flows](01-product/USER_FLOWS.md)

## Implementation gate

Do not begin application coding based only on this overview. Stage 2 must produce complete architecture, data, screen, AI, security, and service-contract specifications consistent with Product Definition v1.0. Stage 3 must produce implementation and verification detail. The product scope cannot be silently expanded during those stages.
