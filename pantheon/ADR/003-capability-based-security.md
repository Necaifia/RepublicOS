# ADR 003: Capability-Based Security Model

**Status**: Accepted
**Date**: Genesis
**Author**: Pantheon Founding Security Council

---

## Context

Pantheon must support millions of cells with fine-grained access control. Traditional ACL-based security (subject/action/resource tuples) does not scale because:
1. ACLs require central administration
2. ACLs cannot be delegated
3. ACLs are coarse-grained
4. ACLs create a single point of failure

## Decision

Adopt **Capability-Based Security** as the universal security model.

### Key Design Choices

1. **Capabilities are unforgeable tokens**: A capability is a signed data structure. Without the issuer's signature, a capability is invalid. This eliminates the need for a central authority to validate access.

2. **Capabilities are delegatable**: Cell A can delegate a capability to Cell B. B can further delegate (with possible attenuation) to Cell C. This enables trust propagation without a central authority.

3. **Capabilities are attenuated on delegation**: A → B cannot grant more than A has. It can only narrow scope, reduce actions, or add constraints. This ensures the principle of least privilege.

4. **No ambient authority**: A Cell cannot perform any action without an explicit capability. There is no "root" or "administrator" role — only capabilities.

5. **Capabilities have short TTLs**: By default, capabilities expire within 24 hours. Long-lived capabilities require explicit renewal. This limits the blast radius of compromised capabilities.

## Consequences

**Positive**:
- Fine-grained, principle-of-least-privilege access control
- No central authority bottleneck
- Delegation enables complex authorization chains
- Short TTLs limit compromise blast radius
- Full audit trail of all capability usage

**Negative**:
- Capability management overhead (creating, delegating, revoking)
- Cryptographic operations for every capability check
- Revocation propagation requires Event Bus messages
- Debugging access issues is more complex than ACLs

## Alternatives Considered

1. **ACL-based (AWS IAM model)**: Rejected — central administration does not scale to millions of cells
2. **Role-based (RBAC)**: Rejected — roles are too coarse-grained, cannot delegate
3. **Attribute-based (ABAC)**: Rejected — policy evaluation is expensive and complex
4. **Trust-on-first-use (TOFU)**: Rejected — insecure for autonomous systems

## Future Evolution

- Capability chains may be optimized with caching
- Zero-knowledge proofs for capability verification without revealing delegation chain
- Economic capabilities (capabilities that cost resources to use)
