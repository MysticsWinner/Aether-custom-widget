# ADR 009: Reactive Signal-Binding Widget Architecture

* **Status**: Accepted
* **Context**: Continuously polling telemetry metrics in tick loops causes unnecessary redraws and CPU context switches for static components.
* **Decision**: Implement `Signal<T>` and `SignalBinding` in `widget_sdk`, driving widget updates via metric signals (`Telemetry/Event -> Signal -> Binding -> Widget State -> Dirty Region -> Render`).
* **Consequences**:
  - *Positive*: Redraws occur exclusively when bound metric values change beyond hysteresis thresholds.
  - *Positive*: Zero allocation in signal version comparisons.
