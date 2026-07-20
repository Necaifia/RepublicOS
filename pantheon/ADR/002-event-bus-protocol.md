# ADR 002: Event Bus as Universal Communication Backbone

**Status**: Accepted
**Date**: Genesis
**Author**: Pantheon Founding Architecture Council

---

## Context

Cells need to communicate. Traditional approaches (RPC, REST, shared memory, message queues) each have trade-offs. Pantheon needs a single communication substrate that supports all communication patterns (request/response, pub/sub, streaming, batch) at planetary scale with strong ordering and durability guarantees.

## Decision

Adopt the **Event Bus** as the universal communication substrate.

### Key Design Choices

1. **Partitioned, ordered log**: Modeled after Apache Kafka. Each partition is a totally ordered, append-only log of events. Partition key is derived from the target capability hash.

2. **Exactly-once within partition**: Within a partition, exactly-once delivery semantics. Across partitions, at-least-once with deduplication.

3. **Events addressed by capability, not cell ID**: Location independence. A Cell can move without affecting event routing. The system resolves capability → current Cell binding.

4. **Signed events**: Every event is cryptographically signed by the source Cell. Provides non-repudiation and audit trail.

5. **Causal tracking**: Every event carries its parent event ID, enabling reconstruction of causal chains across partitions.

## Consequences

**Positive**:
- Single substrate for all communication patterns
- Strong ordering enables deterministic replay
- Location independence enables Cell migration
- Cryptographic audit trail for governance
- Proven scalability (Kafka model at trillion-event scale)

**Negative**:
- Partitioned design means cross-partition operations are expensive
- Exactly-once across partitions requires coordination
- Event retention requires significant storage

## Alternatives Considered

1. **Direct cell-to-cell RPC**: Rejected — couples cells, no audit trail, failure propagates
2. **Shared memory**: Rejected — violates no-shared-state principle, security concerns
3. **Traditional message queue**: Rejected — weak ordering guarantees
4. **gRPC/Protobuf**: Rejected — request/response only, no pub/sub, no replay

## Future Evolution

- Event bus may become federated across Pantheon instances
- Event compression and tiered storage for cost optimization
- Event time-to-live and automatic compaction policies
- Integration with external event systems (webhooks, Kafka bridges)
