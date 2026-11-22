# Documentation Index

This repository contains the product and engineering specifications for Survey Copilot. The project remains in the specification stage; documentation does not imply that application functionality has been implemented.

## Read this first

- [Product Definition v1.0](01-product/PRODUCT_DEFINITION.md) — authoritative product scope, user promise, boundaries, and acceptance baseline.
- [Product Requirements](01-product/PRODUCT_REQUIREMENTS.md) — testable product requirements derived from the definition.
- [Feature Specification](01-product/FEATURE_SPECIFICATION.md) — required feature behavior and user-visible rules.
- [User Flows](01-product/USER_FLOWS.md) — normative end-to-end workflows.
- [Technical Blueprint](BLUEPRINT.md) — legacy high-level overview; subordinate to the detailed specifications.

## Product
- [Product Definition v1.0](01-product/PRODUCT_DEFINITION.md)
- [Product Requirements](01-product/PRODUCT_REQUIREMENTS.md)
- [Feature Specification](01-product/FEATURE_SPECIFICATION.md)
- [Target User and Usage Model](01-product/USER_PERSONAS.md)
- [User Flows](01-product/USER_FLOWS.md)

## Architecture
- [System architecture](02-architecture/SYSTEM_ARCHITECTURE.md)
- [Database schema](02-architecture/DATABASE_SCHEMA.md)
- [AI architecture](02-architecture/AI_ARCHITECTURE.md)
- [Security architecture](02-architecture/SECURITY_ARCHITECTURE.md)
- [Technology stack](02-architecture/TECH_STACK.md)

## AI engine
- [Prompt specification](03-ai-engine/AI_PROMPT_SPECIFICATION.md)
- [Question classifier](03-ai-engine/QUESTION_CLASSIFIER.md)
- [Answer engine](03-ai-engine/ANSWER_ENGINE.md)
- [Memory system](03-ai-engine/MEMORY_SYSTEM.md)
- [Consistency engine](03-ai-engine/CONSISTENCY_ENGINE.md)
- [AI evaluation](03-ai-engine/AI_EVALUATION.md)

## Design
- [UI/UX specification](04-design/UI_UX_SPECIFICATION.md)
- [Design system](04-design/DESIGN_SYSTEM.md)
- [Desktop layout](04-design/DESKTOP_LAYOUT.md)

## Development
- [Development roadmap](05-development/DEVELOPMENT_ROADMAP.md)
- [Implementation plan](05-development/IMPLEMENTATION_PLAN.md)
- [Coding standards](05-development/CODING_STANDARDS.md)
- [Testing strategy](05-development/TESTING_STRATEGY.md)
- [Release plan](05-development/RELEASE_PLAN.md)

## Operations and governance
- [Installation](06-operations/INSTALLATION.md)
- [Configuration](06-operations/CONFIGURATION.md)
- [Privacy policy draft](06-operations/PRIVACY_POLICY.md)
- [Troubleshooting](06-operations/TROUBLESHOOTING.md)
- [Architecture decision records](07-decisions/README.md)

## Document governance

- Product Definition v1.0 is the authoritative source for product scope.
- Product requirements, feature behavior, and user flows must remain consistent with that definition.
- Architecture and implementation documents must implement the product contract; they may not expand product scope.
- A conflict between documents must be resolved before implementation, not left for developers to interpret.
- **MUST** and **MUST NOT** indicate binding requirements. **SHOULD** indicates a strong recommendation. **MAY** indicates an option.
- Do not represent planned functionality as shipped functionality.
- Update all affected documents in the same change as an approved requirement or behavior change.
