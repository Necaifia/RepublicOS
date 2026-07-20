# ADR 004: Deterministic Kernel with Sandboxed Non-Determinism

**Status**: Accepted
**Date**: Genesis
**Author**: Pantheon Founding Engineering Council

---

## Context

Pantheon must be testable, debuggable, and replayable. Non-determinism (from LLMs, network, wall-clock time, random number generators) makes this impossible if it infects the entire system. However, intelligence fundamentally requires non-deterministic components (LLMs).

## Decision

Adopt a **Deterministic Kernel** with explicit sandboxing of all non-deterministic operations.

### Key Design Choices

1. **Kernel is fully deterministic**: Given the same initial state and same event log, the kernel produces identical outputs. This includes scheduling decisions, event ordering, and state transitions.

2. **Deterministic time**: Logical (Lamport) time only. No wall-clock time in the kernel. Timeouts are expressed in event counts.

3. **Deterministic RNG**: PRNGs are seeded with the Cell ID and event count. Same sequence every time for the same inputs.

4. **Explicit non-determinism boundary**: LLM calls, network I/O, and external inputs are wrapped in special "non-determinism" event types. These events are logged with full input/output for replay.

5. **Replay mode**: In replay mode, non-deterministic events are served from the log instead of being executed. This enables deterministic replay of any past execution.

## Consequences

**Positive**:
- Full deterministic replay: debug any past state
- Reproducible tests: same test produces same result every time
- Simulation: run "what-if" scenarios by replaying events with changes
- Audit: every non-deterministic decision is logged

**Negative**:
- LLM calls cannot be replayed (different model version, different output)
- Replay of long executions is expensive
- Deterministic scheduling may be suboptimal for some workloads
- Wall-clock-dependent operations (e.g., "run for 5 seconds") require special handling

## Alternatives Considered

1. **Full non-determinism**: Rejected — impossible to debug, test, or audit
2. **Snapshot-based replay**: Rejected — too coarse-grained, cannot step through events
3. **Only log, no replay guarantee**: Rejected — insufficient for debugging

## Future Evolution

- LLM response caching for deterministic replay of common queries
- Compressed event logs for efficient long-period replay
- Partial replay (replay only a subsystem) for focused debugging
