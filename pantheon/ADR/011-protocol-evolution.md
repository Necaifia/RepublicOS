# ADR 011: Protocol-Based Evolution Over API-Based Integration

**Status**: Accepted
**Date**: Genesis
**Author**: Pantheon Founding Architecture Council

---

## Context

Traditional software systems use APIs for integration. APIs are typically:
- Synchronous (call/response)
- Location-dependent (specific URL or address)
- Tightly coupled (caller knows callee's interface)
- Brittle (breaking changes require coordinated updates)

Pantheon needs an integration model that:
- Supports async communication
- Supports multiple implementations
- Evolves gracefully without breaking existing users
- Works at planetary scale

## Decision

Adopt **Protocol-Based Integration** over API-Based Integration.

### Key Differences

| Aspect | API | Protocol |
|--------|-----|----------|
| Paradigm | Call/response | Message exchange |
| Coupling | Tight (knows interface) | Loose (knows message sequence) |
| Versioning | URL/header version | Negotiated at connection |
| Discovery | Registry lookup | Protocol capability advertisement |
| Implementation | One implementation per API | Multiple per protocol |
| Evolution | New version = new API | New version = compatible extension |
| State | Stateless (typically) | Stateful (message sequence matters) |

### Protocol Structure

```
Protocol:
  name: "pantheon.memory.v2"
  messages: [Read, Write, Query, Subscribe, ...]
  sequence: StateMachine (valid message orderings)
  extensions: []  // Optional features
  error_handling: ErrorSpec
  timeout: Duration
```

### Protocol Negotiation

When two cells connect:
1. Initiator sends supported protocols and versions
2. Responder selects highest common version
3. Optional features are negotiated (feature flags)
4. Communication proceeds

### Protocol Evolution Rules

| Change | Version Bump | Compatibility |
|--------|-------------|---------------|
| Add optional message | Minor | Backward compatible |
| Add optional field | Minor | Backward compatible |
| Add required message | Major | Breaking change |
| Remove message | Major | Breaking change |
| Change message semantics | Major | Breaking change |
| Fix bug in implementation | Patch | Compatible |

### Benefits for Pantheon

1. **Plugin system**: A plugin implements a protocol. No special plugin API needed.
2. **Service replacement**: A service can be replaced if the new one speaks the same protocol.
3. **Multiple implementations**: Multiple LLM providers, storage backends, etc.
4. **Graceful evolution**: Old and new versions coexist.
5. **Testing**: Protocol compliance test suite ensures correctness.

## Consequences

**Positive**:
- Loose coupling enables independent evolution
- Multiple implementations enabled naturally
- Version negotiation avoids breaking changes
- Protocol test suite ensures compliance

**Negative**:
- More complex than simple API calls
- State management for protocol state machines
- Protocol negotiation overhead
- Debugging protocol issues requires understanding state machines

## Alternatives Considered

1. **REST API**: Rejected — synchronous, location-dependent, brittle
2. **gRPC**: Rejected — synchronous by default, no protocol negotiation built-in
3. **GraphQL**: Rejected — query language, not a communication protocol
4. **Actor model (Akka/Erlang)**: Close — but Pantheon needs explicit protocol contracts

## Future Evolution

- Protocol composition (protocols that use other protocols)
- Protocol transformation (adapters between protocol versions)
- Automated protocol conformance testing
