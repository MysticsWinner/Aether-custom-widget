# ADR 012: AI Desktop Composer & Mandatory Security Validation Gate

* **Status**: Accepted
* **Context**: Synthesizing complete desktop themes and layouts via AI natural language prompts could bypass schema validation or security boundaries.
* **Decision**: Implement `AiDesktopComposer` in `ai_engine`, routing prompt synthesis through strict schema validation, capability checks (`capability_broker`), and performance prediction before presenting output for explicit user approval.
* **Consequences**:
  - *Positive*: AI-generated desktop layouts cannot execute unauthorized commands or violate platform performance/security boundaries.
  - *Positive*: User retains full approval control before changes are applied.
