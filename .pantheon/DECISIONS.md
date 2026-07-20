# Architectural Decisions

## ADR-001: Everything is a Cell
Humans, AI, schedulers, plugins — all cells. The engine routes events,
not API calls. This makes the system AI-neutral by construction.

## ADR-002: Capability-based security, not ACLs
Capabilities are first-class: issued, attenuated, delegated, revoked.
No ambient authority. Every event carries a capability reference.

## ADR-003: Plugin SDK over configuration
Plugins are Rust traits, not config files. This gives full type safety
and compile-time verification of plugin interfaces.

## ADR-004: Agent runtime over infinite loop
Pantheon reads `.pantheon/` state at session start. It does not loop
forever — it stops when SUCCESS.md criteria are met. This prevents
refactoring spirals and token waste.

## ADR-005: Property tests over unit tests where possible
13 proptests now cover kernel state machines, clocks, and capabilities
with random inputs. Catches edge cases unit tests miss.
