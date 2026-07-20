# Pantheon Communication Protocols
## Formal Specification of All Inter-Cell Communication

**Document**: 7 of 20
**Version**: Genesis
**Status**: Architectural Specification

---

## 0. Protocol Design Principles

1. **Versioned**: Every protocol has a semantic version
2. **Negotiated**: Client and server negotiate highest common version
3. **Self-describing**: Protocol messages carry type metadata
4. **Backward compatible**: New versions must not break old clients
5. **Failed**: Protocols specify failure modes and error handling
6. **Observable**: All protocol messages are logged for audit

---

## 1. Core Protocols

### 1.1 Cell Lifecycle Protocol (`pantheon.cell.lifecycle.v1`)

Manages Cell creation, execution, and termination.

**Messages**:

```
CellLifecycle.Create {
    spec: CellSpec
    parent_capability: Capability
}
→ Response {
    cell_id: CellID
    initial_capability: Capability
}

CellLifecycle.Start {
    cell_id: CellID
}
→ Response {
    status: Running | Failed(error)
}

CellLifecycle.Suspend {
    cell_id: CellID
    reason: String
}
→ Response {
    checkpoint_id: CheckpointID
}

CellLifecycle.Resume {
    cell_id: CellID
    checkpoint_id: Option<CheckpointID>
}
→ Response {
    status: Running | Failed(error)
}

CellLifecycle.Terminate {
    cell_id: CellID
    reason: TerminationReason  // Success | Failure | Timeout | ParentTerminated | Governance(action)
}
→ Response {
    final_state: CellSnapshot
}
```

**State Machine**:

```
[Created] → Start → [Running] → Suspend → [Suspended] → Resume → [Running]
[Running] → Timeout → [Terminated]
[Running] → Terminate → [Terminated]
[Running] → Fail → [Terminated]
```

**Error Handling**:
- `CellNotFound`: Specified cell does not exist
- `CellAlreadyRunning`: Cannot start an already running cell
- `CapabilityViolation`: Caller lacks required capability
- `ResourceExhausted`: Cannot allocate resources for cell
- `InvalidTransition`: Requested state transition is invalid

---

### 1.2 Event Bus Protocol (`pantheon.eventbus.v1`)

Core message exchange between Cells.

**Messages**:

```
EventBus.Publish {
    events: Vec<Event>
    partition_key: Option<Hash>  // Default: target capability hash
}
→ Response {
    offsets: Vec<(PartitionID, Offset)>
}

EventBus.Subscribe {
    subscription: SubscriptionSpec
    // capability_pattern: "memory://team-alpha/*"
    // event_types: ["command", "notification"]
    // since: Option<Offset>
}
→ Stream<Event>  // Long-lived stream

EventBus.Acknowledge {
    partition: PartitionID
    offset: Offset
}
→ Response { }

EventBus.Replay {
    partition: PartitionID
    start_offset: Offset
    end_offset: Option<Offset>
}
→ Stream<Event>
```

**Delivery Guarantees**:
- Within a partition: exactly-once, in order
- Across partitions: at-least-once, no ordering guarantee
- Consumers must deduplicate across partitions (using EventID)

**Flow Control**:
- Consumers control their own pace (pull-based)
- Backpressure: slow consumers are pushed to older offsets
- Dead-letter: unprocessable events are moved to DLQ

---

### 1.3 Capability Protocol (`pantheon.capability.v1`)

Capability delegation, verification, and revocation.

**Messages**:

```
Capability.Delegate {
    capability: CapabilityID
    delegate_to: CellID
    attenuation: AttenuationSpec  // What to narrow
    ttl: Duration
}
→ Response {
    delegated_capability: Capability
}

Capability.Verify {
    capability: Capability
    action: Action
    resource: Resource
}
→ Response {
    is_valid: bool
    reason: Option<String>
}

Capability.Revoke {
    capability: CapabilityID
    reason: String
}
→ Response { }

Capability.List {
    for_cell: CellID
}
→ Response {
    capabilities: Vec<CapabilityInfo>
}
```

**Delegation Chain Verification**:
```
verify(capability, action, resource):
    if not capability.signature_valid(): return false
    if capability.has_expired(): return false
    if not capability.permits(action, resource): return false
    if capability.issuer != ROOT:
        parent_capability = get_parent(capability)
        return verify(parent_capability, capability.original_action, capability.original_resource)
    return true
```

---

### 1.4 Discovery Protocol (`pantheon.discovery.v1`)

Cell and service discovery.

**Messages**:

```
Discovery.Find {
    protocol: ProtocolRef
    constraints: Vec<Constraint>
    // e.g., protocol: "pantheon.memory.v1", proximity: "same_node"
}
→ Response {
    cells: Vec<CellEndpoint>
}

Discovery.Register {
    protocols: Vec<ProtocolRef>
    endpoint: CellEndpoint
    metadata: HashMap<String, String>
}
→ Response { }

Discovery.Heartbeat {
    cell_id: CellID
    status: HealthStatus
}
→ Response { }
```

---

## 2. Service Protocols

### 2.1 Memory Protocol (`pantheon.memory.v1`)

Read and write memory across all four memory systems.

**Messages**:

```
Memory.Write {
    memory_type: MemoryType  // Working | Episodic | Semantic | Procedural
    region: MemoryRegionID
    data: Blob
    metadata: Option<Metadata>
}
→ Response {
    version: u64
}

Memory.Read {
    memory_type: MemoryType
    query: MemoryQuery
}
→ Response {
    results: Vec<MemoryItem>
}

Memory.Query {
    memory_type: MemoryType
    query: StructuredQuery
    // e.g., graph pattern, vector similarity, SQL
}
→ Response {
    results: QueryResult
}

Memory.Subscribe {
    region: MemoryRegionID
    event_types: Vec<MemoryEventType>
}
→ Stream<MemoryEvent>
```

### 2.2 Scheduling Protocol (`pantheon.scheduler.v1`)

Task submission and management.

**Messages**:

```
Scheduler.Submit {
    task_spec: TaskSpec
    // input, context, capabilities, deadline, priority
}
→ Response {
    task_id: TaskID
}

Scheduler.Status {
    task_id: TaskID
}
→ Response {
    status: Queued | Running(progress) | Completed(result) | Failed(error)
}

Scheduler.Cancel {
    task_id: TaskID
    reason: String
}
→ Response { }

Scheduler.Query {
    filter: TaskFilter  // by status, type, assignee, time range
}
→ Response {
    tasks: Vec<TaskInfo>
}
```

### 2.3 Governance Protocol (`pantheon.governance.v1`)

Proposals, voting, elections, and dispute resolution.

**Messages**:

```
Governance.SubmitProposal {
    proposal: Proposal
    // type, title, body, sponsor, affected_protocols
}
→ Response {
    proposal_id: ProposalID
    status: ProposalStatus
}

Governance.Vote {
    proposal_id: ProposalID
    vote: Vote  // For | Against | Abstain | StrongFor | StrongAgainst
    rationale: Option<String>
}
→ Response { }

Governance.QueryProposal {
    proposal_id: ProposalID
}
→ Response {
    proposal: ProposalDetail
    status: ProposalStatus
    votes: VoteSummary
}

Governance.InitiateElection {
    position: Position  // CEO | CTO | ChiefScientist | Parliament | Judge
}
→ Response {
    election_id: ElectionID
}

Governance.CastElectionVote {
    election_id: ElectionID
    candidate_id: CellID
    ranked_choices: Option<Vec<CellID>>
}
→ Response { }

Governance.FileDispute {
    plaintiff: CellID
    defendant: CellID
    claim: String
    evidence: Vec<EventID>
}
→ Response {
    case_id: CaseID
}
```

---

## 3. Intelligence Protocols

### 3.1 LLM Protocol (`pantheon.llm.v1`)

Abstracted LLM access.

**Messages**:

```
LLM.Complete {
    model: String
    messages: Vec<Message>
    parameters: CompletionParams
    // temperature, max_tokens, stop, etc.
}
→ Response {
    content: String
    usage: TokenUsage
    model: String
}

LLM.Embed {
    model: String
    input: Vec<String>
}
→ Response {
    embeddings: Vec<Vec<f64>>
    model: String
}

LLM.StreamComplete {
    model: String
    messages: Vec<Message>
    parameters: CompletionParams
}
→ Stream<Chunk>
```

### 3.2 Agent Protocol (`pantheon.agent.v1`)

Agent-to-agent and agent-to-system communication.

**Messages**:

```
Agent.Delegate {
    task: Task
    to_agent: CellID
    context: ContextReference
}
→ Response {
    subtask_id: TaskID
}

Agent.Report {
    to: CellID
    status: StatusReport
    // what was done, result, issues, next steps
}
→ Response { }

Agent.RequestInfo {
    from: CellID
    topic: String
    specificity: Specificity
}
→ Response {
    information: Blob
}

Agent.Collaborate {
    with: Vec<CellID>
    on: TaskID
    mode: CollaborationMode  // Swarm | Debate | Consensus
}
→ Stream<CollaborationMessage>
```

---

## 4. Protocol Version Negotiation

When two Cells establish communication:

```
1. Initiator sends: ProtocolRequest {
       protocol: "pantheon.memory.v1"
       supported_versions: [1, 2, 3]
       features: ["vector_search", "graph_query"]
   }

2. Responder sends: ProtocolResponse {
       protocol: "pantheon.memory.v1"
       selected_version: 2
       supported_features: ["vector_search"]  // graph_query not supported
       max_message_size: 1048576
   }

3. Communication proceeds using selected version and features.

4. If no common version: ProtocolError { reason: "No compatible version" }
```

---

## 5. Error Handling

All protocols follow a uniform error model:

```json
{
    "code": "RESOURCE_EXHAUSTED",
    "message": "Memory region 'team-alpha' is at capacity",
    "details": {
        "region_id": "memory://team-alpha",
        "current_size": "10GB",
        "max_size": "10GB",
        "retry_after_ms": 5000
    }
}
```

**Standard Error Codes**:

| Code | HTTP Equivalent | Meaning |
|------|----------------|---------|
| INVALID_ARGUMENT | 400 | Malformed request |
| UNAUTHENTICATED | 401 | No capability provided |
| PERMISSION_DENIED | 403 | Capability insufficient |
| NOT_FOUND | 404 | Resource not found |
| RESOURCE_EXHAUSTED | 429 | Rate limit or capacity |
| INTERNAL_ERROR | 500 | Unexpected system error |
| UNAVAILABLE | 503 | Service temporarily unavailable |
| DEADLINE_EXCEEDED | 504 | Operation timed out |
| PROTOCOL_VIOLATION | - | Protocol sequence error |
