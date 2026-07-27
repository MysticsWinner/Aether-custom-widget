# Development Rules & Guidelines

The following mandatory development rules apply to all engineering, architecture, and coding tasks across this codebase:

## Workflow & Process Governance
1. **Never skip phases**: Follow the phased architecture roadmap strictly.
2. **Architecture Prerequisite**: Never implement features whose architecture has not been finalized.
3. **Comprehensive Deliverables**: Every feature/pull request must include:
   - Unit tests
   - Benchmarks
   - Documentation
   - Logging
   - Error handling
   - Performance analysis
   - Security review

## System & Software Architecture
4. **Interface Isolation**: Every subsystem must expose interfaces (e.g. traits/abstract APIs) instead of concrete implementations.
5. **Composition**: Prefer composition over inheritance.
6. **State & Immutability**:
   - Avoid global state.
   - Prefer immutable data structures and zero-side-effect functions.
7. **Dependency & Build Constraints**:
   - Never optimize prematurely.
   - Keep dependencies minimal and audit regularly.
   - Everything must compile cleanly on Windows 11 (`x86_64` & `ARM64`).

## Documentation, Trade-Offs & Quality
8. **Trade-Off Analysis**: Every design decision must explicitly outline alternatives considered and rationale.
9. **Architectural Visualizations**: Generate visual diagrams (Mermaid) whenever architecture or data flows change.
10. **Production Quality**: Maintain production-grade quality, strict linting, and error handling at every step.
