# ADR 001: Cell as Universal Primitive

**Status**: Accepted
**Date**: Genesis
**Author**: Pantheon Founding Architecture Council

---

## Context

Pantheon needs a universal computational primitive that serves as the foundation for all higher-level abstractions: agents, services, plugins, and system components. This primitive must support identity, isolation, communication, lifecycle management, and security at scale.

## Decision

Adopt the **Cell** as the universal primitive of computation in Pantheon.

### Key Design Choices

1. **Cells are the ONLY computational unit**: Everything — agents, services, plugins, system components — is a Cell. There is no distinction between "user code" and "system code" at the primitive level.

2. **Cryptographic identity**: Every Cell has an Ed25519 keypair. This provides unforgeable identity, message signing, and capability delegation without external infrastructure.

3. **No shared state**: Cells own their state exclusively. Inter-cell communication happens exclusively through messages on the Event Bus. This eliminates concurrency bugs and makes reasoning about Cell behavior local.

4. **Hierarchical lifecycle**: Cells form a tree (parent/children). Parents manage children. This provides structured concurrency: when a parent terminates, all children are terminated.

5. **Capability-based security**: A Cell's capabilities act as its "membrane" — only declared capabilities cross the boundary. This provides fine-grained, delegatable, revocable access control.

## Consequences

**Positive**:
- Simple, universal abstraction reduces conceptual overhead
- Cryptographic identity enables distributed trust without central authority
- Tree hierarchy enables resource management and lifecycle scoping
- Capabilities provide security that scales with the system

**Negative**:
- Message-passing adds latency compared to shared memory
- Cryptographic operations add CPU overhead
- Tree hierarchy can create bottlenecks at parents

## Alternatives Considered

1. **Thread-based model**: Rejected because shared state leads to concurrency bugs at scale
2. **Actor model (Erlang-style)**: Close, but Pantheon needs stronger security (capabilities) and lifecycle guarantees
3. **Microservice model**: Rejected — too coarse-grained, too much infrastructure overhead per unit
4. **Pure functional (Haskell-style)**: Rejected — too restrictive for agent behavior that is inherently stateful

## Future Evolution

- Cell specialization may emerge (e.g., hardware-accelerated Cells)
- Cell migration across machine boundaries enables elastic scaling
- Cell composition patterns (e.g., cell pools, cell swarms) will develop organically
