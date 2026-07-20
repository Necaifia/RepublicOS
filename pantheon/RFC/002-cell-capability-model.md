# RFC 002: Cell Capability Delegation and Attenuation

**Status**: Draft
**Date**: Genesis
**Author**: Pantheon Founding Security Council

---

## Summary

Capabilities in Pantheon can be delegated from parent to child cells, and attenuated (narrowed) at each delegation. This RFC specifies the exact delegation and attenuation rules.

## Motivation

Without clear delegation rules:
- Capabilities may be amplified (security violation)
- Delegation chains may become unverifiable
- Revocation may be incomplete

## Delegation Rules

### Rule 1: No Amplification

A cell cannot delegate a capability that is broader than what it possesses:

```
valid: A has "memory://team-alpha/*" → A delegates "memory://team-alpha/docs" to B
invalid: A has "memory://team-alpha/*" → A delegates "memory://*" to B
```

### Rule 2: Monotonic Attenuation

Each delegation in the chain must be at least as restrictive as the parent:

```
A → B → C
If A delegates "memory://team-a/*" to B
B can delegate "memory://team-a/docs" to C (narrower ✓)
B cannot delegate "memory://team-*/docs" to C (broader ✗)
```

### Rule 3: Constraint Accumulation

Constraints (TTL, max uses, cost budget) are accumulated, not replaced:

```
A grants B: capability X, TTL=3600
B grants C: capability X (narrowed), TTL=1800
C's effective TTL: min(3600, 1800) = 1800
```

### Rule 4: Revocation Propagation

Revocation of a parent capability invalidates all derived capabilities:

```
If A revokes the capability granted to B:
  - B's capability is invalidated
  - All capabilities B derived from it are also invalidated
  - C's capability (derived from B) is invalidated
```

## Attenuation Dimensions

A capability can be attenuated along:

| Dimension | Description | Example |
|-----------|-------------|---------|
| Resource | Narrower resource pattern | `*` → `docs/*` |
| Actions | Fewer allowed actions | `[read, write]` → `[read]` |
| TTL | Shorter expiration | `3600` → `1800` |
| Max uses | Fewer allowed uses | `1000` → `100` |
| Cost budget | Lower cost limit | `100K` → `10K` |
| Delegation depth | Shorter delegation chain | `5` → `2` |

## Verification Algorithm

```
fn verify_delegation(parent: Capability, child: Capability) -> bool:
    // Check resource pattern is subset
    if not is_subset(child.resource, parent.resource): return false
    
    // Check actions are subset
    if not is_subset(child.actions, parent.actions): return false
    
    // Check TTL is not longer
    if child.ttl > parent.ttl: return false
    
    // Check max uses is not more
    if child.max_uses > parent.max_uses: return false
    
    // Check delegation depth is not deeper
    if child.delegation_depth >= parent.delegation_depth: return false
    
    return true
```

## Implementation

Capability verification is a kernel function to ensure determinism and performance. The kernel maintains a capability cache (LRU) for fast verification of recently used capabilities.
