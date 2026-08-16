# ADR 011: Desktop Profiles & Context-Aware Automation Engine

* **Status**: Accepted
* **Context**: User desktop requirements change depending on activities (e.g. gaming vs coding vs streaming vs travelling).
* **Decision**: Implement `ProfileManager` (`Gaming`, `Coding`, `Streaming`, `Work`, `Minimal`, `Travel`, `Custom`) and `ContextAwareEngine` in `config_manager` to automatically detect context signals (fullscreen apps, battery saver, active processes) and trigger atomic profile switches.
* **Consequences**:
  - *Positive*: Seamless adaptation of desktop layout, active widgets, materials, and refresh rates based on active context.
  - *Positive*: Full rollback recovery on profile switch failure.
