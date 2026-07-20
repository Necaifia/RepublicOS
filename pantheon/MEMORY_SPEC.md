# Pantheon Memory Specification
## Distributed, Hierarchical, Persistent Knowledge Architecture

**Document**: 14 of 20
**Version**: Genesis
**Status**: Architectural Specification

---

## 0. Philosophy

Memory in Pantheon is not a single database. It is a **distributed, hierarchical, multi-modal storage system** that mirrors how biological memory works: different types of memory with different retention, access patterns, and consistency requirements.

---

## 1. Memory Types

### 1.1 The Four Memory Systems

```
┌──────────────────────────────────────────────────────────┐
│                     MEMORY SYSTEMS                        │
│                                                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Working    │  │   Episodic  │  │   Semantic  │     │
│  │   Memory     │  │   Memory    │  │   Memory    │     │
│  │              │  │             │  │             │     │
│  │ Current ctx  │  │ Past events │  │ Facts/con-  │     │
│  │ Volatile     │  │ Immutable   │  │ cepts       │     │
│  │ Cell-local   │  │ Event-based │  │ Graph-based │     │
│  └──────┬───────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                 │                 │            │
│  ┌──────▼─────────────────▼─────────────────▼──────┐     │
│  │              Procedural Memory                   │     │
│  │              (How to do things)                   │     │
│  │              Process specs, patterns, templates │     │
│  └──────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────┘
```

### 1.2 Working Memory

**Purpose**: Current execution context for a Cell

**Characteristics**:
- Cell-local (not shared)
- Volatile (lost on Cell termination)
- Size-limited (configurable, default 1MB)
- Fast access (in-process memory)

**Contents**:
- Current task context
- Relevant knowledge snippets
- Active reasoning state
- Tool call results
- Partial computations

**Implementation**:
- In-process LRU cache
- Serialized as part of Cell checkpoint
- Eviction policy: LRU + priority

### 1.3 Episodic Memory

**Purpose**: Record of past events, decisions, and outcomes

**Characteristics**:
- Immutable (append-only)
- Cell-scoped (each Cell has its own episode log)
- Durable (survives Cell restarts)
- Queryable by time, type, and content

**Contents**:
- Every decision made by the Cell
- Every action taken
- Every observation received
- Outcomes and feedback

**Implementation**:
- Partitioned event log (same as Event Bus)
- Indexed by: CellID, timestamp, event type
- Retention: configurable (default: full history)
- Compaction: older episodes can be summarized

**Query Patterns**:
- "What did I decide about X last time?"
- "What happened when I tried Y?"
- "How did Z turn out?"

### 1.4 Semantic Memory

**Purpose**: Structured knowledge about the world (codebase, domain, concepts)

**Characteristics**:
- Shared across Cells (with access control)
- Graph-structured (nodes + edges)
- Versioned
- Queryable by structure and semantics

**Contents**:
- Code entities and their relationships
- Domain concepts and their definitions
- System architecture knowledge
- Agent capabilities and expertise
- Project goals and status

**Implementation**:
- Distributed knowledge graph
- Storage backends: PostgreSQL / Dgraph / custom
- Vector embeddings for semantic search
- Full-text index for text search
- Versioning: event-sourced graph mutations

**Node Types**:
```
CodeModule
CodeFile
CodeFunction
CodeClass
CodeInterface
Concept
Decision
Goal
Task
Agent
Capability
Protocol
Test
Bug
Release
Dependency
```

**Edge Types**:
```
depends_on
implements
extends
calls
defines
references
decides
causes
relates_to
part_of
satisfies
tests
documents
```

### 1.5 Procedural Memory

**Purpose**: How to perform tasks — processes, patterns, templates

**Characteristics**:
- Shared across Cells (read-only by default)
- Versioned and reviewed
- Executable (can be run as workflows)
- Learnable (new procedures can be induced)

**Contents**:
- Workflow definitions
- Code generation patterns
- Refactoring recipes
- Testing strategies
- Deployment procedures
- Decision trees/heuristics

**Implementation**:
- Immutable procedure library
- Procedures are executable DAGs
- Versioned with semantic versions
- Reviewed by Quality Council before publication
- Can be composed and parametrized

---

## 2. Knowledge Graph Architecture

### 2.1 Graph Structure

```
Node {
  id: UUID
  type: NodeType
  properties: HashMap<String, Value>
  created_at: LamportTimestamp
  version: u64
}

Edge {
  id: UUID
  source: UUID
  target: UUID
  type: EdgeType
  weight: f64  // 0.0 to 1.0
  properties: HashMap<String, Value>
  created_at: LamportTimestamp
  version: u64
}
```

### 2.2 Distribution

The knowledge graph is sharded by domain:
- Code knowledge: sharded by module/repository
- Domain knowledge: sharded by concept area
- Operational knowledge: sharded by system component

Cross-shard queries are supported via:
- Query federation (execute on all shards, merge results)
- Reference edges (edges that cross shard boundaries)
- Caching (frequently accessed cross-shard data is cached)

### 2.3 Vector Embeddings

Every node and edge can have associated vector embeddings:
- Embedding model: configurable (default: ada-002 equivalent)
- Dimensions: configurable (default: 1536)
- Index: HNSW (Hierarchical Navigable Small World) for approximate nearest neighbor
- Updates: async batch update

### 2.4 Query Interface

```
query_graph(query: GraphQuery) → ResultSet

GraphQuery:
  - pattern_match(pattern: GraphPattern, constraints: Constraints) → Subgraph
  - semantic_search(text: String, top_k: u32, filters: Filters) → Nodes
  - traversal(start: NodeID, path: PathSpec, max_depth: u32) → Paths
  - temporal_range(start: Timestamp, end: Timestamp) → Subgraph
  - aggregation(query: AggregationSpec) → Statistics
```

---

## 3. Memory Access Control

Memory access is controlled through capabilities:

| Operation | Capability Required |
|-----------|---------------------|
| Read working memory | `memory.working.read` |
| Write working memory | `memory.working.write` |
| Read episodic memory | `memory.episodic.read:{cell_id}` |
| Write episodic memory | `memory.episodic.write:{cell_id}` (own only) |
| Read semantic memory | `memory.semantic.read:{domain}` |
| Write semantic memory | `memory.semantic.write:{domain}` |
| Create procedure | `memory.procedural.create` |
| Execute procedure | `memory.procedural.execute:{procedure_id}` |

---

## 4. Memory Lifecycle

### 4.1 Cell-Level Memory

When a Cell is created:
1. Working memory is initialized empty
2. Episodic memory is a new partition
3. Semantic memory access is inherited from parent
4. Procedural memory access is inherited from parent

When a Cell is terminated:
1. Working memory is discarded
2. Episodic memory is sealed (read-only)
3. Semantic memory contributions remain
4. Procedural memory contributions remain (if published)

### 4.2 Memory Retention

| Memory Type | Retention | Capacity | Eviction |
|-------------|-----------|----------|----------|
| Working | Task lifetime | 1 MB | LRU |
| Episodic | Permanent | Unlimited | Compaction |
| Semantic | Permanent | Unlimited | None |
| Procedural | Permanent | Unlimited | Versioned |

### 4.3 Memory Compaction

Episodic memory can be compacted to save space:
1. Multiple related episodes are summarized
2. The summary replaces the original episodes
3. Original episodes are archived (slow access)
4. Compaction is triggered at configurable thresholds

---

## 5. Cross-Cell Memory Integration

### 5.1 Memory Sharing Protocol

Cells can share memory through the Memory Sharing Protocol:
1. Cell A grants Cell B a capability to read a specific memory region
2. Cell B queries the region through the Memory Service
3. All accesses are logged
4. Capabilities can be revoked at any time

### 5.2 Memory Channels

A Memory Channel is a shared memory space that multiple Cells can read/write:
- Created by a Coordinator Cell
- Subscribe/notify pattern
- Conflict resolution: last-writer-wins (with history)
- Use case: shared project context

### 5.3 Memory Federation

Cross-Pantheon-instance memory sharing:
- Federation protocol for connecting independent Pantheon instances
- Each instance controls its own memory
- Query federation for cross-instance knowledge
- Consent-based data sharing

---

## 6. Caching Architecture

### 6.1 Cache Layers

```
L1: Cell-local cache (in-process, microsecond access)
L2: Node-local cache (shared memory, millisecond access)
L3: Cluster cache (distributed, Redis/Memcached)
L4: Persistent storage (database, tens of milliseconds)
```

### 6.2 Cache Invalidation

- Write-through for critical data
- Write-behind for performance-critical data
- Invalidation via event bus notifications
- TTL-based expiration
- Explicit invalidation on data change

---

## 7. Query Optimization

### 7.1 Query Planning

The Memory Service includes a query optimizer that:
1. Parses the query
2. Determines which memory systems are needed
3. Plans parallel execution across memory systems
4. Optimizes access patterns (index selection)
5. Caches intermediate results

### 7.2 Adaptive Querying

- Queries are monitored for performance
- Slow queries are automatically optimized
- Missing indexes are detected and suggested
- Query patterns are analyzed for batching opportunities

---

## 8. Memory Service Implementation

The Memory Service runs as a privileged System Cell with:
- Multiple worker threads (one per memory type)
- Connection pooling for backends
- Circuit breakers for backend failures
- Health checks and self-healing
- Metrics export for observability
