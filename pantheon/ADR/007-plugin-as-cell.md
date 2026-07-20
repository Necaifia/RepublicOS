# ADR 007: Plugin as a Cell, Not a Separate Concept

**Status**: Accepted
**Date**: Genesis
**Author**: Pantheon Founding Architecture Council

---

## Context

Plugin systems in existing products (VS Code, Jenkins, WordPress) typically have a separate plugin API, lifecycle, and security model from the core system. This creates complexity, fragmentation, and security gaps.

## Decision

A **Plugin IS a Cell**. It uses the same identity, lifecycle, security, communication, and memory systems as everything else.

### Key Design Choices

1. **No special plugin API**: Plugins use the same Cell API. The SDK provides convenience wrappers but no special privileges.

2. **Protocol-based extension**: Plugins extend the system by implementing protocols. Any Cell can implement any protocol. There is no distinction between "core protocols" and "plugin protocols."

3. **Manifest declaration**: Plugins have a manifest that declares protocols, capabilities, and hooks. This is metadata, not a different security model.

4. **Registry for discovery**: The Plugin Registry provides discovery, versioning, and verification. But any Cell can act as a plugin — the registry is just a directory.

## Consequences

**Positive**:
- No special infrastructure for plugins (everything is a Cell)
- Plugins have full Cell capabilities (identity, memory, security)
- No privilege escalation (plugins have only their declared capabilities)
- Simpler mental model: "Everything is a Cell"

**Negative**:
- Plugin-specific features (registry, manifest) are bolted on top of the Cell model
- Plugin lifecycle (install/uninstall) is not inherent to Cells
- Some convenience will be needed in the Plugin SDK

## Alternatives Considered

1. **Separate plugin framework**: Rejected — creates parallel infrastructure
2. **Script-based plugins (Lua/Python)**: Rejected — security and performance concerns
3. **Dynamic linking**: Rejected — crashes in plugins crash the system

## Future Evolution

- Plugin hot-reload without Cell restart
- Plugin composition (plugins that compose other plugins)
- Plugin market with automated quality verification
