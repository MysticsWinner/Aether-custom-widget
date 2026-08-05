# Aether — AI Output Verification Checklist

**Quality Assurance Protocol for AI-Generated Artifacts**

---

## Output Validation Checklist

- [ ] **Manifest Syntactic Validity**: AI-generated TOML strings parse cleanly through `widget_parser::parse_manifest()`.
- [ ] **Color Contrast Verification**: AI-generated theme colors maintain WCAG AA contrast ratio ($\ge 4.5:1$) between text and card backgrounds.
- [ ] **Dimension Bounds**: Synthesized layout bounds comply with minimum ($100 \times 50$) and maximum ($3840 \times 2160$) screen constraints.
- [ ] **Lua Code Safety**: AI-generated Lua scripts pass static analysis without unsafe global mutations.
