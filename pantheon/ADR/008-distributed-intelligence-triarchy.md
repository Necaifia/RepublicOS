# ADR 008: Tripartite Intelligence Model (Central + Decentralized + Collective)

**Status**: Accepted
**Date**: Genesis
**Author**: Pantheon Founding Scientific Council

---

## Context

Intelligence in Pantheon must balance:
- **Coordination**: Some decisions need global consistency
- **Scale**: Millions of agents need local autonomy
- **Wisdom**: Better decisions through aggregation
- **Speed**: Some decisions must be instant
- **Depth**: Some decisions need deep analysis

No single intelligence architecture satisfies all requirements.

## Decision

Adopt a **Tripartite Intelligence Model**: Central Intelligence for coordination, Decentralized Intelligence for scale, Collective Intelligence for wisdom.

### Key Design Choices

1. **Central Intelligence (CEO/CTO/Chief Scientist)**: Long-lived, system-wide context, high authority. Used for strategic direction, architecture decisions, and resource allocation. CHECK: elected, reviewed, recallable.

2. **Decentralized Intelligence (worker agents)**: Short-lived, local context, narrow authority. Used for task execution. Capable of spawning, merging, splitting, and retiring. CHECK: bounded by capabilities and resource budgets.

3. **Collective Intelligence (voting, markets, deliberation)**: Emergent intelligence from group interaction. Used for high-uncertainty decisions, resource allocation, and quality evaluation. CHECK: algorithm selection based on decision type.

## Consequences

**Positive**:
- Appropriate intelligence type for each decision
- Central intelligence provides coherence
- Decentralized intelligence provides scale
- Collective intelligence provides wisdom
- Natural failure mode isolation (one type failing doesn't affect others)

**Negative**:
- Three systems to maintain
- Potential for conflict between intelligence types (e.g., central vs. collective)
- Complex interaction patterns between types

## Alternatives Considered

1. **Fully centralized (one big brain)**: Rejected — single point of failure, does not scale
2. **Fully decentralized (ant colony)**: Rejected — lacks coherence for software production
3. **Fully collective (pure democracy)**: Rejected — too slow for routine decisions

## Future Evolution

- Intelligence type selection may become adaptive (system learns what type works for what)
- New intelligence types may emerge (e.g., analogical, intuitive)
- Cross-instance collective intelligence for multi-Pantheon systems
