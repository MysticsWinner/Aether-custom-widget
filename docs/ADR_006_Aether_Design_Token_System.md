# ADR 006: Aether Design Token System

* **Status**: Accepted
* **Context**: Legacy widget systems rely on hardcoded colors and static fonts, leading to fragmented visual themes and breaking consistency across widget skins.
* **Decision**: Implement a first-class, 12-category semantic Design Token architecture in `theme_engine`. Tokens encapsulate Colors, Typography, Spacing, Sizing, Shape, Borders, Elevation, Materials, Motion, Opacity, Accessibility, and Performance parameters.
* **Consequences**:
  - *Positive*: Complete visual consistency across all widgets; central theme changes instantly propagate to widgets via variable substitution e.g., `{colors.accent}`.
  - *Positive*: Zero visual regressions during theme hot reloading.
  - *Negative*: Widgets must consume semantic tokens rather than raw hex values.
