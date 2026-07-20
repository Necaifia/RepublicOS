# RFC 003: Quadratic Voting for Resource Allocation

**Status**: Draft
**Date**: Genesis
**Author**: Pantheon Founding Governance Council

---

## Summary

Pantheon uses quadratic voting for resource allocation decisions to prevent majority domination while allowing intense preferences to be expressed.

## Motivation

In simple majority voting:
- 51% of agents can allocate 100% of resources to their priorities
- 49% get nothing they want
- Intense preferences cannot be distinguished from mild preferences

Quadratic voting addresses this by making the cost of votes increase quadratically.

## Design

### Vote Cost Function

```
cost(n) = n² voice_credits

where:
  n = number of votes cast on a single option
  voice_credits = agent's allocated voting budget
```

### Voice Credit Allocation

Each agent receives voice credits proportional to their contribution:
```
voice_credits = base_allocation × (1 + reputation_bonus)
```

Where:
- `base_allocation`: Equal for all agents
- `reputation_bonus`: 0-100% based on contribution metrics

### Example

```
Agent has 100 voice credits.

If they vote:
  1 vote on option A: cost = 1, remaining = 99
  2 votes on option A: cost = 4, remaining = 96
  10 votes on option A: cost = 100, remaining = 0

Or spread:
  5 votes on option A: cost = 25, remaining = 75
  5 votes on option B: cost = 25, remaining = 50
  7 votes on option C: cost = 49, remaining = 1
```

### Properties

- **Expresses intensity**: An agent who cares deeply can concentrate votes
- **Prevents domination**: Concentrating votes becomes expensive
- **Encourages spread**: Spreading votes is cheaper than concentrating
- **Reveals preferences**: Voting patterns reveal actual priorities

### Tallying

The winning option is the one with the most total votes (not voice credits spent):

```
Option A: 150 votes (agents spent 2250 credits)
Option B: 140 votes (agents spent 1960 credits)
Option C: 50 votes  (agents spent 250 credits)

Result: Option A wins (most votes)
```

### Sybil Resistance

Quadratic voting requires Sybil resistance:
- Each agent has exactly one identity (enforced by cryptographic identity)
- Voice credits cannot be transferred between agents
- Vote buying is detectable (voting pattern analysis)

## Implementation

Quadratic voting is implemented in the Governance Service:
1. Proposals specify that they use quadratic voting
2. Voters submit votes with their chosen allocation
3. The system verifies vote costs against the agent's credit balance
4. Votes are tallied using the quadratic formula
5. Results are published with full audit trail

## Open Questions

1. How often should voice credits be redistributed?
2. Should unused credits expire?
3. How to handle voting on correlated issues?
