# ADR 008: Material Engine & Adaptive Fallback Pipeline

* **Status**: Accepted
* **Context**: Complex blur, Acrylic, and Mica effects can cause frame drops on lower-end GPUs, laptops on battery, or remote desktop sessions.
* **Decision**: Create an explicit `MaterialEngine` abstraction providing material types (`Solid`, `Transparent`, `Glass`, `Acrylic`, `Mica`, `Elevated`, `Custom`) and an adaptive degradation pipeline (`Advanced Material -> GPU Check -> Power State Check -> Accessibility Check -> Fallback Material`).
* **Consequences**:
  - *Positive*: Preserves Aether's strict performance guarantees (<0.08% idle CPU) while providing visual effects.
  - *Positive*: Seamlessly respects High Contrast and Reduce Transparency Windows accessibility settings.
