# ADR 010: Event Sourcing as the Single Source of Truth

**Status**: Accepted
**Date**: Genesis
**Author**: Pantheon Founding Engineering Council

---

## Context

Pantheon needs a durable, auditable, replayable record of all state changes. Traditional databases (update-in-place) lose history and provide no audit trail. Event sourcing records every state change as an immutable event.

## Decision

Adopt **Event Sourcing** as the single source of truth for ALL state.

### Key Design Choices

1. **Event log is the primary storage**: The current state is derived from the event log. There is no "database" separate from the event log.

2. **Materialized views for performance**: CQRS pattern. Commands go to the event log. Queries come from materialized views that are rebuilt from events.

3. **Snapshots for performance**: Periodic snapshots bound the replay time for recovery.

4. **No deletes**: Events are never deleted. Retention policies archive old events to cold storage.

5. **Schema evolution**: Events have schema versions. Old events can be read by new code.

### Event Types

| Event Type | Description | Example |
|-----------|-------------|---------|
| Command | Request to do something | `CreateCell { spec }` |
| Event | Something happened | `CellCreated { id, spec }` |
| Notification | Information for other cells | `CellTerminated { id, reason }` |
| Query | Request for information | `GetCellState { id }` |
| Response | Response to a query | `CellState { id, state }` |

### CQRS Pattern

```
Command → [Event Log] → [Event Processor] → [Materialized View]

Query → [Materialized View] → Response
```

The materialized view is eventually consistent with the event log. View rebuild is:
```
rebuild(view):
    for event in event_log since view.last_event:
        view.apply(event)
```

## Consequences

**Positive**:
- Complete audit trail of every state change
- Time-travel debugging (query state at any point in time)
- Disaster recovery (rebuild from event log)
- Concurrent systems (events can be processed in order)
- Testing (replay events in test environment)

**Negative**:
- Storage for event log (mitigated by tiered storage)
- Event schema evolution complexity
- Materialized view consistency lag
- Replay time for long-lived systems (mitigated by snapshots)

## Alternatives Considered

1. **Traditional database (update-in-place)**: Rejected — no audit trail, no replay
2. **Dual-write (DB + log)**: Rejected — consistency issues, complexity
3. **Command sourcing (store commands)**: Rejected — commands may fail, events are the truth

## Future Evolution

- Event log federation across Pantheon instances
- Event-driven analytics (real-time aggregation from event stream)
- Automated event schema migration
