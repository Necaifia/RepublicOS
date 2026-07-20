# RFC 001: Deterministic Event Replay for Debugging and Audit

**Status**: Draft
**Date**: Genesis
**Author**: Pantheon Founding Engineering Council

---

## Summary

Pantheon must support full deterministic replay of any past execution for debugging, audit, and simulation purposes. This RFC specifies the mechanism.

## Motivation

Without deterministic replay:
- Debugging production issues requires guesswork
- Auditing past decisions is incomplete
- "What-if" simulation is impossible
- Reproducing bugs becomes "it works on my machine"

## Design

### Event Log Structure

Every event is logged with:
1. Event ID (partition, offset)
2. Timestamp (Lamport clock value)
3. Source cell ID
4. Target capability
5. Input payload (serialized)
6. Output payload (serialized) — for commands
7. Non-determinism marker (if the event involved LLM/external input)
8. Parent event ID chain

### Replay Algorithm

```
replay(target_state):
    state = EMPTY
    for event in event_log since_beginning():
        if event.is_non_deterministic():
            // Use cached output from original execution
            state.apply(event, event.cached_output())
        else:
            // Re-execute deterministically
            output = deterministic_execute(event, state)
            state.apply(event, output)
        if state == target_state:
            return OK
    return NOT_FOUND
```

### Non-Determinism Handling

Non-deterministic events (LLM calls, external API calls) are handled by:
1. **Recording**: During original execution, both input AND output are logged
2. **Replaying**: During replay, the cached output is served instead of re-executing
3. **Verification**: The replayed output can be compared to the original (for drift detection)

### Partial Replay

Replay can be scoped to a subset of cells:
- Only replay events involving cell IDs matching a pattern
- Other cells start from their checkpoint at replay start time
- This enables focused debugging without replaying the entire system

### Simulation Mode

In simulation mode:
- Events are generated artificially (not from log)
- Non-deterministic outputs are sampled from a configurable distribution
- Multiple simulations can be run in parallel with different parameters
- This enables "what-if" analysis

## Performance

- Full replay of 1B events: ~30 minutes (single node, sequential)
- Full replay with indexing: ~5 minutes
- Partial replay (single cell): ~seconds

## Open Questions

1. How long to retain full event logs vs. summarized logs?
2. Should we support parallel replay across multiple nodes?
3. How to handle LLM model version changes during replay?
