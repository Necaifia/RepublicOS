# ADR 006: Four-Memory Hierarchical Architecture

**Status**: Accepted
**Date**: Genesis
**Author**: Pantheon Founding Scientific Council

---

## Context

Agents need memory to function effectively. A single memory approach (e.g., "everything in a vector database") fails because different memory types have radically different access patterns, consistency requirements, and retention needs.

## Decision

Adopt a **Four-Memory Architecture**: Working, Episodic, Semantic, and Procedural.

### Key Design Choices

1. **Working memory**: Cell-local, volatile, small. For current execution context. Fastest access. Lost on Cell termination.

2. **Episodic memory**: Cell-scoped, immutable, durable. Record of past events and decisions. Append-only. Supports replay and learning.

3. **Semantic memory**: Shared, graph-structured, versioned. Facts, concepts, and relationships. The "knowledge graph." Supports structured and semantic queries.

4. **Procedural memory**: Shared, executable, versioned. How to do things. Workflows, patterns, recipes. Can be composed and parameterized.

## Consequences

**Positive**:
- Each memory type optimized for its access pattern
- Clear separation of concerns
- Memory can be scaled independently
- Episodic + semantic = learning from experience
- Procedural memory enables skill transfer between agents

**Negative**:
- Complexity of four systems instead of one
- Cross-memory queries require orchestration
- Memory synchronization between types is non-trivial

## Alternatives Considered

1. **Single universal memory**: Rejected — cannot optimize for conflicting requirements
2. **Two-memory (short-term + long-term)**: Rejected — too coarse, misses procedural knowledge
3. **External database only**: Rejected — latency too high for working memory

## Future Evolution

- Sixth sense memory (predictive memory — pre-loading likely needed information)
- Compressed memory (lossy compression of old episodic memories)
- Memory consolidation (periodic transfer from episodic to semantic)
