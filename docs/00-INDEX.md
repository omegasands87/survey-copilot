# Documentation Index

Survey Copilot is a Windows 11 x64 desktop AI copilot. This repository contains its product and engineering specifications; documentation does not mean the application has already been implemented.

## Start here
- [Antigravity Engineering Handoff](08-agent-handoff/ANTIGRAVITY_HANDOFF.md)
- [Antigravity Start Prompt](08-agent-handoff/ANTIGRAVITY_START_PROMPT.md)
- [Implementation Backlog](08-agent-handoff/IMPLEMENTATION_BACKLOG.md)
- [Acceptance Test Matrix](08-agent-handoff/ACCEPTANCE_TEST_MATRIX.md)
- [Documentation Gaps](08-agent-handoff/DOCUMENTATION_GAPS.md)
- [Repository Map](08-agent-handoff/REPOSITORY_MAP.md)
- [Addendum: Next Question Intelligence & Survey Memory](08-agent-handoff/ADDENDUM_NEXT_QUESTION_INTELLIGENCE_AND_SURVEY_MEMORY.md) — supplemental scope; read after the active implementation task.
- [Product Definition](01-product/PRODUCT_DEFINITION.md)
- [Product Requirements](01-product/PRODUCT_REQUIREMENTS.md)
- [Feature Specification](01-product/FEATURE_SPECIFICATION.md)
- [User Flows](01-product/USER_FLOWS.md)
- [Technical Blueprint](BLUEPRINT.md)

## Product
- [Product Definition](01-product/PRODUCT_DEFINITION.md)
- [Product Requirements](01-product/PRODUCT_REQUIREMENTS.md)
- [Feature Specification](01-product/FEATURE_SPECIFICATION.md)
- [User Personas](01-product/USER_PERSONAS.md)
- [User Flows](01-product/USER_FLOWS.md)
- [Screen Intelligence](01-product/SCREEN_INTELLIGENCE.md)
- [Multi-Persona Specification](01-product/MULTI_PERSONA_SPECIFICATION.md)
- [Survey Fit Learning](01-product/SURVEY_FIT_LEARNING.md)

## Architecture
- [System Architecture](02-architecture/SYSTEM_ARCHITECTURE.md)
- [Database Schema](02-architecture/DATABASE_SCHEMA.md)
- [AI Architecture](02-architecture/AI_ARCHITECTURE.md)
- [Security Architecture](02-architecture/SECURITY_ARCHITECTURE.md)
- [Technology Stack](02-architecture/TECH_STACK.md)
- [AI Provider and API Pooling](02-architecture/API_PROVIDER_POOLING.md)
- [Screen Intelligence Technical Specification](02-architecture/SCREEN_INTELLIGENCE_TECHNICAL_SPEC.md)
- [Tauri Command Contracts](02-architecture/TAURI_COMMAND_CONTRACTS.md)
- [Provider Adapter Contract](02-architecture/PROVIDER_ADAPTER_CONTRACT.md)

## AI engine
- [Prompt Specification](03-ai-engine/AI_PROMPT_SPECIFICATION.md)
- [Question Classifier](03-ai-engine/QUESTION_CLASSIFIER.md)
- [Answer Engine](03-ai-engine/ANSWER_ENGINE.md)
- [Structured Output Schema](03-ai-engine/STRUCTURED_OUTPUT_SCHEMA.md)
- [Memory System](03-ai-engine/MEMORY_SYSTEM.md)
- [Consistency Engine](03-ai-engine/CONSISTENCY_ENGINE.md)
- [AI Evaluation](03-ai-engine/AI_EVALUATION.md)

## Design
- [UI/UX Specification](04-design/UI_UX_SPECIFICATION.md)
- [Design System](04-design/DESIGN_SYSTEM.md)
- [Desktop Layout](04-design/DESKTOP_LAYOUT.md)

## Development
- [Development Roadmap](05-development/DEVELOPMENT_ROADMAP.md)
- [Implementation Plan](05-development/IMPLEMENTATION_PLAN.md)
- [Build Checklist](05-development/BUILD_CHECKLIST.md)
- [Coding Standards](05-development/CODING_STANDARDS.md)
- [Testing Strategy](05-development/TESTING_STRATEGY.md)
- [Release Plan](05-development/RELEASE_PLAN.md)

## Operations
- [Installation](06-operations/INSTALLATION.md)
- [Configuration](06-operations/CONFIGURATION.md)
- [Privacy Policy Draft](06-operations/PRIVACY_POLICY.md)
- [Troubleshooting](06-operations/TROUBLESHOOTING.md)
- [Architecture Decision Records](07-decisions/README.md)

## Agent handoff
- Root [AGENTS.md](../AGENTS.md) defines mandatory coding-agent rules.
- [Antigravity workspace rule](../.agent/rules/survey-copilot.md) applies project constraints.
- [Implement Next workflow](../.agent/workflows/implement-next.md) defines the incremental coding loop.
- Backlog checkboxes require implementation and test evidence before being checked.

## Governance
- Product Definition is the authoritative product scope.
- Architecture and implementation documents must implement that scope, not silently expand it.
- Resolve document conflicts before implementation.
- MUST/MUST NOT are binding requirements; SHOULD is a strong recommendation; MAY is optional.
- Never describe planned functionality as shipped.
