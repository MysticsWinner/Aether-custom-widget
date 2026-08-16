# ADR 010: Adaptive Performance Budgets & Degradation Hierarchy

* **Status**: Accepted
* **Context**: Misbehaving or unoptimized 3rd-party widget code can consume excessive CPU or memory without warning.
* **Decision**: Introduce `PerformanceBudget` declarations and `BudgetEvaluator` tracking declared vs actual resource consumption. State machine (`Normal -> SoftLimit -> Warning -> Degraded -> HardLimit`) automatically degrades visual effects or throttling before system performance is impacted.
* **Consequences**:
  - *Positive*: Host system protection against resource starvation.
  - *Positive*: Proactive feedback to widget developers via dev tools inspector.
