# Pantheon Dependency Graph
## Module and Protocol Dependency Architecture

**Document**: 4 of 20
**Version**: Genesis
**Status**: Architectural Specification

---

## 0. Dependency Principles

1. **Acyclic**: Dependencies form a DAG at the module level
2. **Protocol-Based**: Dependencies are on protocols, not implementations
3. **Explicit**: All dependencies are declared and versioned
4. **Minimal**: A module depends only on what it directly uses
5. **Stable**: Stable abstractions depend on stable abstractions
6. **Inward**: Inner layers (kernel) know nothing about outer layers (interface)

---

## 1. Layer Dependency Hierarchy

```
                    ┌─────────────┐
                    │ Interface   │
                    │ (CLI, Web,  │
                    │  Desktop,   │
                    │  API)       │
                    └──────┬──────┘
                           │ depends on
                    ┌──────▼──────┐
                    │ Intelligence│
                    │ (Agents,    │
                    │  Reasoning, │
                    │  Planning)  │
                    └──────┬──────┘
                           │ depends on
                    ┌──────▼──────┐
                    │ Service     │
                    │ (Memory,    │
                    │  Scheduler, │
                    │  Gov,       │
                    │  Security)  │
                    └──────┬──────┘
                           │ depends on
                    ┌──────▼──────┐
                    │ Runtime     │
                    │ (Sandbox,   │
                    │  Checkpoint,│
                    │  Migration) │
                    └──────┬──────┘
                           │ depends on
                    ┌──────▼──────┐
                    │ Kernel      │
                    │ (Cell,      │
                    │  Scheduler, │
                    │  Event Bus, │
                    │  Caps)      │
                    └─────────────┘
```

---

## 2. Module Dependency Matrix

### 2.1 Kernel Dependencies

| Module | Depends On | Provides Protocol |
|--------|-----------|-------------------|
| cell | core.types | `pantheon.cell.v1` |
| scheduler | cell, resource, determinism | `pantheon.scheduler.v1` |
| event_bus | core.types, core.serialization | `pantheon.eventbus.v1` |
| capabilities | core.types, core.cryptography | `pantheon.capability.v1` |
| protocol | core.types | `pantheon.protocol.v1` |
| determinism | core.types | `pantheon.determinism.v1` |
| resource | core.types | `pantheon.resource.v1` |
| security | core.types, core.cryptography | `pantheon.security.v1` |
| kernel_api | (all kernel modules) | `pantheon.kernel.v1` |

### 2.2 Runtime Dependencies

| Module | Depends On | Provides Protocol |
|--------|-----------|-------------------|
| sandbox | kernel.kernel_api | `pantheon.sandbox.v1` |
| supervision | kernel.kernel_api | `pantheon.supervision.v1` |
| checkpoint | kernel.kernel_api, core.serialization | `pantheon.checkpoint.v1` |
| migration | kernel.kernel_api, checkpoint, connection | `pantheon.migration.v1` |
| discovery | kernel.kernel_api | `pantheon.discovery.v1` |
| connection | kernel.kernel_api, security | `pantheon.connection.v1` |

### 2.3 Service Dependencies

| Module | Depends On | Provides Protocol |
|--------|-----------|-------------------|
| memory | kernel.kernel_api, runtime.sandbox | `pantheon.memory.v1` |
| scheduler_service | kernel.kernel_api, memory | `pantheon.scheduler_service.v1` |
| identity | kernel.kernel_api, security | `pantheon.identity.v1` |
| governance | kernel.kernel_api, identity, memory | `pantheon.governance.v1` |
| security_service | kernel.kernel_api, identity | `pantheon.security_service.v1` |
| telemetry | kernel.kernel_api, memory | `pantheon.telemetry.v1` |

### 2.4 Intelligence Dependencies

| Module | Depends On | Provides Protocol |
|--------|-----------|-------------------|
| llm | kernel.kernel_api, memory, security_service | `pantheon.llm.v1` |
| reasoning | llm, memory | `pantheon.reasoning.v1` |
| planning | reasoning, memory, scheduler_service | `pantheon.planning.v1` |
| research | llm, memory | `pantheon.research.v1` |
| innovation | reasoning, research, memory | `pantheon.innovation.v1` |
| agents | llm, reasoning, planning, memory, governance | `pantheon.agent.v1` |

### 2.5 Engine Dependencies

| Module | Depends On | Provides Protocol |
|--------|-----------|-------------------|
| architecture | memory, reasoning, code_analysis | `pantheon.architecture.v1` |
| refactoring | architecture, memory, testing | `pantheon.refactoring.v1` |
| simulation | kernel.kernel_api, all services | `pantheon.simulation.v1` |
| testing | kernel.kernel_api, sandbox | `pantheon.testing.v1` |

### 2.6 Interface Dependencies

| Module | Depends On | Provides Protocol |
|--------|-----------|-------------------|
| cli | all services (via API) | `pantheon.cli.v1` |
| web | all services (via API) | `pantheon.web.v1` |
| desktop | all services (via API) | `pantheon.desktop.v1` |
| api | all services | `pantheon.api.v1` |

---

## 3. Protocol Dependency Graph

```
pantheon.core.v1
├── pantheon.cell.v1
│   ├── pantheon.scheduler.v1
│   ├── pantheon.eventbus.v1
│   ├── pantheon.capability.v1
│   └── pantheon.protocol.v1
│
├── pantheon.sandbox.v1
│   ├── pantheon.supervision.v1
│   ├── pantheon.checkpoint.v1
│   └── pantheon.migration.v1
│
├── pantheon.discovery.v1
├── pantheon.connection.v1
│
├── pantheon.memory.v1
│   ├── pantheon.identity.v1
│   ├── pantheon.governance.v1
│   ├── pantheon.security_service.v1
│   └── pantheon.telemetry.v1
│
├── pantheon.llm.v1
│   ├── pantheon.reasoning.v1
│   ├── pantheon.planning.v1
│   ├── pantheon.research.v1
│   └── pantheon.innovation.v1
│
├── pantheon.agent.v1
├── pantheon.architecture.v1
├── pantheon.refactoring.v1
├── pantheon.simulation.v1
├── pantheon.testing.v1
│
├── pantheon.cli.v1
├── pantheon.web.v1
├── pantheon.desktop.v1
├── pantheon.api.v1
│
└── pantheon.plugin.v1
```

---

## 4. Circular Dependency Prevention

### 4.1 Rules

1. **Strict layering**: A layer may only depend on layers below it
2. **No cycles**: The dependency graph must remain a DAG
3. **Protocol inversion**: If A needs B and B needs A, extract protocol P that both depend on
4. **Build-time enforcement**: Circular dependencies fail at module compile time
5. **Runtime detection**: Dependency cycles at runtime trigger circuit breaker

### 4.2 Cycle Breaking Strategy

If a cycle is detected:

```
Original:  A → B → C → A

Resolution:
  1. Identify the abstraction both directions need
  2. Extract Protocol P from A and C
  3. A → P, B → P, C → P
  4. All depend on P, cycle broken

Result:    A → P ← B → P ← C
                ↓
              P ← (cycle broken)
```

### 4.3 Dependency Inversion Example

Instead of `Agent → Memory` and `Memory needs Agent info`:

```
Agent → MemoryProtocol ← Memory
        ↓
      AgentProtocol ← (provides agent info without agent dependency)
```

---

## 5. External Dependencies

### 5.1 Required External Systems

| Dependency | Purpose | Versioning | Replacement Strategy |
|-----------|---------|-----------|---------------------|
| Content-addressed storage (IPFS/S3) | File storage | Swappable backend | `pantheon.storage.v1` |
| Event log (Kafka/Redpanda) | Event bus | Swappable backend | `pantheon.eventlog.v1` |
| SQL database (CockroachDB/FoundationDB) | Structured data | Swappable backend | `pantheon.database.v1` |
| Graph database (Dgraph/Neo4j) | Knowledge graph | Swappable backend | `pantheon.graphdb.v1` |
| Vector database (Qdrant/Pinecone) | Embeddings | Swappable backend | `pantheon.vectordb.v1` |
| Container runtime (Docker/WASM) | Sandbox | Swappable backend | `pantheon.sandboxbackend.v1` |
| Message queue (NATS/RabbitMQ) | Internal messaging | Swappable backend | `pantheon.messaging.v1` |
| LLM API (OpenAI/Anthropic/local) | Intelligence | Swappable provider | `pantheon.llmprovider.v1` |
| Key-value store (etcd/Redis) | Coordination | Swappable backend | `pantheon.kvstore.v1` |

### 5.2 Abstraction Principle

Every external dependency is behind a protocol. No code imports external libraries directly — they import protocol types and use a provider that implements the protocol. This allows swapping any dependency without changing application code.
