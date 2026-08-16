# ADR 007: Theme Inheritance & Cascading Resolution Architecture

* **Status**: Accepted
* **Context**: Creating modular themes requires duplicating entire token dictionaries, leading to maintenance overhead.
* **Decision**: Implement cascading theme resolution: `System Defaults -> Base Theme -> Derived Theme -> Widget Theme -> Component Theme -> Instance Override`, with cycle detection (`detect_cycle`).
* **Consequences**:
  - *Positive*: Child themes only need to specify overrides.
  - *Positive*: Prevents infinite recursion loops during theme inheritance resolution.
  - *Negative*: Cascading token lookup requires deterministic precedence evaluation.
