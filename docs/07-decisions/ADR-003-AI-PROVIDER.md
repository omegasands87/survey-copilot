# ADR-003: Abstract the AI Provider

**Status:** Proposed

## Context
Model quality, pricing, privacy terms, structured-output support, and availability can change. Binding domain logic to one provider would make evaluation and migration harder.

## Decision
Define a provider adapter interface and keep provider-specific request/response handling outside domain services. Select the initial provider only after evaluating quality, privacy, cost, and operational requirements.

## Consequences
**Positive:** provider can be changed without rewriting domain logic; evaluation can compare models.  
**Trade-offs:** abstraction adds work and cannot erase provider-specific capability differences.

## Revisit if
A provider's unique capability becomes a core requirement or abstraction materially impairs reliability.
