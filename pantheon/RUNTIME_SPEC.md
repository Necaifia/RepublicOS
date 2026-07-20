# Pantheon Runtime Specification
## Cell Execution Environment and Lifecycle

**Document**: 15 of 20
**Version**: Genesis
**Status**: Architectural Specification

---

## 0. Philosophy

The Runtime is not a single process. It is a **distributed cell execution environment** that can span machines, data centers, and clouds while presenting a unified execution model.

---

## 1. Runtime Architecture

### 1.1 Runtime Instance

A Runtime Instance is a single node in the Pantheon cluster:

```
┌──────────────────────────────────────────────┐
│          Pantheon Runtime Instance            │
│                                              │
│  ┌──────────────────────────────────────────┐│
│  │         Cell Execution Sandbox           ││
│  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐      ││
│  │  │Cell │ │Cell │ │Cell │ │Cell │ ...   ││
│  │  │  A  │ │  B  │ │  C  │ │  D  │      ││
│  │  └──┬──┘ └──┬──┘ └──┬──┘ └──┬──┘      ││
│  │     │       │       │       │          ││
│  │  ┌──▼───────▼───────▼───────▼────────┐ ││
│  │  │       Event Bus Client            │ ││
│  │  └───────────────────────────────────┘ ││
│  └──────────────────────────────────────────┘│
│                                              │
│  ┌──────────────────────────────────────────┐│
│  │         System Service Stubs             ││
│  │  Memory │ Scheduler │ Identity │ Gov     ││
│  └──────────────────────────────────────────┘│
│                                              │
│  ┌──────────────────────────────────────────┐│
│  │         Runtime Services                  ││
│  │  Sandbox │ Checkpoint │ GC │ Metrics     ││
│  └──────────────────────────────────────────┘│
│                                              │
│  ┌──────────────────────────────────────────┐│
│  │         Connection Manager               ││
│  │  Cluster │ Cloud │ Offline Sync          ││
│  └──────────────────────────────────────────┘│
└──────────────────────────────────────────────┘
```

### 1.2 Cell Execution Sandbox

Every Cell executes in an isolated sandbox:

| Dimension | Isolation Mechanism | Configurability |
|-----------|-------------------|-----------------|
| Process | Container (Docker/WASM) | OS process for trusted |
| Memory | cgroup / namespace | Hard limit, soft limit |
| CPU | cgroup / scheduler | Shares, cores, priority |
| Network | Network namespace + capabilities | Full, restricted, none |
| Filesystem | Mount namespace + overlay | Read-only, temp, persistent |
| System calls | Seccomp filter | Default-deny, allowlist |

### 1.3 Sandbox Profiles

| Profile | Use Case | Isolation Level |
|---------|----------|-----------------|
| Untrusted | Third-party plugins | Full container + seccomp |
| Semi-trusted | Standard agents | Lightweight container |
| Trusted | System services | OS process + cgroup |
| Critical | Kernel services | No sandbox (kernel space) |

---

## 2. Cell Lifecycle

### 2.1 Lifecycle State Machine

```
                       ┌──────────┐
                       │  NULL    │
                       └────┬─────┘
                            │ create
                            ▼
                    ┌───────────────┐
             ┌─────│   CREATED     │
             │     └───────┬───────┘
             │             │ start
             │             ▼
             │     ┌───────────────┐
             │     │   RUNNING     │◄───────────────────┐
             │     └───┬───┬───┬──┘                    │
             │         │   │   │                        │
             │         │   │   └────────────────┐       │
             │         │   │                    │       │
             │         │   ▼                    ▼       │
             │         │ ┌─────────┐  ┌─────────────┐  │
             │         │ │SUSPENDED│  │  BLOCKED    │  │
             │         │ └────┬────┘  └──────┬──────┘  │
             │         │      │              │          │
             │         │      └──────────────┘          │
             │         │                                │
             │         ▼                                │
             │     ┌───────────┐                        │
             │     │ TERMINATED│                        │
             │     └─────┬─────┘                        │
             │           │                              │
             │           ▼                              │
             │     ┌──────────────┐                     │
             └─────│    RETIRED   │                     │
                   └──────────────┘                     │
                        │                               │
                        ▼                               │
                   ┌──────────┐                         │
                   │  DESTROYED│─────────────────────────┘
                   └──────────┘
```

### 2.2 State Descriptions

| State | Description | Persistence |
|-------|-------------|-------------|
| NULL | Logical initial state, not yet created | None |
| CREATED | Cell definition accepted, resources allocated | Metadata |
| RUNNING | Cell is executing | Full state |
| SUSPENDED | Cell paused, state preserved | Full state snapshot |
| BLOCKED | Cell waiting for external event | State preserved |
| TERMINATED | Cell finished execution (success or failure) | Final state |
| RETIRED | Cell terminated, being cleaned up | Episodic memory only |
| DESTROYED | All resources released | Nothing |

### 2.3 Lifecycle Events

| Event | Trigger | Description |
|-------|---------|-------------|
| `cell.create` | Parent cell or system | Create cell definition |
| `cell.start` | Scheduler | Allocate resources, begin execution |
| `cell.suspend` | Scheduler or cell itself | Pause execution, checkpoint state |
| `cell.resume` | Scheduler | Restore from checkpoint, continue execution |
| `cell.block` | Cell itself (implicit) | Wait for event |
| `cell.unblock` | Scheduler | Event received, ready to run |
| `cell.terminate` | Cell itself or parent | End execution |
| `cell.retire` | Scheduler | Clean up resources |
| `cell.destroy` | Garbage collector | Final cleanup |
| `cell.fail` | Cell itself or watchdog | Unexpected termination |

---

## 3. Checkpoint and Restore

### 3.1 Checkpoint Types

| Type | Frequency | Contents | Cost |
|------|-----------|----------|------|
| Full | Every N events | Complete state | High |
| Incremental | Between full | Changes since last checkpoint | Low |
| Periodic | Configurable interval | Full or incremental | Configurable |

### 3.2 Checkpoint Contents

```
Checkpoint {
  cell_id: CellID
  version: u64
  state: Blob  // Deterministic state
  event_log_position: (partition, offset)
  capabilities: CapabilitySet
  memory_regions: Vec<MemoryRegionID>
  timestamp: LamportTimestamp
  parent_event: EventID
}
```

### 3.3 Restore Process

1. Load most recent full checkpoint
2. Apply incremental checkpoints (in order)
3. Replay events from checkpoint position to current
4. Verify state consistency (hash check)
5. Resume execution

### 3.4 Recovery Point Objective (RPO)

| Cell Type | Target RPO | Checkpoint Frequency |
|-----------|------------|---------------------|
| Worker | Events since last yield | After each task |
| Coordinator | < 1 second | Every event |
| Service | < 100ms | Every event (synchronous) |
| Agent | < 1 second | Every reasoning step |

---

## 4. Scheduling

### 4.1 Scheduler Architecture

```
┌──────────────────────────────────────────┐
│              Scheduler                    │
│                                          │
│  ┌──────────┐  ┌──────────┐             │
│  │ Priority  │  │ Resource │             │
│  │ Queue     │  │ Monitor  │             │
│  ├──────────┤  ├──────────┤             │
│  │ Dispatch  │  │ Preemption│            │
│  │ Loop      │  │ Handler  │             │
│  └──────────┘  └──────────┘             │
│                                          │
│  ┌──────────────────────────────────────┐│
│  │        Scheduling Policy             ││
│  │  (Pluggable: FIFO, RoundRobin,       ││
│  │   EDF, Proportional, Custom)         ││
│  └──────────────────────────────────────┘│
└──────────────────────────────────────────┘
```

### 4.2 Priority Calculation

```
priority = base_priority × urgency × importance × dependency_depth

where:
  base_priority = assigned at creation (system > service > agent > worker)
  urgency = 1 / (deadline - current_time + 1)
  importance = assigned by parent (based on goal criticality)
  dependency_depth = number of cells waiting on this cell's output
```

### 4.3 Preemption

- Higher-priority cells can preempt lower-priority cells
- Preemption saves cell state (checkpoint)
- Preempted cell is resumed when resources are available
- Starvation prevention: priority aging (priority increases over time)

### 4.4 Distributed Scheduling

- Global scheduler assigns cells to runtime instances
- Local scheduler manages cells on its instance
- Load balancing: cells are moved from overloaded to underloaded instances
- Affinity: cells that communicate frequently are scheduled on the same instance
- Migration: cells can be migrated (checkpoint + restore on new instance)

---

## 5. Resource Management

### 5.1 Resource Types

| Resource | Unit | Limit Mechanism |
|----------|------|-----------------|
| CPU | Millicores | cgroup CPU quota |
| Memory | Bytes | cgroup memory limit |
| Storage | Bytes | Filesystem quota |
| Network | Bytes/sec | Traffic shaping |
| LLM Tokens | Tokens | Token bucket |
| Event Bus | Events/sec | Rate limiter |

### 5.2 Resource Accounting

- Every resource used by a Cell is tracked
- Resource usage is attributed to the Cell chain (parent → child)
- Budgets are enforced at capability check time
- Overspend triggers: warning → throttling → termination

### 5.3 Resource Quotas

```
CellQuota {
  cpu: Range(u64),         // min, max millicores
  memory: Range(u64),      // min, max bytes
  storage: u64,            // max bytes
  network_ingress: u64,    // bytes/sec
  network_egress: u64,     // bytes/sec
  llm_tokens_per_sec: u64,
  llm_tokens_per_day: u64,
  event_rate: u64,         // events/sec
  child_limit: u32,        // max children
  lifetime: Duration,      // max lifetime
}
```

---

## 6. Garbage Collection

### 6.1 Cell GC

- RETIRED cells are cleaned up after a configurable delay
- Childless cells with no references are collected first
- Orphaned cells (parent terminated) are re-parented or retired

### 6.2 Memory GC

- Unreferenced memory regions are deallocated
- Old episodic memory is compacted
- Stale cache entries are evicted
- Orphaned knowledge graph nodes are archived

### 6.3 Event Log GC

- Events beyond retention period are archived to cold storage
- Compaction removes redundant events (e.g., overwritten state)
- Snapshots older than retention period are deleted

---

## 7. Connection Management

### 7.1 Cluster Mode

- Runtime instances connect via gRPC/QUIC
- Event Bus partitions are distributed across instances
- Heartbeat protocol for liveness detection
- Leader election for partition ownership

### 7.2 Cloud Mode

- Runtime instances connect to Pantheon Cloud control plane
- Control plane handles: identity, scheduling, event routing
- Data plane is direct peer-to-peer
- Auto-scaling: instances are provisioned and deprovisioned based on load

### 7.3 Offline Mode

- Local event log, synced when connectivity is available
- Conflict resolution: last-writer-wins with causual ordering
- Priority: consistency is restored on sync
- Human-in-the-loop for conflict resolution

---

## 8. Security

### 8.1 Runtime Security

- All inter-instance communication is encrypted (TLS 1.3+)
- Instance identity is attested (mutual TLS)
- Audit log of all cross-instance operations
- Network segmentation (cells cannot see each other's traffic)

### 8.2 Supply Chain Security

- Code bundles are verified before execution
- Dependencies are verified (hash + signature)
- Runtime binaries are measured and attested
- Updates are signed and verified

### 8.3 Side-Channel Protection

- CPU cache partitioning (CAT)
- Memory bandwidth monitoring
- Timing attack mitigation (deterministic timing in kernel)
- Co-location detection and prevention
